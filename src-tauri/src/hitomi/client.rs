use super::gg::{self, GgData};
use super::models::{Gallery, GalleryPage, GalleryType, SortOrder, Suggestion};
use super::nozomi::{
    build_browse_nozomi_urls, fetch_all_gallery_ids, fetch_gallery_ids, nozomi_url_from_args,
};
use super::search::{build_constraints, constraint_to_nozomi_url, parse as parse_query};
use super::{config, http, image_cache, url_gen};
use crate::error::{AppError, AppResult};
use futures::future::join_all;
use futures::stream::{self, StreamExt};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use tokio::sync::{Mutex as AsyncMutex, RwLock};

const GG_TTL: Duration = Duration::from_secs(30 * 60);
const CACHE_SWEEP_EVERY: usize = 200;

struct GgCache {
    data: Arc<GgData>,
    fetched_at: Instant,
}

pub struct HitomiClient {
    http: reqwest::Client,
    gg: RwLock<Option<GgCache>>,
    gg_refresh: AsyncMutex<()>,
    cache_dir: OnceLock<PathBuf>,
    cache_limit: AtomicU64,
    writes_since_sweep: AtomicUsize,
    downloads: Mutex<HashMap<i64, Arc<AtomicBool>>>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadResult {
    pub id: i64,
    pub gallery: Gallery,
    pub folder: String,
    pub done: usize,
    pub total: usize,
    pub failed: usize,
    pub failed_pages: Vec<usize>,
    pub skipped: usize,
}

impl HitomiClient {
    pub fn new() -> HitomiClient {
        HitomiClient {
            http: http::build_client(),
            gg: RwLock::new(None),
            gg_refresh: AsyncMutex::new(()),
            cache_dir: OnceLock::new(),
            cache_limit: AtomicU64::new(config::DEFAULT_IMAGE_CACHE_BYTES),
            writes_since_sweep: AtomicUsize::new(0),
            downloads: Mutex::new(HashMap::new()),
        }
    }

    pub fn set_cache_limit(&self, bytes: u64) {
        self.cache_limit.store(bytes, Ordering::Relaxed);
        if let Some(dir) = self.cache_dir.get() {
            if bytes != 0 {
                let dir = dir.clone();
                tauri::async_runtime::spawn_blocking(move || {
                    image_cache::enforce_limit(&dir, bytes)
                });
            }
        }
    }

    fn maybe_sweep_cache(&self) {
        let Some(dir) = self.cache_dir.get() else {
            return;
        };
        let limit = self.cache_limit.load(Ordering::Relaxed);
        if limit == 0 {
            return;
        }
        let n = self.writes_since_sweep.fetch_add(1, Ordering::Relaxed) + 1;
        if n < CACHE_SWEEP_EVERY {
            return;
        }
        self.writes_since_sweep.store(0, Ordering::Relaxed);
        let dir = dir.clone();
        tauri::async_runtime::spawn_blocking(move || image_cache::enforce_limit(&dir, limit));
    }

    fn downloads_guard(&self) -> std::sync::MutexGuard<'_, HashMap<i64, Arc<AtomicBool>>> {
        self.downloads.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn register_download(&self, id: i64) -> AppResult<Arc<AtomicBool>> {
        let mut downloads = self.downloads_guard();
        if downloads.contains_key(&id) {
            return Err(AppError::Other("download already running".into()));
        }
        let flag = Arc::new(AtomicBool::new(false));
        downloads.insert(id, flag.clone());
        Ok(flag)
    }

    fn finish_download(&self, id: i64) {
        self.downloads_guard().remove(&id);
    }

    pub fn cancel_download(&self, id: i64) {
        if let Some(flag) = self.downloads_guard().get(&id) {
            flag.store(true, Ordering::SeqCst);
        }
    }

    pub fn set_cache_dir(&self, dir: PathBuf) {
        let _ = self.cache_dir.set(dir);
    }

    pub fn clear_cache(&self) {
        if let Some(dir) = self.cache_dir.get() {
            image_cache::clear(dir);
        }
    }

