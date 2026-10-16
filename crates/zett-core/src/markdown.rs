//! The Markdown mirror: a page ⇄ a `.md` file, so git, grep and Neovim keep
//! working on a zett workspace.
//!
//! Front matter holds the title. Each block is one Markdown construct:
//!
//! | block | Markdown |
//! |---|---|
//! | heading 1–6 | `# …` to `###### …` |
//! | todo | `- [ ] …` / `- [x] …` |
//! | bullet | `- …` |
//! | quote | `> …` (several lines allowed) |
//! | code | fenced with its language |
//! | paragraph | text up to a blank line |
//!
//! Blocks are separated by a blank line, except list items, which stay tight.
//! Reading is forgiving (`*` bullets, `* [X]` todos); writing is canonical, so
//! a file written by zett reads back to the same blocks.

use crate::blocks::{BlockId, Kind, Result, Workspace};

/// A page as Markdown.
pub fn to_markdown(ws: &Workspace, page: BlockId) -> String {
    let mut out = format!("---\ntitle: {}\n---\n", yaml_str(&ws.text(page)));
    let mut prev_list = false;
    for b in ws.children(page) {
        let kind = ws.kind(b);
        let text = ws.text(b);
        let list = matches!(kind, Kind::Todo { .. } | Kind::Bullet);
        // a blank line between blocks, but list items stay one list
        if !(list && prev_list) {
            out.push('\n');
        }
        prev_list = list;
        match kind {
            Kind::Heading(l) => out += &format!("{} {}\n", "#".repeat(l as usize), text),
            Kind::Todo { checked } => out += &format!("- [{}] {}\n", if checked { "x" } else { " " }, text),
            Kind::Bullet => out += &format!("- {text}\n"),
            Kind::Quote => {
                for line in text.split('\n') {
                    out += &if line.is_empty() { ">\n".to_string() } else { format!("> {line}\n") };
                }
            }
            Kind::Code { lang } => {
                // a fence longer than any run of backticks inside
                let fence = "`".repeat(longest_run(&text, '`').max(2) + 1);
                out += &format!("{fence}{lang}\n{text}\n{fence}\n");
            }
            Kind::Paragraph | Kind::Page => out += &format!("{text}\n"),
        }
    }
    out
}

/// Reads Markdown into a new page and returns it. A file without front matter
/// takes `fallback_title` (the file name, for the folder mirror).
pub fn from_markdown(ws: &Workspace, md: &str, fallback_title: &str) -> Result<BlockId> {
    let (title, body) = split_front_matter(md);
    let page = ws.new_page(title.as_deref().unwrap_or(fallback_title))?;
    let lines: Vec<&str> = body.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let t = line.trim_start();
        if t.is_empty() {
            i += 1;
            continue;
        }
        // fenced code
        if let Some(fence) = fence_of(t) {
            let lang = t[fence.len()..].trim().to_string();
            let mut code = Vec::new();
            i += 1;
            while i < lines.len() && !lines[i].trim_start().starts_with(&fence) {
                code.push(lines[i]);
                i += 1;
            }
            i += 1; // the closing fence (or the end of the file)
            ws.append(page, Kind::Code { lang }, &code.join("\n"))?;
            continue;
        }
        // heading
        let hashes = t.chars().take_while(|c| *c == '#').count();
        if (1..=6).contains(&hashes) && t[hashes..].starts_with(' ') {
            ws.append(page, Kind::Heading(hashes as u8), t[hashes + 1..].trim())?;
            i += 1;
            continue;
        }
        // todo / bullet
        if let Some(rest) = t.strip_prefix("- ").or_else(|| t.strip_prefix("* ")).or_else(|| t.strip_prefix("+ ")) {
            let lower = rest.to_ascii_lowercase();
            if lower.starts_with("[ ] ") || lower.starts_with("[x] ") || lower == "[ ]" || lower == "[x]" {
                let checked = lower.starts_with("[x]");
                ws.append(page, Kind::Todo { checked }, rest.get(4..).unwrap_or("").trim_end())?;
            } else {
                ws.append(page, Kind::Bullet, rest.trim_end())?;
            }
            i += 1;
            continue;
        }
        // quote: consecutive `>` lines
        if t.starts_with('>') {
            let mut q = Vec::new();
            while i < lines.len() && lines[i].trim_start().starts_with('>') {
                let l = lines[i].trim_start()[1..].strip_prefix(' ').unwrap_or(&lines[i].trim_start()[1..]);
                q.push(l);
                i += 1;
            }
            ws.append(page, Kind::Quote, &q.join("\n"))?;
            continue;
        }
        // paragraph: up to a blank line or the start of another construct
        let mut p = vec![line.trim_end()];
        i += 1;
        while i < lines.len() {
            let n = lines[i].trim_start();
            if n.is_empty() || n.starts_with('#') || n.starts_with("- ") || n.starts_with("* ")
                || n.starts_with('>') || fence_of(n).is_some() {
                break;
            }
            p.push(lines[i].trim_end());
            i += 1;
        }
        ws.append(page, Kind::Paragraph, &p.join("\n"))?;
    }
    Ok(page)
}

