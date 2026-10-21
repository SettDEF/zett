//! Positions and words: line starts and ends, the cursor kept on a character, Vim's w b e.

use super::*;

impl Editor {
    // ── positions ──

    pub(crate) fn line_start(&self, line: usize) -> usize {
        self.buf.to_char(Pos::new(line, 0))
    }
    pub(crate) fn line_end(&self, line: usize) -> usize {
        // the char index just past the line's last char (before "\n")
        self.line_start(line) + self.buf.line_len(line)
    }
    pub(crate) fn cur_line(&self) -> usize {
        self.cursor().line
    }
    /// Normal mode keeps the cursor ON a character, never past the last one.
    pub(crate) fn clamp(&mut self) {
        let line = self.cur_line().min(self.buf.len_lines() - 1);
        let (s, len) = (self.line_start(line), self.buf.line_len(line));
        // normal and visual stay on a character; visual after `$` may sit on the line break
        let on_char = self.mode == Mode::Normal || (matches!(self.mode, Mode::Visual | Mode::VisualLine | Mode::VisualBlock) && self.want_col != usize::MAX);
        let max = if on_char && len > 0 { s + len - 1 } else { s + len };
        self.cursor = self.cursor.clamp(s, max);
    }
    pub(crate) fn char(&self, i: usize) -> Option<char> {
        self.buf.char_at(i)
    }

    // ── word motions (Vim's `w b e`: keyword runs, punctuation runs, blank lines stop `w`) ──

    pub(crate) fn word_forward(&self, mut i: usize) -> usize {
        let n = self.buf.len_chars();
        if i >= n {
            return n;
        }
        let c0 = self.kw(self.char(i).unwrap());
        if c0 != 0 {
            while i < n && self.kw(self.char(i).unwrap()) == c0 {
                i += 1;
            }
        }
        // skip blanks; an empty line is a word of its own
        while i < n {
            let c = self.char(i).unwrap();
            if c == '\n' && i + 1 < n && self.char(i + 1) == Some('\n') {
                return i + 1;
            }
            if !c.is_whitespace() {
                break;
            }
            i += 1;
        }
        i
    }

    pub(crate) fn word_end(&self, mut i: usize) -> usize {
        let n = self.buf.len_chars();
        if n == 0 {
            return 0;
        }
        i += 1;
        while i < n && self.char(i).unwrap().is_whitespace() {
            i += 1;
        }
        if i >= n {
            return n - 1;
        }
        let c0 = self.kw(self.char(i).unwrap());
        while i + 1 < n && self.kw(self.char(i + 1).unwrap()) == c0 {
            i += 1;
        }
        i
    }

    pub(crate) fn word_back(&self, mut i: usize) -> usize {
        if i == 0 {
            return 0;
        }
        i -= 1;
        while i > 0 && self.char(i).unwrap().is_whitespace() {
            i -= 1;
        }
        let c0 = self.kw(self.char(i).unwrap());
        while i > 0 && self.kw(self.char(i - 1).unwrap()) == c0 {
            i -= 1;
        }
        i
    }

    /// The whitespace a line starts with.
    pub(crate) fn indent_of(&self, line: usize) -> String {
        let s = self.line_start(line);
        self.buf.slice(s, self.first_non_blank(line).max(s))
    }

    pub(crate) fn first_non_blank(&self, line: usize) -> usize {
        let s = self.line_start(line);
        let len = self.buf.line_len(line);
        let mut i = s;
        while i < s + len && self.char(i).is_some_and(|c| c == ' ' || c == '\t') {
            i += 1;
        }
        i
    }

}
