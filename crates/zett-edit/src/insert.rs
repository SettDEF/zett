//! Insert mode, comments and edits applied as one change.

use super::*;

impl Editor {
    // ── insert mode ──

    pub(crate) fn start_insert(&mut self, at: usize) {
        self.begin();
        self.count = None;
        self.mode = Mode::Insert;
        self.cursor = at;
        self.vim.insert_start = at;
    }

    pub(crate) fn insert_key(&mut self, k: Key) {
        // CTRL-K {char1} {char2}: a digraph (digraph.rs); anything else than a character stops it
        if let Some(first) = self.vim.digraph {
            let Some(c) = k.char().filter(|_| !k.ctrl && !k.alt) else {
                self.vim.digraph = None;
                return;
            };
            match first {
                None => self.vim.digraph = Some(Some(c)),
                Some(a) => {
                    self.vim.digraph = None;
                    self.insert_key(Key::ch(crate::digraph::lookup(a, c)));
                }
            }
            return;
        }
        // CTRL-G u: a new undo step from here; CTRL-G U: the next Left/Right stays in this one (plugins send
        // both around the keys they type); CTRL-G j / k: down / up to where typing started
        if std::mem::take(&mut self.vim.ctrl_g) {
            match k.code {
                Code::Char('u') if !k.ctrl => { self.close(); self.begin(); }
                Code::Char('j' | 'k') if !k.ctrl => {
                    let col = self.vim.insert_start.saturating_sub(self.line_start(self.buf.to_pos(self.vim.insert_start.min(self.buf.len_chars())).line));
                    let l = self.cur_line();
                    let to = if k.code == Code::Char('j') { (l + 1).min(self.buf.len_lines() - 1) } else { l.saturating_sub(1) };
                    self.cursor = (self.line_start(to) + col).min(self.line_end(to));
                }
                _ => {}
            }
            return;
        }
        if k.code == Code::Char('g') && k.ctrl && !k.alt {
            self.vim.ctrl_g = true;
            return;
        }
        // 'digraph': {char1} <BS> {char2}; any other key forgets char1
        let dg = (self.vim.dg_last.take(), self.vim.dg_bs.take());
        match k.code {
            Code::Char('k') if k.ctrl && !k.alt => {
                self.vim.digraph = Some(None);
                return;
            }
            Code::Backspace if self.opt_on("digraph", false) => self.vim.dg_bs = dg.0,
            _ => {}
        }
        let k = match (dg.1, k.code) {
            (Some(a), Code::Char(c)) if !k.ctrl && !k.alt => crate::digraph::known(a, c).map_or(k, Key::ch),
            _ => k,
        };
        if let Code::Char(c) = k.code {
            if !k.ctrl && !k.alt {
                self.vim.dg_last = Some(c);
            }
        }
        match k.code {
            Code::Esc => {
                // a block insert: what was typed goes on the other lines too
                self.block_insert_done();
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
                let ind = self.next_indent(l, true, Some(self.cursor - self.line_start(l)));
                self.ins(self.cursor, &format!("\n{ind}"));
                self.cursor += 1 + ind.chars().count();
                self.indent_expr_line(l + 1);
            }
            Code::Tab => self.insert_tab(),
            Code::Backspace => {
                let from = self.vim.insert_start;
                self.vim_backspace(from);
            }
            Code::Delete => {
                self.rem(self.cursor, self.cursor + 1);
            }
            // CTRL-W: the word before the cursor (blanks first); CTRL-U: back to where typing
            // started, or the line's indent (both stop at the line's start)
            Code::Char(c @ ('w' | 'u')) if k.ctrl && !k.alt => {
                let ls = self.line_start(self.cur_line());
                let before: Vec<char> = self.buf.slice(ls, self.cursor).chars().collect();
                let mut n = before.len();
                if c == 'w' {
                    while n > 0 && before[n - 1].is_whitespace() {
                        n -= 1;
                    }
                    let word = |ch: char| ch.is_alphanumeric() || ch == '_';
                    let kind = n.checked_sub(1).map(|i| word(before[i]));
                    while n > 0 && Some(word(before[n - 1])) == kind && !before[n - 1].is_whitespace() {
                        n -= 1;
                    }
                } else {
                    let from = self.vim.insert_start.clamp(ls, self.cursor);
                    let indent = before.iter().take_while(|ch| ch.is_whitespace()).count();
                    n = if from < self.cursor { from - ls } else { indent.min(n) };
                    if n == before.len() {
                        n = 0;
                    }
                }
                let a = ls + n;
                if a < self.cursor {
                    self.rem(a, self.cursor);
                    self.cursor = a;
                    self.vim.insert_start = self.vim.insert_start.min(a);
                }
            }
            Code::Left => self.cursor = self.cursor.saturating_sub(1).max(self.line_start(self.cur_line())),
            Code::Right => self.cursor = (self.cursor + 1).min(self.line_end(self.cur_line())),
            Code::Char(c) if !k.ctrl && !k.alt => {
                // R: typing replaces
                self.vim_replace_char();
                self.ins(self.cursor, &c.to_string());
                self.cursor += 1;
                // past textwidth the line breaks at its last blank (formatoptions t)
                if c != ' ' {
                    self.vim_autowrap();
                }
            }
            _ => {}
        }
    }

