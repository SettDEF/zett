//! Visual mode: v and V, the selection, operators and keys on it.

use super::*;

impl Editor {
    // ── visual mode ──

    /// The selection as a span: characters from the anchor to the cursor (both
    /// included), or whole lines in `V`.
    pub(crate) fn selection(&self) -> Span {
        if self.mode == Mode::VisualLine {
            return Span::Lines(self.buf.to_pos(self.anchor).line, self.cur_line());
        }
        let (a, b) = (self.anchor.min(self.cursor), self.anchor.max(self.cursor));
        Span::Chars(a, (b + 1).min(self.buf.len_chars()))
    }

    /// The selected characters (start, end exclusive), for drawing and for commands on a
    /// selection: Vim's visual selection, or the Emacs profile's active region.
    pub fn selection_range(&self) -> Option<(usize, usize)> {
        if self.emacs {
            return self.region().filter(|(a, b)| a < b);
        }
        match self.mode {
            Mode::Visual | Mode::VisualLine => Some(match self.selection() {
                Span::Chars(a, b) => (a, b),
                Span::Lines(l1, l2) => {
                    let (l1, l2) = (l1.min(l2), l1.max(l2));
                    (self.line_start(l1), self.line_end(l2))
                }
            }),
            _ => None,
        }
    }

    pub(crate) fn leave_visual(&mut self) {
        self.vim.last_visual = Some((self.anchor, self.cursor, self.mode));
        self.mode = Mode::Normal;
        self.want_col = self.want_col.min(usize::MAX - 1);
        self.clamp();
    }

    /// An operator on the selection (`d`, `c`, `y`); `lines` forces whole lines (`D`, `Y`, `X`, `S`).
    pub(crate) fn visual_op(&mut self, op: Op, lines: bool) {
        self.vim.last_visual = Some((self.anchor, self.cursor, self.mode));
        let span = match self.selection() {
            Span::Chars(a, b) if lines => Span::Lines(self.buf.to_pos(a).line, self.buf.to_pos(b.saturating_sub(1).max(a)).line),
            s => s,
        };
        let start = match span {
            Span::Chars(a, _) => a,
            Span::Lines(l1, l2) => self.line_start(l1.min(l2)),
        };
        self.mode = Mode::Normal;
        self.want_col = 0;
        if op == Op::Yank {
            // the cursor goes to the start of what was yanked
            self.apply(op, span);
            self.cursor = match span {
                Span::Chars(a, _) => a,
                Span::Lines(l1, l2) => self.line_start(l1.min(l2)) + self.buf.to_pos(self.anchor.min(self.cursor)).col.min(0),
            };
            if let Span::Lines(..) = span {
                self.cursor = start.max(self.cursor.min(start));
            }
            self.clamp();
            self.want_col = self.cursor().col;
            return;
        }
        self.apply(op, span);
    }

