//! Vim's file options on saving: write, backup / writebackup / backupdir / backupext / backupskip,
//! backupcopy, fsync, fileformat, fileencoding, bomb, endofline, binary; and on opening, the
//! buffer takes its file's format as those options.

use super::*;
use std::path::{Path, PathBuf};

impl App {
    /// A buffer seen for the first time: its file's line ends, encoding, byte-order mark and last
    /// line break as its own fileformat, fileencoding, bomb and endofline.
    pub(crate) fn take_disk(&mut self, di: usize) {
        let disk = self.docs[di].ed.buf.disk.clone();
        let local = &mut self.docs[di].local;
        local.insert("fileformat".into(), disk.format);
        local.insert("fileencoding".into(), disk.encoding);
        local.insert("bomb".into(), disk.bom.to_string());
        local.insert("endofline".into(), disk.eol.to_string());
        self.docs[di].mtime = self.disk_time(di);
    }

    /// Saves buffer `di` as the file options say; Err is the message when it cannot.
    pub(crate) fn write_buffer(&mut self, di: usize) -> Result<(), String> {
        if !self.flag(di, "write") {
            return Err("E142: File not written: Writing is disabled by 'write' option".into());
        }
        let encoding = match self.setting(di, "fileencoding").to_lowercase().as_str() {
            "" | "utf-8" | "utf8" => "utf-8",
            "latin1" | "iso-8859-1" | "iso8859-1" => "latin1",
            e => return Err(format!("E213: Cannot convert to {e} (zett writes utf-8 and latin1)")),
        };
        let format = match self.setting(di, "fileformat").as_str() {
            f @ ("unix" | "dos" | "mac") => f.to_string(),
            _ => "unix".into(),
        };
        let bom = self.flag(di, "bomb");
        let (fixeol, eol) = (self.flag(di, "final_newline") && !self.flag(di, "binary"), self.flag(di, "endofline"));
        let path = self.docs[di].ed.buf.path().map(Path::to_path_buf).ok_or("no file name: :w name")?;
        // backup / writebackup: the old file kept aside while writing (and after, with backup)
        let backup = if (self.flag(di, "backup") || self.flag(di, "writebackup")) && path.is_file() && !self.backup_skipped(di, &path) {
            let to = self.backup_path(di, &path).ok_or("E510: Can't make backup file (add ! to write anyway)")?;
            std::fs::copy(&path, &to).map_err(|e| format!("E509: Cannot create backup file: {e}"))?;
            Some(to)
        } else {
            None
        };
        // backupcopy: yes writes into the file itself, no renames a new one over it, auto picks
        // in place for a link (symbolic or hard) so the link stays
        let in_place = match self.setting(di, "backupcopy").split(',').next().unwrap_or("auto") {
            "yes" => true,
            "no" => false,
            _ => std::fs::symlink_metadata(&path).is_ok_and(|m| m.file_type().is_symlink() || std::os::unix::fs::MetadataExt::nlink(&m) > 1),
        };
        let fsync = self.flag(di, "fsync");
        let b = &mut self.docs[di].ed.buf;
        b.disk.format = format;
        b.disk.encoding = encoding.into();
        b.disk.bom = bom;
        // the last line break: fixendofline adds it (not for binary), else endofline keeps what the file had
        b.disk.eol = fixeol || eol;
        let r = b.save_with(in_place, fsync).map_err(|e| format!("could not save: {e}"));
        if let Some(to) = backup.filter(|_| r.is_ok() && !self.flag(di, "backup")) {
            let _ = std::fs::remove_file(to);
        }
        r
    }

    /// backupskip: patterns of files not backed up ($TMPDIR/*, /tmp/*, …).
    fn backup_skipped(&self, di: usize, path: &Path) -> bool {
        let p = path.to_string_lossy();
        self.setting(di, "backupskip").split(',').map(expand_env).any(|g| !g.is_empty() && glob_match(&g, &p))
    }