    /// Comments lines `from..=to` out with `prefix` (`//`, `#`), or back in when every
    /// non-empty one already is (evil-commentary's gc). One undo step; leaves visual mode.
    pub fn toggle_comment(&mut self, from: usize, to: usize, prefix: &str) {
        self.toggle_comment_with(from, to, prefix, "");
    }

    /// As toggle_comment, with a closing part too (commentstring "/* %s */").
    pub fn toggle_comment_with(&mut self, from: usize, to: usize, prefix: &str, suffix: &str) {
        let last = self.buf.len_lines().saturating_sub(1);
        let (from, to) = (from.min(last), to.min(last));
        let lines: Vec<(usize, String)> = (from..=to).map(|l| (l, self.buf.line(l))).collect();
        let body = |s: &str| s.trim_end_matches(['\n', '\r']).to_string();
        let filled: Vec<&(usize, String)> = lines.iter().filter(|(_, t)| !body(t).trim().is_empty()).collect();
        if filled.is_empty() {
            return;
        }
        let commented = filled.iter().all(|(_, t)| body(t).trim_start().starts_with(prefix) && body(t).trim_end().ends_with(suffix));
        let indent = filled.iter().map(|(_, t)| body(t).len() - body(t).trim_start().len()).min().unwrap_or(0);
        self.begin();
        // bottom up, so earlier char positions stay right
        for (l, t) in filled.iter().rev() {
            let t = body(t);
            let start = self.buf.to_char(Pos { line: *l, col: 0 });
            if commented {
                // the closing part first (at the end), then the opening one
                if !suffix.is_empty() {
                    let end = start + t.trim_end().chars().count();
                    let mut n = suffix.chars().count();
                    if t.trim_end()[..t.trim_end().len() - suffix.len()].ends_with(' ') {
                        n += 1;
                    }
                    self.rem(end - n, end);
                }
                let lead = t.chars().count() - t.trim_start().chars().count();
                let mut n = prefix.chars().count();
                if t.trim_start()[prefix.len()..].starts_with(' ') {
                    n += 1;
                }
                self.rem(start + lead, start + lead + n);
            } else {
                if !suffix.is_empty() {
                    self.ins(start + t.trim_end().chars().count(), &format!(" {suffix}"));
                }
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
        self.apply_edits_in(edits, false);
        self.mode = Mode::Normal;
    }

    /// The edits (from a server), insert mode left as it is: an import added above keeps the
    /// cursor on its text; an edit over the cursor keeps its line and column.
    pub fn apply_edits_in(&mut self, mut edits: Vec<(Pos, Pos, String)>, keep_mode: bool) {
        if edits.is_empty() {
            return;
        }
        let keep = self.cursor();
        let mut at = self.cursor;
        let mut over = false;
        // last first, so earlier positions stay right
        edits.sort_by(|a, b| (b.0.line, b.0.col).cmp(&(a.0.line, a.0.col)));
        self.begin();
        for (a, b, text) in edits {
            let (s, e) = (self.buf.to_char(a), self.buf.to_char(b));
            if e > s {
                self.rem(s, e);
            }
            self.ins(s, &text);
            if e <= at {
                at = at + text.chars().count() - (e - s).min(at);
            } else if s < at {
                over = true;
            }
        }
        self.close();
        if over || !keep_mode {
            self.set_cursor(keep);
        } else {
            self.cursor = at.min(self.buf.len_chars());
        }
    }

    /// The cursor and the column j/k keep, exactly (drawing another window on the buffer puts
    /// them back as they were).
    pub fn cursor_state(&self) -> (usize, usize) {
        (self.cursor, self.want_col)
    }
    pub fn set_cursor_state(&mut self, s: (usize, usize)) {
        self.cursor = s.0.min(self.buf.len_chars());
        self.want_col = s.1;
    }

    /// The cursor as a character offset, and set from one.
    pub fn offset(&self) -> usize {
        self.cursor
    }
    pub fn set_offset(&mut self, at: usize) {
        self.cursor = at.min(self.buf.len_chars());
        self.want_col = self.cursor().col;
    }

    /// Ranges that move with edits (a snippet's stops), and setting them.
    pub fn ranges(&self) -> &[(usize, usize)] {
        &self.vim.ranges
    }
    pub fn set_ranges(&mut self, r: Vec<(usize, usize)>) {
        self.vim.ranges = r;
    }

}
