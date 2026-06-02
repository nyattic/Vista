use super::config;
use super::gg::GgData;
use once_cell::sync::Lazy;
use regex::Regex;

static HASH_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"/[0-9a-f]{61}([0-9a-f]{2})([0-9a-f])").unwrap());
static SUBDOMAIN_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"//..?\.(?:gold-usergeneratedcontent\.net|hitomi\.la)/").unwrap()
});

pub fn generate_all_urls(
    hash: &str,
    is_thumbnail: bool,
    gg: &GgData,
    preferred_formats: &[&str],
) -> Vec<String> {
    if hash.len() < 3 {
        return Vec::new();
    }

    let mut urls: Vec<String> = Vec::new();

    if is_thumbnail {
        let real_path = real_full_path_from_hash(hash);
        let base_url = format!(
            "https://a.{}/webpsmalltn/{}.webp",
            config::CDN_DOMAIN,
            real_path
        );
        urls.push(url_from_url(&base_url, Some("tn"), Some("webpsmalltn"), gg));
    } else {
        let full_path = full_path_from_hash(hash, gg);
        for format in normalized_formats(preferred_formats) {
            let dir = image_directory(&format);
            let base_url = format!("https://a.{}/{}.{}", config::CDN_DOMAIN, full_path, format);
            urls.push(url_from_url(&base_url, None, dir, gg));
        }
    }

    dedup(urls)
}

fn real_full_path_from_hash(hash: &str) -> String {
    if hash.len() < 3 {
        return hash.to_string();
    }
    let last_three = &hash[hash.len() - 3..];
    let last_two = &last_three[..2];
    let third_from_end = &last_three[2..];
    format!("{third_from_end}/{last_two}/{hash}")
}

fn full_path_from_hash(hash: &str, gg: &GgData) -> String {
    let s = pupil_s(hash);
    let base = normalized_base_path(&gg.b);
    format!("{base}{s}/{hash}")
}

fn pupil_s(hash: &str) -> String {
    if hash.len() < 3 {
        return "0".to_string();
    }
    let last_three = &hash[hash.len() - 3..];
    let last_two = &last_three[..2];
    let third_from_end = &last_three[2..];
    let combined = format!("{third_from_end}{last_two}");
    i64::from_str_radix(&combined, 16)
        .map(|v| v.to_string())
        .unwrap_or_else(|_| "0".to_string())
}

fn url_from_url(base_url: &str, base: Option<&str>, dir: Option<&str>, gg: &GgData) -> String {
    let subdomain = subdomain_from_url(base_url, base, dir, gg);
    let replacement = format!("//{subdomain}.{}/", config::CDN_DOMAIN);
    SUBDOMAIN_RE
        .replace(base_url, replacement.as_str())
        .into_owned()
}

fn subdomain_from_url(url: &str, base: Option<&str>, dir: Option<&str>, gg: &GgData) -> String {
    let base_empty = base.map_or(true, |b| b.is_empty());
    let mut retval = String::new();

    if base_empty {
        if dir == Some("webp") {
            retval.push('w');
        } else if dir == Some("avif") {
            retval.push('a');
        }
    }

    if let Some(caps) = HASH_RE.captures(url) {
        let g1 = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let g2 = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        let combined = format!("{g2}{g1}");
        if let Ok(g) = i64::from_str_radix(&combined, 16) {
            let m_value = *gg.m.get(&g).unwrap_or(&gg.o);
            if base_empty {
                retval.push_str(&(1 + m_value).to_string());
            } else {
                let ch = char::from_u32((97 + m_value) as u32).unwrap_or('a');
                retval = format!("{ch}{}", base.unwrap_or(""));
            }
        }
    }

    if retval.is_empty() {
        "a".to_string()
    } else {
        retval
    }
}

fn normalized_base_path(value: &str) -> String {
    let trimmed = value.trim_matches('/');
    if trimmed.is_empty() {
        String::new()
    } else {
        format!("{trimmed}/")
    }
}

fn normalized_formats(formats: &[&str]) -> Vec<String> {
    let sanitized: Vec<String> = formats
        .iter()
        .map(|f| f.to_lowercase().trim_matches(['.', ' ', '\n', '\r', '\t']).to_string())
        .filter(|f| !f.is_empty())
        .collect();
    let with_default = if sanitized.is_empty() {
        vec!["webp".to_string(), "avif".to_string()]
    } else {
        sanitized
    };
    dedup(with_default)
}

fn image_directory(format: &str) -> Option<&'static str> {
    match format {
        "webp" => Some("webp"),
        "avif" => Some("avif"),
        _ => None,
    }
}

fn dedup(items: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    items.into_iter().filter(|s| seen.insert(s.clone())).collect()
}
