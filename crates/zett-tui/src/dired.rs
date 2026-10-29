//! Emacs's dired with wdired: a folder as a buffer of names (folders end in /), edited like any text, :w renames.
//! A line taken out is a file to the trash, done only with :w! (:w says what would go). Enter opens the name
//! under the cursor (a folder: its listing), - the folder above; renames go through temporary names, so names
//! can swap.
use super::*;
use std::collections::{HashMap, HashSet};

pub(crate) struct Dired {
    pub dir: PathBuf,
    /// the names as listed (folders with their /)
    pub names: Vec<String>,
}

/// A folder's names: folders first, then files, each by name (case left as it is); hidden ones too.
pub(crate) fn listing(dir: &Path) -> std::io::Result<Vec<String>> {
    let mut dirs = Vec::new();
    let mut files = Vec::new();
    for e in std::fs::read_dir(dir)?.flatten() {
        let n = e.file_name().to_string_lossy().to_string();
        if e.file_type().is_ok_and(|t| t.is_dir()) { dirs.push(format!("{n}/")) } else { files.push(n) }
    }
    dirs.sort_by_key(|s| s.to_lowercase());
    files.sort_by_key(|s| s.to_lowercase());
    dirs.extend(files);
    Ok(dirs)
}

/// What an edit of the listing asks for: renames (from, to) and names to the trash; or why it cannot be done.
pub(crate) fn plan(old: &[String], new: &[String]) -> Result<(Vec<(String, String)>, Vec<String>), String> {
    let clean = |s: &String| s.trim_end_matches('/').to_string();
    let new: Vec<String> = new.iter().map(|s| s.trim_end().to_string()).filter(|s| !s.is_empty()).collect();
    for n in &new {
        if n == "." || n == ".." || n.contains('\0') {
            return Err(format!("\"{n}\" cannot be a name"));
        }
    }
    let mut seen = HashSet::new();
    for n in &new {
        if !seen.insert(clean(n)) {
            return Err(format!("\"{n}\" is there twice"));
        }
    }
    if new.len() == old.len() {
        let renames = old.iter().zip(&new).filter(|(a, b)| clean(a) != clean(b)).map(|(a, b)| (clean(a), clean(b))).collect();
        return Ok((renames, Vec::new()));
    }
    if new.len() < old.len() {
        // lines taken out: every line left must be a name that was there (renaming in the same save is not clear)
        let olds: HashSet<String> = old.iter().map(clean).collect();
        if let Some(n) = new.iter().find(|n| !olds.contains(&clean(n))) {
            return Err(format!("\"{n}\": rename and delete in separate saves"));
        }
        let kept: HashSet<String> = new.iter().map(clean).collect();
        return Ok((Vec::new(), old.iter().map(clean).filter(|n| !kept.contains(n)).collect()));
    }
    Err("a line was added: new files are made with :e NAME, folders with :!mkdir".into())
}

/// Carries out the renames in `dir` (each through a temporary name first, so a ↔ b works); the first error stops it.
pub(crate) fn rename_all(dir: &Path, renames: &[(String, String)]) -> Result<usize, String> {
    for (_, to) in renames {
        let target = dir.join(to);
        let taken_by_other = target.exists() && !renames.iter().any(|(from, _)| from == to);
        if taken_by_other {
            return Err(format!("\"{to}\" exists already"));
        }
    }
    // a temporary name each (by its place in the list: unique), in the folder it comes from
    let tmp: Vec<(PathBuf, PathBuf)> = renames.iter().enumerate().map(|(i, (from, _))| {
        let a = dir.join(from);
        let t = a.with_file_name(format!(".zett-rename-{}-{i}", std::process::id()));
        (a, t)
    }).collect();
    for (a, t) in &tmp {
        std::fs::rename(a, t).map_err(|e| format!("{}: {e}", a.display()))?;
    }
    for ((_, t), (_, to)) in tmp.iter().zip(renames) {
        let b = dir.join(to);
        if let Some(p) = b.parent() {
            std::fs::create_dir_all(p).map_err(|e| format!("{}: {e}", p.display()))?;
        }
        std::fs::rename(t, &b).map_err(|e| format!("{}: {e}", b.display()))?;
    }
    Ok(renames.len())
}

impl App {
    /// C-x d, :Dired [folder]: the folder as an editable listing (here: the current file's folder).
    pub(crate) fn open_dired(&mut self, dir: Option<PathBuf>) {
        let dir = dir.or_else(|| self.docs[self.cur].abs_path().and_then(|p| p.parent().map(Path::to_path_buf))).unwrap_or_else(|| self.root.clone());
        let dir = std::fs::canonicalize(&dir).unwrap_or(dir);
        let names = match listing(&dir) {
            Ok(n) => n,
            Err(e) => return self.say(format!("{}: {e}", dir.display())),
        };
        let name = format!("{}/", dir.display());
        if let Some(i) = self.docs.iter().position(|d| d.name == name) {
            self.docs.remove(i);
        }
        let mut d = Doc::new(Editor::from_str(&names.join("\n")), name.clone(), "Plain Text".into());
        d.local.insert("buftype".into(), "acwrite".into());
        self.docs.push(d);
        self.dired.insert(name, Dired { dir, names });
        let i = self.docs.len() - 1;
        self.show_doc(i);
        self.say("edit the names, :w renames · a line taken out goes to the trash with :w! · Enter opens · - the folder above");
    }

