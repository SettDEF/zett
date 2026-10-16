//! The folder mirror: a workspace on disk is a folder of Markdown files, one
//! page per file. Loading reads every `.md` into the block tree; saving writes
//! each page back to its file (named from its title when it is new).
//!
//! Behind the `fs` feature: the WASM build has no file system.

use crate::blocks::{BlockId, Result, Workspace};
use crate::markdown;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct Folder {
    pub dir: PathBuf,
    pub ws: Workspace,
    /// which file each page came from (or will be written to)
    pub files: HashMap<BlockId, PathBuf>,
    /// per page: the file's change time and content as zett last read or wrote it,
    /// so a change on disk by someone else can be told from zett's own
    synced: std::cell::RefCell<HashMap<BlockId, (Option<std::time::SystemTime>, String)>>,
}

/// A file name for a title: lowercase words joined by '-', safe on every system.
pub fn slug(title: &str) -> String {
    let s: String = title
        .chars()
        .map(|c| if c.is_alphanumeric() { c.to_lowercase().next().unwrap_or(c) } else { '-' })
        .collect();
    let s = s.split('-').filter(|p| !p.is_empty()).collect::<Vec<_>>().join("-");
    if s.is_empty() { "untitled".into() } else { s }
}

impl Folder {
    /// Reads every `.md` file in `dir` (not recursive yet), sorted by name.
    pub fn open(dir: impl AsRef<Path>) -> std::io::Result<Folder> {
        let dir = dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&dir)?;
        let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)?
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e == "md") && p.is_file())
            .collect();
        paths.sort();
        let ws = Workspace::new();
        let mut files = HashMap::new();
        let mut synced = HashMap::new();
        for p in paths {
            let text = std::fs::read_to_string(&p)?;
            let stem = p.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
            let page = markdown::from_markdown(&ws, &text, &stem).map_err(|e| std::io::Error::other(e.to_string()))?;
            synced.insert(page, (mtime(&p), text));
            files.insert(page, p);
        }
        Ok(Folder { dir, ws, files, synced: std::cell::RefCell::new(synced) })
    }

    /// A new page, with the file it will be saved to.
    pub fn new_page(&mut self, title: &str) -> Result<BlockId> {
        let page = self.ws.new_page(title)?;
        let mut name = slug(title);
        let mut n = 1;
        while self.dir.join(format!("{name}.md")).exists() || self.files.values().any(|p| p.file_stem().is_some_and(|s| s == name.as_str())) {
            n += 1;
            name = format!("{}-{n}", slug(title));
        }
        self.files.insert(page, self.dir.join(format!("{name}.md")));
        Ok(page)
    }

    pub fn path_of(&self, page: BlockId) -> Option<&Path> {
        self.files.get(&page).map(|p| p.as_path())
    }

    /// Writes one page to its file, through a temporary file.
    pub fn save_page(&self, page: BlockId) -> std::io::Result<()> {
        let Some(path) = self.files.get(&page) else { return Err(std::io::Error::other("page has no file")) };
        let md = markdown::to_markdown(&self.ws, page);
        // unchanged files are not rewritten (no needless mtime churn for watchers)
        if std::fs::read_to_string(path).is_ok_and(|old| old == md) {
            return Ok(());
        }
        let tmp = path.with_extension("md.zett-tmp");
        std::fs::write(&tmp, &md)?;
        std::fs::rename(tmp, path)?;
        self.synced_set(page, mtime(path), md);
        Ok(())
    }

    fn synced_set(&self, page: BlockId, t: Option<std::time::SystemTime>, md: String) {
        // interior: saving takes &self (the window saves from many places)
        self.synced.borrow_mut().insert(page, (t, md));
    }

    /// Takes in what changed on disk since zett last read or wrote it: pages
    /// edited elsewhere (Neovim, `zett add`, a git pull) are reloaded, new files
    /// become pages, deleted files go. Returns whether anything changed.
    /// Cheap when nothing did: one stat per file.
    pub fn sync_from_disk(&mut self) -> bool {
        let mut changed = false;
        let pages: Vec<(BlockId, PathBuf)> = self.files.iter().map(|(b, p)| (*b, p.clone())).collect();
        for (page, path) in pages {
            let now = mtime(&path);
            let (seen_t, seen_md) = self.synced.borrow().get(&page).cloned().unwrap_or((None, String::new()));
            if now.is_none() {
                if path.exists() { continue; }
                // deleted elsewhere
                let _ = self.ws.remove(page);
                self.files.remove(&page);
                self.synced.borrow_mut().remove(&page);
                changed = true;
                continue;
            }
            if now == seen_t { continue; }
            let Ok(text) = std::fs::read_to_string(&path) else { continue };
            if text != seen_md {
                let _ = markdown::replace_page(&self.ws, page, &text);
                changed = true;
            }
            self.synced.borrow_mut().insert(page, (now, text));
        }
        // files that appeared
        let known: std::collections::HashSet<PathBuf> = self.files.values().cloned().collect();
        if let Ok(rd) = std::fs::read_dir(&self.dir) {
            for p in rd.flatten().map(|e| e.path()) {
                if p.extension().is_some_and(|e| e == "md") && p.is_file() && !known.contains(&p) {
                    let Ok(text) = std::fs::read_to_string(&p) else { continue };
                    let stem = p.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
                    if let Ok(page) = markdown::from_markdown(&self.ws, &text, &stem) {
                        self.synced.borrow_mut().insert(page, (mtime(&p), text));
                        self.files.insert(page, p);
                        changed = true;
                    }
                }
            }
        }
        changed
    }

    pub fn save_all(&self) -> std::io::Result<()> {
        for p in self.ws.pages() {
            if self.files.contains_key(&p) {
                self.save_page(p)?;
            }
        }
        Ok(())
    }

    /// A page by its title, or by its file name without `.md`.
    pub fn find(&self, name: &str) -> Option<BlockId> {
        self.ws.find_page(name).or_else(|| {
            self.files.iter().find(|(_, p)| p.file_stem().is_some_and(|s| s.to_string_lossy() == name)).map(|(id, _)| *id)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Kind;

    fn tmpdir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("zett-folder-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    #[test]
    fn slugs() {
        assert_eq!(slug("Plans: M0 & more"), "plans-m0-more");
        assert_eq!(slug("   "), "untitled");
        assert_eq!(slug("Überblick"), "überblick");
    }

    #[test]
    fn folder_roundtrip() {
        let d = tmpdir("rt");
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("ideas.md"), "---\ntitle: Ideas\n---\n\n# Big\n\n- [ ] build zett\n").unwrap();
        std::fs::write(d.join("notes.txt"), "not markdown").unwrap();
        let mut f = Folder::open(&d).unwrap();
        assert_eq!(f.ws.pages().len(), 1, "only .md files are pages");
        let ideas = f.find("Ideas").unwrap();
        assert_eq!(f.find("ideas"), Some(ideas), "by file name too");
        f.ws.append(ideas, Kind::Paragraph, "more").unwrap();
        let fresh = f.new_page("Fresh Page").unwrap();
        f.ws.append(fresh, Kind::Bullet, "one").unwrap();
        f.save_all().unwrap();
        assert!(d.join("fresh-page.md").exists());
        let again = Folder::open(&d).unwrap();
        let p = again.find("Ideas").unwrap();
        assert_eq!(again.ws.children(p).len(), 3);
        std::fs::remove_dir_all(&d).unwrap();
    }
}

fn mtime(p: &Path) -> Option<std::time::SystemTime> {
    std::fs::metadata(p).and_then(|m| m.modified()).ok()
}

#[cfg(test)]
mod sync_tests {
    use super::*;
    use crate::Kind;

    #[test]
    fn picks_up_changes_made_elsewhere_but_not_its_own() {
        let d = std::env::temp_dir().join(format!("zett-sync-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("a.md"), "---\ntitle: A\n---\n\none\n").unwrap();
        let mut f = Folder::open(&d).unwrap();
        let a = f.find("A").unwrap();
        // zett's own save: not a change from outside
        f.ws.append(a, Kind::Paragraph, "two").unwrap();
        f.save_page(a).unwrap();
        assert!(!f.sync_from_disk(), "its own write is not reloaded");
        // someone else edits the file (mtime must move on)
        std::thread::sleep(std::time::Duration::from_millis(20));
        std::fs::write(d.join("a.md"), "---\ntitle: A\n---\n\nfrom neovim\n").unwrap();
        assert!(f.sync_from_disk());
        assert_eq!(f.ws.children(a).iter().map(|b| f.ws.text(*b)).collect::<Vec<_>>(), vec!["from neovim"]);
        // a new file, then a deleted one
        std::fs::write(d.join("b.md"), "# B\n").unwrap();
        assert!(f.sync_from_disk());
        assert!(f.find("b").is_some());
        std::fs::remove_file(d.join("a.md")).unwrap();
        assert!(f.sync_from_disk());
        assert!(f.find("A").is_none());
        std::fs::remove_dir_all(&d).unwrap();
    }
}
