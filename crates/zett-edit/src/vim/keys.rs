//! Single keys, Ctrl keys and prefixes (m ' " q @ Z z [ ]) of normal mode.

use super::*;

impl Editor {
    /// A Ctrl key in normal mode; false when it is not one of these.
    pub(crate) fn vim_ctrl(&mut self, c: char) -> bool {
        if self.vim_ctrl2(c) {
            return true;
        }
        match c {
            'a' | 'x' => {
                let n = self.take_count() as i64;
                self.vim_add(if c == 'a' { n } else { -n });
            }
            'n' | 'j' => self.normal_key(Key::ch('j')),
            'p' => self.normal_key(Key::ch('k')),
            'h' => self.vim_back_char(),
            'm' => self.vim_line_motion(1),
            _ => return false,
        }
        true
    }

    pub(crate) fn vim_back_char(&mut self) {
        // <BS>: left, onto the line above at a line start
        let n = self.take_count();
        let to = self.cursor.saturating_sub(n);
        self.motion(Span::Chars(to, self.cursor), Some(to));
    }

    /// `+` `<CR>` (down) and `-` (up) to the first non-blank, `_` (count-1 lines down).
    pub(crate) fn vim_line_motion(&mut self, dir: i64) {
        let n = self.take_count() as i64;
        let line = self.cur_line() as i64;
        let last = self.buf.len_lines() as i64 - 1;
        let to = if dir == 0 { (line + n - 1).min(last) } else { (line + dir * n).clamp(0, last) } as usize;
        self.motion(Span::Lines(self.cur_line(), to), Some(self.first_non_blank(to)));
    }

    /// CTRL-A / CTRL-X: the number at or after the cursor on this line, plus `by`.
    pub(crate) fn vim_add(&mut self, by: i64) {
        let line = self.cur_line();
        let (s, e) = (self.line_start(line), self.line_end(line));
        let text: Vec<char> = self.buf.slice(s, e).chars().collect();
        let col = self.cursor - s;
        // the number: a run of digits touching or after the cursor (0x… hex, a - before it)
        let mut i = col.min(text.len());
        while i > 0 && text.get(i).is_some_and(|c| c.is_ascii_hexdigit()) && text[i - 1].is_ascii_hexdigit() {
            i -= 1;
        }
        let start_digit = (i..text.len()).find(|&j| text[j].is_ascii_digit());
        let Some(d) = start_digit else { return };
        let hex = d + 1 < text.len() && text[d] == '0' && matches!(text[d + 1], 'x' | 'X');
        let (num_start, mut end) = if hex { (d + 2, d + 2) } else { (d, d) };
        while end < text.len() && (if hex { text[end].is_ascii_hexdigit() } else { text[end].is_ascii_digit() }) {
            end += 1;
        }
        let digits: String = text[num_start..end].iter().collect();
        let (from, new) = if hex {
            let v = i64::from_str_radix(&digits, 16).unwrap_or(0).wrapping_add(by);
            let upper = digits.chars().any(|c| c.is_ascii_uppercase());
            let s = if upper { format!("{:0w$X}", v.max(0), w = digits.len()) } else { format!("{:0w$x}", v.max(0), w = digits.len()) };
            (num_start, s)
        } else {
            let neg = d > 0 && text[d - 1] == '-';
            let v: i64 = digits.parse::<i64>().unwrap_or(0) * if neg { -1 } else { 1 };
            let v = v.saturating_add(by);
            (if neg { d - 1 } else { d }, v.to_string())
        };
        self.begin();
        self.rem(s + from, s + end);
        self.ins(s + from, &new);
        self.close();
        self.cursor = s + from + new.chars().count() - 1;
    }