    pub(crate) fn visual_key(&mut self, k: Key) {
        if k.code == Code::Esc {
            self.count = None;
            self.prefix = None;
            return self.leave_visual();
        }
        if k.ctrl || self.prefix.is_some() {
            return self.normal_key(k);
        }
        let Some(c) = k.char() else { return self.normal_key(k) };
        if c.is_ascii_digit() && (c != '0' || self.count.is_some()) {
            return self.normal_key(k);
        }
        let line = self.cur_line();
        match c {
            'v' | 'V' => {
                let want = if c == 'v' { Mode::Visual } else { Mode::VisualLine };
                if self.mode == want { self.leave_visual() } else { self.mode = want }
            }
            'o' | 'O' => std::mem::swap(&mut self.anchor, &mut self.cursor),
            'i' | 'a' => self.prefix = Some(c),
            'd' | 'x' => self.visual_op(Op::Delete, false),
            'X' | 'D' => self.visual_op(Op::Delete, true),
            'c' | 's' => self.visual_op(Op::Change, false),
            'C' | 'S' | 'R' => self.visual_op(Op::Change, true),
            'y' => self.visual_op(Op::Yank, false),
            'Y' => self.visual_op(Op::Yank, true),
            'J' => {
                let (l1, l2) = match self.selection() {
                    Span::Lines(a, b) => (a.min(b), a.max(b)),
                    Span::Chars(a, b) => (self.buf.to_pos(a).line, self.buf.to_pos(b.saturating_sub(1).max(a)).line),
                };
                self.mode = Mode::Normal;
                self.cursor = self.line_start(l1);
                self.join((l2 - l1).max(1));
            }
            '~' | 'u' | 'U' => self.visual_case(c),
            'p' | 'P' => self.visual_put(),
            'I' | 'A' => {
                let (a, b) = self.selection_range().unwrap_or((self.cursor, self.cursor));
                self.mode = Mode::Normal;
                self.start_insert(if c == 'I' { a } else { b });
            }
            '$' => {
                // past the last character: the line break is selected too
                let n = self.take_count();
                let target = (line + n - 1).min(self.buf.len_lines() - 1);
                self.cursor = self.line_end(target);
                self.want_col = usize::MAX;
            }
            _ => self.normal_key(k),
        }
    }

    pub(crate) fn visual_case(&mut self, how: char) {
        let (a, b) = self.selection_range().unwrap_or((self.cursor, self.cursor));
        let text = self.buf.slice(a, b);
        let changed: String = match how {
            'u' => text.to_lowercase(),
            'U' => text.to_uppercase(),
            _ => text.chars().map(|ch| if ch.is_uppercase() { ch.to_lowercase().next().unwrap_or(ch) } else { ch.to_uppercase().next().unwrap_or(ch) }).collect(),
        };
        self.mode = Mode::Normal;
        if changed != text {
            self.begin();
            self.rem(a, b);
            self.ins(a, &changed);
        }
        self.cursor = a;
        self.clamp();
        self.want_col = self.cursor().col;
    }

    pub(crate) fn visual_replace(&mut self, ch: char) {
        let (a, b) = self.selection_range().unwrap_or((self.cursor, self.cursor));
        let text = self.buf.slice(a, b);
        let changed: String = text.chars().map(|c| if c == '\n' { c } else { ch }).collect();
        self.mode = Mode::Normal;
        self.begin();
        self.rem(a, b);
        self.ins(a, &changed);
        self.cursor = a;
        self.clamp();
    }

    /// `p` on a selection: the register replaces it, and what was selected goes to the register.
    pub(crate) fn visual_put(&mut self) {
        let saved = self.reg.clone();
        if self.mode == Mode::VisualLine {
            let Span::Lines(l1, l2) = self.selection() else { return };
            let (l1, l2) = (l1.min(l2), l1.max(l2));
            let at_end = l2 + 1 >= self.buf.len_lines();
            self.visual_op(Op::Delete, true);
            let removed = std::mem::replace(&mut self.reg, Register { text: saved.text.clone(), linewise: true, block: false });
            if !self.reg.text.ends_with('\n') {
                self.reg.text.push('\n');
            }
            // the lines go where the selection was: after the line above when it was the end
            self.close();
            if at_end && l1 > 0 {
                self.cursor = self.line_start(self.buf.len_lines() - 1);
                self.put(true, 1);
            } else {
                self.put(false, 1);
            }
            self.reg = removed;
            return;
        }
        let Span::Chars(a, b) = self.selection() else { return };
        self.mode = Mode::Normal;
        self.begin();
        let removed = self.rem(a, b);
        if saved.linewise {
            // whole lines into the middle of a line: they go on lines of their own
            let text = format!("\n{}\n", saved.text.trim_end_matches('\n'));
            self.ins(a, &text);
            self.cursor = a + 1;
        } else {
            self.ins(a, &saved.text);
            self.cursor = a + saved.text.chars().count().saturating_sub(1);
        }
        self.reg = Register { text: removed, linewise: false, block: false };
        self.clamp();
        self.want_col = self.cursor().col;
    }

}