    pub fn cache_size(&self) -> u64 {
        self.cache_dir
            .get()
            .map(|d| image_cache::size(d))
            .unwrap_or(0)
    }

    pub async fn gg_data(&self) -> Arc<GgData> {
        {
            let guard = self.gg.read().await;
            if let Some(c) = guard.as_ref() {
                if c.fetched_at.elapsed() < GG_TTL {
                    return c.data.clone();
                }
            }
        }

        let _gate = self.gg_refresh.lock().await;

        {
            let guard = self.gg.read().await;
            if let Some(c) = guard.as_ref() {
                if c.fetched_at.elapsed() < GG_TTL {
                    return c.data.clone();
                }
            }
        }

        match gg::fetch(&self.http).await {
            Ok(fresh) => {
                let data = Arc::new(fresh);
                let mut guard = self.gg.write().await;
                *guard = Some(GgCache {
                    data: data.clone(),
                    fetched_at: Instant::now(),
                });
                data
            }
            Err(_) => {
                let mut guard = self.gg.write().await;
                match guard.as_mut() {
                    Some(c) => {
                        c.fetched_at = Instant::now();
                        c.data.clone()
                    }
                    None => {
                        let data = Arc::new(gg::fallback());
                        *guard = Some(GgCache {
                            data: data.clone(),
                            fetched_at: Instant::now(),
                        });
                        data
                    }
                }
            }
        }
    }

    pub async fn image_urls(
        &self,
        hash: &str,
        is_thumbnail: bool,
        formats: &[&str],
    ) -> Vec<String> {
        if !super::is_valid_hash(hash) {
            return Vec::new();
        }
        let gg = self.gg_data().await;
        url_gen::generate_all_urls(hash, is_thumbnail, &gg, formats)
    }

    pub async fn fetch_gallery_info(&self, id: i64) -> AppResult<Gallery> {
        let url = format!("https://{}/galleries/{}.js", config::LTN_DOMAIN, id);
        let resp = self
            .http
            .get(&url)
            .headers(http::api_headers())
            .send()
            .await?;
        let status = resp.status();
        if status.as_u16() != 200 {
            return Err(AppError::Http {
                status: status.as_u16(),
                url,
            });
        }

        let text = resp.text().await?;
        let mut json = text.trim();
        if json.starts_with('<') {
            return Err(AppError::Decode(format!("html response for gallery {id}")));
        }
        for prefix in ["var galleryinfo = ", "galleryinfo = "] {
            if let Some(rest) = json.strip_prefix(prefix) {
                json = rest;
                break;
            }
        }
        json = json.trim();
        json = json.strip_suffix(';').unwrap_or(json).trim();

        if !json.starts_with('{') {
            return Err(AppError::Decode(format!("invalid json for gallery {id}")));
        }

        let value: serde_json::Value =
            serde_json::from_str(json).map_err(|e| AppError::Decode(e.to_string()))?;
        Gallery::from_json(&value)
    }

