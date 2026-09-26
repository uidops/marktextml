//! Site build: frontmatter → markdown → templates → `dist/`.
//!
//! Output layout (relative base paths so the site works from any subdirectory
//! and straight off the filesystem):
//!
//! ```text
//! dist/
//!   index.html            post list
//!   posts/<slug>.html     one page per content file
//!   assets/…              copied verbatim from assets/ (css, katex, mermaid)
//!   favicon*              also copied to the root for the shell's <link> tags
//! ```

use crate::config::SiteConfig;
use crate::frontmatter::{self, Meta};
use crate::markdown;
use crate::template;
use std::path::{Path, PathBuf};

/// One rendered post, newest first after sorting.
pub struct Post {
    pub meta: Meta,
    pub html: String,
    pub has_mermaid: bool,
    pub has_math: bool,
    /// Word count of the rendered text (for reading time + excerpts).
    pub words: usize,
    /// Source path, for error messages.
    pub src: PathBuf,
}

const FAVICONS: [&str; 4] = [
    "favicon.ico",
    "favicon-16x16.png",
    "favicon-32x32.png",
    "apple-touch-icon.png",
];

pub fn build(cfg: &SiteConfig) -> Result<(), String> {
    let shell_path = cfg.abs(&cfg.templates_dir).join("shell.html");
    let shell = std::fs::read_to_string(&shell_path)
        .map_err(|e| format!("cannot read template {}: {e}", shell_path.display()))?;

    let posts = collect(cfg)?;
    let out = prepare_out(cfg)?;
    copy_assets(cfg, &out)?;

    // Index page — base "." keeps every href relative to the site root.
    let items: String = posts.iter().map(post_item_html).collect();
    let list = if items.is_empty() {
        "<div class=\"post-empty\">no posts yet — run <code>marktextml new &lt;slug&gt;</code></div>"
            .to_string()
    } else {
        format!("<div class=\"post-list\">\n{items}</div>")
    };
    let index_content = format!(
        r#"<section class="sec" id="posts" aria-label="Posts">
    <p class="sec-idx">// index</p>
    <div class="sec-head">
        <h2>posts</h2>
        <p class="sec-sub">{desc}</p>
    </div>
    {list}
</section>"#,
        desc = markdown::escape(&cfg.description),
        list = list
    );
    let index = render_page(
        &shell,
        &Page {
            title: cfg.title.clone(),
            description: cfg.description.clone(),
            base: ".",
            content: index_content,
            head_extra: "<link rel=\"stylesheet\" href=\"./assets/css/post.css\">".into(),
            scripts_extra: String::new(),
        },
    )?;
    write(&out.join("index.html"), &index)?;

    // Post pages.
    for post in &posts {
        let content = post_content(post, "..");
        // katex.min.css first, post.css last: the theme rules in post.css
        // deliberately override KaTeX's (same specificity, later cascade).
        let mut head = String::new();
        if post.has_math {
            head.push_str("<link rel=\"stylesheet\" href=\"../assets/katex/katex.min.css\">\n");
        }
        head.push_str("<link rel=\"stylesheet\" href=\"../assets/css/post.css\">");
        let mut scripts = String::new();
        if post.has_mermaid {
            scripts = format!(
                "\n<script src=\"../assets/mermaid.min.js\"></script>\n\
                 <script>mermaid.initialize({{ startOnLoad: true, theme: \"dark\" }});</script>"
            );
        }
        let title = format!("{} — {}", post.meta.title, cfg.title);
        let description = if post.meta.description.is_empty() {
            excerpt(&post.html, 180)
        } else {
            post.meta.description.clone()
        };
        let page = render_page(
            &shell,
            &Page {
                title,
                description,
                base: "..",
                content,
                head_extra: head,
                scripts_extra: scripts,
            },
        )?;
        let path = out.join("posts").join(format!("{}.html", post.meta.slug));
        write(&path, &page)?;
    }

    println!("built {} post(s) → {}", posts.len(), out.display());
    Ok(())
}

/// Everything a page needs to be substituted into the shell.
struct Page {
    title: String,
    description: String,
    base: &'static str,
    content: String,
    head_extra: String,
    scripts_extra: String,
}

