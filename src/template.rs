//! Minimal `{{key}}` template substitution — deliberately no dependency.
//!
//! Rules:
//! - `{{ key }}` and `{{key}}` are the same; keys are trimmed.
//! - An unknown key is a hard error so a build never silently emits
//!   `{{title}}` into a page.
//! - `{{{raw}}}` is *not* supported — values are inserted verbatim (callers
//!   escape their own content), because nearly every value here is HTML.

use std::collections::HashMap;

/// Substitute every `{{key}}` in `tpl`. Errors on keys missing from `vars`.
pub fn render(tpl: &str, vars: &HashMap<&str, String>) -> Result<String, String> {
    let mut out = String::with_capacity(tpl.len());
    let bytes = tpl.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        match find(tpl, i, "{{") {
            None => {
                out.push_str(&tpl[i..]);
                break;
            }
            Some(open) => {
                out.push_str(&tpl[i..open]);
                match find(tpl, open + 2, "}}") {
                    None => {
                        // unbalanced `{{` — treat the rest as literal text
                        out.push_str(&tpl[open..]);
                        break;
                    }
                    Some(close) => {
                        let key = tpl[open + 2..close].trim();
                        match vars.get(key) {
                            Some(value) => out.push_str(value),
                            None => return Err(format!("template: unknown key `{key}`")),
                        }
                        i = close + 2;
                    }
                }
            }
        }
    }
    Ok(out)
}

/// Byte index of `needle` at or after `from`, respecting char boundaries.
fn find(s: &str, from: usize, needle: &str) -> Option<usize> {
    if from > s.len() {
        return None;
    }
    s.get(from..)
        .and_then(|rest| rest.find(needle))
        .map(|off| from + off)
}

/// Build a var map from `(key, value)` pairs.
pub fn vars<'a>(pairs: &[(&'a str, &str)]) -> HashMap<&'a str, String> {
    pairs.iter().map(|(k, v)| (*k, (*v).to_string())).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> HashMap<&'static str, String> {
        vars(&[
            ("title", "Zp×Zp Field"),
            ("content", "<p>hi</p>"),
            ("base", "/blog/"),
        ])
    }

    #[test]
    fn substitutes_all_keys() {
        let out = render("<title>{{title}}</title>{{content}}", &sample()).unwrap();
        assert_eq!(out, "<title>Zp×Zp Field</title><p>hi</p>");
    }

    #[test]
    fn whitespace_inside_braces_is_tolerated() {
        let out = render("{{ title }} / {{  base  }}", &sample()).unwrap();
        assert_eq!(out, "Zp×Zp Field / /blog/");
    }

    #[test]
    fn unknown_key_is_an_error() {
        let err = render("{{nope}}", &sample()).unwrap_err();
        assert!(err.contains("`nope`"), "{err}");
    }

    #[test]
    fn unbalanced_braces_pass_through() {
        let out = render("a {{ b", &sample()).unwrap();
        assert_eq!(out, "a {{ b");
        let out = render("a }} b", &sample()).unwrap();
        assert_eq!(out, "a }} b");
    }

    #[test]
    fn values_may_contain_braces() {
        let mut v = sample();
        v.insert("content", "{{title}}".into());
        let out = render("{{content}}", &v).unwrap();
        assert_eq!(out, "{{title}}", "values must not be re-scanned");
    }

    #[test]
    fn no_placeholders_leaves_text_untouched() {
        let out = render("plain text", &sample()).unwrap();
        assert_eq!(out, "plain text");
    }

    #[test]
    fn unicode_is_safe() {
        let out = render("× {{title}} ×", &sample()).unwrap();
        assert_eq!(out, "× Zp×Zp Field ×");
    }
}
