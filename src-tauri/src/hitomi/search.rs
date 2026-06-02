use super::nozomi::nozomi_url_from_args;

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

    for token in query.split_whitespace() {
        match token.split_once(':') {
            Some((prefix, value)) => {
                let value = value.trim();
                if value.is_empty() {
                    continue;
                }
                match prefix {
                    "language" => t.language = Some(value.to_string()),
                    "artist" => t.artists.push(value.to_string()),
                    "tag" => t.tags.push(format!("tag:{value}")),
                    "female" => t.tags.push(format!("female:{value}")),
                    "male" => t.tags.push(format!("male:{value}")),
                    "title" => t.titles.push(value.to_string()),
                    "type" => t.types.push(value.to_string()),
                    "series" => t.series.push(value.to_string()),
                    "character" => t.characters.push(value.to_string()),
                    "group" => t.groups.push(value.to_string()),
                    _ => t.general.push(token.to_string()),
                }
            }
            None => t.general.push(token.to_string()),
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_general_term() {
        let t = parse("naruto");
        assert_eq!(t.general, vec!["naruto"]);
        assert_eq!(build_constraints(&t), vec!["tag:naruto"]);
    }

    #[test]
    fn multiple_general_terms_are_anded() {
        let t = parse("naruto sasuke");
        assert_eq!(t.general, vec!["naruto", "sasuke"]);
        assert_eq!(build_constraints(&t), vec!["tag:naruto", "tag:sasuke"]);
    }

    #[test]
    fn mixed_prefixes_and_free_text() {
        let t = parse("big_breasts artist:foo language:korean female:sole_female");
        assert_eq!(t.general, vec!["big_breasts"]);
        assert_eq!(t.artists, vec!["foo"]);
        assert_eq!(t.language.as_deref(), Some("korean"));
        assert_eq!(t.tags, vec!["female:sole_female"]);
    }

    #[test]
    fn leading_free_text_before_prefix_is_kept() {
        let t = parse("naruto artist:kishimoto");
        assert_eq!(t.general, vec!["naruto"]);
        assert_eq!(t.artists, vec!["kishimoto"]);
    }

    #[test]
    fn extra_whitespace_is_ignored() {
        let t = parse("   tag:foo    bar   ");
        assert_eq!(t.tags, vec!["tag:foo"]);
        assert_eq!(t.general, vec!["bar"]);
    }

    #[test]
    fn empty_prefix_value_is_skipped() {
        let t = parse("artist: tag:foo");
        assert!(t.artists.is_empty());
        assert_eq!(t.tags, vec!["tag:foo"]);
    }

    #[test]
    fn unknown_prefix_kept_as_general() {
        let t = parse("weird:thing");
        assert_eq!(t.general, vec!["weird:thing"]);
    }

    #[test]
    fn empty_query_yields_no_constraints() {
        let t = parse("   ");
        assert!(build_constraints(&t).is_empty());
    }
}