fn render_page(shell: &str, page: &Page) -> Result<String, String> {
    let vars = template::vars(&[
        ("title", page.title.as_str()),
        ("description", page.description.as_str()),
        ("base", page.base),
        ("content", page.content.as_str()),
        ("head_extra", page.head_extra.as_str()),
        ("scripts_extra", page.scripts_extra.as_str()),
    ]);
    template::render(shell, &vars)
}

/// Read + parse + render every `content/*.md`, sorted newest first.
fn collect(cfg: &SiteConfig) -> Result<Vec<Post>, String> {
    let dir = cfg.abs(&cfg.content_dir);
    let mut posts = Vec::new();
    if !dir.is_dir() {
        return Ok(posts);
    }
    for entry in std::fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        let default_slug = name.strip_suffix(".md").unwrap_or(&name).to_string();
        let source = std::fs::read_to_string(&path)
            .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        let (yaml, body) = frontmatter::split(&source).map_err(|e| format!("{name}: {e}"))?;
        let meta = frontmatter::parse(&yaml, &default_slug);
        let rendered = markdown::render(&body);
        posts.push(Post {
            words: word_count(&rendered.html),
            html: rendered.html,
            has_mermaid: rendered.has_mermaid,
            has_math: rendered.has_math,
            meta,
            src: path,
        });
    }

    // newest first; undated posts sink, ties break alphabetically by slug
    posts.sort_by(|a, b| {
        b.meta
            .date
            .cmp(&a.meta.date)
            .then_with(|| a.meta.slug.cmp(&b.meta.slug))
    });

    let mut seen: Vec<&str> = Vec::new();
    for p in &posts {
        if seen.contains(&p.meta.slug.as_str()) {
            return Err(format!(
                "{}: duplicate slug `{}` (also used by another post)",
                p.src.display(),
                p.meta.slug
            ));
        }
        seen.push(&p.meta.slug);
    }
    Ok(posts)
}

/// Wipe and recreate the output directory.
fn prepare_out(cfg: &SiteConfig) -> Result<PathBuf, String> {
    let out = cfg.abs(&cfg.out_dir);
    if out == cfg.root || out == Path::new("/") {
        return Err(format!(
            "refusing to use {} as the output directory",
            out.display()
        ));
    }
    if out.exists() {
        std::fs::remove_dir_all(&out)
            .map_err(|e| format!("cannot clear {}: {e}", out.display()))?;
    }
    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    Ok(out)
}

/// `assets/` → `dist/assets/`, plus favicons at the site root.
fn copy_assets(cfg: &SiteConfig, out: &Path) -> Result<(), String> {
    let src = cfg.abs(&cfg.assets_dir);
    if src.is_dir() {
        copy_dir(&src, &out.join("assets"))?;
    }
    for icon in FAVICONS {
        let from = out.join("assets").join(icon);
        if from.is_file() {
            std::fs::copy(&from, out.join(icon)).map_err(|e| format!("copy {icon}: {e}"))?;
        }
    }
    Ok(())
}

fn copy_dir(from: &Path, to: &Path) -> Result<(), String> {
    std::fs::create_dir_all(to).map_err(|e| e.to_string())?;
    for entry in std::fs::read_dir(from).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let dest = to.join(entry.file_name());
        if path.is_dir() {
            copy_dir(&path, &dest)?;
        } else {
            std::fs::copy(&path, &dest).map_err(|e| format!("copy {}: {e}", path.display()))?;
        }
    }
    Ok(())
}

fn write(path: &Path, body: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(path, body).map_err(|e| format!("write {}: {e}", path.display()))
}

/// Index entry for one post.
fn post_item_html(post: &Post) -> String {
    let mut meta = String::new();
    if !post.meta.date.is_empty() {
        meta.push_str(&format_date(&post.meta.date));
    }
    let mins = reading_minutes(post.words);
    if !meta.is_empty() {
        meta.push_str(" · ");
    }
    meta.push_str(&format!("{mins} min read"));

    let desc = if post.meta.description.is_empty() {
        excerpt(&post.html, 180)
    } else {
        post.meta.description.clone()
    };
    let desc = markdown::escape(&desc);

    format!(
        r#"    <a class="post-item" href="./posts/{slug}.html" data-reveal>
        <h2>{title}</h2>
        <span class="post-meta">{meta}</span>
        <p class="excerpt">{desc}</p>
    </a>
"#,
        slug = post.meta.slug,
        title = markdown::escape(&post.meta.title),
    )
}

