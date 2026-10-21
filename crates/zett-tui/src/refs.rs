//! Live block references: a line `![[target]]` or `![[target#section]]` draws that note or file (or
//! one section of it: a Markdown heading, or a block of code such as `fn main`) under itself, read
//! again every 3 s, so the reference always shows what is there now. A note's end lists what links
//! to it (`[[name]]` in notes and in code comments). C-c C-o on a reference opens it there.

use crate::query::{self, Cache};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use zett_edit::outline::{self, OutlineKind};

/// How many lines a reference shows (the rest: open it)
const SHOWN: usize = 15;

/// `![[target#section]]` alone on a line: (target, section)
pub(crate) fn embed(line: &str) -> Option<(String, Option<String>)> {
    let inner = line.trim().strip_prefix("![[")?.strip_suffix("]]")?;
    let inner = inner.split('|').next().unwrap_or(inner);
    let (t, s) = match inner.split_once('#') {
        Some((t, s)) => (t.trim(), Some(s.trim().to_string()).filter(|s| !s.is_empty())),
        None => (inner.trim(), None),
    };
    (!t.is_empty()).then(|| (t.to_string(), s))
}

/// The file a target names: a path from the note's folder or the project, with or without .md,
/// else the note of that name anywhere in the project.
pub(crate) fn resolve(root: &Path, here: &Path, target: &str) -> Option<PathBuf> {
    for base in [here, root] {
        for cand in [base.join(target), base.join(format!("{target}.md"))] {
            if cand.is_file() {
                return Some(cand);
            }
        }
    }
    let want = target.to_lowercase();
    query::files(root).into_iter().map(|(p, _)| p).find(|p| p.file_stem().is_some_and(|s| s.to_string_lossy().to_lowercase() == want))
}

fn bare(title: &str) -> String {
    title.trim_start_matches('#').trim().to_lowercase()
}

/// The lines a reference stands for: the whole file, or its section (the heading named exactly, else
/// the first one containing the words); with the line it starts at (0-based).
pub(crate) fn extract(path: &Path, section: Option<&str>) -> Result<(usize, Vec<String>), String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let lines: Vec<String> = text.lines().map(String::from).collect();
    let Some(want) = section else { return Ok((0, lines)) };
    let kind = if path.extension().is_some_and(|e| e == "md" || e == "markdown") { OutlineKind::Markdown } else { OutlineKind::Code };
    let secs = outline::sections(&lines, kind);
    let w = want.to_lowercase();
    let s = secs.iter().find(|s| bare(&s.title) == w).or_else(|| secs.iter().find(|s| bare(&s.title).contains(&w)));
    match s {
        Some(s) => Ok((s.start, lines[s.start..s.end.min(lines.len())].to_vec())),
        None => Err(format!("no section \"{want}\" in {}", path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default())),
    }
}

/// Embeds' contents by (file, section), each read again after 3 s.
#[derive(Default)]
pub(crate) struct Embeds(std::collections::HashMap<(PathBuf, Option<String>), (Instant, Result<(usize, Vec<String>), String>)>);

impl Embeds {
    fn get(&mut self, path: &Path, section: Option<&str>) -> Result<(usize, Vec<String>), String> {
        let key = (path.to_path_buf(), section.map(String::from));
        if let Some((at, r)) = self.0.get(&key) {
            if at.elapsed() < Duration::from_secs(3) {
                return r.clone();
            }
        }
        let r = extract(path, section);
        self.0.insert(key, (Instant::now(), r.clone()));
        r
    }
}