    pub async fn fetch_galleries(
        &self,
        page: usize,
        gtype: GalleryType,
        sort: SortOrder,
        language: &str,
    ) -> AppResult<GalleryPage> {
        let lang_specific = !language.is_empty() && language != "all";
        if sort == SortOrder::Latest && gtype != GalleryType::All && lang_specific {
            let constraints = vec![
                format!("type:{}", gtype.slug()),
                format!("language:{language}"),
            ];
            let ids = unique_sorted_desc(self.fetch_ids_for_constraints(constraints).await?);
            return Ok(self.paginate_ids(ids, page).await);
        }

        if let Some(period) = sort.popular_period() {
            if gtype != GalleryType::All {
                let lang = if language.is_empty() { "all" } else { language };
                let popular_url = format!(
                    "https://{}/popular/{}-{}.nozomi",
                    config::LTN_DOMAIN,
                    period,
                    lang
                );
                let type_url = nozomi_url_from_args("type", gtype.slug(), "all");
                let popular_ids = fetch_all_gallery_ids(&self.http, &popular_url).await?;
                let type_set: HashSet<i64> = fetch_all_gallery_ids(&self.http, &type_url)
                    .await?
                    .into_iter()
                    .collect();
                let ids: Vec<i64> = popular_ids
                    .into_iter()
                    .filter(|id| type_set.contains(id))
                    .collect();
                return Ok(self.paginate_ids(unique_preserve_order(ids), page).await);
            }
        }

        let urls = build_browse_nozomi_urls(gtype, sort, language);
        let mut last_err = AppError::Other("all nozomi endpoints failed".into());

        for url in &urls {
            match fetch_gallery_ids(&self.http, url, page, config::PAGE_SIZE).await {
                Ok((ids, total)) => {
                    if ids.is_empty() && total.is_none() {
                        continue;
                    }
                    let ordered = unique_preserve_order(ids);
                    let items = self.load_galleries_in_parallel(ordered).await;
                    let total = total.unwrap_or(items.len());
                    let total_pages = total.div_ceil(config::PAGE_SIZE).max(1);
                    return Ok(GalleryPage {
                        items,
                        total,
                        total_pages,
                        page,
                    });
                }
                Err(e) => last_err = e,
            }
        }
        Err(last_err)
    }

    pub async fn search_galleries(
        &self,
        query: &str,
        page: usize,
        language: &str,
    ) -> AppResult<GalleryPage> {
        let ids = self.resolve_search_ids(query, language).await?;
        Ok(self.paginate_ids(ids, page).await)
    }

    async fn paginate_ids(&self, ids: Vec<i64>, page: usize) -> GalleryPage {
        let total = ids.len();
        let total_pages = total.div_ceil(config::PAGE_SIZE).max(1);

        let start = page.saturating_sub(1) * config::PAGE_SIZE;
        let items = if start >= total {
            Vec::new()
        } else {
            let end = (start + config::PAGE_SIZE).min(total);
            self.load_galleries_in_parallel(ids[start..end].to_vec())
                .await
        };

        GalleryPage {
            items,
            total,
            total_pages,
            page,
        }
    }

    async fn resolve_search_ids(&self, query: &str, language: &str) -> AppResult<Vec<i64>> {
        let mut terms = parse_query(query);
        if terms.language.is_none() && !language.is_empty() && language != "all" {
            terms.language = Some(language.to_string());
        }
        let constraints = build_constraints(&terms);

        let ids = if !constraints.is_empty() {
            self.fetch_ids_for_constraints(constraints).await?
        } else {
            let url = nozomi_url_from_args("all", "index", "all");
            fetch_all_gallery_ids(&self.http, &url).await?
        };

        Ok(unique_sorted_desc(ids))
    }

    async fn fetch_ids_for_constraints(&self, constraints: Vec<String>) -> AppResult<Vec<i64>> {
        let futures = constraints.iter().map(|c| {
            let client = &self.http;
            async move {
                match constraint_to_nozomi_url(c) {
                    Some(url) => fetch_all_gallery_ids(client, &url).await,
                    None => Ok(Vec::new()),
                }
            }
        });

        let results: Vec<Vec<i64>> = join_all(futures)
            .await
            .into_iter()
            .collect::<AppResult<Vec<_>>>()?;
        let mut sets = results
            .into_iter()
            .map(|v| v.into_iter().collect::<HashSet<i64>>());

        let Some(mut acc) = sets.next() else {
            return Ok(Vec::new());
        };
        for s in sets {
            acc = acc.intersection(&s).copied().collect();
        }
        Ok(acc.into_iter().collect())
    }

    async fn load_galleries_in_parallel(&self, ids: Vec<i64>) -> Vec<Gallery> {
        let results: Vec<(i64, Option<Gallery>)> = stream::iter(ids.clone())
            .map(|id| async move { (id, self.fetch_gallery_info(id).await.ok()) })
            .buffer_unordered(config::MAX_CONCURRENT_GALLERY_INFO)
            .collect()
            .await;

        let map: HashMap<i64, Gallery> = results
            .into_iter()
            .filter_map(|(id, g)| g.map(|g| (id, g)))
            .collect();

        ids.iter().filter_map(|id| map.get(id).cloned()).collect()
    }

