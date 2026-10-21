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
        let (n, fold) = (q.chars().count(), self.folds_for(q));
        let mut out = Vec::new();
        let mut at = from;
        while let Some(i) = self.buf.find(q, at, true, fold) {
            if i >= to || out.len() >= 1000 {
                break;
            }
            out.push((i, i + n));
            at = i + 1;
        }
        out
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

    /// :nohlsearch: the matches stop showing until the next search.
    pub fn nohlsearch(&mut self) {
        self.vim.hl_off = true;
    }
}