fn fence_of(t: &str) -> Option<String> {
    let n = t.chars().take_while(|c| *c == '`').count();
    (n >= 3).then(|| "`".repeat(n))
}

fn longest_run(s: &str, c: char) -> usize {
    let (mut best, mut cur) = (0, 0);
    for ch in s.chars() {
        cur = if ch == c { cur + 1 } else { 0 };
        best = best.max(cur);
    }
    best
}

fn split_front_matter(md: &str) -> (Option<String>, &str) {
    let Some(rest) = md.strip_prefix("---\n") else { return (None, md) };
    let Some(end) = rest.find("\n---\n").map(|e| (e, e + 5)).or_else(|| rest.strip_suffix("\n---").map(|r| (r.len(), rest.len()))) else {
        return (None, md);
    };
    let (fm, body) = (&rest[..end.0], &rest[end.1..]);
    let title = fm.lines().find_map(|l| l.strip_prefix("title:")).map(|v| unyaml_str(v.trim()));
    (title, body)
}

/// Quotes a title only when YAML needs it.
fn yaml_str(s: &str) -> String {
    let plain = !s.is_empty()
        && !s.starts_with(|c: char| "-?:,[]{}#&*!|>'\"%@`".contains(c) || c.is_whitespace())
        && !s.ends_with(char::is_whitespace)
        && !s.contains(": ") && !s.contains(" #") && !s.contains('\n');
    if plain { s.to_string() } else { format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")) }
}

fn unyaml_str(s: &str) -> String {
    if let Some(inner) = s.strip_prefix('"').and_then(|r| r.strip_suffix('"')) {
        inner.replace("\\\"", "\"").replace("\\\\", "\\")
    } else if let Some(inner) = s.strip_prefix('\'').and_then(|r| r.strip_suffix('\'')) {
        inner.replace("''", "'")
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOC: &str = "---\ntitle: \"Plans: M0\"\n---\n\n# Goals\n\nShip the foundation\nthis week.\n\n- [ ] block tree\n- [x] plan\n- loose idea\n\n> quoted\n> twice\n\n```rust\nfn main() {}\n```\n";

    fn kinds(ws: &Workspace, p: BlockId) -> Vec<Kind> {
        ws.children(p).into_iter().map(|b| ws.kind(b)).collect()
    }

    #[test]
    fn reads_every_block_kind() {
        let ws = Workspace::new();
        let p = from_markdown(&ws, DOC, "fallback").unwrap();
        assert_eq!(ws.text(p), "Plans: M0");
        assert_eq!(kinds(&ws, p), vec![
            Kind::Heading(1), Kind::Paragraph, Kind::Todo { checked: false }, Kind::Todo { checked: true },
            Kind::Bullet, Kind::Quote, Kind::Code { lang: "rust".into() },
        ]);
        let c = ws.children(p);
        assert_eq!(ws.text(c[1]), "Ship the foundation\nthis week.");
        assert_eq!(ws.text(c[5]), "quoted\ntwice");
        assert_eq!(ws.text(c[6]), "fn main() {}");
    }

    #[test]
    fn writes_canonical_markdown_that_reads_back() {
        let ws = Workspace::new();
        let p = from_markdown(&ws, DOC, "x").unwrap();
        let md = to_markdown(&ws, p);
        assert_eq!(md, DOC, "canonical input comes back byte for byte");
        let ws2 = Workspace::new();
        let p2 = from_markdown(&ws2, &md, "x").unwrap();
        assert_eq!(kinds(&ws2, p2), kinds(&ws, p));
    }

    #[test]
    fn forgiving_input() {
        let ws = Workspace::new();
        let p = from_markdown(&ws, "* [X] done\n+ item\n####### not a heading\n", "untitled").unwrap();
        assert_eq!(ws.text(p), "untitled");
        assert_eq!(kinds(&ws, p), vec![Kind::Todo { checked: true }, Kind::Bullet, Kind::Paragraph]);
    }

    #[test]
    fn code_with_backticks_inside_gets_a_longer_fence() {
        let ws = Workspace::new();
        let p = ws.new_page("p").unwrap();
        ws.append(p, Kind::Code { lang: "md".into() }, "```\nnested\n```").unwrap();
        let md = to_markdown(&ws, p);
        assert!(md.contains("````md\n"));
        let ws2 = Workspace::new();
        let p2 = from_markdown(&ws2, &md, "p").unwrap();
        assert_eq!(ws2.text(ws2.children(p2)[0]), "```\nnested\n```");
    }
}
