//! g and a key.

use super::*;

impl Editor {
    /// `g` and a key.
    pub(crate) fn vim_g(&mut self, c: char) -> bool {
        let line = self.cur_line();
        match c {
            '~' => self.vim_op(Op::Toggle),
            'u' => self.vim_op(Op::Lower),
            'U' => self.vim_op(Op::Upper),
            '?' => self.vim_op(Op::Rot13),
            'q' | 'w' => self.vim_op(Op::Format),
            '@' => self.vim_op(Op::Func),
            'J' => {
                let n = self.take_count().max(2) - 1;
                self.begin();
                for _ in 0..n {
                    let l = self.cur_line();
                    if l + 1 >= self.buf.len_lines() {
                        break;
                    }
                    let e = self.line_end(l);
                    self.rem(e, e + 1);
                    self.cursor = e;
                }
                self.close();
                self.clamp();
            }
            'i' => {
                let at = self.vim.last_insert.unwrap_or(self.cursor).min(self.buf.len_chars());
                self.start_insert(at);
            }
            'I' => {
                let s = self.line_start(line);
                self.start_insert(s);
            }
            'v' => {
                if let Some((a, b, m)) = self.vim.last_visual {
                    let n = self.buf.len_chars();
                    self.anchor = a.min(n);
                    self.cursor = b.min(n);
                    self.mode = m;
                }
            }
            ';' | ',' => {
                let n = self.vim.changes.len();
                if n == 0 {
                    self.vim.requests.push(Request::Message("E664: changelist is empty".into()));
                    return true;
                }
                let i = if c == ';' { self.vim.change_at.saturating_sub(1) } else { (self.vim.change_at + 1).min(n - 1) };
                self.vim.change_at = i;
                self.cursor = self.vim.changes[i].min(self.buf.len_chars());
                self.clamp();
            }
            'e' | 'E' => {
                // back to the end of the previous word
                let n = self.take_count();
                let mut i = self.cursor;
                for _ in 0..n {
                    let mut j = i;
                    let cls = |e: &Self, k: usize| e.char(k).map(|ch| if ch.is_whitespace() { 0 } else if c == 'E' || is_word(ch) { 1 } else { 2 }).unwrap_or(0);
                    let start = cls(self, j);
                    while j > 0 && cls(self, j) == start && start != 0 {
                        j -= 1;
                    }
                    while j > 0 && cls(self, j) == 0 {
                        j -= 1;
                    }
                    i = j;
                }
                self.motion(Span::Chars(i, self.cursor + 1), Some(i));
            }
            '_' => {
                let n = self.take_count();
                let to = (line + n - 1).min(self.buf.len_lines() - 1);
                let (s, mut e) = (self.line_start(to), self.line_end(to));
                while e > s && self.char(e - 1).is_some_and(char::is_whitespace) {
                    e -= 1;
                }
                let at = e.saturating_sub(1).max(s);
                self.motion(Span::Chars(self.cursor.min(at), self.cursor.max(at) + 1), Some(at));
            }
            '0' | '^' | '$' | 'm' | 'M' => {
                // the screen row the cursor is on (the whole line when lines do not wrap)
                let (s, e) = (self.line_start(line), self.line_end(line));
                let w = self.vim.screen.2.max(1);
                let (rs, re) = if self.vim.wrap {
                    let row = (self.cursor - s) / w;
                    (s + row * w, (s + (row + 1) * w).min(e))
                } else {
                    (s, e)
                };
                let at = match c {
                    '0' => rs,
                    '^' => (rs..re).find(|&i| self.char(i).is_some_and(|ch| !ch.is_whitespace())).unwrap_or(rs),
                    '$' => re.saturating_sub(1).max(rs),
                    'm' => (rs + w / 2).min(re.saturating_sub(1).max(rs)),
                    _ => s + (e - s) / 2,
                };
                self.motion(Span::Chars(self.cursor.min(at), self.cursor.max(at)), Some(at));
            }
            'j' | 'k' => {
                // down or up a screen row; without wrap the same as j and k
                if !self.vim.wrap {
                    self.normal_key(Key::ch(c));
                    return true;
                }
                let w = self.vim.screen.2.max(1);
                let n = self.take_count();
                let mut at = self.cursor;
                for _ in 0..n {
                    let l = self.buf.to_pos(at).line;
                    let (s, e) = (self.line_start(l), self.line_end(l));
                    let col = (at - s) % w;
                    at = if c == 'j' {
                        if at + w <= e.max(s) && (at - s) / w < (e - s) / w { (at + w).min(e.saturating_sub(1).max(s)) } else if l + 1 < self.buf.len_lines() { (self.line_start(l + 1) + col).min(self.line_end(l + 1).saturating_sub(1).max(self.line_start(l + 1))) } else { at }
                    } else if at - s >= w {
                        at - w
                    } else if l > 0 {
                        let (ps, pe) = (self.line_start(l - 1), self.line_end(l - 1));
                        let last_row = (pe - ps) / w;
                        (ps + last_row * w + col).min(pe.saturating_sub(1).max(ps))
                    } else {
                        at
                    };
                }
                self.motion(Span::Chars(self.cursor.min(at), self.cursor.max(at) + 1), Some(at));
            }
            '\'' | '`' => {
                // g' g`: to a mark without the jump list
                self.vim.no_jump = true;
                self.prefix = Some(c);
            }
            ']' => self.vim.requests.push(Request::Definition),
            'o' => {
                // to byte N (1 by default)
                let n = self.count.take().unwrap_or(1).saturating_sub(1);
                let text = self.buf.text();
                let at = text.char_indices().take_while(|(b, _)| *b <= n).count().saturating_sub(1);
                self.motion(Span::Chars(self.cursor.min(at), self.cursor.max(at)), Some(at));
            }
            'p' | 'P' => {
                let n = self.take_count();
                self.put(c == 'p', n);
                // after what was put
                let len = self.reg.text.chars().count() * n;
                if self.reg.linewise {
                    let l = (self.cur_line() + self.reg.text.matches('\n').count() * n).min(self.buf.len_lines() - 1);
                    self.cursor = self.line_start(l);
                } else {
                    self.cursor = (self.cursor + 1).min(self.buf.len_chars()).max(self.cursor + len.min(1));
                }
            }
            '*' | '#' => self.vim_star(c == '*', false),
            'n' | 'N' => {
                // select the next match
                let back = self.vim.search_back;
                let fwd = if c == 'n' { !back } else { back };
                self.vim_next(fwd);
                let n = self.last_search.chars().count();
                if n > 0 {
                    self.anchor = self.cursor;
                    self.cursor = (self.cursor + n - 1).min(self.buf.len_chars());
                    self.mode = Mode::Visual;
                }
            }
            'a' | '8' => {
                if let Some(ch) = self.char(self.cursor) {
                    let m = if c == 'a' { format!("<{ch}> {}, Hex {:02x}, Oct {:o}", ch as u32, ch as u32, ch as u32) } else { ch.to_string().bytes().map(|b| format!("{b:02x}")).collect::<Vec<_>>().join(" ") };
                    self.vim.requests.push(Request::Message(m));
                }
            }
            's' => {
                let n = self.take_count();
                self.vim.requests.push(Request::Sleep(n as u64 * 1000));
            }
            '&' => self.vim.requests.push(Request::Ex("%s//~/&".into())),
            '<' => self.vim.requests.push(Request::Ex("messages".into())),
            'Q' => self.vim.requests.push(Request::CmdLine(String::new())),
            't' | 'T' => {
                let n = self.count.take();
                self.vim.requests.push(Request::Tab(match (c, n) { ('t', Some(n)) => 1000 + n as i32, ('t', None) => 1, _ => -(n.unwrap_or(1) as i32) }));
            }
            'x' | 'f' | 'F' => {
                // the address or file name under the cursor
                let (s, e) = (self.line_start(line), self.line_end(line));
                let ok = |ch: char| !ch.is_whitespace() && !matches!(ch, '"' | '\'' | '<' | '>' | '(' | ')' | '[' | ']' | '`');
                let (mut a, mut b) = (self.cursor, self.cursor);
                while a > s && self.char(a - 1).is_some_and(ok) {
                    a -= 1;
                }
                while b < e && self.char(b).is_some_and(ok) {
                    b += 1;
                }
                if b > a {
                    self.vim.requests.push(Request::Open(self.buf.slice(a, b)));
                }
            }
            'd' | 'D' => {
                // the word's first use (in the file: gD; Vim's gd looks from the function start)
                let Some((a, b)) = self.word_here() else { return true };
                let w = self.buf.slice(a, b);
                let mut from = 0;
                while let Some(i) = self.buf.find(&w, from, true, false) {
                    if self.whole_word_at(i, w.chars().count()) {
                        self.vim.marks.insert('\'', self.cursor);
                        self.last_search = w.clone();
                        self.vim.search_word = true;
                        self.cursor = i;
                        self.clamp();
                        break;
                    }
                    from = i + 1;
                }
            }
            'h' | 'H' | 'V' => {
                // select mode: zett's visual
                self.anchor = self.cursor;
                self.mode = if c == 'H' { Mode::VisualLine } else { Mode::Visual };
            }
            'R' | 'r' => {
                if c == 'R' {
                    self.start_insert(self.cursor);
                    self.vim.replace = true;
                } else {
                    self.prefix = Some('r');
                }
            }
            _ => return false,
        }
        true
    }

}
