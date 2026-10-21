//! Insert mode, comments and edits applied as one change.

use super::*;

impl Editor {
    // ── insert mode ──

    pub(crate) fn start_insert(&mut self, at: usize) {
        self.begin();
        self.count = None;
        self.mode = Mode::Insert;
        self.cursor = at;
    }

    pub(crate) fn insert_key(&mut self, k: Key) {
        match k.code {
            Code::Esc => {
                self.mode = Mode::Normal;
                self.vim.last_insert = Some(self.cursor);
                self.vim.replace = false;
                // Vim steps back onto the last typed char
                if self.cursor > self.line_start(self.cur_line()) {
                    self.cursor -= 1;
                }
                self.clamp();
                self.want_col = self.cursor().col;
            }
            Code::Enter => {
                // autoindent: the indent before the cursor carries over
                let l = self.cur_line();
                let ind: String = self.indent_of(l).chars().take(self.cursor - self.line_start(l)).collect();
                self.ins(self.cursor, &format!("\n{ind}"));
                self.cursor += 1 + ind.chars().count();
            }
            Code::Tab => self.insert_tab(),
            Code::Backspace => {
                if self.cursor > 0 {
                    self.rem(self.cursor - 1, self.cursor);
                    self.cursor -= 1;
                }
            }
            Code::Delete => {
                self.rem(self.cursor, self.cursor + 1);
            }
            Code::Left => self.cursor = self.cursor.saturating_sub(1).max(self.line_start(self.cur_line())),
            Code::Right => self.cursor = (self.cursor + 1).min(self.line_end(self.cur_line())),
            Code::Char(c) if !k.ctrl && !k.alt => {
                // R: typing replaces
                self.vim_replace_char();
                self.ins(self.cursor, &c.to_string());
                self.cursor += 1;
            }
            _ => {}
        }
    }

    /// Comments lines `from..=to` out with `prefix` (`//`, `#`), or back in when every
    /// non-empty one already is (evil-commentary's gc). One undo step; leaves visual mode.
    pub fn toggle_comment(&mut self, from: usize, to: usize, prefix: &str) {
        let last = self.buf.len_lines().saturating_sub(1);
        let (from, to) = (from.min(last), to.min(last));
        let lines: Vec<(usize, String)> = (from..=to).map(|l| (l, self.buf.line(l))).collect();
        let body = |s: &str| s.trim_end_matches(['\n', '\r']).to_string();
        let filled: Vec<&(usize, String)> = lines.iter().filter(|(_, t)| !body(t).trim().is_empty()).collect();
        if filled.is_empty() {
            return;
        }
        let commented = filled.iter().all(|(_, t)| body(t).trim_start().starts_with(prefix));
        let indent = filled.iter().map(|(_, t)| body(t).len() - body(t).trim_start().len()).min().unwrap_or(0);
        self.begin();
        // bottom up, so earlier char positions stay right
        for (l, t) in filled.iter().rev() {
            let t = body(t);
            let start = self.buf.to_char(Pos { line: *l, col: 0 });
            if commented {
                let lead = t.chars().count() - t.trim_start().chars().count();
                let mut n = prefix.chars().count();
                if t.trim_start()[prefix.len()..].starts_with(' ') {
                    n += 1;
                }
                self.rem(start + lead, start + lead + n);
            } else {
                self.ins(start + t.chars().take(indent).count(), &format!("{prefix} "));
            }
        }
        self.close();
        self.mode = Mode::Normal;
        let p = self.cursor();
        self.set_cursor(p);
    }

    /// Replaces ranges (from, to, new text) as one undo step: a rename or a format
    /// from a language server. Ranges must not overlap; any order.
    pub fn apply_edits(&mut self, mut edits: Vec<(Pos, Pos, String)>) {
        if edits.is_empty() {
            return;
        }
        let keep = self.cursor();
        // last first, so earlier positions stay right
        edits.sort_by(|a, b| (b.0.line, b.0.col).cmp(&(a.0.line, a.0.col)));
        self.begin();
        for (a, b, text) in edits {
            let (s, e) = (self.buf.to_char(a), self.buf.to_char(b));
            if e > s {
                self.rem(s, e);
            }
            self.ins(s, &text);
        }
        self.close();
        self.mode = Mode::Normal;
        self.set_cursor(keep);
    }

}