    /// One key in normal mode that lib.rs does not know; false when it is not one of these either.
    pub(crate) fn vim_more(&mut self, c: char) -> bool {
        let line = self.cur_line();
        match c {
            '/' | '?' => self.vim_search_start(c == '/'),
            'n' => {
                let back = self.vim.search_back;
                self.vim_next(!back);
            }
            'N' => {
                let back = self.vim.search_back;
                self.vim_next(back);
            }
            '*' | '#' => self.vim_star(c == '*', true),
            'm' | '\'' | '`' | '"' | 'q' | '@' | 'Z' | 'z' | '[' | ']' => {
                // q alone stops a recording
                if c == 'q' && self.vim.recording.is_some() {
                    let (r, mut keys) = self.vim.recording.take().unwrap();
                    keys.pop();
                    let text = keys.iter().map(|k| k.notation()).collect::<String>();
                    self.vim.regs.insert(r, Register { text, linewise: false });
                    self.vim.macros.insert(r, keys);
                    return true;
                }
                self.prefix = Some(c);
            }
            '.' => {
                let n = self.count.take();
                let keys = self.vim.dot.clone();
                self.vim.replaying += 1;
                for _ in 0..n.unwrap_or(1) {
                    // a count given to . replaces the one the change had
                    for k in &keys {
                        self.key(*k);
                    }
                }
                self.vim.replaying -= 1;
            }
            '~' => {
                let n = self.take_count();
                let e = (self.cursor + n).min(self.line_end(line));
                if e > self.cursor {
                    let t: String = self.buf.slice(self.cursor, e).chars().map(toggle).collect();
                    self.begin();
                    let at = self.cursor;
                    self.rem(at, e);
                    self.ins(at, &t);
                    self.close();
                    self.cursor = e;
                    self.clamp();
                }
            }
            's' => {
                let n = self.take_count();
                let e = (self.cursor + n).min(self.line_end(line));
                self.apply(Op::Change, Span::Chars(self.cursor, e));
            }
            'S' => {
                let n = self.take_count();
                self.apply(Op::Change, Span::Lines(line, (line + n - 1).min(self.buf.len_lines() - 1)));
            }
            'Y' => {
                let n = self.take_count();
                self.apply(Op::Yank, Span::Lines(line, (line + n - 1).min(self.buf.len_lines() - 1)));
            }
            'R' => {
                self.start_insert(self.cursor);
                self.vim.replace = true;
            }
            '+' => self.vim_line_motion(1),
            '-' => self.vim_line_motion(-1),
            '_' => self.vim_line_motion(0),
            '|' => {
                let n = self.take_count();
                let to = (self.line_start(line) + n - 1).min(self.line_end(line));
                self.motion(Span::Chars(self.cursor.min(to), self.cursor.max(to)), Some(to));
            }
            ' ' => self.normal_key(Key::ch('l')),
            '>' | '<' | '=' | '!' => {
                let op = match c { '>' => Op::ShiftRight, '<' => Op::ShiftLeft, '=' => Op::Reindent, _ => Op::Filter };
                self.vim_op(op);
            }
            '(' | ')' => self.vim_sentence(c == ')'),
            '{' | '}' => self.vim_paragraph(c == '}'),
            '%' => self.vim_percent(),
            'H' | 'M' | 'L' => {
                let (top, height, _) = self.vim.screen;
                let last = self.buf.len_lines() - 1;
                let bottom = (top + height.max(1) - 1).min(last);
                let n = self.count.take().unwrap_or(1).saturating_sub(1);
                let to = match c { 'H' => (top + n).min(bottom), 'L' => bottom.saturating_sub(n).max(top), _ => (top + bottom) / 2 };
                self.motion(Span::Lines(line, to), Some(self.first_non_blank(to)));
            }
            'K' => {
                if let Some((a, b)) = self.word_here() {
                    self.vim.requests.push(Request::Keyword(self.buf.slice(a, b)));
                }
            }
            '&' => self.vim.requests.push(Request::Ex("s".into())),
            'Q' => self.vim.requests.push(Request::CmdLine(String::new())),
            _ => return false,
        }
        true
    }

    /// An operator key: twice (or with the same g-key) means the lines (`>>`, `g~~`, `gUU`).
    pub(crate) fn vim_op(&mut self, op: Op) {
        if self.op == Some(op) {
            let n = self.take_count();
            let line = self.cur_line();
            let to = (line + n - 1).min(self.buf.len_lines() - 1);
            return self.apply(op, Span::Lines(line, to));
        }
        self.op = Some(op);
        self.op_count = self.count.take();
    }

