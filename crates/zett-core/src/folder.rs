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
        for p in paths {
            let text = std::fs::read_to_string(&p)?;
            let stem = p.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
            let page = markdown::from_markdown(&ws, &text, &stem).map_err(|e| std::io::Error::other(e.to_string()))?;
            files.insert(page, p);
        }
        Ok(Folder { dir, ws, files })
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
        std::fs::write(&tmp, md)?;
        std::fs::rename(tmp, path)
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
