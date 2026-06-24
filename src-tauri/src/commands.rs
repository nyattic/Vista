use crate::db::{Db, DownloadRecord, Progress};
use crate::error::{AppError, AppResult};
use crate::hitomi::client::existing_page;
use crate::hitomi::{
    is_valid_hash, Gallery, GalleryPage, GalleryType, HitomiClient, SortOrder, Suggestion,
};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};

const MAX_QUERY_CHARS: usize = 300;
const MAX_QUERY_TERMS: usize = 12;
const MAX_SUGGEST_CHARS: usize = 80;
const MAX_CACHE_LIMIT_BYTES: u64 = 20 * 1024 * 1024 * 1024;
const MIN_PAGE_SIZE: usize = 12;
const MAX_PAGE_SIZE: usize = 80;

fn validate_id(id: i64) -> AppResult<i64> {
    if id <= 0 {
        return Err(AppError::Other("invalid gallery id".into()));
    }
    Ok(id)
}

fn validate_page(page: usize) -> usize {
    page.clamp(1, 100_000)
}

fn validate_page_size(page_size: usize) -> usize {
    page_size.clamp(MIN_PAGE_SIZE, MAX_PAGE_SIZE)
}

fn validate_language(language: &str) -> AppResult<()> {
    match language {
        "" | "all" | "korean" | "english" | "japanese" | "chinese" => Ok(()),
        _ => Err(AppError::Other("invalid language".into())),
    }
}

fn validate_query(query: &str) -> AppResult<()> {
    if query.chars().count() > MAX_QUERY_CHARS {
        return Err(AppError::Other("query is too long".into()));
    }
    if query.split_whitespace().count() > MAX_QUERY_TERMS {
        return Err(AppError::Other("too many search terms".into()));
    }
    Ok(())
}

fn validate_gallery_payload(gallery: &Gallery) -> AppResult<()> {
    validate_id(gallery.id)?;
    if gallery.title.chars().count() > 500
        || gallery.gtype.chars().count() > 40
        || gallery.date.chars().count() > 80
        || gallery.files.len() > 1_500
        || gallery.tags.len() > 1_000
    {
        return Err(AppError::Other("gallery payload is too large".into()));
    }
    for value in gallery
        .artists
        .iter()
        .chain(gallery.groups.iter())
        .chain(gallery.series.iter())
        .chain(gallery.characters.iter())
        .chain(gallery.tags.iter())
    {
        if value.chars().count() > 200 {
            return Err(AppError::Other("gallery metadata is too large".into()));
        }
    }
    for file in &gallery.files {
        if file.name.chars().count() > 260 || !is_valid_hash(&file.hash) {
            return Err(AppError::Other("invalid gallery file".into()));
        }
    }
    Ok(())
}

fn validate_download_dir(dir: &str) -> AppResult<String> {
    if dir.is_empty() || dir.chars().count() > 4096 || dir.chars().any(char::is_control) {
        return Err(AppError::Other("invalid download folder".into()));
    }
    let path = PathBuf::from(dir);
    if !path.is_absolute() {
        return Err(AppError::Other("download folder must be absolute".into()));
    }
    if !path.is_dir() {
        return Err(AppError::Other("download folder does not exist".into()));
    }
    Ok(path.canonicalize()?.to_string_lossy().to_string())
}

fn with_local_paths(mut record: DownloadRecord) -> DownloadRecord {
    let folder = PathBuf::from(&record.folder);
    for (i, file) in record.gallery.files.iter_mut().enumerate() {
        file.local_path = existing_page(&folder, i).map(|p| p.to_string_lossy().to_string());
    }
    record
}

#[tauri::command]
pub async fn fetch_galleries(
    client: State<'_, Arc<HitomiClient>>,
    page: usize,
    gtype: String,
    sort: String,
    language: String,
    page_size: usize,
) -> AppResult<GalleryPage> {
    validate_language(&language)?;
    let page = validate_page(page);
    let page_size = validate_page_size(page_size);
    client
        .fetch_galleries(
            page,
            page_size,
            GalleryType::from_str(&gtype),
            SortOrder::from_str(&sort),
            &language,
        )
        .await
}

#[tauri::command]
pub async fn fetch_gallery(client: State<'_, Arc<HitomiClient>>, id: i64) -> AppResult<Gallery> {
    let id = validate_id(id)?;
    client.fetch_gallery_info(id).await
}

#[tauri::command]
pub async fn search_galleries(
    client: State<'_, Arc<HitomiClient>>,
    query: String,
    page: usize,
    sort: String,
    language: String,
    page_size: usize,
) -> AppResult<GalleryPage> {
    validate_language(&language)?;
    validate_query(&query)?;
    client
        .search_galleries(
            &query,
            validate_page(page),
            validate_page_size(page_size),
            SortOrder::from_str(&sort),
            &language,
        )
        .await
}

#[tauri::command]
pub async fn tag_suggestions(
    client: State<'_, Arc<HitomiClient>>,
    query: String,
) -> AppResult<Vec<Suggestion>> {
    if query.chars().count() > MAX_SUGGEST_CHARS {
        return Ok(Vec::new());
    }
    client.tag_suggestions(&query).await
}