    /// Where the backup goes: the first backupdir that is there or can be made ("." the file's own
    /// folder; a dir ending in // takes the whole path, / as %), the name plus backupext.
    fn backup_path(&self, di: usize, path: &Path) -> Option<PathBuf> {
        let ext = self.setting(di, "backupext");
        let ext = if ext.is_empty() { "~".to_string() } else { ext };
        let name = path.file_name()?.to_string_lossy().to_string();
        for d in self.setting(di, "backupdir").split(',').map(expand_env).filter(|d| !d.is_empty()) {
            let (dir, whole) = match d.strip_suffix("//") {
                Some(x) => (x.to_string(), true),
                None => (d.clone(), false),
            };
            let dir = if dir == "." { path.parent().map(Path::to_path_buf).unwrap_or_default() } else { PathBuf::from(dir.replace("/nvim/", "/zett/")) };
            if !dir.is_dir() && std::fs::create_dir_all(&dir).is_err() {
                continue;
            }
            let file = if whole { std::fs::canonicalize(path).unwrap_or(path.to_path_buf()).to_string_lossy().replace('/', "%") } else { name.clone() };
            return Some(dir.join(format!("{file}{ext}")));
        }
        None
    }

    /// The file as it is on disk again (:e!, autoread), the buffer's changes dropped; the cursor,
    /// the scroll position and the buffer's settings stay, its file's format is read anew.
    pub(crate) fn reload_doc(&mut self, i: usize) -> Result<PathBuf, String> {
        let p = self.docs[i].abs_path().ok_or("no file to reload")?;
        let mut nd = Doc::open(&p, &self.ss).map_err(|e| e.to_string())?;
        let old = &self.docs[i];
        nd.name = old.name.clone();
        nd.top = old.top;
        nd.local = old.local.clone();
        nd.ed.set_cursor(old.ed.cursor());
        self.docs[i] = nd;
        self.lsp_open(i);
        Ok(p)
    }

    /// When the file was last changed on disk.
    pub(crate) fn disk_time(&self, i: usize) -> Option<std::time::SystemTime> {
        std::fs::metadata(self.docs[i].ed.buf.path()?).and_then(|m| m.modified()).ok()
    }

    /// :checktime, and every second or so: files changed outside zett load again when their buffer
    /// has no changes and autoread is on, else a warning says so (once per change).
    pub(crate) fn checktime(&mut self) {
        for i in 0..self.docs.len() {
            let (Some(was), Some(now)) = (self.docs[i].mtime, self.disk_time(i)) else { continue };
            if now == was {
                continue;
            }
            self.docs[i].mtime = Some(now);
            let name = self.docs[i].name.clone();
            if !self.docs[i].dirty() && self.flag(i, "autoread") {
                match self.reload_doc(i) {
                    Ok(_) => self.say(format!("\"{name}\" changed outside zett: loaded again")),
                    Err(e) => self.say(e),
                }
            } else {
                self.say(format!("W11: Warning: File \"{name}\" has changed since editing started (:e! loads it, :w! keeps yours)"));
            }
        }
    }

    /// autowrite (autowriteall): before going to another buffer, :make or a shell command, the
    /// changed buffer is written.
    pub(crate) fn auto_write(&mut self) {
        let i = self.cur;
        if self.docs[i].dirty() && self.docs[i].ed.buf.path().is_some() && (self.flag(i, "autowrite") || self.flag(i, "autowriteall")) {
            self.save(None);
        }
    }
}

/// $VAR, ${VAR} and a leading ~ in an option's path.
pub(crate) fn expand_env(s: &str) -> String {
    let s = s.trim();
    let home = std::env::var("HOME").unwrap_or_default();
    let mut s = match s.strip_prefix('~') {
        Some(r) => format!("{home}{r}"),
        None => s.to_string(),
    };
    while let Some(i) = s.find('$') {
        let rest = &s[i + 1..];
        let (name, len) = match rest.strip_prefix('{') {
            Some(r) => match r.find('}') {
                Some(e) => (r[..e].to_string(), e + 2),
                None => break,
            },
            None => {
                let n: String = rest.chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '_').collect();
                let l = n.len();
                (n, l)
            }
        };
        if name.is_empty() {
            break;
        }
        let v = std::env::var(&name).unwrap_or_else(|_| match name.as_str() {
            "XDG_STATE_HOME" => format!("{home}/.local/state"),
            "TMPDIR" => "/tmp".into(),
            _ => String::new(),
        });
        s.replace_range(i..i + 1 + len, &v);
    }
    s
}

