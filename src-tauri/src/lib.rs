mod commands;
mod db;
mod error;
mod hitomi;
mod image_protocol;

use db::Db;
use hitomi::HitomiClient;
use std::sync::Arc;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let client = Arc::new(HitomiClient::new());
    let proto_client = client.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(client)
        .register_asynchronous_uri_scheme_protocol("vimg", move |_ctx, request, responder| {
            image_protocol::handle(proto_client.clone(), request, responder);
        })
        .setup(|app| {
            if let Ok(cache) = app.path().app_cache_dir() {
                app.state::<Arc<HitomiClient>>()
                    .set_cache_dir(cache.join("images"));
            }
            if let Ok(data_dir) = app.path().app_data_dir() {
                std::fs::create_dir_all(&data_dir).ok();
                match Db::open(&data_dir.join("vista.db")) {
                    Ok(db) => {
                        app.manage(db);
                    }
                    Err(e) => log::error!("failed to open database: {e}"),
                }
            }
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::fetch_galleries,
            commands::fetch_gallery,
            commands::search_galleries,
            commands::tag_suggestions,
            commands::download_gallery,
            commands::cancel_download,
            commands::default_download_dir,
            commands::clear_image_cache,
            commands::image_cache_size,
            commands::set_cache_limit,
            commands::toggle_favorite,
            commands::remove_favorite,
            commands::favorite_ids,
            commands::list_favorites,
            commands::record_view,
            commands::list_history,
            commands::remove_history,
            commands::clear_history,
            commands::set_progress,
            commands::all_progress,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
