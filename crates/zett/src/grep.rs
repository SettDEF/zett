//! Search and replace across the workspace's files (Search, Files tab): every text file but the pages
//! (Markdown lives in the workspace's own store), hidden folders, target and node_modules left out.
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// the pattern: plain text escaped unless `regex`, `case` sensitive or not, `word` whole words only
fn pattern(query: &str, o: &Value) -> Option<regex::Regex> {
    if query.is_empty() { return None }
    let src = if o["regex"] == true { query.to_string() } else { regex::escape(query) };
    let src = if o["word"] == true { format!(r"\b{src}\b") } else { src };
    regex::RegexBuilder::new(&src).case_insensitive(o["case"] != true).build().ok()
}

/// the files searched, relative paths beside them
fn files(root: &Path) -> Vec<(PathBuf, String)> {
    fn walk(d: &Path, root: &Path, out: &mut Vec<(PathBuf, String)>) {
        let Ok(rd) = std::fs::read_dir(d) else { return };
        for e in rd.flatten() {
            let p = e.path();
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with('.') || matches!(name.as_str(), "target" | "node_modules" | "build" | "dist") || out.len() > 20_000 { continue }
            if p.is_dir() { walk(&p, root, out) }
            else if p.extension().is_none_or(|x| x != "md") && e.metadata().is_ok_and(|m| m.len() < 2 << 20) {
                out.push((p.clone(), p.strip_prefix(root).unwrap_or(&p).to_string_lossy().to_string()))
            }
        }
    }
    let mut out = Vec::new();
    walk(root, root, &mut out);
    out.sort_by(|a, b| a.1.cmp(&b.1));
    out
}

/// a file's text, None for binary
fn text(p: &Path) -> Option<String> {
    let b = std::fs::read(p).ok()?;
    if b.contains(&0) { return None }
    String::from_utf8(b).ok()
}

/// Every hit: {path, rel, line (0-based), col, len, text}; 2000 at most
pub fn search(root: &Path, query: &str, o: &Value) -> Value {
    let Some(re) = pattern(query, o) else { return json!([]) };
    let mut hits = Vec::new();
    for (p, rel) in files(root) {
        let Some(t) = text(&p) else { continue };
        for (i, line) in t.lines().enumerate() {
            for m in re.find_iter(line) {
                hits.push(json!({ "path": p, "rel": rel, "line": i, "col": line[..m.start()].chars().count(), "len": m.as_str().chars().count(), "text": line.chars().take(240).collect::<String>() }));
                if hits.len() >= 2000 { return Value::Array(hits) }
            }
        }
    }
    Value::Array(hits)
}

/// Replaces every match in the files given (relative paths; all when empty); how many places changed
pub fn replace(root: &Path, query: &str, with: &str, o: &Value, only: &[String]) -> usize {
    let Some(re) = pattern(query, o) else { return 0 };
    let mut n = 0;
    for (p, rel) in files(root) {
        if !only.is_empty() && !only.contains(&rel) { continue }
        let Some(t) = text(&p) else { continue };
        let count = re.find_iter(&t).count();
        if count == 0 { continue }
        // a plain replacement takes $ literally; a regex one may use $1
        let out = if o["regex"] == true { re.replace_all(&t, with).to_string() } else { re.replace_all(&t, regex::NoExpand(with)).to_string() };
        if std::fs::write(&p, out).is_ok() { n += count }
    }
    n
}
