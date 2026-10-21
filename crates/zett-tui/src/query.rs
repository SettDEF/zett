//! Live query blocks: a fenced ```query block in a note whose results are drawn under it and kept
//! current (a 10 s cache), searched in the project's files: open tasks, TODOs, a text, file names,
//! recent files, backlinks. `C-c C-o` on the block lists them in the minibuffer to jump to one.
//!
//!   tasks [done|all]   - [ ] lines in Markdown files (done: - [x])
//!   todo               TODO / FIXME / XXX in any text file
//!   grep TEXT          lines with TEXT (any case unless TEXT has capitals)
//!   files GLOB         file names matching GLOB (* ?), newest first
//!   recent [N(d|h)]    files changed in the last N days or hours (7d), newest first
//!   links NAME         lines linking to NAME: [[NAME]] or (NAME.md)
//! with `in DIR` (under the project) and `limit N` anywhere.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Hit {
    pub path: PathBuf,
    /// 1-based; 0 for a file-only hit
    pub line: usize,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Kind {
    Tasks { done: Option<bool> },
    Todo,
    Grep(String),
    Files(String),
    Recent(Duration),
    Links(String),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Query {
    pub kind: Kind,
    pub dir: Option<String>,
    pub limit: usize,
}

/// One block's text as a query; None with a reason when it is not one.
pub(crate) fn parse(text: &str) -> Result<Query, String> {
    // " # …" ends a line's words (a comment)
    let joined = text.lines().map(|l| l.split(" #").next().unwrap_or(l)).collect::<Vec<_>>().join(" ");
    let words: Vec<&str> = joined.split_whitespace().collect();
    let (mut dir, mut limit, mut rest) = (None, 50usize, Vec::new());
    let mut i = 0;
    while i < words.len() {
        match words[i] {
            "in" if i + 1 < words.len() => { dir = Some(words[i + 1].trim_end_matches('/').to_string()); i += 2; }
            "limit" if i + 1 < words.len() => { limit = words[i + 1].parse().map_err(|_| format!("limit wants a number, not {}", words[i + 1]))?; i += 2; }
            w => { rest.push(w); i += 1; }
        }
    }
    let arg = rest.get(1..).map(|r| r.join(" ")).unwrap_or_default();
    let need = |what: &str| if arg.is_empty() { Err(format!("{} needs {what}", rest[0])) } else { Ok(arg.clone()) };
    let kind = match rest.first().copied() {
        None => return Err("an empty query: tasks, todo, grep TEXT, files GLOB, recent 7d or links NAME".into()),
        Some("tasks") => Kind::Tasks { done: match arg.as_str() { "" | "open" => Some(false), "done" => Some(true), "all" => None, x => return Err(format!("tasks open, done or all, not {x}")) } },
        Some("todo") => Kind::Todo,
        Some("grep") => Kind::Grep(need("a text")?),
        Some("files") => Kind::Files(need("a pattern like *.md")?),
        Some("recent") => {
            let a = if arg.is_empty() { "7d".to_string() } else { arg.clone() };
            let (n, unit) = a.split_at(a.len().saturating_sub(1));
            let n: u64 = n.parse().map_err(|_| format!("recent wants 7d or 12h, not {a}"))?;
            Kind::Recent(Duration::from_secs(n * if unit == "h" { 3600 } else if unit == "d" { 86400 } else { return Err(format!("recent wants 7d or 12h, not {a}")) }))
        }
        Some("links") => Kind::Links(need("a note name")?),
        Some(x) => return Err(format!("no query called {x}: tasks, todo, grep, files, recent, links")),
    };
    Ok(Query { kind, dir, limit })
}

/// `*` and `?` over a whole name.
fn wild(p: &[u8], s: &[u8]) -> bool {
    match (p.first(), s.first()) {
        (None, None) => true,
        (Some(b'*'), _) => wild(&p[1..], s) || (!s.is_empty() && wild(p, &s[1..])),
        (Some(b'?'), Some(_)) => wild(&p[1..], &s[1..]),
        (Some(a), Some(b)) if a.eq_ignore_ascii_case(b) => wild(&p[1..], &s[1..]),
        _ => false,
    }
}

/// The project's files: no hidden folders, build output or dependencies; at most 5000.
pub(crate) fn files(root: &Path) -> Vec<(PathBuf, SystemTime)> {
    walk(root)
}

fn walk(root: &Path) -> Vec<(PathBuf, SystemTime)> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else { continue };
        for e in rd.flatten() {
            let name = e.file_name();
            let name = name.to_string_lossy();
            if name.starts_with('.') || matches!(name.as_ref(), "target" | "node_modules" | "dist" | "build") {
                continue;
            }
            let Ok(md) = e.metadata() else { continue };
            if md.is_dir() {
                stack.push(e.path());
            } else if md.is_file() && md.len() < 1 << 20 {
                out.push((e.path(), md.modified().unwrap_or(SystemTime::UNIX_EPOCH)));
                if out.len() >= 5000 {
                    return out;
                }
            }
        }
    }
    out
}

