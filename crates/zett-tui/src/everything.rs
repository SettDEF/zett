//! Search everything: the project's files as they are (ripgrep) and the text they once had, from
//! every saved undo history (what a change deleted or replaced, and when). Enter on a past hit opens
//! the file in time travel at the state that still has the text, to look at it or take it back.

use super::*;

/// The histories' index: "hash<TAB>path" per line (the history files are named by the hash).
fn index_file() -> PathBuf {
    doom::state_dir().join("history").join("undo").join("index")
}

/// Notes a file's history in the index (once).
pub(crate) fn remember(hash: &str, path: &Path) {
    let f = index_file();
    let have = std::fs::read_to_string(&f).unwrap_or_default();
    let line = format!("{hash}\t{}", path.display());
    if have.lines().any(|l| l == line) {
        return;
    }
    let _ = std::fs::write(&f, format!("{have}{line}\n"));
}

/// The histories of files under `root`: (path, its history file).
fn histories(root: &Path) -> Vec<(PathBuf, PathBuf)> {
    let dir = index_file().parent().map(Path::to_path_buf).unwrap_or_default();
    std::fs::read_to_string(index_file()).unwrap_or_default().lines()
        .filter_map(|l| l.split_once('\t'))
        .map(|(h, p)| (PathBuf::from(p), dir.join(h)))
        .filter(|(p, h)| p.starts_with(root) && h.is_file())
        .collect()
}

/// Past text with `q` in it, newest first per file: "⟲ path@state · when: line".
pub(crate) fn past(root: &Path, q: &str, max: usize) -> Vec<String> {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
    let mut out = Vec::new();
    for (p, h) in histories(root) {
        let Ok(bytes) = std::fs::read(&h) else { continue };
        let rel = p.strip_prefix(root).unwrap_or(&p).display().to_string();
        for (node, t, line) in Editor::history_search(&bytes, q, 20) {
            out.push(format!("⟲ {rel}@{node} · {}: {line}", travel::when(t, now)));
            if out.len() >= max {
                return out;
            }
        }
    }
    out
}

/// "⟲ path@state · …" → (path, state)
pub(crate) fn parse(item: &str) -> Option<(String, usize)> {
    let rest = item.strip_prefix("⟲ ")?;
    let head = rest.split(" · ").next()?;
    let (p, n) = head.rsplit_once('@')?;
    Some((p.to_string(), n.parse().ok()?))
}

impl App {
    /// Enter on a past hit: the file, in time travel at the state before the change that removed it.
    pub(crate) fn open_past(&mut self, root: &Path, item: &str) -> bool {
        let Some((rel, node)) = parse(item) else { return false };
        self.open(&root.join(&rel));
        let states = self.docs[self.cur].ed.undo_states();
        let Some(&(before, _)) = states.get(node) else {
            self.say("that history no longer fits the file");
            return true;
        };
        self.travel_start();
        self.travel_to(before);
        true
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_past_hit_names_its_file_and_state() {
        assert_eq!(super::parse("⟲ notes/a b.md@12 · 3 h ago: the line · with dots"), Some(("notes/a b.md".into(), 12)));
        assert_eq!(super::parse("src/x.rs:3: text"), None);
    }
}
