//! Normal mode: counts, motions, operators, the keys that start insert mode.

use super::*;

impl Editor {
    // ── normal mode ──

    pub(crate) fn take_count(&mut self) -> usize {
        let c = self.count.take().unwrap_or(1) * self.op_count.take().unwrap_or(1);
        c.max(1)
    }

    pub(crate) fn normal_key(&mut self, k: Key) {
        if k.code == Code::Esc {
            self.count = None;
            self.op = None;
            self.op_count = None;
            self.prefix = None;
            return;
        }
        if k.ctrl {
            if let Code::Char(c) = k.code.clone() {
                if c != 'r' && self.vim_ctrl(c) {
                    return;
                }
            }
            if k.code == Code::Char('r') {
                let n = self.take_count();
                for _ in 0..n {
                    self.redo_once();
                }
            }
            return;
        }
        let Some(c) = k.char() else {
            // arrows behave like h j k l
            let mapped = match k.code {
                Code::Left => 'h',
                Code::Right => 'l',
                Code::Up => 'k',
                Code::Down => 'j',
                Code::Home => '0',
                Code::End => '$',
                Code::Enter if self.prefix == Some('z') => {
                    self.prefix = None;
                    self.vim_z('\r');
                    return;
                }
                Code::Enter => return self.normal_key(Key::ch('+')),
                Code::Tab => {
                    self.vim_ctrl('i');
                    return;
                }
                Code::Backspace => return self.normal_key(Key::ctrl('h')),
                Code::Delete => 'x',
                _ => return,
            };
            return self.normal_key(Key::ch(mapped));
        };

        // second key of a two-key command
        if let Some(p) = self.prefix.take() {
            match (p, c) {
                // g- g+: back and forth in time, across branches
                ('g', '-') => {
                    let n = self.take_count();
                    return self.undo_goto(self.undo.cur.saturating_sub(n));
                }
                ('g', '+') => {
                    let n = self.take_count();
                    return self.undo_goto((self.undo.cur + n).min(self.undo.nodes.len() - 1));
                }
                ('g', 'g') => {
                    let line = self.count.take().map(|n| n.saturating_sub(1)).unwrap_or(0);
                    return self.motion(Span::Lines(self.cur_line(), line.min(self.buf.len_lines() - 1)), Some(self.first_non_blank(line.min(self.buf.len_lines() - 1))));
                }
                ('r', ch) if matches!(self.mode, Mode::Visual | Mode::VisualLine) => return self.visual_replace(ch),
                ('r', ch) => return self.replace_chars(ch),
                ('i' | 'a', obj) => return self.text_object_key(p == 'i', obj),
                ('f' | 'F' | 't' | 'T', ch) => {
                    self.last_find = Some((p, ch));
                    return self.find(p, ch);
                }
                _ => {
                    if !self.vim_prefixed(p, c) {
                        self.count = None;
                        self.op = None;
                    }
                    return;
                }
            }
        }
        // the same operator twice: its lines (g~~, gUU, >>, ==)
        if self.vim_double(c) {
            return;
        }

        if c.is_ascii_digit() && (c != '0' || self.count.is_some()) {
            let d = c.to_digit(10).unwrap() as usize;
            self.count = Some(self.count.unwrap_or(0).saturating_mul(10).saturating_add(d));
            return;
        }

        let line = self.cur_line();
        match c {
            // motions
            'h' => {
                let n = self.take_count();
                let to = self.cursor.saturating_sub(n).max(self.line_start(line));
                self.motion(Span::Chars(to, self.cursor), Some(to));
            }
            'l' => {
                let n = self.take_count();
                let to = (self.cursor + n).min(self.line_end(line));
                self.motion(Span::Chars(self.cursor, to), Some(to));
            }
            'j' | 'k' => {
                let n = self.take_count();
                let last = self.buf.len_lines() - 1;
                let to = if c == 'j' { (line + n).min(last) } else { line.saturating_sub(n) };
                self.motion(Span::Lines(line, to), None);
                if self.op.is_none() {
                    self.vertical_to(to);
                }
            }
            'w' | 'W' => {
                let n = self.take_count();
                let mut to = self.cursor;
                for _ in 0..n {
                    to = self.word_forward(to);
                }
                if let Some(op) = self.op {
                    // `cw` on a word is `ce`; `dw` never eats the line break
                    if op == Op::Change && self.char(self.cursor).is_some_and(|ch| !ch.is_whitespace()) {
                        let mut e = self.cursor;
                        for i in 0..n {
                            e = if i == 0 && self.is_word_end(e) { e } else { self.word_end(e) };
                        }
                        return self.motion(Span::Chars(self.cursor, e + 1), None);
                    }
                    let end = to.min(self.line_end(line).max(self.cursor + 1));
                    let end = if self.buf.to_pos(to).line > line { self.line_end(line) } else { end };
                    return self.motion(Span::Chars(self.cursor, end), None);
                }
                self.motion(Span::Chars(self.cursor, to), Some(to));
            }
            'e' | 'E' => {
                let n = self.take_count();
                let mut to = self.cursor;
                for _ in 0..n {
                    to = self.word_end(to);
                }
                self.motion(Span::Chars(self.cursor, to + 1), Some(to));
            }
            'b' | 'B' => {
                let n = self.take_count();
                let mut to = self.cursor;
                for _ in 0..n {
                    to = self.word_back(to);
                }
                self.motion(Span::Chars(to, self.cursor), Some(to));
            }
            '0' => {
                let s = self.line_start(line);
                self.motion(Span::Chars(s, self.cursor), Some(s));
            }
            '^' => {
                let s = self.first_non_blank(line);
                self.motion(Span::Chars(s.min(self.cursor), s.max(self.cursor)), Some(s));
            }
            '$' => {
                let n = self.take_count();
                let target = (line + n - 1).min(self.buf.len_lines() - 1);
                let e = self.line_end(target);
                self.motion(Span::Chars(self.cursor, e), Some(e));
                self.want_col = usize::MAX;
            }
            'G' => {
                let last = self.buf.len_lines() - 1;
                let to = self.count.take().map(|n| n.saturating_sub(1).min(last)).unwrap_or(last);
                self.motion(Span::Lines(line, to), Some(self.first_non_blank(to)));
            }
            'g' | 'r' | 'f' | 'F' | 't' | 'T' => self.prefix = Some(c),
            ';' | ',' => {
                if let Some((k, ch)) = self.last_find {
                    // , goes the other way
                    let k = if c == ',' { match k { 'f' => 'F', 'F' => 'f', 't' => 'T', _ => 't' } } else { k };
                    self.find(k, ch);
                } else {
                    self.count = None;
                    self.op = None;
                }
            }

            // operators: `dd` `cc` `yy` act on lines
            'd' | 'c' | 'y' => {
                let op = match c { 'd' => Op::Delete, 'c' => Op::Change, _ => Op::Yank };
                if self.op == Some(op) {
                    let n = self.take_count();
                    let to = (line + n - 1).min(self.buf.len_lines() - 1);
                    return self.apply(op, Span::Lines(line, to));
                }
                self.op = Some(op);
                self.op_count = self.count.take();
            }

            // shortcuts
            'x' => { let n = self.take_count(); let e = (self.cursor + n).min(self.line_end(line)); if e > self.cursor { self.apply(Op::Delete, Span::Chars(self.cursor, e)); } }
            'X' => { let n = self.take_count(); let s = self.cursor.saturating_sub(n).max(self.line_start(line)); if s < self.cursor { self.apply(Op::Delete, Span::Chars(s, self.cursor)); } }
            'D' => { let e = self.line_end(line); self.apply(Op::Delete, Span::Chars(self.cursor, e)); }
            'C' => { let e = self.line_end(line); self.apply(Op::Change, Span::Chars(self.cursor, e)); }
            'p' | 'P' => { let n = self.take_count(); self.put(c == 'p', n); }
            'J' => { let n = self.take_count().max(2) - 1; self.join(n); }

            // into insert mode
            // with an operator pending, i and a start a text object (`diw`, `ca"`)
            'i' | 'a' if self.op.is_some() => self.prefix = Some(c),
            'v' => { self.anchor = self.cursor; self.mode = Mode::Visual; self.count = None; }
            'V' => { self.anchor = self.cursor; self.mode = Mode::VisualLine; self.count = None; }
            'i' => self.start_insert(self.cursor),
            'a' => { let at = (self.cursor + 1).min(self.line_end(line)); self.start_insert(at) }
            'I' => self.start_insert(self.first_non_blank(line)),
            'A' => self.start_insert(self.line_end(line)),
            // autoindent (Neovim's default): the new line starts at this one's indent
            'o' => { self.begin(); let ind = self.indent_of(line); let e = self.line_end(line); self.ins(e, &format!("\n{ind}")); self.mode = Mode::Insert; self.cursor = e + 1 + ind.chars().count(); }
            'O' => { self.begin(); let ind = self.indent_of(line); let s = self.line_start(line); self.ins(s, &format!("{ind}\n")); self.mode = Mode::Insert; self.cursor = s + ind.chars().count(); }

            'u' => { let n = self.take_count(); for _ in 0..n { self.undo_once(); } }
            _ if self.vim_more(c) => {}
            _ => { self.count = None; self.op = None; }
        }
    }

}
