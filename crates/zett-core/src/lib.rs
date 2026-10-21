//! zett documents. For now: the text buffer every view edits.
//!
//! Positions are char indices into the whole text (what ropey indexes by), or
//! (line, column) pairs in chars for the editor. Lines include their "\n".

pub mod blocks;
pub mod markdown;
#[cfg(feature = "fs")]
pub mod folder;
pub use blocks::{BlockId, Kind, Workspace};

use ropey::Rope;
use std::path::{Path, PathBuf};

/// A position in a buffer: line and column, both 0-based, column in chars.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pos {
    pub line: usize,
    pub col: usize,
}

impl Pos {
    pub const fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }
}

/// How a file is kept on disk (Vim's fileformat, fileencoding, bomb, endofline): what opening
/// found, and what saving writes back.
#[derive(Clone, Debug, PartialEq)]
pub struct Disk {
    /// "unix" (\n), "dos" (\r\n), "mac" (\r)
    pub format: String,
    /// "utf-8" or "latin1"
    pub encoding: String,
    pub bom: bool,
    /// the file ended with a line break
    pub eol: bool,
}

impl Default for Disk {
    fn default() -> Self {
        Disk { format: "unix".into(), encoding: "utf-8".into(), bom: false, eol: true }
    }
}

impl Disk {
    /// A file's bytes as text: a UTF-8 byte-order mark taken off, UTF-8 else Latin-1, DOS line ends
    /// made plain (Vim's fileencodings ucs-bom,utf-8,latin1 and fileformats unix,dos).
    pub fn decode(bytes: &[u8]) -> (String, Disk) {
        let mut d = Disk::default();
        let bytes = match bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]) {
            Some(b) => {
                d.bom = true;
                b
            }
            None => bytes,
        };
        let text = match std::str::from_utf8(bytes) {
            Ok(s) => s.to_string(),
            Err(_) => {
                d.encoding = "latin1".into();
                bytes.iter().map(|&b| b as char).collect()
            }
        };
        // dos when every line break has its \r
        let lf = text.matches('\n').count();
        let text = if lf > 0 && text.matches("\r\n").count() == lf {
            d.format = "dos".into();
            text.replace("\r\n", "\n")
        } else {
            text
        };
        d.eol = text.is_empty() || text.ends_with('\n');
        (text, d)
    }

    /// The text as the file's bytes: its line ends, encoding and byte-order mark.
    pub fn encode(&self, text: &str) -> Vec<u8> {
        // no last line break when the file is to have none (Vim's noendofline)
        let text = if self.eol { text } else { text.strip_suffix('\n').unwrap_or(text) };
        let text = match self.format.as_str() {
            "dos" => text.replace('\n', "\r\n"),
            "mac" => text.replace('\n', "\r"),
            _ => text.to_string(),
        };
        let mut out = Vec::with_capacity(text.len() + 3);
        if self.bom && self.encoding != "latin1" {
            out.extend_from_slice(&[0xEF, 0xBB, 0xBF]);
        }
        if self.encoding == "latin1" {
            out.extend(text.chars().map(|c| if (c as u32) < 256 { c as u8 } else { b'?' }));
        } else {
            out.extend_from_slice(text.as_bytes());
        }
        out
    }
}

/// A text buffer: a rope, and where it came from.
#[derive(Clone, Debug, Default)]
pub struct Buffer {
    text: Rope,
    path: Option<PathBuf>,
    dirty: bool,
    /// counts every change (language servers and caches compare it)
    rev: u64,
    /// how its file is kept on disk
    pub disk: Disk,
}

