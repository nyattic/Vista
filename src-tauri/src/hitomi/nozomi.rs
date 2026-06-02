use super::models::{GalleryType, SortOrder};
use super::{config, http};
use crate::error::{AppError, AppResult};
use reqwest::header::CONTENT_RANGE;

pub fn parse_nozomi_data(data: &[u8]) -> Vec<i64> {
    let count = data.len() / 4;
    let mut ids = Vec::with_capacity(count);
    for chunk in data.chunks_exact(4) {
        let id = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        ids.push(id as i64);
    }
    ids
}

fn encode(value: &str) -> String {
    urlencoding::encode(value).into_owned()
}

pub fn nozomi_url_from_args(area: &str, tag: &str, language: &str) -> String {
    if area == "all" {
        format!(
            "https://{}/n/{}-{}.nozomi",
            config::LTN_DOMAIN,
            encode(tag),
            encode(language)
        )
    } else {
        format!(
            "https://{}/n/{}/{}-{}.nozomi",
            config::LTN_DOMAIN,
            encode(area),
            encode(tag),
            encode(language)
        )
    }
}

pub fn build_browse_nozomi_urls(
    gtype: GalleryType,
    sort: SortOrder,
    language: &str,
) -> Vec<String> {
    let lang = if language.is_empty() { "all" } else { language };

    if let Some(period) = sort.popular_period() {
        return vec![format!(
            "https://{}/popular/{}-{}.nozomi",
            config::LTN_DOMAIN,
            period,
            lang
        )];
    }

    let base = format!("https://{}/n", config::LTN_DOMAIN);
    if gtype == GalleryType::All {
        vec![format!("{base}/index-{lang}.nozomi")]
    } else {
        vec![
            format!("{base}/type/{}-{}.nozomi", gtype.slug(), lang),
            format!("{base}/index-{lang}.nozomi"),
        ]
    }
}

fn parse_total_count(content_range: &str) -> Option<usize> {
    content_range
        .rsplit('/')
        .next()?
        .trim()
        .parse::<usize>()
        .ok()
        .map(|bytes| bytes / config::BYTES_PER_ID)
}

pub async fn fetch_gallery_ids(
    client: &reqwest::Client,
    url: &str,
    page: usize,
    page_size: usize,
) -> AppResult<(Vec<i64>, Option<usize>)> {
    if page == 0 || page_size == 0 {
        return Ok((Vec::new(), None));
    }

    let start = (page - 1) * page_size * config::BYTES_PER_ID;
    let requested = page_size * config::BYTES_PER_ID;
    let end = start + requested - 1;

    let resp = client
        .get(url)
        .headers(http::nozomi_headers(Some((start, end))))
        .send()
        .await?;

    let status = resp.status();
    if status.as_u16() != 200 && status.as_u16() != 206 {
        return Err(AppError::Http {
            status: status.as_u16(),
            url: url.to_string(),
        });
    }

    let total = resp
        .headers()
        .get(CONTENT_RANGE)
        .and_then(|v| v.to_str().ok())
        .and_then(parse_total_count);

    let data = resp.bytes().await?;
    if status.as_u16() == 200 && data.len() > requested {
        return Err(AppError::Other(format!(
            "nozomi endpoint ignored range request: {url}"
        )));
    }

    Ok((parse_nozomi_data(&data), total))
}

pub async fn fetch_all_gallery_ids(client: &reqwest::Client, url: &str) -> AppResult<Vec<i64>> {
    let resp = client
        .get(url)
        .headers(http::nozomi_headers(None))
        .send()
        .await?;

    if resp.status().as_u16() != 200 {
        return Ok(Vec::new());
    }

    if let Some(len) = resp.content_length() {
        if len > config::MAX_NOZOMI_INDEX_BYTES {
            return Err(AppError::Other("nozomi index too large".into()));
        }
    }

    let data = resp.bytes().await?;
    if data.is_empty() {
        return Ok(Vec::new());
    }
    if data.len() as u64 > config::MAX_NOZOMI_INDEX_BYTES {
        return Err(AppError::Other("nozomi index exceeded size limit".into()));
    }

    Ok(parse_nozomi_data(&data))
}
