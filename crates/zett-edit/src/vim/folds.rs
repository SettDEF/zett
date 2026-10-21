//! Folds as Vim has them: manual ones (zf zF zd zD zE) that move with the text, opened and closed
//! (zo zO zc zC za zA zv zx zX zm zM zr zR zn zN zi), moved between (zj zk [z ]z). Where no manual
//! fold is, zc zo za fold zett's outline section (headings, blocks) instead.

use super::*;

impl Editor {
    /// The manual folds as line ranges (first, last, closed), innermost last among those around a line.
    pub(crate) fn fold_lines(&self) -> Vec<(usize, usize, bool)> {
        let n = self.buf.len_chars();
        self.vim.folds.iter().map(|&(a, b, c)| (self.buf.to_pos(a.min(n)).line, self.buf.to_pos(b.min(n)).line, c)).collect()
    }

    /// The folds around `line`, outermost first.
    fn folds_at(&self, line: usize) -> Vec<usize> {
        let ls = self.fold_lines();
        let mut v: Vec<usize> = (0..ls.len()).filter(|&i| ls[i].0 <= line && line <= ls[i].1).collect();
        v.sort_by_key(|&i| (ls[i].0, std::cmp::Reverse(ls[i].1)));
        v
    }

    /// The hidden line ranges the closed manual folds make (for the view): [first+1, last+1).
    pub(crate) fn manual_hidden(&self) -> Vec<(usize, usize)> {
        if self.vim.folds_off {
            return Vec::new();
        }
        self.fold_lines().into_iter().filter(|f| f.2 && f.1 > f.0).map(|(a, b, _)| (a + 1, b + 1)).collect()
    }

    fn folds_changed(&mut self) {
        // the view works the hidden lines out again
        self.view_touch();
        // the cursor goes to the start of a fold that closed over it
        let l = self.cur_line();
        if let Some(&(a, _)) = self.manual_hidden().iter().find(|&&(a, b)| l >= a && l < b) {
            self.cursor = self.line_start(a - 1);
            self.clamp();
        }
    }

    /// A fold over lines l1..=l2, closed.
    pub(crate) fn fold_create(&mut self, l1: usize, l2: usize) {
        let (l1, l2) = (l1.min(l2), l1.max(l2));
        let (a, b) = (self.line_start(l1), self.line_end(l2));
        self.vim.folds.push((a, b, true));
        self.vim.folds_off = false;
        self.folds_changed();
    }

