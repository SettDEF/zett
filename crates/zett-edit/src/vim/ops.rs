//! The operators beyond d c y: g~ gu gU g? > < = gq ! g@.

use super::*;

impl Editor {
    /// The new operators on what a motion covers.
    pub(crate) fn vim_apply(&mut self, op: Op, span: Span) -> bool {
        if !matches!(op, Op::Toggle | Op::Lower | Op::Upper | Op::Rot13 | Op::ShiftRight | Op::ShiftLeft | Op::Reindent | Op::Format | Op::Filter | Op::Func | Op::Fold | Op::YankTrim) {
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
                // equalprg: = filters the lines through that program instead
                if op == Op::Reindent {
                    // indentexpr: the window indents each line by it
                    if self.opt("indentexpr").is_some_and(|e| !e.trim().is_empty()) {
                        self.vim.requests.push(Request::IndentExpr(l1, l2));
                        return true;
                    }
                    if let Some(prg) = self.opt("equalprg").filter(|p| !p.trim().is_empty()) {
                        let l = format!("{},{}!{prg}", l1 + 1, l2 + 1);
                        self.vim.requests.push(Request::Ex(l));
                        return true;
                    }
                }
                // shiftwidth steps; shiftround to a multiple of it; preserveindent keeps the
                // indent's own characters and only adds or takes at its end
                let sw = self.shiftwidth().max(1);
                let (round, keep) = (self.opt_on("shiftround", false), self.opt_on("preserveindent", false));
                self.begin();
                for l in l1..=l2 {
                    let ind = self.indent_of(l);
                    let width = self.indent_width(&ind);
                    let blank = self.buf.line_len(l) == ind.chars().count();
                    let new = match op {
                        // > leaves empty lines as they are
                        Op::ShiftRight if blank => continue,
                        Op::ShiftRight if round => (width / sw + 1) * sw,
                        Op::ShiftRight => width + sw,
                        Op::ShiftLeft if round && width % sw != 0 => width / sw * sw,
                        Op::ShiftLeft => width.saturating_sub(sw),
                        // = : the indent of the nearest non-blank line above (Vim's default with no indentexpr)
                        _ => (0..l).rev().find(|&k| self.buf.line_len(k) > self.indent_of(k).chars().count()).map(|k| self.indent_width(&self.indent_of(k))).unwrap_or(0),
                    };
                    let new_ind = if keep && new >= width {
                        format!("{ind}{}", " ".repeat(new - width))
                    } else if keep {
                        let mut kept: String = String::new();
                        for c in ind.chars() {
                            if self.indent_width(&format!("{kept}{c}")) > new {
                                break;
                            }
                            kept.push(c);
                        }
                        let w = self.indent_width(&kept);
                        format!("{kept}{}", " ".repeat(new - w))
                    } else {
                        self.make_indent(new)
                    };
                    if new_ind != ind {
                        let s = self.line_start(l);
                        self.rem(s, s + ind.chars().count());
                        self.ins(s, &new_ind);
                    }
                }
                self.close();
                self.cursor = self.first_non_blank(l1);
            }
            Op::Format => {
                // formatexpr: the window evaluates it (non-zero: the internal formatting after all)
                if self.opt("formatexpr").is_some_and(|e| !e.trim().is_empty()) {
                    self.vim.requests.push(Request::FormatExpr(l1, l2));
                    return true;
                }
                // formatprg: gq filters the lines through that program instead
                if let Some(prg) = self.opt("formatprg").filter(|p| !p.trim().is_empty()) {
                    let l = format!("{},{}!{prg}", l1 + 1, l2 + 1);
                    self.vim.requests.push(Request::Ex(l));
                    return true;
                }
                self.format_lines(l1, l2);
            }
            Op::Filter => self.vim.requests.push(Request::CmdLine(if l1 == l2 { ".!".into() } else { format!(".,.+{}!", l2 - l1) })),
            Op::Func => {
                // g the marks [ and ] around the text, then the window calls the operatorfunc
                self.vim.marks.insert('[', a);
                self.vim.marks.insert(']', b.saturating_sub(1).max(a));
                self.vim.requests.push(Request::OperatorFunc(a, b, lines.is_some()));
            }
            Op::Fold => self.fold_create(l1, l2),
            Op::YankTrim => {
                self.apply(Op::Yank, span);
                self.reg.text = self.reg.text.split('\n').map(str::trim_end).collect::<Vec<_>>().join("\n");
                return true;
            }
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

impl Editor {
    /// gq's own formatting of lines `l1..=l2`: each paragraph filled to the text width, its first
    /// line's prefix (a comment leader, a list mark) kept (also when formatexpr gives non-zero).
    pub fn format_lines(&mut self, l1: usize, l2: usize) {
        let l2 = l2.min(self.buf.len_lines().saturating_sub(1));
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
}
