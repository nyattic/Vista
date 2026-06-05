use super::{config, http};
use crate::error::{AppError, AppResult};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct GgData {
    pub b: String,
    pub m: HashMap<i64, i64>,
    pub o: i64,
}

static CASE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"case\s+(\d+):(?:\s*o\s*=\s*(\d+))?").unwrap());
static IF_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"if\s+\(g\s*===?\s*(\d+)\)[\s{]*o\s*=\s*(\d+)").unwrap());
static DEFAULT_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?:var\s|default:)\s*o\s*=\s*(\d+)").unwrap());
static B_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"b:\s*["'](.+)["']"#).unwrap());

pub fn fallback() -> GgData {
    let mut m = HashMap::with_capacity(4096);
    for i in 0..4096i64 {
        m.insert(i, (i % 3) + 1);
    }
    GgData {
        b: "b".to_string(),
        m,
        o: 1,
    }
}

pub fn parse(js: &str) -> Option<GgData> {
    let mut m: HashMap<i64, i64> = HashMap::new();
    let mut keys: Vec<i64> = Vec::new();

    for cap in CASE_RE.captures_iter(js) {
        if let Some(k) = cap.get(1).and_then(|x| x.as_str().parse::<i64>().ok()) {
            keys.push(k);
            if let Some(vm) = cap.get(2) {
                if !vm.as_str().is_empty() {
                    if let Ok(val) = vm.as_str().parse::<i64>() {
                        for kk in keys.drain(..) {
                            m.insert(kk, val);
                        }
                    }
                }
            }
        }
    }

    for cap in IF_RE.captures_iter(js) {
        if let (Some(k), Some(val)) = (
            cap.get(1).and_then(|x| x.as_str().parse::<i64>().ok()),
            cap.get(2).and_then(|x| x.as_str().parse::<i64>().ok()),
        ) {
            m.insert(k, val);
        }
    }

    let o = DEFAULT_RE
        .captures(js)
        .and_then(|c| c.get(1))
        .and_then(|x| x.as_str().parse::<i64>().ok())
        .unwrap_or(0);

    let b = B_RE
        .captures(js)
        .and_then(|c| c.get(1))
        .map(|x| x.as_str().trim_matches('/').to_string())
        .unwrap_or_default();

    Some(GgData { b, m, o })
}

pub async fn fetch(client: &reqwest::Client) -> AppResult<GgData> {
    let url = format!("https://{}/gg.js", config::LTN_DOMAIN);
    let resp = client.get(&url).headers(http::api_headers()).send().await?;
    let text = resp.text().await?;
    parse(&text)
        .filter(|g| !g.m.is_empty())
        .ok_or_else(|| AppError::Decode("failed to parse gg.js".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "var o = 0;\nfunction f(g){switch(g){case 0:\ncase 1:\no = 1;\nbreak;\ncase 2:\no = 2;\nbreak;}\nif (g === 5) o = 3;}\nb: \"1640642964/\"";

    #[test]
    fn parses_cases_default_if_and_b() {
        let g = parse(SAMPLE).expect("parse");
        assert_eq!(g.b, "1640642964");
        assert_eq!(g.o, 0);
        assert_eq!(g.m.get(&0), Some(&1));
        assert_eq!(g.m.get(&1), Some(&1));
        assert_eq!(g.m.get(&2), Some(&2));
        assert_eq!(g.m.get(&5), Some(&3));
    }

    #[test]
    fn default_without_spaces_is_parsed() {
        let g = parse("var o=7;").expect("parse");
        assert_eq!(g.o, 7);
    }

    #[test]
    fn fallback_has_full_mapping() {
        let f = fallback();
        assert_eq!(f.m.len(), 4096);
        assert!(f.m.contains_key(&0));
        assert!(f.m.contains_key(&4095));
    }
}
