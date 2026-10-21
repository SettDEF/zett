//! Undo: changes made of edits, the undo tree, the history kept across restarts.

use super::*;

impl Editor {
    // ── undo: changes made of edits (a whole insert session is one change) ──

    /// Starts recording a new change; the previous one is finished.
    pub(crate) fn begin(&mut self) {
        self.close();
        self.open = Some(Change { edits: Vec::new(), cursor: self.cursor });
    }
    pub(crate) fn close(&mut self) {
        if let Some(c) = self.open.take() {
            if !c.edits.is_empty() {
                self.undo.push(c);
                // 'undolevels': -1 none, 0 one (as Vi), else that many
                let levels = self.opt_num("undolevels", 1000);
                self.undo.keep(if levels < 0 { 0 } else { levels.max(1) as usize });
            }
        }
    }
    /// Inserts, recording the edit in the open change.
    pub(crate) fn ins(&mut self, at: usize, s: &str) {
        if self.refuses_change() {
            return;
        }
        self.buf.insert(at, s);
        self.vim.shift(at, s.chars().count() as isize);
        if let Some(c) = self.open.as_mut() {
            c.edits.push(Edit { at, removed: String::new(), inserted: s.to_string() });
        }
    }
    /// Removes `a..b`, recording the edit in the open change.
    pub(crate) fn rem(&mut self, a: usize, b: usize) -> String {
        if self.refuses_change() {
            return String::new();
        }
        let gone = self.buf.remove(a, b);
        self.vim.removed(a.min(b), gone.chars().count());
        if let Some(c) = self.open.as_mut() {
            c.edits.push(Edit { at: a.min(b), removed: gone.clone(), inserted: String::new() });
        }
        gone
    }
    pub(crate) fn undo_once(&mut self) {
        self.close();
        let n = self.undo.cur;
        if n == 0 || n == self.undo.floor {
            return;
        }
        self.undo.nodes[n].change.revert(&mut self.buf);
        self.cursor = self.undo.nodes[n].change.cursor;
        let p = self.undo.nodes[n].parent;
        self.undo.nodes[p].redo = Some(n);
        self.undo.cur = p;
        self.mode = Mode::Normal;
        self.clamp();
    }
    pub(crate) fn redo_once(&mut self) {
        self.close();
        if let Some(n) = self.undo.nodes[self.undo.cur].redo {
            self.forward(n);
            // Vim puts the cursor where the change starts
            let c = &self.undo.nodes[n].change;
            self.cursor = c.edits.first().map_or(c.cursor, |e| e.at);
            self.clamp();
        }
    }
    /// Applies child `n` of the current node.
    pub(crate) fn forward(&mut self, n: usize) {
        self.undo.nodes[n].change.apply(&mut self.buf);
        let p = self.undo.nodes[n].parent;
        self.undo.nodes[p].redo = Some(n);
        self.undo.cur = n;
    }

    // ── the undo tree, for undotree-like views ──

    /// Ends the change being typed, so the history includes it: always in the Emacs profile
    /// (modeless, always "inserting"), in Vim's only outside an insert session (one session is
    /// one undo step there).
    pub(crate) fn settle(&mut self) {
        if self.emacs || self.mode != Mode::Insert {
            self.close();
        }
    }

    /// Every state: (its parent, when it was made in seconds since the epoch). State 0 is
    /// the text as opened; a state's number is its place in time.
    pub fn undo_states(&mut self) -> Vec<(usize, u64)> {
        // a finished change counts at once; one still being typed does not
        self.settle();
        self.undo.nodes.iter().map(|n| (n.parent, n.time)).collect()
    }
    /// `u` and `C-r` once, for scripts.
    pub fn undo(&mut self) {
        self.undo_once();
    }
    pub fn redo(&mut self) {
        self.redo_once();
    }
    /// The state the text is in.
    pub fn undo_current(&mut self) -> usize {
        self.settle();
        self.undo.cur
    }
    /// The text as it is at state `n`, without going there.
    pub fn text_at(&mut self, n: usize) -> String {
        self.settle();
        if n >= self.undo.nodes.len() { self.text() } else { self.undo.text_at(&self.text(), n) }
    }
    /// The whole undo history as bytes, to keep it across restarts.
    pub fn history_save(&mut self) -> Vec<u8> {
        self.settle();
        self.undo.save(&self.text())
    }