/// The article itself: header, body, back link.
fn post_content(post: &Post, base: &str) -> String {
    let mut meta = String::new();
    if !post.meta.date.is_empty() {
        meta.push_str(&format_date(&post.meta.date));
        meta.push_str(" · ");
    }
    meta.push_str(&format!("{} min read", reading_minutes(post.words)));

    let tags = if post.meta.tags.is_empty() {
        String::new()
    } else {
        let items: String = post
            .meta
            .tags
            .iter()
            .map(|t| format!("<span>{}</span>", markdown::escape(t)))
            .collect();
        format!("\n        <div class=\"post-tags\">{items}</div>")
    };

    // Banner sits at the top of the header, above the title: it is an image
    // file, so `base` (.. on post pages) points back at the site root.
    let banner = if post.meta.banner.is_empty() {
        String::new()
    } else {
        format!(
            "\n        <img class=\"post-banner\" src=\"{}/{}\" alt=\"\" width=\"1056\" height=\"320\" fetchpriority=\"high\">",
            base,
            post.meta.banner.trim_start_matches('/')
        )
    };

    format!(
        r#"<article class="article">
    <header class="post-head" data-reveal>{banner}
        <h1>{title}</h1>
        <div class="post-meta">{meta}</div>{tags}
    </header>
{body}
    <p class="post-foot"><a href="{base}#posts">← all posts</a></p>
</article>"#,
        title = markdown::escape(&post.meta.title),
        banner = banner,
        meta = markdown::escape(&meta),
        tags = tags,
        body = post.html,
        base = base,
    )
}

/// `2026-09-26` → `September 26, 2026`; anything else passes through.
fn format_date(raw: &str) -> String {
    let parts: Vec<&str> = raw.split('-').collect();
    if parts.len() != 3 {
        return raw.to_string();
    }
    let (Ok(y), Ok(m), Ok(d)) = (
        parts[0].parse::<u32>(),
        parts[1].parse::<u32>(),
        parts[2].parse::<u32>(),
    ) else {
        return raw.to_string();
    };
    let months = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    match months.get(m.wrapping_sub(1) as usize) {
        Some(name) if (1..=31).contains(&d) => format!("{name} {d}, {y}"),
        _ => raw.to_string(),
    }
}

fn reading_minutes(words: usize) -> usize {
    (words / 200).max(1)
}

/// Visible words in a rendered HTML fragment.
fn word_count(html: &str) -> usize {
    html_to_text(html).split_whitespace().count()
}

/// First paragraph of rendered HTML as plain text (for meta descriptions).
fn excerpt(html: &str, max: usize) -> String {
    let text = html_to_text(html);
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if text.chars().count() <= max {
        return text;
    }
    let mut cut = text
        .char_indices()
        .take_while(|(i, _)| *i < max)
        .last()
        .map(|(i, _)| i)
        .unwrap_or(max);
    // don't split a word in half
    if let Some(space) = text[..cut].rfind(' ') {
        cut = space;
    }
    format!("{}…", text[..cut].trim_end())
}