    pub async fn tag_suggestions(&self, query: &str) -> AppResult<Vec<Suggestion>> {
        let last = query
            .split(' ')
            .next_back()
            .unwrap_or("")
            .trim()
            .to_string();
        if last.is_empty() {
            return Ok(Vec::new());
        }

        const ALLOWED: [&str; 10] = [
            "global",
            "tag",
            "female",
            "male",
            "language",
            "artist",
            "group",
            "series",
            "character",
            "type",
        ];
        let mut field = "global";
        let mut term = last.as_str();
        if let Some((cand, rest)) = last.split_once(':') {
            if ALLOWED.contains(&cand) {
                field = cand;
                term = rest;
            }
        }

        let term = term.to_lowercase();
        let encoded: Vec<String> = term.chars().map(encode_suggest_char).collect();
        let path = if encoded.is_empty() {
            format!("/{field}.json")
        } else {
            format!("/{field}/{}.json", encoded.join("/"))
        };
        let url = format!("https://tagindex.hitomi.la{path}");

        let resp = self
            .http
            .get(&url)
            .headers(http::suggestions_headers())
            .send()
            .await?;
        let status = resp.status().as_u16();
        if status == 404 {
            return Ok(Vec::new());
        }
        if status != 200 {
            return Err(AppError::Http { status, url });
        }

        let rows: Vec<Vec<serde_json::Value>> = resp.json().await.unwrap_or_default();
        let mut out = Vec::new();
        for row in rows {
            if row.len() < 3 {
                continue;
            }
            let label = row[0].as_str().unwrap_or("").to_string();
            let count = row[1].as_i64().unwrap_or(0);
            let namespace = row[2].as_str().unwrap_or("").to_string();
            if label.is_empty() {
                continue;
            }
            let value = suggestion_value(&namespace, &label);
            out.push(Suggestion {
                label,
                value,
                count,
                namespace,
            });
        }
        Ok(out)
    }

    pub async fn fetch_image_bytes(
        &self,
        hash: &str,
        is_thumbnail: bool,
    ) -> AppResult<(Vec<u8>, String)> {
        if !super::is_valid_hash(hash) {
            return Err(AppError::NotFound("invalid image hash".into()));
        }
        if let Some(dir) = self.cache_dir.get() {
            if let Some(hit) = image_cache::read(dir, hash, is_thumbnail) {
                return Ok(hit);
            }
        }

        let formats: &[&str] = if is_thumbnail {
            &["webp"]
        } else {
            &["webp", "avif"]
        };
        let urls = self.image_urls(hash, is_thumbnail, formats).await;
        let mut last_err: Option<AppError> = None;

        for url in urls {
            match self.try_fetch_image(&url).await {
                Ok((bytes, ct)) => {
                    if let Some(dir) = self.cache_dir.get() {
                        image_cache::write(dir, hash, is_thumbnail, &bytes, &ct);
                        self.maybe_sweep_cache();
                    }
                    return Ok((bytes, ct));
                }
                Err(e) => last_err = Some(e),
            }
        }
        Err(last_err.unwrap_or_else(|| AppError::NotFound("no image url candidates".into())))
    }

