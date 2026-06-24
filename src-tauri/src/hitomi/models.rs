use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Gallery {
    pub id: i64,
    pub title: String,
    #[serde(rename = "type")]
    pub gtype: String,
    pub language: Option<String>,
    pub artists: Vec<String>,
    pub groups: Vec<String>,
    pub series: Vec<String>,
    pub characters: Vec<String>,
    pub tags: Vec<String>,
    pub date: String,
    pub files: Vec<GalleryFile>,
    pub page_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GalleryFile {
    pub name: String,
    pub hash: String,
    pub width: i64,
    pub height: i64,
    pub haswebp: i64,
    pub hasavif: i64,
    pub hasavifsmalltn: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_path: Option<String>,
}

impl Gallery {
    pub fn from_json(v: &Value) -> AppResult<Gallery> {
        let obj = v
            .as_object()
            .ok_or_else(|| AppError::Decode("galleryinfo is not an object".into()))?;

        let id =
            flexible_int(obj.get("id")).ok_or_else(|| AppError::Decode("missing id".into()))?;
        if id <= 0 {
            return Err(AppError::Decode(format!("invalid gallery id: {id}")));
        }

        let title = obj
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or("Unknown Gallery")
            .to_string();
        let gtype = obj
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let language = obj
            .get("language")
            .and_then(Value::as_str)
            .map(str::to_string);
        let date = obj
            .get("date")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();

        let files: Vec<GalleryFile> = obj
            .get("files")
            .and_then(Value::as_array)
            .map(|arr| arr.iter().filter_map(parse_file).collect())
            .unwrap_or_default();

        let artists = name_list(obj, "artists", "artist");
        let groups = name_list(obj, "groups", "group");
        let series = name_list(obj, "parodys", "parody");
        let characters = name_list(obj, "characters", "character");
        let tags = parse_tags(obj);

        let page_count = files.len();
        Ok(Gallery {
            id,
            title,
            gtype,
            language,
            artists,
            groups,
            series,
            characters,
            tags,
            date,
            files,
            page_count,
        })
    }
}

fn parse_file(v: &Value) -> Option<GalleryFile> {
    let obj = v.as_object()?;
    let name = obj.get("name").and_then(Value::as_str)?.to_string();
    let hash = obj.get("hash").and_then(Value::as_str)?.to_string();
    Some(GalleryFile {
        name,
        hash,
        width: flexible_int(obj.get("width")).unwrap_or(0),
        height: flexible_int(obj.get("height")).unwrap_or(0),
        haswebp: flexible_int(obj.get("haswebp")).unwrap_or(0),
        hasavif: flexible_int(obj.get("hasavif")).unwrap_or(0),
        hasavifsmalltn: flexible_int(obj.get("hasavifsmalltn")),
        local_path: None,
    })
}

fn parse_tags(obj: &serde_json::Map<String, Value>) -> Vec<String> {
    let Some(arr) = obj.get("tags").and_then(Value::as_array) else {
        return Vec::new();
    };
    arr.iter()
        .filter_map(|t| {
            let tag = t.get("tag").and_then(Value::as_str)?;
            let mut s = String::new();
            if truthy(t.get("female")) {
                s.push_str("female:");
            }
            if truthy(t.get("male")) {
                s.push_str("male:");
            }
            s.push_str(tag);
            Some(s)
        })
        .collect()
}

fn name_list(obj: &serde_json::Map<String, Value>, plural: &str, singular: &str) -> Vec<String> {
    if let Some(arr) = obj.get(plural).and_then(Value::as_array) {
        arr.iter()
            .filter_map(|e| e.get(singular).and_then(Value::as_str).map(str::to_string))
            .collect()
    } else if let Some(arr) = obj.get(singular).and_then(Value::as_array) {
        arr.iter()
            .filter_map(|e| e.as_str().map(str::to_string))
            .collect()
    } else {
        Vec::new()
    }
}

fn flexible_int(v: Option<&Value>) -> Option<i64> {
    match v {
        Some(Value::Number(n)) => n.as_i64().or_else(|| n.as_f64().map(|f| f as i64)),
        Some(Value::String(s)) => s.trim().parse::<i64>().ok(),
        _ => None,
    }
}

fn truthy(v: Option<&Value>) -> bool {
    match v {
        Some(Value::Bool(b)) => *b,
        Some(Value::Number(n)) => n.as_i64().map(|i| i != 0).unwrap_or(false),
        Some(Value::String(s)) => !s.is_empty() && s != "0",
        _ => false,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GalleryType {
    All,
    Doujinshi,
    Manga,
    ArtistCG,
    GameCG,
    Anime,
}

impl GalleryType {
    pub fn from_str(s: &str) -> GalleryType {
        match s {
            "doujinshi" => GalleryType::Doujinshi,
            "manga" => GalleryType::Manga,
            "artistcg" => GalleryType::ArtistCG,
            "gamecg" => GalleryType::GameCG,
            "anime" => GalleryType::Anime,
            _ => GalleryType::All,
        }
    }

    pub fn slug(&self) -> &'static str {
        match self {
            GalleryType::All => "",
            GalleryType::Doujinshi => "doujinshi",
            GalleryType::Manga => "manga",
            GalleryType::ArtistCG => "artistcg",
            GalleryType::GameCG => "gamecg",
            GalleryType::Anime => "anime",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Latest,
    Today,
    Week,
    Month,
    Year,
}

impl SortOrder {
    pub fn from_str(s: &str) -> SortOrder {
        match s {
            "today" => SortOrder::Today,
            "week" => SortOrder::Week,
            "month" => SortOrder::Month,
            "year" => SortOrder::Year,
            _ => SortOrder::Latest,
        }
    }

    pub fn popular_period(&self) -> Option<&'static str> {
        match self {
            SortOrder::Latest => None,
            SortOrder::Today => Some("today"),
            SortOrder::Week => Some("week"),
            SortOrder::Month => Some("month"),
            SortOrder::Year => Some("year"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GalleryPage {
    pub items: Vec<Gallery>,
    pub total: usize,
    pub total_pages: usize,
    pub page: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Suggestion {
    pub label: String,
    pub value: String,
    pub count: i64,
    pub namespace: String,
}