    /// z and a fold key; false when `c` is not one.
    pub(crate) fn vim_fold(&mut self, c: char) -> bool {
        let line = self.cur_line();
        let around = self.folds_at(line);
        let none = || Request::Message("E490: No fold found".into());
        match c {
            'y' => {
                self.op = Some(Op::YankTrim);
                self.op_count = self.count.take();
                return true;
            }
            'p' | 'P' => {
                // put without the blanks at line ends
                let n = self.take_count();
                let saved = self.reg.clone();
                self.reg.text = saved.text.split('\n').map(str::trim_end).collect::<Vec<_>>().join("\n");
                self.put(c == 'p', n);
                self.reg = saved;
            }
            '=' => self.spell_suggest(),
            'g' | 'G' | 'w' | 'W' => self.spell_mark(c, false),
            'u' => {
                self.prefix = Some(super::spelling::ZU);
                return true;
            }
            'f' => {
                // zf{motion}: an operator
                self.op = Some(Op::Fold);
                self.op_count = self.count.take();
                return true;
            }
            'F' => {
                let n = self.take_count();
                self.fold_create(line, (line + n - 1).min(self.buf.len_lines() - 1));
            }
            'd' | 'D' => {
                if around.is_empty() {
                    self.vim.requests.push(none());
                } else if c == 'd' {
                    self.vim.folds.remove(*around.last().unwrap());
                } else {
                    let mut gone = around.clone();
                    gone.sort();
                    for i in gone.into_iter().rev() {
                        self.vim.folds.remove(i);
                    }
                }
                self.folds_changed();
            }
            'E' => {
                self.vim.folds.clear();
                self.folds_changed();
            }
            'o' | 'O' | 'c' | 'C' | 'a' | 'A' | 'v' => {
                if around.is_empty() {
                    // no manual fold here: the outline section (a heading, a block)
                    if matches!(c, 'o' | 'O' | 'v') && !self.is_fold_start(line) {
                        if c != 'v' {
                            self.vim.requests.push(none());
                        }
                        return true;
                    }
                    if !self.outline_exec("outline-cycle", 1) {
                        self.vim.requests.push(none());
                    }
                    return true;
                }
                let inner = *around.last().unwrap();
                let all = matches!(c, 'O' | 'C' | 'A' | 'v');
                let close = match c {
                    'o' | 'O' | 'v' => false,
                    'c' | 'C' => true,
                    _ => !self.vim.folds[inner].2,
                };
                if all {
                    for i in around {
                        self.vim.folds[i].2 = close;
                    }
                } else {
                    self.vim.folds[inner].2 = close;
                }
                self.folds_changed();
            }
            'x' | 'X' => {
                // back to every fold closed (zx: but the cursor's line shown)
                for f in self.vim.folds.iter_mut() {
                    f.2 = true;
                }
                if c == 'x' {
                    for i in self.folds_at(line) {
                        self.vim.folds[i].2 = false;
                    }
                }
                self.folds_changed();
            }
            'm' | 'M' | 'r' | 'R' => {
                // more folded (m: the innermost open ones close; M: all) or less (r: the outermost
                // closed ones open; R: all)
                let ls = self.fold_lines();
                let depth = |i: usize| ls.iter().filter(|o| o.0 <= ls[i].0 && ls[i].1 <= o.1).count();
                match c {
                    'M' | 'R' => self.vim.folds.iter_mut().for_each(|f| f.2 = c == 'M'),
                    'm' => {
                        if let Some(d) = (0..ls.len()).filter(|&i| !ls[i].2).map(depth).max() {
                            for i in 0..ls.len() {
                                if !ls[i].2 && depth(i) == d {
                                    self.vim.folds[i].2 = true;
                                }
                            }
                        }
                    }
                    _ => {
                        if let Some(d) = (0..ls.len()).filter(|&i| ls[i].2).map(depth).min() {
                            for i in 0..ls.len() {
                                if ls[i].2 && depth(i) == d {
                                    self.vim.folds[i].2 = false;
                                }
                            }
                        }
                    }
                }
                if c == 'R' {
                    self.outline_exec("outline-show-all", 1);
                }
                self.folds_changed();
            }
            'n' | 'N' | 'i' => {
                self.vim.folds_off = match c { 'n' => true, 'N' => false, _ => !self.vim.folds_off };
                self.folds_changed();
            }
            'j' | 'k' => {
                // to the start of the next fold, or the end of the previous one
                let ls = self.fold_lines();
                let to = if c == 'j' { ls.iter().map(|f| f.0).filter(|&s| s > line).min() } else { ls.iter().map(|f| f.1).filter(|&e| e < line).max() };
                if let Some(l) = to {
                    self.motion(Span::Lines(line, l), Some(self.first_non_blank(l)));
                }
            }
            _ => return false,
        }
        true
    }

    /// [z ]z: the start or end of the open fold the cursor is in.
    pub(crate) fn fold_edge(&mut self, fwd: bool) -> bool {
        let line = self.cur_line();
        let ls = self.fold_lines();
        let Some(&i) = self.folds_at(line).last() else {
            self.vim.requests.push(Request::Message("E490: No fold found".into()));
            return true;
        };
        let to = if fwd { ls[i].1 } else { ls[i].0 };
        self.motion(Span::Lines(line, to), Some(self.first_non_blank(to)));
        true
    }
}
