//! Motions: f F t T, the motion every key goes through (or the operator it completes), up and down.

use super::*;

impl Editor {
    /// f F t T: to (or up to) the next or previous `ch` on the line, count times.
    pub(crate) fn find(&mut self, kind: char, ch: char) {
        let n = self.take_count();
        let line = self.cur_line();
        let (ls, le) = (self.line_start(line), self.line_end(line));
        let forward = kind == 'f' || kind == 't';
        let till = kind == 't' || kind == 'T';
        let mut i = self.cursor;
        let mut found = None;
        let mut left = n;
        loop {
            // t/T start one further, so repeating them moves on
            let next = if forward { i + 1 } else { i.wrapping_sub(1) };
            if (forward && next >= le) || (!forward && (i == 0 || next < ls)) {
                break;
            }
            i = next;
            if self.char(i) == Some(ch) {
                left -= 1;
                if left == 0 {
                    found = Some(i);
                    break;
                }
            }
        }
        let Some(p) = found else {
            self.op = None;
            self.op_count = None;
            return;
        };
        let to = match (forward, till) {
            (true, true) => p - 1,
            (false, true) => p + 1,
            _ => p,
        };
        if forward {
            // f and t include the character they land on
            self.motion(Span::Chars(self.cursor, to + 1), Some(to));
        } else {
            self.motion(Span::Chars(to, self.cursor), Some(to));
        }
    }

    /// A motion: moves the cursor, or, with an operator pending, applies it.
    pub(crate) fn motion(&mut self, span: Span, to: Option<usize>) {
        if let Some(op) = self.op.take() {
            self.op_count = None;
            return self.apply(op, span);
        }
        if let Some(t) = to {
            self.cursor = t;
            self.clamp();
            self.want_col = self.cursor().col;
        }
    }

    pub(crate) fn vertical_to(&mut self, line: usize) {
        let len = self.buf.line_len(line);
        let max = if len > 0 { len - 1 } else { 0 };
        self.cursor = self.line_start(line) + self.want_col.min(max);
    }

}
