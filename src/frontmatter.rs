//! Minimal YAML frontmatter: `---` fences, single-line `key: value`, inline
//! `[a, b]` lists. Deliberately tiny — post metadata is flat and small.

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Meta {
    pub title: String,
    pub date: String,
    pub slug: String,
    pub description: String,
    /// Optional banner/hero image, path relative to the site root
    /// (e.g. `assets/img/zp-zp-field.svg`). Empty = no banner.
    pub banner: String,
    pub tags: Vec<String>,
}

/// Split `---\n yaml \n---\n body` → (yaml, body).
pub fn split(source: &str) -> Result<(String, String), String> {
    let src = source.replace("\r\n", "\n");
    let mut lines = src.lines();
    if lines.next().map(str::trim) != Some("---") {
        return Err("frontmatter: file must start with `---`".into());
    }
    let mut yaml = String::new();
    let mut closed = false;
    for line in lines {
        if line.trim() == "---" {
            closed = true;
            break;
        }
        yaml.push_str(line);
        yaml.push('\n');
    }
    if !closed {
        return Err("frontmatter: missing closing `---`".into());
    }
    let body = src[src.len() - src.lines().count().min(usize::MAX)..].to_string();
    // recompute body robustly: everything after the closing fence line
    let after = &src;
    let mut seen = 0;
    let mut body_start = 0;
    for (i, line) in after.lines().enumerate() {
        if seen == 0 {
            if line.trim() != "---" {
                return Err("frontmatter: file must start with `---`".into());
            }
            seen = 1;
        } else if line.trim() == "---" {
            // byte offset just past this line
            body_start = after
                .lines()
                .take(i + 1)
                .map(|l| l.len() + 1)
                .sum::<usize>();
            break;
        }
    }
    let _ = body;
    Ok((yaml, src[body_start.min(src.len())..].to_string()))
}

/// Parse the yaml block into [`Meta`]. `default_slug` comes from the file name.
pub fn parse(yaml: &str, default_slug: &str) -> Meta {
    let mut kv: HashMap<String, String> = HashMap::new();
    for line in yaml.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once(':') {
            kv.insert(k.trim().to_string(), v.trim().to_string());
        }
    }

    let unquote = |s: &str| -> String {
        let s = s.trim();
        if s.len() >= 2
            && ((s.starts_with('"') && s.ends_with('"'))
                || (s.starts_with('\'') && s.ends_with('\'')))
        {
            s[1..s.len() - 1].to_string()
        } else {
            s.to_string()
        }
    };

    let slug = kv
        .get("slug")
        .map(|s| unquote(s))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| default_slug.to_string());
    let title = kv
        .get("title")
        .map(|s| unquote(s))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| title_from_slug(&slug));
    let date = kv.get("date").cloned().unwrap_or_default();
    let description = kv
        .get("description")
        .map(|s| unquote(s))
        .unwrap_or_default();
    let tags = kv.get("tags").map(|s| parse_list(s)).unwrap_or_default();
    let banner = kv
        .get("banner")
        .map(|s| unquote(s))
        .filter(|s| !s.is_empty())
        .unwrap_or_default();

    Meta {
        title,
        date,
        slug,
        description,
        banner,
        tags,
    }
}

/// `[a, b]`, `a, b`, or single value → Vec (empty strings dropped).
fn parse_list(raw: &str) -> Vec<String> {
    let s = raw.trim();
    let s = s.strip_prefix('[').unwrap_or(s);
    let s = s.strip_suffix(']').unwrap_or(s);
    s.split(',')
        .map(|p| unquote_simple(p))
        .filter(|p| !p.is_empty())
        .collect()
}

fn unquote_simple(s: &str) -> String {
    let s = s.trim();
    if s.len() >= 2
        && ((s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')))
    {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

pub fn title_from_slug(slug: &str) -> String {
    slug.split('-')
        .filter(|p| !p.is_empty())
        .map(|p| {
            let mut c = p.chars();
            match c.next() {
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_frontmatter() {
        let src = "---\ntitle: Hello\nslug: hi\n---\n\nBody text.\n";
        let (yaml, body) = split(src).unwrap();
        assert!(yaml.contains("title: Hello"));
        assert_eq!(body.trim(), "Body text.");
    }

    #[test]
    fn missing_open_fence_errors() {
        assert!(split("no fence").is_err());
        assert!(split("---\ntitle: x\n").is_err()); // no close
    }

    #[test]
    fn parses_meta_with_tags() {
        let yaml = "title: \"Zp×Zp Field\"\ndate: 2026-09-26\nslug: zp-zp-field\ndescription: fields\ntags: [math, algebra]\nbanner: assets/img/zp-zp-field.svg\n";
        let m = parse(yaml, "fallback");
        assert_eq!(m.title, "Zp×Zp Field");
        assert_eq!(m.slug, "zp-zp-field");
        assert_eq!(m.tags, vec!["math", "algebra"]);
        assert_eq!(m.banner, "assets/img/zp-zp-field.svg");
        assert_eq!(parse("title: x\n", "x").banner, "");
    }

    #[test]
    fn title_defaults_from_slug() {
        let m = parse("", "my-cool-post");
        assert_eq!(m.title, "My Cool Post");
        assert_eq!(m.slug, "my-cool-post");
    }
}