/// A file pattern as Vim's options take them: * any run (not across /), ** across, ? one character.
pub(crate) fn glob_match(pat: &str, s: &str) -> bool {
    fn go(p: &[char], s: &[char]) -> bool {
        match p.first() {
            None => s.is_empty(),
            Some('*') if p.get(1) == Some(&'*') => (0..=s.len()).any(|i| go(&p[2..], &s[i..])),
            Some('*') => (0..=s.len()).take_while(|&i| i == 0 || s[i - 1] != '/').any(|i| go(&p[1..], &s[i..])),
            Some('?') => !s.is_empty() && go(&p[1..], &s[1..]),
            Some(c) => s.first() == Some(c) && go(&p[1..], &s[1..]),
        }
    }
    go(&pat.chars().collect::<Vec<_>>(), &s.chars().collect::<Vec<_>>())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn globs_and_paths() {
        assert!(glob_match("/tmp/*", "/tmp/a.txt") && !glob_match("/tmp/*", "/tmp/a/b.txt"));
        assert!(glob_match("/tmp/**", "/tmp/a/b.txt") && glob_match("*.git/*", ".git/x"));
        assert!(glob_match("a?c", "abc") && !glob_match("a?c", "ac"));
        let home = std::env::var("HOME").unwrap();
        assert_eq!(expand_env("$HOME/y"), format!("{home}/y"));
        assert_eq!(expand_env("${HOME}z"), format!("{home}z"));
        assert_eq!(expand_env("~/b"), format!("{home}/b"));
    }
}

impl App {
    /// File names for the : line (:e :w :sp :cd …): what the folder typed so far holds; not what
    /// wildignore names; wildignorecase (or fileignorecase) matches either case, wildoptions=fuzzy
    /// takes the letters in order anywhere; names ending in one of suffixes come last; folders end in /.
    pub(crate) fn file_suggestions(&self, head: &str, arg: &str) -> Vec<crate::cmdline::Suggestion> {
        let cur = self.cur;
        let (dir_part, name_part) = match arg.rfind('/') {
            Some(i) => (&arg[..=i], &arg[i + 1..]),
            None => ("", arg),
        };
        let dir = if dir_part.is_empty() { PathBuf::from(".") } else { PathBuf::from(expand_env(dir_part)) };
        let dir = if dir.is_absolute() { dir } else { std::env::current_dir().unwrap_or_default().join(dir) };
        let ignore: Vec<String> = self.setting(cur, "wildignore").split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
        let suffixes: Vec<String> = self.setting(cur, "suffixes").split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
        let ic = self.flag(cur, "wildignorecase") || self.flag(cur, "fileignorecase");
        let fuzzy = self.setting(cur, "wildoptions").split(',').any(|o| o == "fuzzy");
        let norm = |s: &str| if ic { s.to_lowercase() } else { s.to_string() };
        let want = norm(name_part);
        let fits = |n: &str| {
            let n = norm(n);
            if fuzzy {
                let mut it = n.chars();
                want.chars().all(|c| it.any(|x| x == c))
            } else {
                n.starts_with(&want)
            }
        };
        let Ok(rd) = std::fs::read_dir(&dir) else { return Vec::new() };
        let mut found: Vec<(bool, bool, String)> = rd
            .flatten()
            .filter_map(|e| {
                let n = e.file_name().to_string_lossy().to_string();
                // dot files only when asked for
                if n.starts_with('.') && !name_part.starts_with('.') {
                    return None;
                }
                let full = e.path().to_string_lossy().to_string();
                if !fits(&n) || ignore.iter().any(|g| glob_match(g, &n) || glob_match(g, &full)) {
                    return None;
                }
                let is_dir = e.file_type().is_ok_and(|t| t.is_dir());
                let late = suffixes.iter().any(|s| n.ends_with(s.as_str()));
                Some((late, !is_dir, if is_dir { format!("{n}/") } else { n }))
            })
            .collect();
        found.sort();
        found
            .into_iter()
            .take(200)
            .map(|(_, file, n)| crate::cmdline::Suggestion { fill: format!("{head} {dir_part}{n}"), name: n, args: String::new(), about: if file { "file".into() } else { "folder".into() }, example: String::new() })
            .collect()
    }
}
