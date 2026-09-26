//! Site configuration: defaults + optional `site.toml` at the project root.
//!
//! `site.toml` is intentionally a tiny `key = "value"` subset — no external
//! TOML dependency.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct SiteConfig {
    pub title: String,
    pub description: String,
    pub base_url: String,
    pub content_dir: PathBuf,
    pub out_dir: PathBuf,
    pub templates_dir: PathBuf,
    pub assets_dir: PathBuf,
    pub root: PathBuf,
}

impl Default for SiteConfig {
    fn default() -> Self {
        SiteConfig {
            title: "uidops".into(),
            description: "notes on code, math and systems".into(),
            base_url: String::new(),
            content_dir: PathBuf::from("content"),
            out_dir: PathBuf::from("dist"),
            templates_dir: PathBuf::from("templates"),
            assets_dir: PathBuf::from("assets"),
            root: PathBuf::from("."),
        }
    }
}

impl SiteConfig {
    /// Load config for a project root: defaults, overridden by `site.toml`.
    pub fn load(root: &Path) -> Result<Self, String> {
        let mut cfg = SiteConfig::default();
        cfg.root = root.to_path_buf();

        let path = root.join("site.toml");
        if path.exists() {
            let text = std::fs::read_to_string(&path)
                .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
            let kv = parse_kv(&text);
            for (k, v) in kv {
                match k.as_str() {
                    "title" => cfg.title = v,
                    "description" => cfg.description = v,
                    "base_url" => cfg.base_url = v.trim_end_matches('/').to_string(),
                    "content" => cfg.content_dir = PathBuf::from(v),
                    "out" => cfg.out_dir = PathBuf::from(v),
                    "templates" => cfg.templates_dir = PathBuf::from(v),
                    "assets" => cfg.assets_dir = PathBuf::from(v),
                    other => eprintln!("warning: unknown site.toml key `{other}`"),
                }
            }
        }
        Ok(cfg)
    }

    /// Absolute path helper for a config-relative dir.
    pub fn abs(&self, rel: &Path) -> PathBuf {
        if rel.is_absolute() {
            rel.to_path_buf()
        } else {
            self.root.join(rel)
        }
    }
}

/// Parse `key = "value"` / `key = value` lines; `#` comments; blank lines.
fn parse_kv(text: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let k = k.trim().to_string();
        let mut v = v.trim().to_string();
        if (v.starts_with('"') && v.ends_with('"') && v.len() >= 2)
            || (v.starts_with('\'') && v.ends_with('\'') && v.len() >= 2)
        {
            v = v[1..v.len() - 1].to_string();
        }
        if !k.is_empty() {
            map.insert(k, v);
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_kv() {
        let m = parse_kv("# comment\ntitle = \"Zp\"\nbase_url = https://x.dev/\n\nout = dist");
        assert_eq!(m.get("title").unwrap(), "Zp");
        assert_eq!(m.get("base_url").unwrap(), "https://x.dev/");
        assert_eq!(m.get("out").unwrap(), "dist");
        assert_eq!(m.len(), 3);
    }
}
