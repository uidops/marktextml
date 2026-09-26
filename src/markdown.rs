//! Markdown → HTML pipeline: GFM + build-time math (katex-rs) + mermaid blocks.
//!
//! Design decisions (see plan):
//! - Math is rendered at build time so pages work with JavaScript disabled.
//! - ```mermaid``` fences become `<div class="mermaid">…</div>`; the page shell
//!   only loads `mermaid.min.js` when a post actually produced one.

use katex::{render_to_string, KatexContext, Settings, StrictMode, StrictSetting};
use pulldown_cmark::{html, CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use std::sync::LazyLock;

/// Rendering result for one markdown document.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Rendered {
    pub html: String,
    /// True when the source contained at least one ```mermaid block.
    pub has_mermaid: bool,
    /// True when at least one `$…$` / `$$…$$` span was rendered — the page
    /// only pulls in the KaTeX stylesheet when this is set.
    pub has_math: bool,
}

/// The KaTeX context caches fonts/macros/symbols — build it once per process.
static CTX: LazyLock<KatexContext> = LazyLock::new(KatexContext::default);

// `Settings` holds a `RefCell` (custom macros) so it can't be a `static`;
// thread-locals keep one instance per build thread instead of one per formula.
thread_local! {
    static INLINE_SETTINGS: Settings = settings(false);
    static DISPLAY_SETTINGS: Settings = settings(true);
}

fn settings(display_mode: bool) -> Settings {
    let mut s = Settings::default();
    s.display_mode = display_mode;
    // Bad LaTeX must degrade to an inline error span, never fail the build.
    s.throw_on_error = false;
    // `mathVsTextUnits` and friends fire on legitimate blog prose; ignore.
    s.strict = StrictSetting::Mode(StrictMode::Ignore);
    s
}

/// GFM + math + footnotes + heading attributes.
fn options() -> Options {
    let mut o = Options::empty();
    o.insert(Options::ENABLE_TABLES);
    o.insert(Options::ENABLE_STRIKETHROUGH);
    o.insert(Options::ENABLE_TASKLISTS);
    o.insert(Options::ENABLE_FOOTNOTES);
    o.insert(Options::ENABLE_MATH);
    o.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    o
}

/// Render a markdown body to HTML.
pub fn render(src: &str) -> Rendered {
    let events: Vec<Event> = Parser::new_ext(src, options()).collect();
    let mut out = String::with_capacity(src.len() * 2);
    let mut has_mermaid = false;
    let mut has_math = false;

    let mut i = 0;
    while i < events.len() {
        match &events[i] {
            Event::InlineMath(tex) => {
                out.push_str(&render_math(tex, false));
                has_math = true;
                i += 1;
            }
            Event::DisplayMath(tex) => {
                out.push_str(&render_math(tex, true));
                has_math = true;
                i += 1;
            }
            Event::Start(Tag::CodeBlock(info)) if is_mermaid(info) => {
                has_mermaid = true;
                let mut code = String::new();
                let mut j = i + 1;
                while j < events.len() {
                    match &events[j] {
                        Event::Text(t) | Event::Code(t) => {
                            code.push_str(t);
                            j += 1;
                        }
                        Event::End(TagEnd::CodeBlock) => {
                            j += 1;
                            break;
                        }
                        _ => break,
                    }
                }
                out.push_str("<div class=\"mermaid\">");
                out.push_str(&escape(&code));
                out.push_str("</div>\n");
                i = j;
            }
            other => {
                // Feed the remaining events to push_html in batches: flush at
                // each math/mermaid boundary so we never clone past it.
                let batch_end = next_special(&events, i + 1);
                html::push_html(&mut out, events[i..batch_end].iter().cloned());
                i = batch_end;
                let _ = other;
            }
        }
    }

    Rendered {
        html: out,
        has_mermaid,
        has_math,
    }
}

/// First index ≥ `from` that starts a math/mermaid construct, else `len`.
fn next_special(events: &[Event], from: usize) -> usize {
    let mut j = from;
    while j < events.len() {
        match &events[j] {
            Event::InlineMath(_) | Event::DisplayMath(_) => return j,
            Event::Start(Tag::CodeBlock(info)) if is_mermaid(info) => return j,
            _ => j += 1,
        }
    }
    events.len()
}

