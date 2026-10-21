//! Changes: an operator on what a motion covers, put, join, replace.

use super::*;

impl Editor {
    pub(crate) fn apply(&mut self, op: Op, span: Span) {
        if self.vim_apply(op, span) {
            return;
        }
        self.op = None;
        self.count = None;
        match span {
            Span::Chars(a, b) => {
                let (a, b) = (a.min(b), a.max(b));
                self.reg = Register { text: self.buf.slice(a, b), linewise: false, block: false };
                if op == Op::Yank {
                    self.cursor = a;
                } else {
                    self.begin();
                    self.rem(a, b);
                    self.cursor = a;
                    if op == Op::Change {
                        self.mode = Mode::Insert;
                        return;
                    }
                }
            }
            Span::Lines(l1, l2) => {
                let (l1, l2) = (l1.min(l2), l1.max(l2));
                let s = self.line_start(l1);
                let e = if l2 + 1 < self.buf.len_lines() { self.line_start(l2 + 1) } else { self.buf.len_chars() };
                let mut text = self.buf.slice(s, e);
                if !text.ends_with('\n') {
                    text.push('\n');
                }
                self.reg = Register { text, linewise: true, block: false };
                match op {
                    Op::Yank => self.cursor = s.min(self.cursor),
                    Op::Delete => {
                        self.begin();
                        // the last lines take the line break BEFORE them
                        let (s2, e2) = if e == self.buf.len_chars() && s > 0 { (s - 1, e) } else { (s, e) };
                        self.rem(s2, e2);
                        let line = l1.min(self.buf.len_lines() - 1);
                        self.cursor = self.first_non_blank(line);
                    }
                    Op::Change => {
                        self.begin();
                        let indent = self.first_non_blank(l1) - s;
                        let keep = self.buf.slice(s, s + indent);
                        let end = self.line_end(l2);
                        self.rem(s, end);
                        self.ins(s, &keep);
                        self.cursor = s + indent;
                        self.mode = Mode::Insert;
                        return;
                    }
                    // the rest are vim.rs's (vim_apply took them before)
                    _ => {}
                }
            }
        }
        self.clamp();
        self.want_col = self.cursor().col;
    }

    pub(crate) fn put(&mut self, after: bool, n: usize) {
        if self.reg.block {
            return self.put_block(after);
        }
        if self.reg.text.is_empty() {
            return;
        }
        self.begin();
        let text = self.reg.text.repeat(n);
        let line = self.cur_line();
        if self.reg.linewise {
            let last = line + 1 >= self.buf.len_lines();
            let no_final_break = self.buf.len_chars() > 0 && self.buf.char_at(self.buf.len_chars() - 1) != Some('\n');
            let at = if after && last && no_final_break {
                // after the last line of a file without a final line break: the
                // break goes BEFORE the new lines, so the file still has none
                let e = self.buf.len_chars();
                self.ins(e, &format!("\n{}", text.strip_suffix('\n').unwrap_or(&text)));
                e + 1
            } else {
                let at = if after {
                    if last { self.buf.len_chars() } else { self.line_start(line + 1) }
                } else {
                    self.line_start(line)
                };
                self.ins(at, &text);
                at
            };
            self.cursor = at;
            let l = self.cur_line();
            self.cursor = self.first_non_blank(l);
        } else {
            let at = if after && self.buf.line_len(line) > 0 { self.cursor + 1 } else { self.cursor };
            self.ins(at, &text);
            self.cursor = at + text.chars().count() - 1;
        }
        self.clamp();
    }

    pub(crate) fn join(&mut self, times: usize) {
        self.begin();
        for _ in 0..times {
            let line = self.cur_line();
            if line + 1 >= self.buf.len_lines() {
                break;
            }
            let e = self.line_end(line);
            let next = self.first_non_blank(line + 1);
            let joined_at = e;
            self.rem(e, next);
            let needs_space = joined_at > self.line_start(line) && self.char(joined_at).is_some_and(|c| c != '\n' && c != ')');
            if needs_space {
                self.ins(joined_at, " ");
            }
            self.cursor = joined_at;
        }
        self.clamp();
    }

    pub(crate) fn replace_chars(&mut self, ch: char) {
        let n = self.take_count();
        let line = self.cur_line();
        if self.cursor + n > self.line_end(line) {
            return;
        }
        self.begin();
        self.rem(self.cursor, self.cursor + n);
        self.ins(self.cursor, &ch.to_string().repeat(n));
        self.cursor += n - 1;
    }

}