    /// :w in a listing: the renames (and with ! the deletions), then the folder listed again.
    pub(crate) fn dired_save(&mut self, bang: bool) {
        let name = self.docs[self.cur].name.clone();
        let Some(d) = self.dired.get(&name) else { return };
        let dir = d.dir.clone();
        let text = self.docs[self.cur].ed.text();
        let new: Vec<String> = text.lines().map(str::to_string).collect();
        let (renames, gone) = match plan(&d.names, &new) {
            Ok(p) => p,
            Err(e) => return self.say(format!("nothing changed: {e}")),
        };
        if !gone.is_empty() && !bang {
            let list = gone.iter().take(5).cloned().collect::<Vec<_>>().join(", ");
            return self.say(format!("{} to the trash ({list}{}): :w! to do it", gone.len(), if gone.len() > 5 { ", …" } else { "" }));
        }
        let mut said = Vec::new();
        match rename_all(&dir, &renames) {
            Ok(0) => {}
            Ok(n) => said.push(format!("{n} renamed")),
            Err(e) => said.push(format!("stopped: {e}")),
        }
        if !gone.is_empty() {
            let mut cmd = std::process::Command::new("gio");
            cmd.arg("trash").arg("--");
            for g in &gone {
                cmd.arg(dir.join(g));
            }
            match cmd.status() {
                Ok(s) if s.success() => said.push(format!("{} to the trash", gone.len())),
                _ => said.push("the trash refused (gio trash)".into()),
            }
        }
        let cursor = self.docs[self.cur].ed.cursor();
        self.open_dired(Some(dir));
        let ed = &mut self.docs[self.cur].ed;
        let line = cursor.line.min(ed.buf.len_lines().saturating_sub(1));
        ed.set_cursor(zett_core::Pos { line, col: 0 });
        self.say(if said.is_empty() { "nothing to change".to_string() } else { said.join(", ") });
    }

    /// Enter and - in a listing (normal mode); false: not a listing, or not one of these keys.
    pub(crate) fn dired_key(&mut self, k: &KeyEvent) -> bool {
        let name = self.docs[self.cur].name.clone();
        let Some(d) = self.dired.get(&name) else { return false };
        let dir = d.dir.clone();
        match k.code {
            KeyCode::Enter => {
                let line = self.docs[self.cur].ed.cursor().line;
                let n = self.docs[self.cur].ed.buf.line(line).to_string().trim_end_matches(['\n', '\r']).to_string();
                if n.is_empty() {
                    return true;
                }
                let p = dir.join(n.trim_end_matches('/'));
                if p.is_dir() { self.open_dired(Some(p)) } else { self.open(&p) }
                true
            }
            KeyCode::Char('-') => {
                if let Some(up) = dir.parent() {
                    self.open_dired(Some(up.to_path_buf()));
                }
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn edits_become_renames_or_deletions() {
        let old = s(&["src/", "a.txt", "b.txt"]);
        assert_eq!(plan(&old, &s(&["src/", "a.md", "b.txt"])), Ok((vec![("a.txt".into(), "a.md".into())], vec![])));
        assert_eq!(plan(&old, &s(&["src/", "b.txt"])), Ok((vec![], vec!["a.txt".into()])));
        assert!(plan(&old, &s(&["src/", "c.txt"])).is_err(), "rename and delete at once");
        assert!(plan(&old, &s(&["src/", "b.txt", "b.txt"])).is_err(), "twice");
        assert!(plan(&old, &s(&["src/", "a.txt", "b.txt", "new"])).is_err(), "added");
    }

    #[test]
    fn names_can_swap() {
        let dir = std::env::temp_dir().join(format!("zett-dired-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a"), "A").unwrap();
        std::fs::write(dir.join("b"), "B").unwrap();
        std::fs::write(dir.join("c"), "C").unwrap();
        rename_all(&dir, &[("a".into(), "b".into()), ("b".into(), "a".into())]).unwrap();
        assert_eq!(std::fs::read_to_string(dir.join("a")).unwrap(), "B");
        assert_eq!(std::fs::read_to_string(dir.join("b")).unwrap(), "A");
        // onto a name that is there and stays: refused, nothing moved
        assert!(rename_all(&dir, &[("a".into(), "c".into())]).is_err());
        assert_eq!(std::fs::read_to_string(dir.join("c")).unwrap(), "C");
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[cfg(test)]
mod app_tests {
    // the listing edited in the editor and saved: a rename on disk; a line taken out is only said, not done
    // (the trash itself is not used here: it would be the user's)
    #[test]
    fn saving_the_listing_renames_on_disk() {
        let dir = std::env::temp_dir().join(format!("zett-dired-app-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("sub")).unwrap();
        std::fs::write(dir.join("old.txt"), "x").unwrap();
        std::fs::write(dir.join("keep.txt"), "y").unwrap();
        let mut app = crate::App::new(Vec::new());
        app.open_dired(Some(dir.clone()));
        assert_eq!(app.docs[app.cur].ed.text(), "sub/\nkeep.txt\nold.txt");
        app.command("%s/old/new/");
        app.command("w");
        assert!(dir.join("new.txt").exists() && !dir.join("old.txt").exists(), "renamed");
        assert_eq!(app.docs[app.cur].ed.text(), "sub/\nkeep.txt\nnew.txt", "listed again");
        app.command("2delete");
        app.command("w");
        assert!(dir.join("keep.txt").exists(), "without ! nothing goes");
        assert!(app.msg.as_ref().is_some_and(|m| m.0.contains(":w!")), "{:?}", app.msg);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