    /// The second key after a prefix lib.rs does not handle; false when not one of these.
    pub(crate) fn vim_prefixed(&mut self, p: char, c: char) -> bool {
        match (p, c) {
            ('m', c) if c.is_ascii_alphabetic() || matches!(c, '\'' | '`' | '[' | ']' | '<' | '>') => {
                self.vim.marks.insert(c, self.cursor);
            }
            ('\'' | '`', m) => {
                let m = if m == '`' { '\'' } else { m };
                let at = match m {
                    '<' => self.vim.last_visual.map(|(a, b, _)| a.min(b)),
                    '>' => self.vim.last_visual.map(|(a, b, _)| a.max(b)),
                    _ => self.vim.marks.get(&m).copied(),
                };
                let Some(at) = at.map(|a| a.min(self.buf.len_chars())) else {
                    self.op = None;
                    self.count = None;
                    self.vim.requests.push(Request::Message("E20: Mark not set".into()));
                    return true;
                };
                // the jump back to here
                self.vim.marks.insert('\'', self.cursor);
                let line = self.cur_line();
                if p == '\'' {
                    let to = self.buf.to_pos(at).line;
                    self.motion(Span::Lines(line, to), Some(self.first_non_blank(to)));
                } else {
                    self.motion(Span::Chars(self.cursor.min(at), self.cursor.max(at)), Some(at));
                }
            }
            ('"', r) if r.is_ascii_alphanumeric() || matches!(r, '"' | '-' | '_' | '+' | '*' | '.' | ':' | '/') => {
                self.vim_use_reg(r);
            }
            ('q', r) if r.is_ascii_alphanumeric() || r == '"' => {
                self.vim.recording = Some((r.to_ascii_lowercase(), Vec::new()));
            }
            ('@', r) => {
                let r = if r == '@' { self.vim.last_macro.unwrap_or('q') } else { r.to_ascii_lowercase() };
                if r == ':' {
                    self.vim.requests.push(Request::Ex("@:".into()));
                    return true;
                }
                self.vim.last_macro = Some(r);
                let keys = self.vim.macros.get(&r).cloned().unwrap_or_else(|| crate::keys::parse(&self.vim.regs.get(&r).map(|x| x.text.clone()).unwrap_or_default()));
                let n = self.take_count();
                if self.vim.replaying > 20 {
                    return true;
                }
                self.vim.replaying += 1;
                for _ in 0..n {
                    for k in &keys {
                        self.key(*k);
                    }
                }
                self.vim.replaying -= 1;
            }
            ('z', c) => return self.vim_z(c),
            ('q', w @ (':' | '/' | '?')) => self.vim.requests.push(Request::CmdWindow(w)),
            (CW | CWG | CBS, c) => return self.vim_window_prefixed(p, c),
            ('Z', 'Z') => self.vim.requests.push(Request::Ex("x".into())),
            ('Z', 'Q') => self.vim.requests.push(Request::Ex("q!".into())),
            ('g', _) => return self.vim_g(c),
            _ => return false,
        }
        true
    }

    /// With an operator pending, its own key again means its lines (`g~~` `guu` `gUU` `g??` `>>`).
    pub(crate) fn vim_double(&mut self, c: char) -> bool {
        let same = matches!((self.op, c), (Some(Op::Toggle), '~') | (Some(Op::Lower), 'u') | (Some(Op::Upper), 'U') | (Some(Op::Rot13), '?') | (Some(Op::Format), 'q' | 'w') | (Some(Op::ShiftRight), '>') | (Some(Op::ShiftLeft), '<') | (Some(Op::Reindent), '=') | (Some(Op::Filter), '!'));
        if same {
            let op = self.op.unwrap();
            self.vim_op(op);
        }
        same
    }


}

impl Editor {
    /// A key no command takes: what was pending is dropped, and it is counted (probe.rs).
    pub(crate) fn vim_unknown(&mut self) {
        self.count = None;
        self.op = None;
        self.vim.unknown += 1;
    }

    /// With an operator waiting, keys that are not motions or text objects cancel it.
    pub(crate) fn vim_op_refuses(&mut self, c: char) -> bool {
        let Some(op) = self.op else { return false };
        let motion = matches!(c, 'h' | 'j' | 'k' | 'l' | 'w' | 'W' | 'e' | 'E' | 'b' | 'B' | '0' | '^' | '$' | 'G' | 'g' | 'f' | 'F' | 't' | 'T' | ';' | ',' | '/' | '?' | 'n' | 'N' | '*' | '#' | '%' | '(' | ')' | '{' | '}' | 'H' | 'M' | 'L' | '|' | '+' | '-' | '_' | '\'' | '`' | '[' | ']' | 'i' | 'a' | 'v' | 'V' | ' ');
        if motion {
            return false;
        }
        if op == Op::Delete && matches!(c, 'o' | 'p') {
            self.op = None;
            self.count = None;
            self.vim.requests.push(Request::Message("E99: Current buffer is not in diff mode".into()));
            return true;
        }
        // the same operator's own key (dd cc yy >> g~~ …) is not refused: it means the lines
        let doubles = matches!((op, c), (Op::Delete, 'd') | (Op::Change, 'c') | (Op::Yank, 'y'));
        if doubles {
            return false;
        }
        self.vim_unknown();
        true
    }
}
