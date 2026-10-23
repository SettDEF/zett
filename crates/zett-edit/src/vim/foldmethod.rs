//! foldmethod indent and marker: folds worked out from the text and kept as Vim's other folds, so
//! every z key works on them, each one's open or closed state kept by its first line across edits;
//! foldlevel, foldlevelstart, foldenable, foldminlines, foldnestmax, foldignore, foldmarker; and
//! foldopen / foldclose after a command.

use super::*;

impl Editor {
    /// The folds again when the text or a fold option changed (before the view is worked out).
    pub(crate) fn fold_update(&mut self) {
        // foldenable: folding on or off when the option changes (zn zN zi switch it too)
        let enable = self.opt_on("foldenable", true);
        if self.vim.fold_enable != Some(enable) {
            self.vim.fold_enable = Some(enable);
            self.vim.folds_off = !enable;
            self.view_stale();
        }
        let method = self.opt("foldmethod").unwrap_or("manual").to_string();
        if !matches!(method.as_str(), "indent" | "marker" | "expr") {
            if self.vim.fold_auto.take().is_some() {
                self.vim.folds.clear();
                self.view_stale();
            }
            return;
        }
        let key = format!(
            "{method}|{}|{}|{}|{}|{}|{}",
            self.vim.fold_expr_version,
            self.buf.rev(),
            self.opt("foldmarker").unwrap_or("{{{,}}}"),
            self.opt("foldignore").unwrap_or("#"),
            self.opt_num("foldnestmax", 20),
            self.opt_num("shiftwidth", 0)
        );
        let level = self.opt_num("foldlevel", 0);
        let first = self.vim.fold_auto.is_none();
        if self.vim.fold_auto.as_deref() == Some(key.as_str()) && self.vim.fold_level == Some(level) {
            return;
        }
        // foldlevelstart: the level a buffer starts with (-1: foldlevel's)
        let start = self.opt_num("foldlevelstart", -1);
        let level = if first && start >= 0 { start } else { level };
        let relevel = self.vim.fold_level.is_some_and(|l| l != level);
        self.vim.fold_level = Some(self.opt_num("foldlevel", 0));
        let ranges = match method.as_str() {
            "indent" => self.indent_folds(),
            "expr" => expr_folds(&self.vim.fold_expr, self.opt_num("foldnestmax", 20).max(0) as usize),
            _ => self.marker_folds(),
        };
        // each fold keeps its state by its first line; new ones are closed deeper than foldlevel
        let was: std::collections::HashMap<usize, bool> = self.fold_lines().into_iter().map(|(a, _, c)| (a, c)).collect();
        self.vim.folds = ranges
            .into_iter()
            .map(|(l1, l2, depth)| {
                let closed = if first || relevel { depth as i64 > level } else { was.get(&l1).copied().unwrap_or(depth as i64 > level) };
                (self.line_start(l1), self.line_end(l2), closed)
            })
            .collect();
        self.vim.fold_auto = Some(key);
        self.view_stale();
    }

    /// foldmethod=indent: runs of lines indented at least N shiftwidths deep, for each N; blank lines
    /// and those starting with foldignore take the lower of their neighbours' levels.
    fn indent_folds(&self) -> Vec<(usize, usize, usize)> {
        let n = self.buf.len_lines();
        let sw = match self.opt_num("shiftwidth", 0) {
            s if s > 0 => s as usize,
            _ => self.soft_tab.unwrap_or(self.opt_num("tabstop", 8).max(1) as usize),
        };
        let ignore = self.opt("foldignore").unwrap_or("#").to_string();
        let nest = self.opt_num("foldnestmax", 20).max(0) as usize;
        let mut lv: Vec<Option<usize>> = (0..n)
            .map(|l| {
                let t = self.buf.line(l);
                let body = t.trim_start();
                if body.trim().is_empty() || body.starts_with(|c| ignore.contains(c)) {
                    None
                } else {
                    Some((self.indent_width(&t[..t.len() - body.len()]) / sw).min(nest))
                }
            })
            .collect();
        // the blank ones: the lower of the levels above and below
        for l in 0..n {
            if lv[l].is_none() {
                let up = (0..l).rev().find_map(|i| lv[i]);
                let down = (l + 1..n).find_map(|i| lv[i]);
                lv[l] = Some(up.unwrap_or(0).min(down.unwrap_or(0)));
            }
        }
        let lv: Vec<usize> = lv.into_iter().map(|x| x.unwrap_or(0)).collect();
        let mut out = Vec::new();
        for d in 1..=lv.iter().copied().max().unwrap_or(0) {
            let mut l = 0;
            while l < n {
                if lv[l] >= d {
                    let s = l;
                    while l + 1 < n && lv[l + 1] >= d {
                        l += 1;
                    }
                    out.push((s, l, d));
                }
                l += 1;
            }
        }
        out
    }