#[tauri::command]
pub async fn download_gallery(
    app: AppHandle,
    client: State<'_, Arc<HitomiClient>>,
    db: State<'_, Db>,
    id: i64,
    dir: String,
    pages: Option<Vec<usize>>,
) -> AppResult<crate::hitomi::client::DownloadResult> {
    let id = validate_id(id)?;
    let dir = validate_download_dir(&dir)?;
    let retry_pages = pages.unwrap_or_default();
    if retry_pages.len() > 500 {
        return Err(AppError::Other("too many pages requested".into()));
    }
    let old_failed = db
        .download_record(id)?
        .map(|r| r.failed_pages)
        .unwrap_or_default();
    let client = client.inner().clone();
    let requested = if retry_pages.is_empty() {
        None
    } else {
        Some(retry_pages.clone())
    };
    let result = client.download_gallery(app, id, dir, requested).await?;
    let failed_pages = if retry_pages.is_empty() {
        result.failed_pages.clone()
    } else {
        let retry_set: std::collections::HashSet<usize> = retry_pages.into_iter().collect();
        let mut merged: Vec<usize> = old_failed
            .into_iter()
            .filter(|page| !retry_set.contains(page))
            .collect();
        merged.extend(result.failed_pages.iter().copied());
        merged.sort_unstable();
        merged.dedup();
        merged
    };
    db.upsert_download(&result.gallery, &result.folder, &failed_pages)?;
    Ok(result)
}

#[tauri::command]
pub fn cancel_download(client: State<'_, Arc<HitomiClient>>, id: i64) {
    if id <= 0 {
        return;
    }
    client.cancel_download(id);
}

#[tauri::command]
pub fn default_download_dir(app: AppHandle) -> Option<String> {
    app.path()
        .download_dir()
        .ok()
        .map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
pub fn clear_image_cache(client: State<'_, Arc<HitomiClient>>) -> AppResult<()> {
    client.clear_cache();
    Ok(())
}

#[tauri::command]
pub fn image_cache_size(client: State<'_, Arc<HitomiClient>>) -> u64 {
    client.cache_size()
}

#[tauri::command]
pub fn set_cache_limit(client: State<'_, Arc<HitomiClient>>, bytes: u64) {
    let bytes = bytes.min(MAX_CACHE_LIMIT_BYTES);
    client.set_cache_limit(bytes);
}

#[tauri::command]
pub fn toggle_favorite(db: State<'_, Db>, gallery: Gallery) -> AppResult<bool> {
    validate_gallery_payload(&gallery)?;
    db.toggle_favorite(&gallery)
}

#[tauri::command]
pub fn remove_favorite(db: State<'_, Db>, id: i64) -> AppResult<()> {
    let id = validate_id(id)?;
    db.remove_favorite(id)
}

#[tauri::command]
pub fn favorite_ids(db: State<'_, Db>) -> AppResult<Vec<i64>> {
    db.favorite_ids()
}

#[tauri::command]
pub fn list_favorites(db: State<'_, Db>) -> AppResult<Vec<Gallery>> {
    db.list_favorites()
}

#[tauri::command]
pub fn record_view(db: State<'_, Db>, gallery: Gallery) -> AppResult<()> {
    validate_gallery_payload(&gallery)?;
    db.record_view(&gallery)
}

#[tauri::command]
pub fn list_history(db: State<'_, Db>) -> AppResult<Vec<Gallery>> {
    db.list_history()
}

#[tauri::command]
pub fn remove_history(db: State<'_, Db>, id: i64) -> AppResult<()> {
    let id = validate_id(id)?;
    db.remove_history(id)
}

#[tauri::command]
pub fn clear_history(db: State<'_, Db>) -> AppResult<()> {
    db.clear_history()
}

#[tauri::command]
pub fn set_progress(db: State<'_, Db>, id: i64, page: i64, total: i64) -> AppResult<()> {
    let id = validate_id(id)?;
    if total <= 0 || page <= 0 || page > total {
        return Err(AppError::Other("invalid progress".into()));
    }
    db.set_progress(id, page, total)
}

#[tauri::command]
pub fn all_progress(db: State<'_, Db>) -> AppResult<Vec<Progress>> {
    db.all_progress()
}

#[tauri::command]
pub fn download_ids(db: State<'_, Db>) -> AppResult<Vec<i64>> {
    db.completed_download_ids()
}

#[tauri::command]
pub fn list_downloads(db: State<'_, Db>) -> AppResult<Vec<DownloadRecord>> {
    Ok(db
        .list_downloads_raw()?
        .into_iter()
        .map(with_local_paths)
        .collect())
}

#[tauri::command]
pub fn remove_download(db: State<'_, Db>, id: i64) -> AppResult<()> {
    let id = validate_id(id)?;
    db.remove_download(id)
}

#[tauri::command]
pub fn open_download_folder(db: State<'_, Db>, id: i64) -> AppResult<()> {
    let id = validate_id(id)?;
    let Some(record) = db.download_record(id)? else {
        return Err(AppError::NotFound("download record".into()));
    };
    let folder = PathBuf::from(record.folder);
    if !folder.is_dir() {
        return Err(AppError::NotFound("download folder".into()));
    }
    tauri_plugin_opener::open_path(folder, None::<&str>).map_err(|e| AppError::Other(e.to_string()))
}