    pub async fn download_gallery(
        self: Arc<Self>,
        app: AppHandle,
        id: i64,
        dir: String,
        pages: Option<Vec<usize>>,
    ) -> AppResult<DownloadResult> {
        let gallery = self.fetch_gallery_info(id).await?;
        let folder = PathBuf::from(&dir).join(sanitize(&format!("[{}] {}", id, gallery.title)));
        std::fs::create_dir_all(&folder)?;

        let cancel = self.register_download(id)?;
        let total = gallery.files.len();
        let done = Arc::new(AtomicUsize::new(0));
        let failed = Arc::new(AtomicUsize::new(0));
        let skipped = Arc::new(AtomicUsize::new(0));
        let failed_pages = Arc::new(Mutex::new(Vec::<usize>::new()));
        let _ = app.emit(
            "download-progress",
            serde_json::json!({ "id": id, "done": 0, "total": total }),
        );

        let requested: Option<HashSet<usize>> = pages.map(|items| {
            items
                .into_iter()
                .filter(|page| *page > 0 && *page <= total)
                .map(|page| page - 1)
                .collect()
        });

        let mut items: Vec<(usize, String)> = Vec::new();
        for (i, file) in gallery.files.iter().enumerate() {
            if requested.as_ref().is_some_and(|set| !set.contains(&i)) {
                continue;
            }
            items.push((i, file.hash.clone()));
        }

        let tasks = items.into_iter().map(|(i, hash)| {
            let me = self.clone();
            let folder = folder.clone();
            let done = done.clone();
            let failed = failed.clone();
            let skipped = skipped.clone();
            let failed_pages = failed_pages.clone();
            let cancel = cancel.clone();
            let app = app.clone();
            async move {
                if cancel.load(Ordering::SeqCst) {
                    return;
                }
                if existing_page(&folder, i).is_some() {
                    skipped.fetch_add(1, Ordering::SeqCst);
                    let d = done.fetch_add(1, Ordering::SeqCst) + 1;
                    let _ = app.emit(
                        "download-progress",
                        serde_json::json!({ "id": id, "done": d, "total": total }),
                    );
                    return;
                }
                let ok = match me.fetch_image_bytes(&hash, false).await {
                    Ok((bytes, ct)) => {
                        let ext = image_cache::ext_from_content_type(&ct);
                        let path = folder.join(format!("{:04}.{}", i + 1, ext));
                        let tmp = folder.join(format!("{:04}.{}.part", i + 1, ext));
                        std::fs::write(&tmp, &bytes)
                            .and_then(|_| std::fs::rename(&tmp, &path))
                            .is_ok()
                    }
                    Err(_) => false,
                };
                if !ok {
                    failed.fetch_add(1, Ordering::SeqCst);
                    if let Ok(mut pages) = failed_pages.lock() {
                        pages.push(i + 1);
                    }
                }
                let d = done.fetch_add(1, Ordering::SeqCst) + 1;
                let _ = app.emit(
                    "download-progress",
                    serde_json::json!({ "id": id, "done": d, "total": total }),
                );
            }
        });

        stream::iter(tasks)
            .buffer_unordered(4)
            .collect::<Vec<()>>()
            .await;
        self.finish_download(id);

        let failed = failed.load(Ordering::SeqCst);
        let done = done.load(Ordering::SeqCst);
        let skipped = skipped.load(Ordering::SeqCst);
        let mut failed_pages = failed_pages.lock().map(|p| p.clone()).unwrap_or_default();
        failed_pages.sort_unstable();
        let folder_str = folder.to_string_lossy().to_string();

        if cancel.load(Ordering::SeqCst) {
            let _ = app.emit(
                "download-cancelled",
                serde_json::json!({ "id": id, "done": done, "total": total }),
            );
        } else {
            let _ = app.emit(
                "download-done",
                serde_json::json!({
                    "id": id,
                    "folder": folder_str,
                    "total": total,
                    "failed": failed,
                    "failedPages": failed_pages,
                    "skipped": skipped
                }),
            );
        }
        Ok(DownloadResult {
            id,
            gallery,
            folder: folder_str,
            done,
            total,
            failed,
            failed_pages,
            skipped,
        })
    }