    /// foldmethod=marker: from a start marker to its end marker ("{{{" "}}}", a number after one
    /// sets the level: {{{2 starts a fold at level 2, ending deeper ones).
    fn marker_folds(&self) -> Vec<(usize, usize, usize)> {
        let fm = self.opt("foldmarker").unwrap_or("{{{,}}}").to_string();
        let (open, close) = fm.split_once(',').unwrap_or(("{{{", "}}}"));
        let n = self.buf.len_lines();
        let mut stack: Vec<(usize, usize)> = Vec::new();
        let mut out = Vec::new();
        let num_after = |t: &str, at: usize, m: &str| t[at + m.len()..].chars().take_while(char::is_ascii_digit).collect::<String>().parse::<usize>().ok();
        for l in 0..n {
            let t = self.buf.line(l);
            if let Some(at) = t.find(open).filter(|_| !open.is_empty()) {
                match num_after(&t, at, open) {
                    Some(level) => {
                        while stack.last().is_some_and(|f| f.1 >= level) {
                            let (s, d) = stack.pop().unwrap();
                            out.push((s, l.saturating_sub(1).max(s), d));
                        }
                        stack.push((l, level));
                    }
                    None => {
                        let d = stack.last().map_or(1, |f| f.1 + 1);
                        stack.push((l, d));
                    }
                }
            }
            if let Some(at) = t.find(close).filter(|_| !close.is_empty()) {
                match num_after(&t, at, close) {
                    Some(level) => {
                        while stack.last().is_some_and(|f| f.1 >= level) {
                            let (s, d) = stack.pop().unwrap();
                            out.push((s, l, d));
                        }
                    }
                    None => {
                        if let Some((s, d)) = stack.pop() {
                            out.push((s, l, d));
                        }
                    }
                }
            }
        }
        while let Some((s, d)) = stack.pop() {
            out.push((s, n.saturating_sub(1), d));
        }
        out.sort();
        out
    }

    /// After a command: foldopen opens the folds around the cursor for the kinds of command it
    /// names; foldclose=all closes the ones the cursor left (deeper than foldlevel).
    pub(crate) fn fold_after(&mut self, keys: &[Key]) {
        if self.vim.folds.is_empty() {
            return;
        }
        let line = self.cur_line();
        let first = keys.iter().find(|k| k.char().is_some_and(|c| !c.is_ascii_digit() && c != '"')).copied();
        let kind = match first.map(|k| (k.ctrl, k.char().unwrap_or(' '))) {
            Some((true, 'o' | 'i')) => "jump",
            Some((true, 'r')) | Some((false, 'u')) => "undo",
            Some((false, '/' | '?' | 'n' | 'N' | '*' | '#')) => "search",
            Some((false, 'h' | 'l' | '0' | '^' | '$' | '|' | 'w' | 'b' | 'e' | 'W' | 'B' | 'E' | 'f' | 'F' | 't' | 'T' | ';' | ',' | ' ')) => "hor",
            Some((false, '\'' | '`')) => "mark",
            Some((false, '%')) => "percent",
            Some((false, 'G')) => "jump",
            Some((false, '[' | ']' | '(' | ')' | '{' | '}')) => "block",
            Some((false, 'i' | 'a' | 'I' | 'A' | 'o' | 'O' | 's' | 'S' | 'c' | 'C' | 'R')) => "insert",
            _ => "",
        };
        let default = "block,hor,mark,percent,quickfix,search,tag,undo";
        let mut changed = false;
        if !kind.is_empty() && (self.opt_has("foldopen", kind, default) || self.opt_has("foldopen", "all", default)) {
            for (i, (a, b, c)) in self.fold_lines().into_iter().enumerate() {
                if c && a < line && line <= b {
                    self.vim.folds[i].2 = false;
                    changed = true;
                }
            }
        }
        if self.opt("foldclose") == Some("all") {
            let level = self.opt_num("foldlevel", 0);
            let ls = self.fold_lines();
            for (i, &(a, b, c)) in ls.iter().enumerate() {
                let depth = ls.iter().filter(|o| o.0 <= a && b <= o.1).count() as i64;
                if !c && !(a <= line && line <= b) && depth > level {
                    self.vim.folds[i].2 = true;
                    changed = true;
                }
            }
        }
        if changed {
            self.view_touch();
        }
    }
}

impl Editor {
    /// foldmethod=expr: the levels the front end worked out from foldexpr, one per line (a fold
    /// starting at a line: Some(its level), Vim's ">N").
    pub fn set_fold_levels(&mut self, levels: Vec<(usize, Option<usize>)>) {
        if levels != self.vim.fold_expr {
            self.vim.fold_expr = levels;
            self.vim.fold_expr_version += 1;
        }
    }
}

/// Folds from levels: for each depth N, the runs of lines at N or deeper; a line where a fold of
/// level N or less starts (">N") ends the run before it.
fn expr_folds(lv: &[(usize, Option<usize>)], nest: usize) -> Vec<(usize, usize, usize)> {
    let n = lv.len();
    let level = |l: usize| lv[l].0.min(nest);
    let mut out = Vec::new();
    for d in 1..=(0..n).map(level).max().unwrap_or(0) {
        let mut l = 0;
        while l < n {
            if level(l) >= d {
                let s = l;
                while l + 1 < n && level(l + 1) >= d && !lv[l + 1].1.is_some_and(|e| e <= d) {
                    l += 1;
                }
                out.push((s, l, d));
            }
            l += 1;
        }
    }
    out
}

#[cfg(test)]
mod expr_tests {
    use super::expr_folds;

    #[test]
    fn levels_become_folds() {
        // 0 1 1 >1 1 0: two folds at level 1 side by side (the > starts the second)
        let lv = [(0, None), (1, Some(1)), (1, None), (1, Some(1)), (1, None), (0, None)];
        assert_eq!(expr_folds(&lv, 20), vec![(1, 2, 1), (3, 4, 1)]);
        // nested: 1 2 2 1
        let lv = [(1, Some(1)), (2, Some(2)), (2, None), (1, None)];
        assert_eq!(expr_folds(&lv, 20), vec![(0, 3, 1), (1, 2, 2)]);
        assert_eq!(expr_folds(&lv, 1), vec![(0, 3, 1)], "foldnestmax");
    }
}