fn is_md(p: &Path) -> bool {
    p.extension().is_some_and(|e| e == "md" || e == "markdown")
}

/// The query over the files under `root` (and its `in DIR`).
pub(crate) fn run(root: &Path, q: &Query) -> Vec<Hit> {
    let base = q.dir.as_ref().map(|d| root.join(d)).unwrap_or(root.to_path_buf());
    let mut files = walk(&base);
    let mut hits = Vec::new();
    let mut by_line = |files: &[(PathBuf, SystemTime)], md_only: bool, f: &dyn Fn(&str) -> bool| {
        for (p, _) in files.iter().filter(|(p, _)| !md_only || is_md(p)) {
            let Ok(text) = std::fs::read_to_string(p) else { continue };
            for (i, l) in text.lines().enumerate() {
                if f(l) {
                    hits.push(Hit { path: p.clone(), line: i + 1, text: l.trim().chars().take(160).collect() });
                    if hits.len() >= q.limit {
                        return;
                    }
                }
            }
        }
    };
    files.sort_by(|a, b| a.0.cmp(&b.0));
    match &q.kind {
        Kind::Tasks { done } => {
            let done = *done;
            by_line(&files, true, &move |l: &str| {
                let t = l.trim_start();
                let open = t.starts_with("- [ ]") || t.starts_with("* [ ]");
                let closed = t.starts_with("- [x]") || t.starts_with("- [X]") || t.starts_with("* [x]");
                match done { Some(false) => open, Some(true) => closed, None => open || closed }
            })
        }
        Kind::Todo => by_line(&files, false, &|l: &str| ["TODO", "FIXME", "XXX"].iter().any(|w| l.contains(w))),
        Kind::Grep(t) => {
            let smart = t.chars().any(char::is_uppercase);
            let t2 = t.to_lowercase();
            let t = t.clone();
            by_line(&files, false, &move |l: &str| if smart { l.contains(&t) } else { l.to_lowercase().contains(&t2) })
        }
        Kind::Links(n) => {
            // in notes and in code (a comment that says [[design]]); a reference ![[n#part]] counts too
            let (a, b) = (format!("[[{n}]]"), format!("({n}.md)"));
            let (c, e) = (format!("[[{n}|"), format!("[[{n}#"));
            by_line(&files, false, &move |l: &str| l.contains(&a) || l.contains(&b) || l.contains(&c) || l.contains(&e))
        }
        Kind::Files(g) => {
            files.sort_by(|a, b| b.1.cmp(&a.1));
            for (p, _) in files.iter().filter(|(p, _)| p.file_name().is_some_and(|n| wild(g.as_bytes(), n.to_string_lossy().as_bytes()))).take(q.limit) {
                hits.push(Hit { path: p.clone(), line: 0, text: String::new() });
            }
        }
        Kind::Recent(age) => {
            let since = SystemTime::now() - *age;
            files.sort_by(|a, b| b.1.cmp(&a.1));
            for (p, t) in files.iter().filter(|(_, t)| *t >= since).take(q.limit) {
                hits.push(Hit { path: p.clone(), line: 0, text: ago(*t) });
            }
        }
    }
    hits
}