type Rows = Vec<(usize, Vec<Vec<(String, &'static str)>>)>;

/// What a note's references draw (after their line), and its backlinks (after its last line).
pub(crate) fn rows(embeds: &mut Embeds, links: &mut Cache, root: &Path, file: Option<&Path>, lines: &[String]) -> Rows {
    let here = file.and_then(|f| f.parent()).unwrap_or(root).to_path_buf();
    let mut out: Rows = Vec::new();
    for (i, l) in lines.iter().enumerate() {
        let Some((target, section)) = embed(l) else { continue };
        let mut r = Vec::new();
        match resolve(root, &here, &target).ok_or_else(|| format!("no note or file called {target}")).and_then(|p| embeds.get(&p, section.as_deref()).map(|x| (p, x))) {
            Err(e) => r.push(vec![("┃ ".into(), "RefBar"), (e, "Error")]),
            Ok((p, (start, body))) => {
                let rel = p.strip_prefix(root).unwrap_or(&p).display().to_string();
                let span = if section.is_some() { format!(" · lines {}–{}", start + 1, start + body.len()) } else { String::new() };
                r.push(vec![("┃ ".into(), "RefBar"), (format!("{rel}{}", section.as_ref().map(|s| format!(" › {s}")).unwrap_or_default()), "RefHead"), (span, "QueryHead")]);
                for b in body.iter().take(SHOWN) {
                    r.push(vec![("┃ ".into(), "RefBar"), (b.replace('\t', "    "), "RefText")]);
                }
                if body.len() > SHOWN {
                    r.push(vec![("┃ ".into(), "RefBar"), (format!("… {} more lines · C-c C-o opens it", body.len() - SHOWN), "QueryHead")]);
                }
            }
        }
        out.push((i, r));
    }
    // backlinks: what links to this note, at its end
    if let Some(stem) = file.and_then(|f| f.file_stem()).map(|s| s.to_string_lossy().to_string()) {
        if let (Ok(hits), _) = links.get(root, &format!("links {stem}")) {
            let hits: Vec<_> = hits.into_iter().filter(|h| Some(h.path.as_path()) != file).collect();
            if !hits.is_empty() && !lines.is_empty() {
                let mut r = vec![vec![("".into(), "QueryHead")], vec![("↩ ".into(), "RefBar"), (format!("Linked from {} place{}", hits.len(), if hits.len() == 1 { "" } else { "s" }), "RefHead")]];
                for h in hits.iter().take(8) {
                    let rel = h.path.strip_prefix(root).unwrap_or(&h.path).display().to_string();
                    r.push(vec![("  ".into(), "QueryHead"), (format!("{rel}:{}", h.line), "QueryPath"), (format!("  {}", h.text), "QueryHit")]);
                }
                if hits.len() > 8 {
                    r.push(vec![("  ".into(), "QueryHead"), (format!("… {} more · SPC s b lists them", hits.len() - 8), "QueryHead")]);
                }
                out.push((lines.len() - 1, r));
            }
        }
    }
    out
}

impl crate::App {
    /// C-c C-o on a reference: opens what it names, at its section.
    pub(crate) fn ref_open(&mut self) -> bool {
        let d = &self.docs[self.cur];
        let Some((target, section)) = embed(&d.ed.buf.line(d.ed.cursor().line)) else { return false };
        let file = std::fs::canonicalize(&d.name).ok();
        let root = file.as_deref().and_then(crate::doom::project_of).unwrap_or(self.root.clone());
        let here = file.as_deref().and_then(Path::parent).unwrap_or(&root).to_path_buf();
        match resolve(&root, &here, &target) {
            None => self.say(format!("no note or file called {target}")),
            Some(p) => {
                let at = section.as_deref().and_then(|s| extract(&p, Some(s)).ok()).map(|(l, _)| l + 1);
                self.open(&p);
                if let Some(l) = at {
                    self.goto_line(l);
                }
            }
        }
        true
    }

    /// SPC s b: what links to this note, in the minibuffer.
    pub(crate) fn backlinks(&mut self) {
        let d = &self.docs[self.cur];
        let file = std::fs::canonicalize(&d.name).ok();
        let Some(stem) = file.as_deref().and_then(Path::file_stem).map(|s| s.to_string_lossy().to_string()) else {
            return self.say("this buffer is not a file");
        };
        let root = file.as_deref().and_then(crate::doom::project_of).unwrap_or(self.root.clone());
        let hits: Vec<_> = self.queries.get(&root, &format!("links {stem}")).0.unwrap_or_default().into_iter().filter(|h| Some(h.path.as_path()) != file.as_deref()).collect();
        if hits.is_empty() {
            return self.say(format!("nothing links to {stem}"));
        }
        let items = hits.iter().map(|h| crate::minibuf::Cand::plain(format!("{}:{}: {}", h.path.strip_prefix(&self.root).unwrap_or(&h.path).display(), h.line, h.text))).collect();
        self.pick_list(crate::doom::Pick::Query, &format!("Linked from {} places", hits.len()), items);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_an_embed_line() {
        assert_eq!(embed("![[ideas]]"), Some(("ideas".into(), None)));
        assert_eq!(embed("  ![[notes/ideas#Next steps]]  "), Some(("notes/ideas".into(), Some("Next steps".into()))));
        assert_eq!(embed("![[ideas|shown name]]"), Some(("ideas".into(), None)));
        assert_eq!(embed("see ![[ideas]] here"), None);
        assert_eq!(embed("[[ideas]]"), None);
        assert_eq!(embed("![[]]"), None);
    }

    #[test]
    fn resolves_and_extracts_note_sections_and_code_blocks() {
        let d = std::env::temp_dir().join(format!("zett-refs-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(d.join("notes/deep")).unwrap();
        std::fs::write(d.join("notes/deep/plan.md"), "# Plan\nintro\n## Next steps\n- one\n- two\n## Later\n- three\n").unwrap();
        std::fs::write(d.join("main.rs"), "fn other() {}\n\nfn main() {\n    run();\n}\n").unwrap();
        let p = resolve(&d, &d, "plan").expect("found by name anywhere");
        assert_eq!(p, d.join("notes/deep/plan.md"));
        assert_eq!(resolve(&d, &d, "notes/deep/plan"), Some(p.clone()), "a path without .md");
        assert_eq!(extract(&p, Some("next steps")).unwrap(), (2, vec!["## Next steps".into(), "- one".into(), "- two".into()]));
        assert_eq!(extract(&p, Some("later")).unwrap().1.len(), 2);
        assert!(extract(&p, Some("nowhere")).is_err());
        let code = resolve(&d, &d, "main.rs").unwrap();
        assert_eq!(extract(&code, Some("fn main")).unwrap(), (2, vec!["fn main() {".into(), "    run();".into(), "}".into()]));
        let _ = std::fs::remove_dir_all(&d);
    }
}
