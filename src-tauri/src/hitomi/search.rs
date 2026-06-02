use super::nozomi::nozomi_url_from_args;

const PREFIXES: [&str; 10] = [
    "language:",
    "artist:",
    "tag:",
    "female:",
    "male:",
    "title:",
    "type:",
    "series:",
    "character:",
    "group:",
];

#[derive(Debug, Default, Clone)]
pub struct SearchTerms {
    pub language: Option<String>,
    pub artists: Vec<String>,
    pub tags: Vec<String>,
    pub titles: Vec<String>,
    pub types: Vec<String>,
    pub series: Vec<String>,
    pub characters: Vec<String>,
    pub groups: Vec<String>,
    pub general: Vec<String>,
}

pub fn parse(query: &str) -> SearchTerms {
    let mut t = SearchTerms::default();
    let mut idx = 0usize;

    while idx < query.len() {
        let slice = &query[idx..];

        let mut nearest: Option<(usize, &str)> = None;
        for p in PREFIXES {
            if let Some(rel) = slice.find(p) {
                let abs = idx + rel;
                if nearest.map_or(true, |(pos, _)| abs < pos) {
                    nearest = Some((abs, p));
                }
            }
        }

        let Some((pos, prefix)) = nearest else {
            let remaining = query[idx..].trim();
            if !remaining.is_empty() {
                t.general.push(remaining.to_string());
            }
            break;
        };

        let value_start = pos + prefix.len();
        let mut value_end = query.len();
        let after = &query[value_start..];
        for np in PREFIXES {
            if let Some(rel) = after.find(np) {
                let abs = value_start + rel;
                if abs < value_end {
                    value_end = abs;
                }
            }
        }

        let value = query[value_start..value_end].trim().to_string();
        let clean = &prefix[..prefix.len() - 1];

        match clean {
            "language" => t.language = Some(value),
            "artist" => t.artists.push(value),
            "tag" => t.tags.push(format!("tag:{value}")),
            "female" => t.tags.push(format!("female:{value}")),
            "male" => t.tags.push(format!("male:{value}")),
            "title" => t.titles.push(value),
            "type" => t.types.push(value),
            "series" => t.series.push(value),
            "character" => t.characters.push(value),
            "group" => t.groups.push(value),
            _ => t.general.push(format!("{clean}:{value}")),
        }

        idx = value_end;
    }

    t
}

pub fn build_constraints(t: &SearchTerms) -> Vec<String> {
    let mut c: Vec<String> = Vec::new();
    if let Some(lang) = &t.language {
        c.push(format!("language:{lang}"));
    }
    c.extend(t.tags.iter().cloned());
    c.extend(t.artists.iter().map(|a| format!("artist:{a}")));
    c.extend(t.types.iter().map(|x| format!("type:{x}")));
    c.extend(t.series.iter().map(|x| format!("series:{x}")));
    c.extend(t.characters.iter().map(|x| format!("character:{x}")));
    c.extend(t.groups.iter().map(|x| format!("group:{x}")));
    c.extend(t.general.iter().map(|x| format!("tag:{x}")));
    c
}

pub fn constraint_to_nozomi_url(constraint: &str) -> Option<String> {
    let mut parts = constraint.splitn(2, ':');
    let ctype = parts.next()?;
    let value = parts.next()?;
    if value.is_empty() {
        return None;
    }

    let url = match ctype {
        "language" => nozomi_url_from_args("all", "index", value),
        "female" | "male" | "tag" => {
            let tag_value = if ctype == "tag" {
                value.to_string()
            } else {
                format!("{ctype}:{value}")
            };
            let processed = tag_value.replace('_', " ");
            nozomi_url_from_args("tag", &processed, "all")
        }
        "artist" => nozomi_url_from_args("artist", &value.replace('_', " "), "all"),
        "type" => nozomi_url_from_args("type", value, "all"),
        "series" => nozomi_url_from_args("series", &value.replace('_', " "), "all"),
        "character" => nozomi_url_from_args("character", &value.replace('_', " "), "all"),
        "group" => nozomi_url_from_args("group", &value.replace('_', " "), "all"),
        _ => return None,
    };
    Some(url)
}
