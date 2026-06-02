use crate::hitomi::HitomiClient;
use std::sync::Arc;
use tauri::http::{header, Request, Response, StatusCode};
use tauri::UriSchemeResponder;

pub fn handle(client: Arc<HitomiClient>, request: Request<Vec<u8>>, responder: UriSchemeResponder) {
    tauri::async_runtime::spawn(async move {
        let uri = request.uri();
        let raw = uri.path().trim_start_matches('/').to_string();
        let is_thumb = uri
            .query()
            .map(|q| q.split('&').any(|kv| kv == "thumb=1"))
            .unwrap_or(false);
        let hash = urlencoding::decode(&raw)
            .map(|c| c.into_owned())
            .unwrap_or(raw);

        if !crate::hitomi::is_valid_hash(&hash) {
            responder.respond(
                Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(Vec::new())
                    .unwrap_or_else(|_| Response::new(Vec::new())),
            );
            return;
        }

        let response = match client.fetch_image_bytes(&hash, is_thumb).await {
            Ok((bytes, content_type)) => Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, content_type)
                .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                .header(header::CACHE_CONTROL, "public, max-age=86400")
                .body(bytes)
                .unwrap_or_else(|_| Response::new(Vec::new())),
            Err(_) => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Vec::new())
                .unwrap_or_else(|_| Response::new(Vec::new())),
        };

        responder.respond(response);
    });
}
