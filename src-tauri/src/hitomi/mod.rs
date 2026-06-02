pub mod client;
pub mod config;
pub mod gg;
pub mod http;
pub mod image_cache;
pub mod models;
pub mod nozomi;
pub mod search;
pub mod url_gen;

pub use client::HitomiClient;
pub use models::{Gallery, GalleryPage, GalleryType, SortOrder, Suggestion};

pub fn is_valid_hash(hash: &str) -> bool {
    let len = hash.len();
    (3..=64).contains(&len) && hash.bytes().all(|b| b.is_ascii_hexdigit())
}