/// "3 min ago", "2 h ago", "4 d ago"
pub(crate) fn ago(t: SystemTime) -> String {
    let s = SystemTime::now().duration_since(t).map(|d| d.as_secs()).unwrap_or(0);
    match s {
        0..=59 => "just now".into(),
        60..=3599 => format!("{} min ago", s / 60),
        3600..=86399 => format!("{} h ago", s / 3600),
        _ => format!("{} d ago", s / 86400),
    }
}

/// The ```query blocks in a buffer: (the opening fence's line, the closing one's, the text between).
pub(crate) fn blocks(lines: &[String]) -> Vec<(usize, usize, String)> {
    let mut out = Vec::new();
    let mut open: Option<usize> = None;
    for (i, l) in lines.iter().enumerate() {
        let t = l.trim_start();
        match open {
            None if t.starts_with("```query") => open = Some(i),
            Some(a) if t.starts_with("```") => {
                out.push((a, i, lines[a + 1..i].join("\n")));
                open = None;
            }
            _ => {}
        }
    }
    out
}

/// Results by query text and root, each kept 10 s.
#[derive(Default)]
pub(crate) struct Cache(std::collections::HashMap<(PathBuf, String), (Instant, Result<Vec<Hit>, String>)>);

impl Cache {
    pub fn get(&mut self, root: &Path, text: &str) -> (Result<Vec<Hit>, String>, Duration) {
        let key = (root.to_path_buf(), text.to_string());
        if let Some((at, r)) = self.0.get(&key) {
            if at.elapsed() < Duration::from_secs(10) {
                return (r.clone(), at.elapsed());
            }
        }
        let r = parse(text).map(|q| run(root, &q));
        self.0.insert(key, (Instant::now(), r.clone()));
        (r, Duration::ZERO)
    }
}

/// How many results a block shows under it (the rest: C-c C-o)
pub(crate) const SHOWN: usize = 12;

