//! What the window draws from the editor's Vim state: the fold column, and the last search's matches
//! for hlsearch (off after :nohlsearch until the next search or n).

use super::*;

impl Editor {
    /// The fold column for `line`: how many manual folds it is in, and whether one starts there
    /// (Some(closed)).
    pub fn fold_mark(&self, line: usize) -> (usize, Option<bool>) {
        if self.vim.folds_off {
            return (0, None);
        }
        let ls = self.fold_lines();
        let depth = ls.iter().filter(|f| f.0 <= line && line <= f.1).count();
        // closed only when it hides lines (foldminlines keeps short ones open)
        let min = self.opt_num("foldminlines", 1).max(0) as usize;
        let start = ls.iter().filter(|f| f.0 == line).min_by_key(|f| f.1).map(|f| f.2 && f.1 > f.0 && f.1 - f.0 >= min);
        (depth, start)
    }

    /// There are manual folds (foldcolumn=auto shows then).
    pub fn has_folds(&self) -> bool {
        !self.vim.folds.is_empty() && !self.vim.folds_off
    }

    /// The last search's matches that start in [from, to) (char indices), for hlsearch.
    pub fn hl_matches(&self, from: usize, to: usize) -> Vec<(usize, usize)> {
        let q = &self.last_search;
        if q.is_empty() || self.vim.hl_off {
            return Vec::new();
        }
        self.vim_find_all(q, self.folds_for(q), from, to)
    }

    /// showmatch: the bracket a closing one just typed (before the cursor) pairs with (matchpairs).
    pub fn match_before_cursor(&self) -> Option<usize> {
        let at = self.cursor.checked_sub(1)?;
        let close = self.char(at)?;
        let (open, _) = self.match_pairs().into_iter().find(|p| p.1 == close)?;
        let mut depth = 0usize;
        for i in (at.saturating_sub(10_000)..at).rev() {
            match self.char(i) {
                Some(c) if c == close => depth += 1,
                Some(c) if c == open && depth == 0 => return Some(i),
                Some(c) if c == open => depth -= 1,
                _ => {}
            }
        }
        None
    }

    /// An operator waits for its motion (guicursor's o mode).
    pub fn pending_op(&self) -> bool {
        self.op.is_some()
    }

    /// The first key of a two-key command waits for its second (q: g? z t ]c …): that key is the
    /// command's, so no mapping takes it (as Vim).
    pub fn prefix_pending(&self) -> bool {
        self.prefix.is_some()
    }

    /// showcmd: the keys of a command not finished yet ("2d", "\"a", "g"), empty when none waits.
    pub fn pending_keys(&self) -> String {
        let waiting = self.mode == Mode::Normal && (self.op.is_some() || self.prefix.is_some() || self.count.is_some() || self.vim.use_reg.is_some_and(|_| self.vim.reg_before.is_none() || self.vim.reg_armed));
        if !waiting && !matches!(self.mode, Mode::Visual | Mode::VisualLine | Mode::VisualBlock) {
            return String::new();
        }
        let keys: String = self.vim.typing.iter().map(|k| if k.ctrl || k.alt || k.char().is_none() { k.notation() } else { k.char().unwrap().to_string() }).collect();
        // in visual mode: the selection's size, as Vim shows it, when nothing else waits
        if !waiting {
            if let Some((a, b)) = self.selection_range() {
                let (l1, l2) = (self.buf.to_pos(a).line, self.buf.to_pos(b.saturating_sub(1).max(a)).line);
                return if l1 == l2 && self.mode == Mode::Visual { (b - a).to_string() } else { (l2 - l1 + 1).to_string() };
            }
        }
        keys.chars().rev().take(10).collect::<Vec<_>>().into_iter().rev().collect()
    }

    /// :marks: each mark set, as (name, line, column), sorted by name.
    pub fn marks(&self) -> Vec<(char, usize, usize)> {
        let n = self.buf.len_chars();
        let mut v: Vec<(char, usize, usize)> = self.vim.marks.iter().map(|(&c, &p)| {
            let p = self.buf.to_pos(p.min(n));
            (c, p.line, p.col)
        }).collect();
        v.sort();
        v
    }

    /// :mark: mark `c` at char position `at`.
    pub fn set_vim_mark(&mut self, c: char, at: usize) {
        self.vim.marks.insert(c, at.min(self.buf.len_chars()));
    }

    /// :delmarks: the marks named (a-d, x, ! for every lowercase one).
    pub fn del_marks(&mut self, spec: &str) {
        if spec.trim() == "!" {
            self.vim.marks.retain(|c, _| !c.is_ascii_lowercase());
            return;
        }
        let cs: Vec<char> = spec.chars().filter(|c| !c.is_whitespace()).collect();
        let mut i = 0;
        while i < cs.len() {
            if i + 2 < cs.len() && cs[i + 1] == '-' {
                for c in cs[i]..=cs[i + 2] {
                    self.vim.marks.remove(&c);
                }
                i += 3;
            } else {
                self.vim.marks.remove(&cs[i]);
                i += 1;
            }
        }
    }

    /// :jumps: the jump list (char positions) and where in it CTRL-O stands.
    pub fn jump_list(&self) -> (Vec<usize>, usize) {
        (self.vim.jumps.clone(), self.vim.jump_at)
    }

    /// :clearjumps.
    pub fn clear_jumps(&mut self) {
        self.vim.jumps.clear();
        self.vim.jump_tops.clear();
        self.vim.jump_at = 0;
    }

    /// :changes: where changes happened, oldest first.
    pub fn change_list(&self) -> Vec<usize> {
        self.vim.changes.clone()
    }

    /// Register `r` holds `text` (:redir @r, :let @r), linewise when it ends in a line break.
    pub fn set_register(&mut self, r: char, text: &str) {
        let reg = Register { text: text.to_string(), linewise: text.ends_with('\n'), block: false };
        if r == '"' {
            self.reg = reg.clone();
        }
        self.vim.regs.insert(r.to_ascii_lowercase(), reg);
    }

    /// :registers: each register with text, as (name, text, linewise).
    pub fn registers(&self) -> Vec<(char, String, bool)> {
        let mut v: Vec<(char, String, bool)> = self.vim.regs.iter().map(|(&c, r)| (c, r.text.clone(), r.linewise)).collect();
        if !self.reg.text.is_empty() && !self.vim.regs.contains_key(&'"') {
            v.push(('"', self.reg.text.clone(), self.reg.linewise));
        }
        v.sort_by_key(|r| r.0);
        v
    }

    /// :nohlsearch: the matches stop showing until the next search.
    pub fn nohlsearch(&mut self) {
        self.vim.hl_off = true;
    }
}