    async fn try_fetch_image(&self, url: &str) -> AppResult<(Vec<u8>, String)> {
        let resp = self
            .http
            .get(url)
            .headers(http::image_headers())
            .send()
            .await?;
        let status = resp.status();
        if status.as_u16() != 200 {
            return Err(AppError::Http {
                status: status.as_u16(),
                url: url.to_string(),
            });
        }
        if let Some(len) = resp.content_length() {
            if len > config::MAX_IMAGE_BYTES {
                return Err(AppError::Other("image too large".into()));
            }
        }
        let content_type = resp
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("image/webp")
            .to_string();
        if !content_type.starts_with("image/") {
            return Err(AppError::Other("response is not an image".into()));
        }
        let mut stream = resp.bytes_stream();
        let mut out = Vec::new();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            let next_len = out.len().saturating_add(chunk.len());
            if next_len as u64 > config::MAX_IMAGE_BYTES {
                return Err(AppError::Other("image too large".into()));
            }
            out.extend_from_slice(&chunk);
        }
        if out.is_empty() {
            return Err(AppError::Other("empty image response".into()));
        }
        Ok((out, content_type))
    }
}

fn unique_sorted_desc(ids: Vec<i64>) -> Vec<i64> {
    let set: HashSet<i64> = ids.into_iter().collect();
    let mut v: Vec<i64> = set.into_iter().collect();
    v.sort_unstable_by(|a, b| b.cmp(a));
    v
}

fn unique_preserve_order(ids: Vec<i64>) -> Vec<i64> {
    let mut seen = HashSet::new();
    ids.into_iter().filter(|id| seen.insert(*id)).collect()
}

fn encode_suggest_char(c: char) -> String {
    match c {
        ' ' => "_".to_string(),
        '/' => "slash".to_string(),
        '.' => "dot".to_string(),
        c if c.is_ascii_alphanumeric() || c == '_' || c == '-' => c.to_string(),
        c => {
            let mut buf = [0u8; 4];
            urlencoding::encode(c.encode_utf8(&mut buf)).into_owned()
        }
    }
}

fn suggestion_value(namespace: &str, tagname: &str) -> String {
    let t = tagname.replace(' ', "_");
    match namespace {
        "female" | "male" | "language" | "artist" | "group" | "series" | "character" | "type" => {
            format!("{namespace}:{t}")
        }
        _ => format!("tag:{t}"),
    }
}

fn sanitize(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if (c as u32) < 0x20 => '_',
            c => c,
        })
        .collect();
    let trimmed = cleaned.trim().trim_matches('.').trim();
    let limited: String = trimmed.chars().take(150).collect();
    if limited.is_empty() {
        return "untitled".to_string();
    }
    if is_reserved_name(&limited) {
        format!("_{limited}")
    } else {
        limited
    }
}

fn is_reserved_name(name: &str) -> bool {
    const RESERVED: [&str; 22] = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    let stem = name.split('.').next().unwrap_or(name).to_ascii_uppercase();
    RESERVED.contains(&stem.as_str())
}