fn is_mermaid(kind: &CodeBlockKind) -> bool {
    match kind {
        CodeBlockKind::Fenced(info) => info.split_whitespace().next() == Some("mermaid"),
        CodeBlockKind::Indented => false,
    }
}

fn render_math(tex: &str, display: bool) -> String {
    let attempt = |s: &Settings| match render_to_string(&CTX, tex, s) {
        Ok(html) => html,
        Err(e) => format!(
            "<span class=\"math-error\" title=\"{}\">{}</span>",
            escape(&e.to_string()),
            escape(tex)
        ),
    };
    if display {
        DISPLAY_SETTINGS.with(attempt)
    } else {
        INLINE_SETTINGS.with(attempt)
    }
}

/// Escape text for inclusion in HTML element content.
pub fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_inline_math_with_katex() {
        let r = render("Euler: $e^{i\\pi} + 1 = 0$.");
        assert!(r.html.contains("class=\"katex\""), "{}", r.html);
        assert!(!r.html.contains("math-inline"), "raw math leaked");
        assert!(!r.has_mermaid);
        assert!(r.has_math);
    }

    #[test]
    fn renders_display_math() {
        let r = render("$$\\sum_{k=1}^{n} k = \\frac{n(n+1)}{2}$$");
        assert!(r.html.contains("katex-display"), "{}", r.html);
        assert!(r.has_math);
    }

    #[test]
    fn no_math_no_flag() {
        let r = render("just prose, no math here");
        assert!(!r.has_math, "{}", r.html);
    }

    #[test]
    fn math_error_does_not_panic() {
        // pulldown only emits a math event when braces balance, but the LaTeX
        // itself is invalid — katex must degrade to red inline text, not panic
        // or abort the build.
        let r = render(r"$\definitelynotacommand$");
        assert!(r.html.contains("class=\"katex\""), "{}", r.html);
        assert!(r.html.contains("\\definitelynotacommand"), "{}", r.html);
        assert!(!r.has_mermaid);
    }

    #[test]
    fn unbalanced_brace_math_stays_literal() {
        // pulldown-cmark refuses to treat `$\frac{$` as math (brace tracking);
        // it must pass through as literal text rather than eat characters.
        let r = render(r"$\frac{$");
        assert!(!r.html.contains("class=\"katex\""), "{}", r.html);
        assert!(r.html.contains("$\\frac{$"), "{}", r.html);
    }

    #[test]
    fn mermaid_block_becomes_div() {
        let src = "```mermaid\ngraph TD;\n  A-->B;\n```\n";
        let r = render(src);
        assert!(r.has_mermaid);
        assert!(r.html.contains("<div class=\"mermaid\">"), "{}", r.html);
        assert!(r.html.contains("A--&gt;B;") || r.html.contains("A-->B;"));
        assert!(!r.html.contains("<pre><code"), "{}", r.html);
    }

    #[test]
    fn ordinary_code_block_untouched() {
        let r = render("```rust\nfn main() {}\n```");
        assert!(!r.has_mermaid);
        assert!(
            r.html.contains("<code class=\"language-rust\">"),
            "{}",
            r.html
        );
    }

    #[test]
    fn gfm_table_strikethrough_tasklist() {
        let src = "| a | b |\n|---|---|\n| 1 | 2 |\n\n- [x] done\n- [ ] todo\n\n~~gone~~\n";
        let r = render(src);
        assert!(r.html.contains("<table>"), "{}", r.html);
        assert!(r.html.contains("type=\"checkbox\""), "{}", r.html);
        assert!(r.html.contains("<del>gone</del>"), "{}", r.html);
    }

    #[test]
    fn footnotes_render() {
        let r = render("Body[^1]\n\n[^1]: the note\n");
        assert!(r.html.contains("footnote"), "{}", r.html);
    }

    #[test]
    fn heading_attributes_render() {
        let r = render("# Title {.custom}\n");
        assert!(r.html.contains("class=\"custom\""), "{}", r.html);
    }

    #[test]
    fn escaping_helpers() {
        assert_eq!(escape("<a & \"b\">"), "&lt;a &amp; &quot;b&quot;&gt;");
    }
}
