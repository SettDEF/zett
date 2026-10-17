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

/// A text buffer: a rope, and where it came from.
#[derive(Clone, Debug, Default)]
pub struct Buffer {
    text: Rope,
    path: Option<PathBuf>,
    dirty: bool,
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
        let text = match std::fs::read_to_string(path) {
            Ok(s) => Rope::from_str(&s),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Rope::new(),
            Err(e) => return Err(e),
        };
        Ok(Self { text, path: Some(path.to_path_buf()), dirty: false })
    }

    #[cfg(feature = "fs")]
    /// Writes to its file, through a temporary file so a crash never leaves half a file.
    pub fn save(&mut self) -> std::io::Result<()> {
        let Some(path) = self.path.clone() else {
            return Err(std::io::Error::other("the buffer has no file"));
        };
        let tmp = path.with_extension(format!(
            "{}.zett-tmp",
            path.extension().and_then(|e| e.to_str()).unwrap_or("")
        ));
        let mut f = std::io::BufWriter::new(std::fs::File::create(&tmp)?);
        self.text.write_to(&mut f)?;
        drop(f);
        std::fs::rename(&tmp, &path)?;
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
    pub fn text(&self) -> String {
        self.text.to_string()
    }
    pub fn len_chars(&self) -> usize {
        self.text.len_chars()
    }

    /// Lines as the editor counts them: a trailing "\n" does not start one more.
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
