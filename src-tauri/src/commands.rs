use crate::db::{Db, Progress};
use crate::error::AppResult;
use crate::hitomi::{Gallery, GalleryPage, GalleryType, HitomiClient, SortOrder, Suggestion};
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};

#[tauri::command]
pub async fn fetch_galleries(
    client: State<'_, Arc<HitomiClient>>,
    page: usize,
    gtype: String,
    sort: String,
    language: String,
) -> AppResult<GalleryPage> {
    client
        .fetch_galleries(
            page,
            GalleryType::from_str(&gtype),
            SortOrder::from_str(&sort),
            &language,
        )
        .await
}

#[tauri::command]
pub async fn fetch_gallery(
    client: State<'_, Arc<HitomiClient>>,
    id: i64,
) -> AppResult<Gallery> {
    client.fetch_gallery_info(id).await
}

#[tauri::command]
pub async fn search_galleries(
    client: State<'_, Arc<HitomiClient>>,
    query: String,
    page: usize,
    language: String,
) -> AppResult<GalleryPage> {
    client.search_galleries(&query, page, &language).await
}

#[tauri::command]
pub async fn tag_suggestions(
    client: State<'_, Arc<HitomiClient>>,
    query: String,
) -> AppResult<Vec<Suggestion>> {
    client.tag_suggestions(&query).await
}

#[tauri::command]
pub async fn download_gallery(
    app: AppHandle,
    client: State<'_, Arc<HitomiClient>>,
    id: i64,
    dir: String,
) -> AppResult<String> {
    let client = client.inner().clone();
    client.download_gallery(app, id, dir).await
}

#[tauri::command]
pub fn cancel_download(client: State<'_, Arc<HitomiClient>>, id: i64) {
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
    client.set_cache_limit(bytes);
}

#[tauri::command]
pub fn toggle_favorite(db: State<'_, Db>, gallery: Gallery) -> AppResult<bool> {
    db.toggle_favorite(&gallery)
}

#[tauri::command]
pub fn remove_favorite(db: State<'_, Db>, id: i64) -> AppResult<()> {
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
    db.record_view(&gallery)
}

#[tauri::command]
pub fn list_history(db: State<'_, Db>) -> AppResult<Vec<Gallery>> {
    db.list_history()
}

#[tauri::command]
pub fn remove_history(db: State<'_, Db>, id: i64) -> AppResult<()> {
    db.remove_history(id)
}

#[tauri::command]
pub fn clear_history(db: State<'_, Db>) -> AppResult<()> {
    db.clear_history()
}

#[tauri::command]
pub fn set_progress(db: State<'_, Db>, id: i64, page: i64, total: i64) -> AppResult<()> {
    db.set_progress(id, page, total)
}

#[tauri::command]
pub fn all_progress(db: State<'_, Db>) -> AppResult<Vec<Progress>> {
    db.all_progress()
}