fn existing_page(folder: &std::path::Path, index: usize) -> Option<PathBuf> {
    const EXTS: [&str; 7] = ["webp", "avif", "jpg", "png", "gif", "jxl", "img"];
    for ext in EXTS {
        let path = folder.join(format!("{:04}.{}", index + 1, ext));
        if path
            .metadata()
            .map(|m| m.is_file() && m.len() > 0)
            .unwrap_or(false)
        {
            return Some(path);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn live_type_language() {
        let client = HitomiClient::new();
        let page = client
            .fetch_galleries(1, GalleryType::Anime, SortOrder::Latest, "korean")
            .await
            .expect("anime + korean browse failed");
        println!(
            "anime+korean: {} items, total={}",
            page.items.len(),
            page.total
        );
        assert!(
            page.items.iter().all(|g| g.gtype == "anime"),
            "all items should be anime"
        );
        assert!(
            page.items
                .iter()
                .all(|g| g.language.as_deref() == Some("korean")),
            "all items should be korean"
        );

        let popular = client
            .fetch_galleries(1, GalleryType::Manga, SortOrder::Week, "english")
            .await
            .expect("popular manga browse failed");
        println!("manga+week+english: {} items", popular.items.len());
        assert!(
            popular.items.iter().all(|g| g.gtype == "manga"),
            "all popular items should be manga"
        );
    }

    #[tokio::test]
    #[ignore]
    async fn live_smoke() {
        let client = HitomiClient::new();

        let gg = client.gg_data().await;
        println!("gg.b={:?} mappings={} default={}", gg.b, gg.m.len(), gg.o);
        assert!(!gg.m.is_empty(), "gg mappings should not be empty");

        let page = client
            .fetch_galleries(1, GalleryType::All, SortOrder::Latest, "english")
            .await
            .expect("fetch_galleries failed");
        println!(
            "page 1: {} items, total={}, totalPages={}",
            page.items.len(),
            page.total,
            page.total_pages
        );
        assert!(!page.items.is_empty(), "expected at least one gallery");
        assert!(page.total_pages > 1, "expected many pages");

        let first = &page.items[0];
        println!(
            "first: id={} type={} title={:?} pages={}",
            first.id, first.gtype, first.title, first.page_count
        );
        assert!(!first.files.is_empty(), "gallery should have files");

        let file = &first.files[0];
        let urls = client
            .image_urls(&file.hash, false, &["webp", "avif"])
            .await;
        println!("image urls for first page: {:#?}", urls);
        assert!(!urls.is_empty(), "expected image url candidates");

        let thumb = client.image_urls(&file.hash, true, &["webp"]).await;
        println!("thumbnail url: {:#?}", thumb);
        assert!(!thumb.is_empty(), "expected thumbnail url");

        let tmp = std::env::temp_dir().join("vista-cache-test");
        let _ = std::fs::remove_dir_all(&tmp);
        client.set_cache_dir(tmp.clone());
        let (b1, _) = client
            .fetch_image_bytes(&file.hash, true)
            .await
            .expect("first image fetch failed");
        let size_after_fetch = client.cache_size();
        let (b2, _) = client
            .fetch_image_bytes(&file.hash, true)
            .await
            .expect("cached image fetch failed");
        println!(
            "image bytes: first={} cached={} cache_size={}",
            b1.len(),
            b2.len(),
            size_after_fetch
        );
        assert_eq!(b1.len(), b2.len(), "cached bytes should match");
        assert!(
            size_after_fetch > 0,
            "cache should contain data after fetch"
        );
        let _ = std::fs::remove_dir_all(&tmp);

        let kr = client
            .fetch_galleries(1, GalleryType::All, SortOrder::Latest, "korean")
            .await
            .expect("korean browse failed");
        println!("korean page 1: {} items, langs:", kr.items.len());
        for g in kr.items.iter().take(5) {
            println!("  - lang={:?} title={}", g.language, g.title);
        }
        assert!(
            kr.items
                .iter()
                .all(|g| g.language.as_deref() == Some("korean")),
            "all korean browse items should be korean"
        );
    }

    #[tokio::test]
    #[ignore]
    async fn live_suggest() {
        let client = HitomiClient::new();
        let s = client
            .tag_suggestions("naru")
            .await
            .expect("suggest failed");
        println!("'naru' -> {} suggestions", s.len());
        for x in s.iter().take(5) {
            println!("  {} [{}] {} -> {}", x.label, x.namespace, x.count, x.value);
        }
        assert!(!s.is_empty(), "expected suggestions for 'naru'");

        let f = client
            .tag_suggestions("female:bon")
            .await
            .expect("suggest2 failed");
        println!("'female:bon' -> {:?}", f.first().map(|x| &x.value));
        assert!(f.iter().any(|x| x.namespace == "female"));
    }

    #[tokio::test]
    #[ignore]
    async fn live_search() {
        let client = HitomiClient::new();
        let result = client
            .search_galleries("male:furry", 3, "all")
            .await
            .expect("search failed");
        println!(
            "search male:furry page 3: {} items, total={}, totalPages={}",
            result.items.len(),
            result.total,
            result.total_pages
        );
        for g in result.items.iter().take(3) {
            println!("  - {} (lang={:?})", g.title, g.language);
        }
        assert!(result.total > 0, "expected search matches");
    }
}