impl Buffer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_str(s: &str) -> Self {
        Self { text: Rope::from_str(s), ..Self::default() }
    }

    #[cfg(feature = "fs")]
    /// Opens a file; a path that does not exist yet is an empty buffer for it.
    pub fn open(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let path = path.as_ref();
        let (text, disk) = match std::fs::read(path) {
            Ok(b) => Disk::decode(&b),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (String::new(), Disk::default()),
            Err(e) => return Err(e),
        };
        Ok(Self { text: Rope::from_str(&text), path: Some(path.to_path_buf()), dirty: false, rev: 0, disk })
    }

    #[cfg(feature = "fs")]
    /// Writes to its file, through a temporary file so a crash never leaves half a file.
    pub fn save(&mut self) -> std::io::Result<()> {
        self.save_with(false, false)
    }

    #[cfg(feature = "fs")]
    /// Writes to its file as `disk` says: through a temporary file renamed over it (the file's
    /// permissions kept), or with `in_place` into the file itself (Vim's backupcopy=yes: links and
    /// the file's identity stay); `fsync` waits until it is on the disk.
    pub fn save_with(&mut self, in_place: bool, fsync: bool) -> std::io::Result<()> {
        use std::io::Write;
        let Some(path) = self.path.clone() else {
            return Err(std::io::Error::other("the buffer has no file"));
        };
        let bytes = self.disk.encode(&self.text.to_string());
        if in_place {
            let mut f = std::fs::OpenOptions::new().write(true).create(true).truncate(true).open(&path)?;
            f.write_all(&bytes)?;
            if fsync {
                f.sync_all()?;
            }
        } else {
            let tmp = path.with_extension(format!(
                "{}.zett-tmp",
                path.extension().and_then(|e| e.to_str()).unwrap_or("")
            ));
            let mut f = std::fs::File::create(&tmp)?;
            f.write_all(&bytes)?;
            if fsync {
                f.sync_all()?;
            }
            drop(f);
            // the file keeps its permissions (a script stays executable)
            if let Ok(m) = std::fs::metadata(&path) {
                let _ = std::fs::set_permissions(&tmp, m.permissions());
            }
            std::fs::rename(&tmp, &path)?;
        }
        self.dirty = false;
        Ok(())
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }
    /// Where `save` writes (`:w name` on a new buffer); marks it changed.
    pub fn set_path(&mut self, p: PathBuf) {
        self.path = Some(p);
        self.dirty = true;
    }
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
    /// :set modified / nomodified.
    pub fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty;
    }
    pub fn text(&self) -> String {
        self.text.to_string()
    }
    pub fn len_chars(&self) -> usize {
        self.text.len_chars()
    }

    /// Lines as the editor counts them: a trailing "\n" does not start one more.
    /// How many changes the text has had (any edit counts one).
    pub fn rev(&self) -> u64 {
        self.rev
    }

    pub fn len_lines(&self) -> usize {
        let n = self.text.len_lines();
        if n > 1 && self.text.line(n - 1).len_chars() == 0 { n - 1 } else { n.max(1) }
    }

    /// A line without its line break.
    pub fn line(&self, line: usize) -> String {
        if line >= self.text.len_lines() {
            return String::new();
        }
        let mut s = self.text.line(line).to_string();
        while s.ends_with('\n') || s.ends_with('\r') {
            s.pop();
        }
        s
    }

    /// Length of a line in chars, without its line break.
    pub fn line_len(&self, line: usize) -> usize {
        self.line(line).chars().count()
    }

    /// (line, col) → char index, clamped into the buffer.
    pub fn to_char(&self, p: Pos) -> usize {
        if self.text.len_chars() == 0 {
            return 0;
        }
        let line = p.line.min(self.text.len_lines() - 1);
        (self.text.line_to_char(line) + p.col.min(self.line_len(line))).min(self.text.len_chars())
    }

    /// char index → (line, col).
    pub fn to_pos(&self, idx: usize) -> Pos {
        let idx = idx.min(self.text.len_chars());
        let line = self.text.char_to_line(idx);
        Pos::new(line, idx - self.text.line_to_char(line))
    }

    pub fn char_at(&self, idx: usize) -> Option<char> {
        (idx < self.text.len_chars()).then(|| self.text.char(idx))
    }

    pub fn insert(&mut self, idx: usize, s: &str) {
        if s.is_empty() {
            return;
        }
        self.text.insert(idx.min(self.text.len_chars()), s);
        self.dirty = true;
        self.rev += 1;
    }

    /// Removes chars `from..to` and returns them.
    pub fn remove(&mut self, from: usize, to: usize) -> String {
        let (a, b) = (from.min(to), from.max(to).min(self.text.len_chars()));
        if a >= b {
            return String::new();
        }
        let gone = self.text.slice(a..b).to_string();
        self.text.remove(a..b);
        self.dirty = true;
        self.rev += 1;
        gone
    }

    pub fn slice(&self, from: usize, to: usize) -> String {
        let (a, b) = (from.min(to), from.max(to).min(self.text.len_chars()));
        self.text.slice(a..b).to_string()
    }

    /// Replaces the whole text (undo restores snapshots this way).
    pub fn set_text(&mut self, s: &str) {
        self.text = Rope::from_str(s);
        self.dirty = true;
    }

    /// Where `needle` next starts: at or after `from` (forward), or before `from` (backward).
    /// `fold`: case does not matter. Walks the rope itself, so a search costs the distance to
    /// the match, not a copy of the text.
    pub fn find(&self, needle: &str, from: usize, forward: bool, fold: bool) -> Option<usize> {
        let low = |c: char| if fold { c.to_lowercase().next().unwrap_or(c) } else { c };
        let want: Vec<char> = needle.chars().map(low).collect();
        let (m, len) = (want.len(), self.text.len_chars());
        if m == 0 || m > len {
            return None;
        }
        let mut window = std::collections::VecDeque::with_capacity(m + 1);
        if forward {
            let mut at = from.min(len);
            for c in self.text.chars_at(at) {
                window.push_back(low(c));
                if window.len() > m {
                    window.pop_front();
                }
                at += 1;
                if window.len() == m && window.iter().eq(want.iter()) {
                    return Some(at - m);
                }
            }
        } else {
            // a match starting before `from` may end after it
            let mut at = (from + m - 1).min(len);
            let mut it = self.text.chars_at(at);
            while let Some(c) = it.prev() {
                at -= 1;
                window.push_front(low(c));
                if window.len() > m {
                    window.pop_back();
                }
                if window.len() == m && at < from && window.iter().eq(want.iter()) {
                    return Some(at);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_both_ways_and_folded() {
        let b = Buffer::from_str("one Two one two");
        assert_eq!(b.find("one", 0, true, false), Some(0));
        assert_eq!(b.find("one", 1, true, false), Some(8));
        assert_eq!(b.find("two", 0, true, false), Some(12));
        assert_eq!(b.find("two", 0, true, true), Some(4));
        // backward: a match that starts before `from`, even one reaching past it
        assert_eq!(b.find("one", 15, false, false), Some(8));
        assert_eq!(b.find("one", 9, false, false), Some(8));
        assert_eq!(b.find("one", 8, false, false), Some(0));
        assert_eq!(b.find("one", 0, false, false), None);
        assert_eq!(b.find("three", 0, true, true), None);
        assert_eq!(b.find("", 0, true, true), None);
        // not ASCII: char positions
        let b = Buffer::from_str("grüße, Grüße");
        assert_eq!(b.find("GRÜSSE", 0, true, true), None);
        assert_eq!(b.find("grüße", 1, true, true), Some(7));
    }

    #[test]
    fn lines_and_positions() {
        let b = Buffer::from_str("one\ntwo\n");
        assert_eq!(b.len_lines(), 2);
        assert_eq!(b.line(1), "two");
        assert_eq!(b.to_char(Pos::new(1, 2)), 6);
        assert_eq!(b.to_pos(6), Pos::new(1, 2));
        // a column past the end clamps to the line
        assert_eq!(b.to_char(Pos::new(0, 99)), 3);
    }

    #[test]
    fn insert_remove() {
        let mut b = Buffer::from_str("hello world");
        b.insert(5, ",");
        assert_eq!(b.text(), "hello, world");
        assert_eq!(b.remove(5, 6), ",");
        assert_eq!(b.text(), "hello world");
        assert!(b.is_dirty());
    }

    #[test]
    fn empty_buffer_has_one_line() {
        let b = Buffer::new();
        assert_eq!(b.len_lines(), 1);
        assert_eq!(b.line(0), "");
        assert_eq!(b.to_char(Pos::new(3, 3)), 0);
    }

    #[cfg(feature = "fs")]
    #[test]
    fn files_come_back_as_they_were() {
        let (t, d) = Disk::decode(b"\xEF\xBB\xBFa\r\nb\r\n");
        assert_eq!((t.as_str(), d.format.as_str(), d.bom, d.eol), ("a\nb\n", "dos", true, true));
        assert_eq!(d.encode(&t), b"\xEF\xBB\xBFa\r\nb\r\n");
        let (t, d) = Disk::decode(b"caf\xE9");
        assert_eq!((t.as_str(), d.encoding.as_str(), d.eol), ("caf\u{e9}", "latin1", false));
        assert_eq!(d.encode(&t), b"caf\xE9");
        let (_, d) = Disk::decode(b"a\r\nb\n");
        assert_eq!(d.format, "unix", "mixed line ends stay as they are");
        // a script stays executable after a save
        use std::os::unix::fs::PermissionsExt;
        let dir = std::env::temp_dir().join(format!("zett-perm-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("s.sh");
        std::fs::write(&p, "echo\n").unwrap();
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
        let mut b = Buffer::open(&p).unwrap();
        b.insert(0, "#!/bin/sh\n");
        b.save().unwrap();
        assert_eq!(std::fs::metadata(&p).unwrap().permissions().mode() & 0o777, 0o755);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn save_and_open_roundtrip() {
        let dir = std::env::temp_dir().join(format!("zett-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("note.md");
        let mut b = Buffer::open(&p).unwrap();
        b.insert(0, "# hi\n");
        b.save().unwrap();
        assert_eq!(Buffer::open(&p).unwrap().text(), "# hi\n");
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