/// Strip tags and decode the handful of entities we emit.
fn html_to_text(html: &str) -> String {
    let chars: Vec<char> = html.chars().collect();
    let mut text = String::with_capacity(html.len());
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '<' {
            while i < chars.len() && chars[i] != '>' {
                i += 1;
            }
            i += 1; // past the closing `>`
                    // `<p>one</p><p>two` must not become "onetwo", but
                    // `<strong>x</strong>,` must not become "x ,"
            match chars.get(i) {
                Some(next) if next.is_alphanumeric() => {
                    if !text.is_empty() && !text.ends_with(char::is_whitespace) {
                        text.push(' ');
                    }
                }
                _ => {}
            }
        } else {
            text.push(chars[i]);
            i += 1;
        }
    }
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_iso_dates() {
        assert_eq!(format_date("2024-01-10"), "January 10, 2024");
        assert_eq!(format_date("2026-12-31"), "December 31, 2026");
        assert_eq!(format_date("not-a-date"), "not-a-date");
        assert_eq!(format_date(""), "");
        assert_eq!(format_date("2026-13-40"), "2026-13-40");
    }

    #[test]
    fn reading_time_floors_at_one_minute() {
        assert_eq!(reading_minutes(0), 1);
        assert_eq!(reading_minutes(199), 1);
        assert_eq!(reading_minutes(400), 2);
    }

    #[test]
    fn excerpt_strips_tags_and_respects_limit() {
        let html = "<p>Hello <strong>world</strong>, this is a paragraph.</p><p>Second.</p>";
        let e = excerpt(html, 20);
        assert!(e.starts_with("Hello world"), "{e}");
        assert!(e.chars().count() <= 21, "{e}");
        assert!(!e.contains('<'), "{e}");
    }

    #[test]
    fn word_count_counts_visible_words() {
        assert_eq!(word_count("<p>one two</p><p>three</p>"), 3);
    }

    #[test]
    fn build_end_to_end() {
        let root = std::env::temp_dir().join(format!("marktextml-e2e-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("templates")).unwrap();
        std::fs::create_dir_all(root.join("content")).unwrap();
        std::fs::create_dir_all(root.join("assets/css")).unwrap();

        std::fs::write(
            root.join("templates/shell.html"),
            "<!doctype html><title>{{title}}</title>\
             <meta content=\"{{description}}\"><link href=\"{{base}}/x\"><body>\
             {{head_extra}}{{content}}{{scripts_extra}}</body>",
        )
        .unwrap();
        std::fs::write(
            root.join("content/hello.md"),
            "---\ntitle: Hello & Welcome\ndate: 2026-01-02\nslug: hello\ntags: [rust, web]\nbanner: assets/img/hello-banner.svg\n---\n\n\
             First *paragraph* with math $x^2$.\n\n```mermaid\ngraph TD; A-->B;\n```\n",
        )
        .unwrap();
        std::fs::write(root.join("assets/css/post.css"), "body{}").unwrap();
        std::fs::write(root.join("assets/favicon.ico"), "icon").unwrap();

        let mut cfg = SiteConfig::default();
        cfg.root = root.clone();
        build(&cfg).unwrap();

        let index = std::fs::read_to_string(root.join("dist/index.html")).unwrap();
        assert!(index.contains("Hello &amp; Welcome"), "{index}");
        assert!(index.contains("./posts/hello.html"), "{index}");
        assert!(index.contains("January 2, 2026"), "{index}");

        let post = std::fs::read_to_string(root.join("dist/posts/hello.html")).unwrap();
        assert!(post.contains("katex"), "{post}"); // math rendered
        assert!(post.contains("assets/katex/katex.min.css"), "{post}"); // math css
        assert!(post.contains("mermaid.min.js"), "{post}"); // mermaid loaded
        assert!(post.contains("data-reveal"), "{post}");
        assert!(
            post.contains("<img class=\"post-banner\" src=\"../assets/img/hello-banner.svg\""),
            "{post}"
        );

        assert!(root.join("dist/assets/css/post.css").is_file());
        assert!(root.join("dist/favicon.ico").is_file());

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn build_without_content_makes_empty_index() {
        let root = std::env::temp_dir().join(format!("marktextml-empty-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("templates")).unwrap();
        std::fs::write(
            root.join("templates/shell.html"),
            "{{title}}|{{description}}|{{base}}|{{content}}|{{head_extra}}|{{scripts_extra}}",
        )
        .unwrap();

        let mut cfg = SiteConfig::default();
        cfg.root = root.clone();
        build(&cfg).unwrap();
        let index = std::fs::read_to_string(root.join("dist/index.html")).unwrap();
        assert!(index.contains("post-empty"), "{index}");
        let _ = std::fs::remove_dir_all(&root);
    }
}
