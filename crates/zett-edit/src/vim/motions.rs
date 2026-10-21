//! Paragraphs, sentences, brackets (% and N%), and typing over in R mode.

use super::*;

impl Editor {
    pub(crate) fn vim_paragraph(&mut self, forward: bool) {
        let n = self.take_count();
        let last = self.buf.len_lines() - 1;
        let mut l = self.cur_line();
        let blank = |e: &Self, l: usize| e.buf.line_len(l) == 0;
        for _ in 0..n {
            if forward {
                while l < last && blank(self, l) {
                    l += 1;
                }
                while l < last && !blank(self, l) {
                    l += 1;
                }
            } else {
                while l > 0 && blank(self, l) {
                    l -= 1;
                }
                while l > 0 && !blank(self, l) {
                    l -= 1;
                }
            }
        }
        let to = if forward && l == last && !blank(self, l) { self.line_end(l) } else { self.line_start(l) };
        self.motion(Span::Chars(self.cursor.min(to), self.cursor.max(to)), Some(to));
    }

    pub(crate) fn vim_sentence(&mut self, forward: bool) {
        let n = self.take_count();
        let len = self.buf.len_chars();
        let mut i = self.cursor;
        let ends = |e: &Self, k: usize| e.char(k).is_some_and(|c| matches!(c, '.' | '!' | '?')) && e.char(k + 1).is_none_or(|c| c.is_whitespace());
        for _ in 0..n {
            if forward {
                let mut k = i;
                while k < len && !ends(self, k) && !(self.char(k) == Some('\n') && self.char(k + 1) == Some('\n')) {
                    k += 1;
                }
                k += 1;
                while k < len && self.char(k).is_some_and(char::is_whitespace) {
                    k += 1;
                }
                i = k.min(len);
            } else {
                let mut k = i.saturating_sub(1);
                while k > 0 && self.char(k).is_some_and(char::is_whitespace) {
                    k -= 1;
                }
                while k > 0 && !ends(self, k - 1) && !(self.char(k - 1) == Some('\n') && self.char(k.saturating_sub(2)) == Some('\n')) {
                    k -= 1;
                }
                i = k;
            }
        }
        self.motion(Span::Chars(self.cursor.min(i), self.cursor.max(i)), Some(i));
    }

    /// `%`: the bracket matching the one at or after the cursor; with a count, that percentage of the file.
    pub(crate) fn vim_percent(&mut self) {
        if let Some(n) = self.count.take() {
            let last = self.buf.len_lines() - 1;
            let to = ((n.min(100) * (last + 1)).div_ceil(100)).saturating_sub(1).min(last);
            let line = self.cur_line();
            return self.motion(Span::Lines(line, to), Some(self.first_non_blank(to)));
        }
        let line = self.cur_line();
        let e = self.line_end(line);
        let pairs = [('(', ')'), ('[', ']'), ('{', '}')];
        let Some((at, ch)) = (self.cursor..e).find_map(|i| self.char(i).filter(|c| pairs.iter().any(|(a, b)| a == c || b == c)).map(|c| (i, c))) else { return };
        let (open, close, fwd) = pairs.iter().find_map(|&(a, b)| if ch == a { Some((a, b, true)) } else if ch == b { Some((a, b, false)) } else { None }).unwrap();
        let mut depth = 0i32;
        let mut i = at as isize;
        let len = self.buf.len_chars() as isize;
        while i >= 0 && i < len {
            let c = self.char(i as usize).unwrap_or(' ');
            if c == open {
                depth += if fwd { 1 } else { -1 };
            } else if c == close {
                depth += if fwd { -1 } else { 1 };
            }
            if depth == 0 {
                let to = i as usize;
                self.vim.marks.insert('\'', self.cursor);
                return self.motion(Span::Chars(self.cursor.min(to), self.cursor.max(to) + 1), Some(to));
            }
            i += if fwd { 1 } else { -1 };
        }
    }

    /// Typing in `R` mode replaces the character under the cursor (not the line break).
    pub(crate) fn vim_replace_char(&mut self) {
        if self.vim.replace && self.char(self.cursor).is_some_and(|c| c != '\n') {
            let at = self.cursor;
            self.rem(at, at + 1);
        }
    }

}
