//! marktextml — Markdown + Mermaid + LaTeX → static HTML blog generator.
//!
//! `marktextml build`      render `content/*.md` into the output dir
//! `marktextml new <slug>` scaffold a content file with YAML frontmatter
//! `marktextml serve`      preview the built site over HTTP

mod config;
mod frontmatter;
mod markdown;
mod site;
mod template;

use config::SiteConfig;
use std::env;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::process::exit;

const USAGE: &str = "marktextml — Markdown + Mermaid + LaTeX -> static HTML blog generator

USAGE:
  marktextml build                 build the site into the output dir
  marktextml new <slug> [title]    scaffold content/<slug>.md
  marktextml serve [--addr A:P]    preview the built site (default 127.0.0.1:8080)
  marktextml help                  show this help";

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let cmd = args.first().map(String::as_str).unwrap_or("help");

    let result = match cmd {
        "build" => cmd_build(&args[1..]),
        "new" => cmd_new(&args[1..]),
        "serve" => cmd_serve(&args[1..]),
        "help" | "-h" | "--help" => {
            println!("{USAGE}");
            Ok(())
        }
        other => {
            eprintln!("error: unknown command `{other}`\n\n{USAGE}");
            exit(2);
        }
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        exit(1);
    }
}

fn cmd_build(_args: &[String]) -> Result<(), String> {
    let cfg = SiteConfig::load(Path::new("."))?;
    site::build(&cfg)
}

fn cmd_new(args: &[String]) -> Result<(), String> {
    let slug = args
        .first()
        .ok_or_else(|| "usage: marktextml new <slug> [title]".to_string())?;
    if !slug
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        || slug.starts_with('-')
    {
        return Err(format!("slug `{slug}` must be lowercase [a-z0-9-]"));
    }
    let title = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| title_from_slug(slug));

    let cfg = SiteConfig::load(Path::new("."))?;
    let dir = cfg.abs(&cfg.content_dir);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("{slug}.md"));
    if path.exists() {
        return Err(format!("{} already exists", path.display()));
    }

    let today = local_date();
    let body = format!(
        "---\ntitle: {title}\ndate: {today}\nslug: {slug}\ndescription:\ntags:\n---\n\nWrite here.\n"
    );
    std::fs::write(&path, body).map_err(|e| e.to_string())?;
    println!("created {}", path.display());
    Ok(())
}

fn title_from_slug(slug: &str) -> String {
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

/// YYYY-MM-DD for today via POSIX `date` (build hosts are macOS/Linux).
fn local_date() -> String {
    std::process::Command::new("date")
        .arg("+%F")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "1970-01-01".into())
}

fn cmd_serve(args: &[String]) -> Result<(), String> {
    let mut addr = "127.0.0.1:8080".to_string();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--addr" => {
                addr = args.get(i + 1).cloned().ok_or("--addr needs a value")?;
                i += 2;
            }
            other => return Err(format!("unknown serve flag `{other}`")),
        }
    }

    let cfg = SiteConfig::load(Path::new("."))?;
    let root = cfg.abs(&cfg.out_dir);
    serve(&root, &addr)
}

/// Minimal static-file HTTP/1.1 server (std only) for local preview.
fn serve(dir: &Path, addr: &str) -> Result<(), String> {
    if !dir.is_dir() {
        return Err(format!(
            "{} does not exist — run `marktextml build` first",
            dir.display()
        ));
    }
    let listener = TcpListener::bind(addr).map_err(|e| format!("bind {addr}: {e}"))?;
    println!("serving {} at http://{}/", dir.display(), addr);
    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                let _ = handle(s, dir);
            }
            Err(e) => eprintln!("accept error: {e}"),
        }
    }
    Ok(())
}

fn handle(mut stream: TcpStream, dir: &Path) -> std::io::Result<()> {
    let mut buf = [0u8; 8192];
    let n = stream.read(&mut buf)?;
    let req = String::from_utf8_lossy(&buf[..n]);
    let (method, target) = match req.lines().next() {
        Some(line) => {
            let mut parts = line.split_whitespace();
            (
                parts.next().unwrap_or("").to_string(),
                parts.next().unwrap_or("/").to_string(),
            )
        }
        None => ("GET".into(), "/".into()),
    };

    if method != "GET" && method != "HEAD" {
        return respond(stream, 405, "text/plain", b"method not allowed");
    }

    let rel = target.split('?').next().unwrap_or("/");
    let rel = rel.trim_start_matches('/');
    let rel = urldecode(rel);

    // reject traversal
    if rel.split('/').any(|c| c == "..") {
        return respond(stream, 403, "text/plain", b"forbidden");
    }

    let mut file = dir.join(&rel);
    if file.is_dir() {
        file = file.join("index.html");
    }

    match std::fs::read(&file) {
        Ok(body) => {
            let ct = content_type(&file);
            if method == "HEAD" {
                respond(stream, 200, ct, b"")
            } else {
                respond(stream, 200, ct, &body)
            }
        }
        Err(_) => respond(stream, 404, "text/plain", b"not found"),
    }
}

fn respond(mut stream: TcpStream, code: u16, ctype: &str, body: &[u8]) -> std::io::Result<()> {
    let reason = match code {
        200 => "OK",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        _ => "Error",
    };
    let head = format!(
        "HTTP/1.1 {code} {reason}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(head.as_bytes())?;
    stream.write_all(body)?;
    stream.flush()
}

fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "json" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "ico" => "image/x-icon",
        "woff2" => "font/woff2",
        "woff" => "font/woff",
        "ttf" => "font/ttf",
        "txt" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

fn urldecode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => {
                let hex = &s[i + 1..i + 3];
                match u8::from_str_radix(hex, 16) {
                    Ok(b) => {
                        out.push(b);
                        i += 3;
                    }
                    Err(_) => {
                        out.push(bytes[i]);
                        i += 1;
                    }
                }
            }
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}
