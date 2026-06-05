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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn gg() -> GgData {
        let mut m = HashMap::new();
        for i in 0..4096i64 {
            m.insert(i, (i % 3) + 1);
        }
        GgData {
            b: "1640642964".to_string(),
            m,
            o: 1,
        }
    }

    const HASH: &str = "8a7b6c5d4e3f2a1b0c9d8e7f6a5b4c3d2e1f0a9b8c7d6e5f4a3b2c1d0e9f8a7b";

    #[test]
    fn pupil_s_is_deterministic() {
        assert_eq!(pupil_s("abc"), 0xcab_i64.to_string());
        assert_eq!(pupil_s("ab"), "0");
    }

    #[test]
    fn real_full_path_splits_last_three() {
        assert_eq!(real_full_path_from_hash("abc"), "c/ab/abc");
    }

    #[test]
    fn normalized_base_path_trims_slashes() {
        assert_eq!(normalized_base_path("/b/"), "b/");
        assert_eq!(normalized_base_path(""), "");
        assert_eq!(normalized_base_path("//"), "");
    }

    #[test]
    fn normalized_formats_sanitizes_and_defaults() {
        assert_eq!(
            normalized_formats(&["WEBP", ".avif", " "]),
            vec!["webp".to_string(), "avif".to_string()]
        );
        assert_eq!(
            normalized_formats(&[]),
            vec!["webp".to_string(), "avif".to_string()]
        );
        assert_eq!(normalized_formats(&["webp", "webp"]), vec!["webp".to_string()]);
    }

    #[test]
    fn image_directory_known_only() {
        assert_eq!(image_directory("webp"), Some("webp"));
        assert_eq!(image_directory("avif"), Some("avif"));
        assert_eq!(image_directory("png"), None);
    }

    #[test]
    fn short_hash_yields_no_urls() {
        assert!(generate_all_urls("ab", false, &gg(), &["webp"]).is_empty());
    }

    #[test]
    fn full_image_urls_cover_requested_formats() {
        let urls = generate_all_urls(HASH, false, &gg(), &["webp", "avif"]);
        assert_eq!(urls.len(), 2);
        assert!(urls.iter().all(|u| u.starts_with("https://")));
        assert!(urls.iter().all(|u| u.contains(config::CDN_DOMAIN)));
        assert!(urls.iter().any(|u| u.ends_with(".webp")));
        assert!(urls.iter().any(|u| u.ends_with(".avif")));
    }

    #[test]
    fn thumbnail_url_uses_webpsmalltn() {
        let urls = generate_all_urls(HASH, true, &gg(), &["webp"]);
        assert_eq!(urls.len(), 1);
        assert!(urls[0].contains("webpsmalltn"));
        assert!(urls[0].ends_with(".webp"));
        assert!(urls[0].contains(config::CDN_DOMAIN));
    }
}