    /// Text that a saved history once removed (deleted or replaced) containing `needle` (any case
    /// unless it has capitals): (the state it was removed by, when, its line). The state before
    /// that one still has it.
    pub fn history_search(bytes: &[u8], needle: &str, max: usize) -> Vec<(usize, u64, String)> {
        let Some((t, _)) = undo::Tree::read(bytes) else { return Vec::new() };
        let smart = needle.chars().any(char::is_uppercase);
        let low = needle.to_lowercase();
        let mut out = Vec::new();
        for (i, n) in t.nodes.iter().enumerate().rev() {
            for e in &n.change.edits {
                let hit = e.removed.lines().find(|l| if smart { l.contains(needle) } else { l.to_lowercase().contains(&low) });
                if let Some(l) = hit {
                    out.push((i, n.time, l.trim().chars().take(160).collect()));
                    break;
                }
            }
            if out.len() >= max {
                break;
            }
        }
        out
    }

    /// Takes back a saved history; false (and nothing changes) if it was saved for other text.
    pub fn history_load(&mut self, bytes: &[u8]) -> bool {
        match undo::Tree::load(bytes, &self.text()) {
            Some(t) => {
                self.undo = t;
                true
            }
            None => false,
        }
    }

    /// Time travel's "restore this section": the section `title` as it was at state `n` replaces
    /// the one of that title now, as one change (later work elsewhere stays). False: it is not
    /// in both.
    pub fn restore_section(&mut self, n: usize, title: &str) -> bool {
        let then = self.text_at(n);
        let old: Vec<String> = then.lines().map(String::from).collect();
        let Some(s_old) = outline::sections(&old, self.view_kind()).into_iter().find(|s| s.title == title) else { return false };
        let Some(s_new) = self.sections().into_iter().find(|s| s.title == title) else { return false };
        let mut body: String = old[s_old.start..s_old.end].iter().map(|l| format!("{l}\n")).collect();
        let a = self.line_start(s_new.start);
        let b = if s_new.end >= self.buf.len_lines() { self.buf.len_chars() } else { self.line_start(s_new.end) };
        // the end of the file without a line break keeps it that way
        if b == self.buf.len_chars() && !self.buf.slice(a, b).ends_with('\n') {
            body.pop();
        }
        self.begin();
        self.rem(a, b);
        self.ins(a, &body);
        self.close();
        self.cursor = a;
        true
    }

    /// Goes to state `n`, along the tree (undoing up to the common state, then redoing down).
    pub fn undo_goto(&mut self, n: usize) {
        self.close();
        if n >= self.undo.nodes.len() || n == self.undo.cur {
            return;
        }
        let (up, down) = self.undo.path(self.undo.cur, n);
        for _ in up {
            self.undo_once();
        }
        for d in down {
            self.forward(d);
            self.cursor = self.undo.nodes[d].change.edits.first().map_or(self.cursor, |e| e.at);
        }
        self.mode = Mode::Normal;
        self.clamp();
    }

}

#[cfg(test)]
mod undolevels {
    use crate::Editor;

    fn changes(e: &mut Editor, n: usize) {
        for i in 0..n {
            e.begin();
            e.ins(e.text().chars().count(), &i.to_string());
            e.close();
        }
    }

    #[test]
    fn undo_goes_back_no_further_than_undolevels() {
        let mut e = Editor::from_str("x");
        e.set_opt("undolevels", "2");
        changes(&mut e, 3);
        assert_eq!(e.text(), "x012");
        for _ in 0..3 {
            e.undo_once();
        }
        assert_eq!(e.text(), "x0", "two steps back, the first change kept");
        e.redo_once();
        assert_eq!(e.text(), "x01");
    }

    #[test]
    fn undolevels_minus_one_keeps_no_undo() {
        let mut e = Editor::from_str("x");
        e.set_opt("undolevels", "-1");
        changes(&mut e, 2);
        e.undo_once();
        assert_eq!(e.text(), "x01");
    }
}
