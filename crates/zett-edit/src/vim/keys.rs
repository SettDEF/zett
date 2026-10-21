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

    /// CTRL-A / CTRL-X: the number at or after the cursor on this line, plus `by`, in the formats
    /// nrformats allows (bin,hex by default; octal; alpha: a single letter).
    pub(crate) fn vim_add(&mut self, by: i64) {
        let line = self.cur_line();
        let (s, e) = (self.line_start(line), self.line_end(line));
        let text: Vec<char> = self.buf.slice(s, e).chars().collect();
        let col = self.cursor - s;
        let has = |f: &str| self.opt_has("nrformats", f, "bin,hex");
        // back to the start of a number the cursor is in
        let mut i = col.min(text.len());
        while i > 0 && text.get(i).is_some_and(|c| c.is_ascii_hexdigit()) && text[i - 1].is_ascii_hexdigit() {
            i -= 1;
        }
        let Some(d) = (i..text.len()).find(|&j| text[j].is_ascii_digit()) else {
            // alpha: the letter under or after the cursor
            if has("alpha") {
                if let Some(j) = (col..text.len()).find(|&j| text[j].is_ascii_alphabetic()) {
                    let c = text[j];
                    let base = if c.is_ascii_lowercase() { b'a' } else { b'A' };
                    let n = ((c as u8 - base) as i64 + by).clamp(0, 25) as u8;
                    let new = ((base + n) as char).to_string();
                    self.replace_range(s + j, s + j + 1, &new);
                    self.cursor = s + j;
                }
            }
            return;
        };
        let prefixed = |p: [char; 2]| d + 2 < text.len() && text[d] == '0' && (text[d + 1] == p[0] || text[d + 1] == p[1]);
        let radix = if has("hex") && prefixed(['x', 'X']) {
            16
        } else if has("bin") && prefixed(['b', 'B']) {
            2
        } else if has("octal") && text[d] == '0' && text.get(d + 1).is_some_and(|c| ('0'..='7').contains(c)) {
            8
        } else {
            10
        };
        let num_start = if radix == 16 || radix == 2 { d + 2 } else { d };
        let mut end = num_start;
        while end < text.len() && text[end].is_digit(radix) {
            end += 1;
        }
        let digits: String = text[num_start..end].iter().collect();
        let (from, new) = if radix == 10 {
            let neg = d > 0 && text[d - 1] == '-' && !has("unsigned");
            let v: i64 = digits.parse::<i64>().unwrap_or(0) * if neg { -1 } else { 1 };
            let v = v.saturating_add(by);
            let v = if has("unsigned") { v.max(0) } else { v };
            (if neg { d - 1 } else { d }, v.to_string())
        } else {
            let v = i64::from_str_radix(&digits, radix).unwrap_or(0).wrapping_add(by).max(0);
            let w = digits.len();
            let upper = digits.chars().any(|c| c.is_ascii_uppercase());
            let s = match radix {
                16 if upper => format!("{v:0w$X}"),
                16 => format!("{v:0w$x}"),
                2 => format!("{v:0w$b}"),
                _ => format!("{v:0w$o}"),
            };
            (num_start, s)
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
                    self.vim.regs.insert(r, Register { text, linewise: false, block: false });
                    self.vim.macros.insert(r, keys);
                    self.vim.last_recorded = Some(r);
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
            '~' if self.opt_on("tildeop", false) => self.vim_op(Op::Toggle),
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
            ' ' => {
                // whichwrap s: Space goes on to the next line
                let line = self.cur_line();
                if self.cursor + 1 >= self.line_end(line) && line + 1 < self.buf.len_lines() && self.opt_letter("whichwrap", 's', "b,s") && self.op.is_none() {
                    self.count = None;
                    self.cursor = self.line_start(line + 1);
                } else {
                    self.normal_key(Key::ch('l'));
                }
            }
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
                self.motion(Span::Lines(line, to), Some(self.sol_target(to)));
            }
            'K' => {
                if let Some((a, b)) = self.word_here() {
                    self.vim.requests.push(Request::Keyword(self.buf.slice(a, b)));
                }
            }
            '&' => self.vim.requests.push(Request::Ex("s".into())),
            // Neovim: Q replays the register recorded last (gQ is Ex mode)
            'Q' => {
                let r = self.vim.last_recorded.unwrap_or('q');
                self.vim_prefixed('@', r);
            }
            'U' => {
                if let Some((l, was)) = self.vim.uline.clone().filter(|u| u.0 < self.buf.len_lines()) {
                    let now = self.buf.line(l);
                    let (s, e) = (self.line_start(l), self.line_end(l));
                    self.replace_range(s, e, was.trim_end_matches('\n'));
                    // U again undoes the U
                    self.vim.uline = Some((l, now));
                    self.cursor = s;
                    self.clamp();
                }
            }
            ':' => {
                let n = self.count.take();
                self.vim.requests.push(Request::CmdLine(n.map(|n| if n > 1 { format!(".,.+{}", n - 1) } else { ".".into() }).unwrap_or_default()));
            }
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
                // the jump back to here (g' and g` leave it and the jump list alone)
                if !std::mem::take(&mut self.vim.no_jump) {
                    self.vim.marks.insert('\'', self.cursor);
                }
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
            ('[' | ']', c) => return self.vim_bracket(p, c),
            ('q', w @ (':' | '/' | '?')) => self.vim.requests.push(Request::CmdWindow(w)),
            (CW | CWG | CBS, c) => return self.vim_window_prefixed(p, c),
            (super::spelling::ZU, c) if matches!(c, 'g' | 'G' | 'w' | 'W') => self.spell_mark(c, true),
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
