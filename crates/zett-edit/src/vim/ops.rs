//! The operators beyond d c y: g~ gu gU g? > < = gq ! g@.

use super::*;

impl Editor {
    /// The new operators on what a motion covers.
    pub(crate) fn vim_apply(&mut self, op: Op, span: Span) -> bool {
        if !matches!(op, Op::Toggle | Op::Lower | Op::Upper | Op::Rot13 | Op::ShiftRight | Op::ShiftLeft | Op::Reindent | Op::Format | Op::Filter | Op::Func) {
            return false;
        }
        self.op = None;
        self.count = None;
        let (a, b, lines) = match span {
            Span::Chars(a, b) => (a.min(b), a.max(b), None),
            Span::Lines(l1, l2) => {
                let (l1, l2) = (l1.min(l2), l1.max(l2));
                (self.line_start(l1), self.line_end(l2), Some((l1, l2)))
            }
        };
        let (l1, l2) = lines.unwrap_or_else(|| (self.buf.to_pos(a).line, self.buf.to_pos(b.saturating_sub(1).max(a)).line));
        match op {
            Op::Toggle | Op::Lower | Op::Upper | Op::Rot13 => {
                let t: String = self.buf.slice(a, b).chars().map(|ch| match op { Op::Toggle => toggle(ch), Op::Lower => ch.to_lowercase().next().unwrap_or(ch), Op::Upper => ch.to_uppercase().next().unwrap_or(ch), _ => rot13(ch) }).collect();
                self.replace_range(a, b, &t);
                self.cursor = a;
            }
            Op::ShiftRight | Op::ShiftLeft | Op::Reindent => {
                let sw = self.soft_tab.unwrap_or(4);
                self.begin();
                for l in l1..=l2 {
                    let s = self.line_start(l);
                    let ind = self.indent_of(l);
                    let width = ind.chars().map(|c| if c == '\t' { sw } else { 1 }).sum::<usize>();
                    let blank = self.buf.line_len(l) == ind.chars().count();
                    let new = match op {
                        Op::ShiftRight if !blank => width + sw,
                        Op::ShiftRight => width,
                        Op::ShiftLeft => width.saturating_sub(sw) / sw * sw + if width % sw != 0 && width > sw { 0 } else { 0 },
                        // = : the indent of the nearest non-blank line above (Vim's default with no indentexpr)
                        _ => (0..l).rev().find(|&k| self.buf.line_len(k) > self.indent_of(k).chars().count()).map(|k| self.indent_of(k).chars().map(|c| if c == '\t' { sw } else { 1 }).sum()).unwrap_or(0),
                    };
                    let n = ind.chars().count();
                    self.rem(s, s + n);
                    self.ins(s, &" ".repeat(new));
                }
                self.close();
                self.cursor = self.first_non_blank(l1);
            }
            Op::Format => {
                let width = self.fill_column.max(10);
                let text = self.buf.slice(self.line_start(l1), self.line_end(l2));
                let mut out: Vec<String> = Vec::new();
                for para in text.split("\n\n") {
                    let first = para.lines().next().unwrap_or("");
                    let (p1, rest) = crate::textops::fill_prefix(first);
                    let words: Vec<&str> = para.split_whitespace().collect();
                    out.push(crate::textops::fill(&words, &p1, &rest, width).join("\n"));
                }
                let (s, e) = (self.line_start(l1), self.line_end(l2));
                self.replace_range(s, e, &out.join("\n\n"));
                self.cursor = self.first_non_blank(l1.min(self.buf.len_lines() - 1));
            }
            Op::Filter => self.vim.requests.push(Request::CmdLine(if l1 == l2 { ".!".into() } else { format!(".,.+{}!", l2 - l1) })),
            Op::Func => self.vim.requests.push(Request::OperatorFunc(a, b)),
            _ => {}
        }
        self.clamp();
        self.want_col = self.cursor().col;
        true
    }

    /// Replaces [a, b) with `t` as one change.
    pub(crate) fn replace_range(&mut self, a: usize, b: usize, t: &str) {
        if self.buf.slice(a, b) == t {
            return;
        }
        self.begin();
        self.rem(a, b);
        self.ins(a, t);
        self.close();
    }

}