/// What a buffer's query blocks draw: for each, the line it goes after and its rows as
/// (text, hl group) pieces. Only Markdown buffers have them.
pub(crate) fn rows(cache: &mut Cache, root: &Path, lines: &[String]) -> Vec<(usize, Vec<Vec<(String, &'static str)>>)> {
    blocks(lines)
        .into_iter()
        .map(|(_, close, text)| {
            let (r, age) = cache.get(root, &text);
            let mut out = Vec::new();
            match r {
                Err(e) => out.push(vec![("┊ ".into(), "QueryHead"), (format!("query: {e}"), "Error")]),
                Ok(hits) => {
                    let what = text.split_whitespace().next().unwrap_or("query");
                    let when = if age.as_secs() < 2 { "now".to_string() } else { format!("{} s ago", age.as_secs()) };
                    out.push(vec![("┊ ".into(), "QueryHead"), (format!("{} {} · {when}", hits.len(), if hits.len() == 1 { "result" } else { "results" }), "QueryHead"), (format!("  ({what})"), "QueryHead")]);
                    for h in hits.iter().take(SHOWN) {
                        let rel = h.path.strip_prefix(root).unwrap_or(&h.path).display().to_string();
                        let at = if h.line > 0 { format!("{rel}:{}", h.line) } else { rel };
                        out.push(vec![("┊ ".into(), "QueryHead"), (at, "QueryPath"), (if h.text.is_empty() { String::new() } else { format!("  {}", h.text) }, "QueryHit")]);
                    }
                    if hits.len() > SHOWN {
                        out.push(vec![("┊ ".into(), "QueryHead"), (format!("… {} more · C-c C-o lists them all", hits.len() - SHOWN), "QueryHead")]);
                    }
                }
            }
            (close, out)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree() -> PathBuf {
        let d = std::env::temp_dir().join(format!("zett-query-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(d.join("notes")).unwrap();
        std::fs::create_dir_all(d.join("target")).unwrap();
        std::fs::write(d.join("notes/a.md"), "# A\n- [ ] write the query blocks\n- [x] the outline\nsee [[b]]\n").unwrap();
        std::fs::write(d.join("notes/b.md"), "# B\n- [ ] sync\n").unwrap();
        std::fs::write(d.join("main.rs"), "fn main() {} // TODO: the rest\n").unwrap();
        std::fs::write(d.join("target/junk.md"), "- [ ] never seen\n").unwrap();
        d
    }

    #[test]
    fn parses_each_kind_and_its_options() {
        assert_eq!(parse("tasks").unwrap().kind, Kind::Tasks { done: Some(false) });
        assert_eq!(parse("tasks done in notes limit 3").unwrap(), Query { kind: Kind::Tasks { done: Some(true) }, dir: Some("notes".into()), limit: 3 });
        assert_eq!(parse("grep hello world").unwrap().kind, Kind::Grep("hello world".into()));
        assert_eq!(parse("recent 12h").unwrap().kind, Kind::Recent(Duration::from_secs(12 * 3600)));
        assert!(parse("recent soon").is_err());
        assert!(parse("files").is_err());
        assert!(parse("frobnicate").is_err());
        assert!(parse("").is_err());
    }

    #[test]
    fn finds_open_tasks_todos_links_and_files_but_not_in_build_output() {
        let d = tree();
        let tasks = run(&d, &parse("tasks").unwrap());
        assert_eq!(tasks.iter().map(|h| h.text.as_str()).collect::<Vec<_>>(), ["- [ ] write the query blocks", "- [ ] sync"]);
        assert_eq!(tasks[0].line, 2);
        assert_eq!(run(&d, &parse("tasks done").unwrap()).len(), 1);
        assert_eq!(run(&d, &parse("todo").unwrap())[0].path, d.join("main.rs"));
        assert_eq!(run(&d, &parse("links b").unwrap())[0].text, "see [[b]]");
        assert_eq!(run(&d, &parse("files *.MD").unwrap()).len(), 2);
        assert_eq!(run(&d, &parse("grep QUERY").unwrap()).len(), 0, "capitals match exactly");
        assert_eq!(run(&d, &parse("grep query").unwrap()).len(), 1);
        assert_eq!(run(&d, &parse("tasks limit 1").unwrap()).len(), 1);
        assert_eq!(run(&d, &parse("recent 1d").unwrap()).len(), 3);
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn finds_the_blocks_and_their_text() {
        let l: Vec<String> = ["# x", "```query", "tasks", "in notes", "```", "```rust", "fn a() {}", "```"].iter().map(|s| s.to_string()).collect();
        assert_eq!(blocks(&l), vec![(1, 4, "tasks\nin notes".to_string())]);
    }
}

impl crate::App {
    /// C-c C-o: the results of the query block the cursor is in (or on), all of them, in the minibuffer.
    pub(crate) fn query_open(&mut self) {
        // on a block reference: open what it names
        if self.ref_open() {
            return;
        }
        let d = &self.docs[self.cur];
        let at = d.ed.cursor().line;
        let lines: Vec<String> = (0..d.ed.buf.len_lines()).map(|i| d.ed.buf.line(i).trim_end_matches('\n').to_string()).collect();
        let Some((_, _, text)) = blocks(&lines).into_iter().find(|(a, b, _)| (*a..=*b).contains(&at)) else {
            return self.say("not in a ```query block");
        };
        let root = std::fs::canonicalize(&d.name).ok().and_then(|p| crate::doom::project_of(&p)).unwrap_or(self.root.clone());
        match self.queries.get(&root, &text).0 {
            Err(e) => self.say(format!("query: {e}")),
            Ok(hits) if hits.is_empty() => self.say("the query finds nothing"),
            Ok(hits) => {
                let items = hits
                    .iter()
                    // relative to the project when it is the one zett is in (the picker resolves those), else whole
                    .map(|h| crate::minibuf::Cand::plain(format!("{}:{}: {}", h.path.strip_prefix(&self.root).unwrap_or(&h.path).display(), h.line.max(1), h.text)))
                    .collect();
                self.pick_list(crate::doom::Pick::Query, &format!("{} results", hits.len()), items);
            }
        }
    }
}
