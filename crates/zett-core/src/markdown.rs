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
    let blocks = ws.children(page);
    let mut out = String::with_capacity(64 + blocks.len() * 48);
    out += &format!("---\ntitle: {}\n", yaml_str(&ws.text(page)));
    for (k, v) in ws.props(page) {
        out += &format!("{k}: {}\n", yaml_str(&v));
    }
    out += "---\n";
    let mut prev_list = false;
    for b in blocks {
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
            // a list item with its nested items under it, two spaces per level
            Kind::Todo { .. } | Kind::Bullet => write_item(ws, b, 0, &mut out),
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

/// One list item and, indented under it, the items nested in it.
fn write_item(ws: &Workspace, b: BlockId, depth: usize, out: &mut String) {
    let pad = "  ".repeat(depth);
    match ws.kind(b) {
        Kind::Todo { checked } => *out += &format!("{pad}- [{}] {}\n", if checked { "x" } else { " " }, ws.text(b)),
        _ => *out += &format!("{pad}- {}\n", ws.text(b)),
    }
    for c in ws.children(b) {
        write_item(ws, c, depth + 1, out);
    }
}

/// Reads Markdown into a new page and returns it. A file without front matter
/// takes `fallback_title` (the file name, for the folder mirror).
pub fn from_markdown(ws: &Workspace, md: &str, fallback_title: &str) -> Result<BlockId> {
    let (title, props, body) = split_front_matter(md);
    let page = ws.new_page(title.as_deref().unwrap_or(fallback_title))?;
    ws.set_props(page, &props)?;
    read_blocks(ws, page, body)?;
    Ok(page)
}

/// Replaces a page's title and blocks with what the Markdown says (after an
/// edit of its text). The page keeps its id, so links to it stay valid.
pub fn replace_page(ws: &Workspace, page: BlockId, md: &str) -> Result<()> {
    let (title, props, body) = split_front_matter(md);
    if let Some(t) = title {
        ws.set_text(page, &t)?;
    }
    ws.set_props(page, &props)?;
    for b in ws.children(page) {
        ws.remove(b)?;
    }
    read_blocks(ws, page, body)
}

fn read_blocks(ws: &Workspace, page: BlockId, body: &str) -> Result<()> {
    let lines: Vec<&str> = body.lines().collect();
    let mut i = 0;
    // the open list items by depth (2 spaces or a tab per level): an indented item goes under the
    // last item one level up; anything that is not a list item ends the list
    let mut items: Vec<(usize, BlockId)> = Vec::new();
    while i < lines.len() {
        let line = lines[i];
        let t = line.trim_start();
        if t.is_empty() {
            i += 1;
            continue;
        }
        if !(t.starts_with("- ") || t.starts_with("* ") || t.starts_with("+ ")) {
            items.clear();
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
            let indent = line.chars().take_while(|c| *c == ' ' || *c == '\t').map(|c| if c == '\t' { 2 } else { 1 }).sum::<usize>() / 2;
            while items.last().is_some_and(|(d, _)| *d >= indent) { items.pop(); }
            let parent = items.last().map(|(_, b)| *b).unwrap_or(page);
            let depth = items.len();
            let lower = rest.to_ascii_lowercase();
            let id = if lower.starts_with("[ ] ") || lower.starts_with("[x] ") || lower == "[ ]" || lower == "[x]" {
                let checked = lower.starts_with("[x]");
                ws.append(parent, Kind::Todo { checked }, rest.get(4..).unwrap_or("").trim_end())?
            } else {
                ws.append(parent, Kind::Bullet, rest.trim_end())?
            };
            items.push((depth, id));
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
    Ok(())
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

/// The title, the other `key: value` lines (the page's properties, in order), and the body.
fn split_front_matter(md: &str) -> (Option<String>, Vec<(String, String)>, &str) {
    let Some(rest) = md.strip_prefix("---\n") else { return (None, Vec::new(), md) };
    let Some(end) = rest.find("\n---\n").map(|e| (e, e + 5)).or_else(|| rest.strip_suffix("\n---").map(|r| (r.len(), rest.len()))) else {
        return (None, Vec::new(), md);
    };
    let (fm, body) = (&rest[..end.0], &rest[end.1..]);
    let mut title = None;
    let mut props = Vec::new();
    for l in fm.lines() {
        let Some((k, v)) = l.split_once(':') else { continue };
        let (k, v) = (k.trim(), unyaml_str(v.trim()));
        if k == "title" { title = Some(v) } else if !k.is_empty() && !k.starts_with(' ') { props.push((k.to_string(), v)) }
    }
    (title, props, body)
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

#[cfg(test)]
mod replace_tests {
    use super::*;
    use crate::Kind;

    #[test]
    fn replace_keeps_the_page_id() {
        let ws = Workspace::new();
        let p = from_markdown(&ws, "# a\n\nold\n", "t").unwrap();
        replace_page(&ws, p, "---\ntitle: New\n---\n\n- [x] done\n").unwrap();
        assert_eq!(ws.text(p), "New");
        assert_eq!(ws.children(p).iter().map(|b| ws.kind(*b)).collect::<Vec<_>>(), vec![Kind::Todo { checked: true }]);
        assert_eq!(ws.pages(), vec![p]);
    }
}

#[cfg(test)]
mod nested_tests {
    use super::*;

    #[test]
    fn nested_lists_round_trip() {
        let md = "---\ntitle: N\n---\n\n- top\n  - child\n    - [ ] grandchild todo\n  - [x] second child\n- next top\n\nA paragraph.\n\n- new list\n";
        let ws = Workspace::new();
        let p = from_markdown(&ws, md, "N").unwrap();
        let top = ws.children(p);
        assert_eq!(top.len(), 4, "two items of the first list, the paragraph, the new list");
        assert_eq!(ws.text(top[0]), "top");
        let kids = ws.children(top[0]);
        assert_eq!(kids.iter().map(|b| ws.text(*b)).collect::<Vec<_>>(), vec!["child", "second child"]);
        assert_eq!(ws.text(ws.children(kids[0])[0]), "grandchild todo");
        assert!(ws.children(top[3]).is_empty(), "a paragraph ends the list: the new list starts at the top");
        assert_eq!(to_markdown(&ws, p), md, "written back the same");
    }
}

#[cfg(test)]
mod props_tests {
    use super::*;

    #[test]
    fn page_properties_round_trip() {
        let md = "---\ntitle: P\nicon: 🌀\nstatus: draft\ntags: zett, ui\n---\n\nBody.\n";
        let ws = Workspace::new();
        let p = from_markdown(&ws, md, "P").unwrap();
        assert_eq!(ws.props(p), vec![("icon".into(), "🌀".into()), ("status".into(), "draft".into()), ("tags".into(), "zett, ui".into())]);
        assert_eq!(to_markdown(&ws, p), md);
        ws.set_prop(p, "status", "done").unwrap();
        ws.set_prop(p, "icon", "").unwrap();
        assert_eq!(ws.props(p), vec![("status".into(), "done".into()), ("tags".into(), "zett, ui".into())]);
    }
}
