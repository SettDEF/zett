//! `[` and `]` and a key: unmatched brackets, sections, methods, comments, #if blocks, marks, puts
//! with the indent, the keyword through the file, diagnostics and changes (asked of the window).

use super::*;

/// What CTRL-I and CTRL-D after `[` `]` arrive as.
pub(crate) const BR_TAB: char = '\u{9}';
pub(crate) const BR_DEF: char = '\u{4}';

impl Editor {
    /// `[x` / `]x`; false when x is not one of these.
    pub(crate) fn vim_bracket(&mut self, p: char, c: char) -> bool {
        let fwd = p == ']';
        let n = self.take_count();
        match c {
            '(' | ')' | '{' | '}' => {
                let (open, close) = if matches!(c, '(' | ')') { ('(', ')') } else { ('{', '}') };
                let mut at = self.cursor;
                for _ in 0..n {
                    match self.unmatched(at, open, close, fwd) {
                        Some(i) => at = i,
                        None => break,
                    }
                }
                self.jump_motion(at, fwd);
            }
            '[' | ']' => {
                // sections: a { (or for [] ][ a }) in the first column, else the file's ends
                let want = if (c == '[') == (p == '[') { '{' } else { '}' };
                let last = self.buf.len_lines() - 1;
                let mut l = self.cur_line();
                for _ in 0..n {
                    loop {
                        if fwd { if l >= last { break } l += 1 } else { if l == 0 { break } l -= 1 }
                        if self.char(self.line_start(l)) == Some(want) {
                            break;
                        }
                    }
                }
                let at = if fwd && l == last && self.char(self.line_start(l)) != Some(want) { self.line_end(l) } else { self.line_start(l) };
                self.jump_motion(at, fwd);
            }
            'm' | 'M' => {
                // a method's start ({) or end (}), the nearest one that way
                let want = if c == 'm' { '{' } else { '}' };
                let mut at = self.cursor;
                for _ in 0..n {
                    let found = if fwd { (at + 1..self.buf.len_chars()).find(|&i| self.char(i) == Some(want)) } else { (0..at).rev().find(|&i| self.char(i) == Some(want)) };
                    match found {
                        Some(i) => at = i,
                        None => break,
                    }
                }
                self.jump_motion(at, fwd);
            }
            '*' | '/' => {
                let pat = if fwd { "*/" } else { "/*" };
                let found = self.buf.find(pat, if fwd { self.cursor + 1 } else { self.cursor }, fwd, false);
                if let Some(i) = found {
                    self.jump_motion(if fwd { i + 1 } else { i }, fwd);
                }
            }
            '#' => {
                // the #if / #else / #endif this line is inside
                let last = self.buf.len_lines() - 1;
                let (mut l, mut depth) = (self.cur_line(), 0);
                let kind = |e: &Self, l: usize| {
                    let t = e.buf.line(l);
                    let t = t.trim_start();
                    if t.starts_with("#if") { 1 } else if t.starts_with("#endif") { -1 } else if t.starts_with("#el") { 0 } else { 2 }
                };
                loop {
                    if fwd { if l >= last { break } l += 1 } else { if l == 0 { break } l -= 1 }
                    match (kind(self, l), fwd) {
                        (1, true) | (-1, false) => depth += 1,
                        (-1, true) | (1, false) if depth > 0 => depth -= 1,
                        (-1, true) | (1, false) | (0, _) if depth == 0 => {
                            let at = self.first_non_blank(l);
                            return {
                                self.jump_motion(at, fwd);
                                true
                            };
                        }
                        _ => {}
                    }
                }
            }
            '\'' | '`' => {
                // the next or previous lower-case mark
                let here = self.cursor;
                let mut marks: Vec<usize> = self.vim.marks.iter().filter(|(k, _)| k.is_ascii_lowercase()).map(|(_, v)| *v).collect();
                marks.sort();
                let line_of = |e: &Self, i: usize| e.buf.to_pos(i).line;
                let target = if fwd { marks.into_iter().find(|&m| if c == '\'' { line_of(self, m) > line_of(self, here) } else { m > here }) } else { marks.into_iter().rev().find(|&m| if c == '\'' { line_of(self, m) < line_of(self, here) } else { m < here }) };
                if let Some(m) = target {
                    let at = if c == '\'' { self.first_non_blank(line_of(self, m)) } else { m };
                    self.jump_motion(at, fwd);
                }
            }
            'p' | 'P' => {
                // put, its lines taking this line's indent
                let ind = self.indent_of(self.cur_line());
                if self.reg.linewise {
                    let base = self.reg.text.lines().filter(|l| !l.trim().is_empty()).map(|l| l.len() - l.trim_start().len()).min().unwrap_or(0);
                    let saved = self.reg.clone();
                    self.reg.text = self.reg.text.lines().map(|l| if l.trim().is_empty() { String::new() } else { format!("{ind}{}", &l[base.min(l.len() - l.trim_start().len())..]) }).collect::<Vec<_>>().join("\n") + "\n";
                    self.put(c == 'p' && fwd, n);
                    self.reg = saved;
                } else {
                    self.put(fwd && c == 'p', n);
                }
            }
            'i' | 'I' | BR_TAB | BR_DEF => {
                // the word under the cursor through the file: the first line with it ([i), all of them
                // ([I), go to the first ([ CTRL-I, ] CTRL-I after the cursor), to its definition (CTRL-D)
                let Some((a, b)) = self.word_here() else { return true };
                let w = self.buf.slice(a, b);
                let lines: Vec<usize> = (0..self.buf.len_lines()).filter(|&l| {
                    let t = self.buf.line(l);
                    t.match_indices(&w).any(|(i, _)| {
                        let before = t[..i].chars().last().is_none_or(|ch| !super::is_word(ch));
                        let after = t[i + w.len()..].chars().next().is_none_or(|ch| !super::is_word(ch));
                        before && after
                    })
                }).filter(|&l| !fwd || l > self.cur_line()).collect();
                match c {
                    'i' => {
                        if let Some(&l) = lines.first() {
                            self.vim.requests.push(Request::Message(format!("{}: {}", l + 1, self.buf.line(l).trim_end())));
                        }
                    }
                    'I' => {
                        let m = lines.iter().take(30).map(|&l| format!("{:>4}: {}", l + 1, self.buf.line(l).trim_end())).collect::<Vec<_>>().join("\n");
                        self.vim.requests.push(Request::Message(m));
                    }
                    BR_TAB => {
                        if let Some(&l) = lines.first() {
                            let at = self.buf.to_char(crate::Pos { line: l, col: 0 });
                            let col = self.buf.line(l).find(&w).map(|i| self.buf.line(l)[..i].chars().count()).unwrap_or(0);
                            self.jump_motion(at + col, fwd);
                        }
                    }
                    _ => self.vim.requests.push(Request::Definition),
                }
            }
            'd' | 'D' => self.vim.requests.push(Request::Diagnostic(match (c, fwd) { ('d', true) => 1, ('d', false) => -1, ('D', true) => 2, _ => -2 })),
            'c' => self.vim.requests.push(Request::Hunk(if fwd { 1 } else { -1 })),
            's' | 'S' => self.spell_jump(fwd),
            'f' => {
                self.vim_g('f');
            }
            'z' => return self.fold_edge(fwd),
            _ => return false,
        };
        true
    }

    /// The `open`/`close` that is not matched between the cursor and it, that way.
    fn unmatched(&self, from: usize, open: char, close: char, fwd: bool) -> Option<usize> {
        let mut depth = 0;
        if fwd {
            for i in from + 1..self.buf.len_chars() {
                match self.char(i) {
                    Some(c) if c == open => depth += 1,
                    Some(c) if c == close && depth == 0 => return Some(i),
                    Some(c) if c == close => depth -= 1,
                    _ => {}
                }
            }
        } else {
            for i in (0..from).rev() {
                match self.char(i) {
                    Some(c) if c == close => depth += 1,
                    Some(c) if c == open && depth == 0 => return Some(i),
                    Some(c) if c == open => depth -= 1,
                    _ => {}
                }
            }
        }
        None
    }

    /// A jump to `at` as a motion (an operator before it acts on the way), on the jump list.
    pub(crate) fn jump_motion(&mut self, at: usize, fwd: bool) {
        let from = self.cursor;
        let span = if fwd { Span::Chars(from, at + 1) } else { Span::Chars(at, from) };
        self.motion(span, Some(at));
    }
}
