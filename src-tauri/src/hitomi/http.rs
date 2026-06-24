use super::config;
use reqwest::header::{
    HeaderMap, HeaderValue, ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, ORIGIN, RANGE, REFERER,
    USER_AGENT,
};
use std::time::Duration;

pub fn build_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(config::USER_AGENT)
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(15))
        .pool_max_idle_per_host(10)
        .build()
        .expect("failed to build reqwest client")
}

fn base_headers() -> HeaderMap {
    let mut h = HeaderMap::new();
    h.insert(REFERER, HeaderValue::from_static(config::REFERER));
    h.insert(ORIGIN, HeaderValue::from_static(config::ORIGIN));
    h.insert(USER_AGENT, HeaderValue::from_static(config::USER_AGENT));
    h
}

pub fn api_headers() -> HeaderMap {
    let mut h = base_headers();
    h.insert(
        ACCEPT,
        HeaderValue::from_static("application/javascript, text/javascript, */*; q=0.01"),
    );
    h
}

pub fn nozomi_headers(range: Option<(usize, usize)>) -> HeaderMap {
    let mut h = base_headers();
    h.insert(ACCEPT, HeaderValue::from_static("application/octet-stream"));
    h.insert(ACCEPT_ENCODING, HeaderValue::from_static("identity"));
    if let Some((start, end)) = range {
        if let Ok(v) = HeaderValue::from_str(&format!("bytes={start}-{end}")) {
            h.insert(RANGE, v);
        }
    }
    h
}

pub fn suggestions_headers() -> HeaderMap {
    let mut h = base_headers();
    h.insert(
        ACCEPT,
        HeaderValue::from_static("application/json, text/plain, */*"),
    );
    h
}

pub fn image_headers() -> HeaderMap {
    let mut h = base_headers();
    h.insert(
        ACCEPT,
        HeaderValue::from_static("image/avif,image/webp,image/apng,image/*,*/*;q=0.8"),
    );
    h.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));
    h
}
