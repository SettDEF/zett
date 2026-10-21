//! Tab pages, as Vim's and Emacs's tab-bar have them: each tab keeps its own windows (their split
//! layout, the focused one, their :match patterns), the buffers are shared. With one tab zett is as
//! before (the tab line lists buffers); from two on the tab line lists the tabs.

use super::*;

/// A tab that is not the one shown: its windows, as they were left.
#[derive(Clone)]
pub(crate) struct TabPage {
    wins: Vec<Option<Win>>,
    tree: WTree,
    focus: usize,
    last_win: Option<usize>,
    cur: usize,
    win_match: std::collections::HashMap<(usize, u8), (String, String)>,
    /// tab-rename's name (else the focused window's buffer)
    pub name: Option<String>,
}

impl App {
    /// How many tabs there are (one when none was ever made).
    pub(crate) fn tab_count(&self) -> usize {
        self.tab_pages.len().max(1)
    }

    /// The tab shown now, kept in its slot (the live windows are App's own fields).
    fn tab_save(&mut self) {
        let cur = self.cur;
        if let Some(Some(w)) = self.wins.get_mut(self.focus) {
            w.doc = cur;
        }
        let page = TabPage {
            wins: self.wins.clone(),
            tree: self.tree.clone(),
            focus: self.focus,
            last_win: self.last_win,
            cur: self.cur,
            win_match: self.win_match.clone(),
            name: self.tab_pages.get(self.tab).and_then(|t| t.name.clone()),
        };
        if self.tab_pages.is_empty() {
            self.tab_pages.push(page);
            self.tab = 0;
        } else {
            self.tab_pages[self.tab] = page;
        }
    }

    fn tab_load(&mut self, i: usize) {
        let Some(t) = self.tab_pages.get(i).cloned() else { return };
        self.tab = i;
        self.wins = t.wins;
        self.tree = t.tree;
        self.last_win = t.last_win;
        self.win_match = t.win_match;
        self.focus = t.focus;
        self.cur = t.cur.min(self.docs.len().saturating_sub(1));
        // the windows' rectangles are the old tab's until the next draw
        self.win_rects.clear();
        let f = self.focus;
        self.focus_win(f);
        if let Some(Some(w)) = self.wins.get(f) {
            let (top, left) = (w.top, w.left);
            self.docs[self.cur].top = top;
            self.docs[self.cur].left = left;
        }
    }

    /// A new tab after this one (at `at`, when given), one window on buffer `doc` (this one's).
    pub(crate) fn tab_new(&mut self, doc: Option<usize>, at: Option<usize>) {
        self.tab_save();
        let doc = doc.unwrap_or(self.cur);
        let w = Win { doc, lcd: self.wins.get(self.focus).and_then(|w| w.as_ref()).and_then(|w| w.lcd.clone()), ..Default::default() };
        let page = TabPage { wins: vec![Some(w)], tree: WTree::Leaf(0), focus: 0, last_win: None, cur: doc, win_match: Default::default(), name: None };
        let i = at.unwrap_or(self.tab + 1).min(self.tab_pages.len());
        self.tab_pages.insert(i, page);
        self.tab_load(i);
    }

    /// Tab `i` (0-based) shown.
    pub(crate) fn tab_goto(&mut self, i: usize) {
        if self.tab_pages.len() < 2 || i >= self.tab_pages.len() || i == self.tab {
            return;
        }
        self.tab_save();
        self.tab_load(i);
    }

    /// `n` tabs on (back when negative), round the ends.
    pub(crate) fn tab_step(&mut self, n: i64) {
        let len = self.tab_count() as i64;
        if len < 2 {
            return;
        }
        let i = (self.tab as i64 + n).rem_euclid(len) as usize;
        self.tab_goto(i);
    }

    /// Tab `i` closed (its windows go; the buffers stay). The last one cannot be.
    pub(crate) fn tab_close(&mut self, i: usize) -> bool {
        if self.tab_count() < 2 {
            self.say("E784: Cannot close last tab page");
            return false;
        }
        self.tab_save();
        let closed = self.tab_pages.remove(i);
        self.closed_tabs.push(closed);
        let next = if i < self.tab { self.tab - 1 } else if i == self.tab { i.min(self.tab_pages.len() - 1) } else { self.tab };
        self.tab_load(next);
        if self.tab_pages.len() == 1 {
            self.tab_pages.clear();
            self.tab = 0;
        }
        true
    }

    /// Every tab but this one closed.
    pub(crate) fn tab_only(&mut self) {
        if self.tab_count() < 2 {
            return;
        }
        self.tab_save();
        let keep = self.tab_pages.remove(self.tab);
        self.closed_tabs.append(&mut self.tab_pages);
        self.tab_pages = vec![keep];
        self.tab_load(0);
        self.tab_pages.clear();
        self.tab = 0;
    }

    /// The tab moved to place `to` (0-based, clamped).
    pub(crate) fn tab_move(&mut self, to: usize) {
        if self.tab_count() < 2 {
            return;
        }
        self.tab_save();
        let t = self.tab_pages.remove(self.tab);
        let to = to.min(self.tab_pages.len());
        self.tab_pages.insert(to, t);
        self.tab = to;
    }

    /// tab-undo: the tab closed last, back.
    pub(crate) fn tab_undo(&mut self) {
        let Some(t) = self.closed_tabs.pop() else { return self.say("No closed tab to bring back") };
        self.tab_save();
        let i = self.tab + 1;
        self.tab_pages.insert(i, t);
        self.tab_load(i);
    }

    /// A tab's label: its name, else the focused window's buffer (and how many windows).
    pub(crate) fn tab_label(&self, i: usize) -> String {
        let (name, doc, n) = if i == self.tab {
            (self.tab_pages.get(i).and_then(|t| t.name.clone()), self.cur, self.tree.ids().len())
        } else {
            let t = &self.tab_pages[i];
            (t.name.clone(), t.cur, t.tree.ids().len())
        };
        let base = name.unwrap_or_else(|| {
            let d = self.docs.get(doc).map(|d| d.name.clone()).unwrap_or_default();
            Path::new(&d).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or(d)
        });
        if n > 1 { format!("{base} +{}", n - 1) } else { base }
    }

    pub(crate) fn tab_rename(&mut self, name: &str) {
        self.tab_save();
        let name = name.trim();
        self.tab_pages[self.tab].name = (!name.is_empty()).then(|| name.to_string());
    }

    /// :tabs, as Vim lists them: each tab and its windows' buffers (> the current one).
    pub(crate) fn tab_list(&mut self) {
        self.tab_save();
        let mut rows = Vec::new();
        for (i, t) in self.tab_pages.iter().enumerate() {
            rows.push((format!("Tab page {}", i + 1), t.name.clone().unwrap_or_default()));
            for id in t.tree.ids() {
                let Some(Some(w)) = t.wins.get(id) else { continue };
                let d = self.docs.get(w.doc).map(|d| d.name.clone()).unwrap_or_default();
                rows.push((format!("{}   ", if i == self.tab && id == t.focus { ">" } else { " " }), d));
            }
        }
        if self.tab_pages.len() == 1 && self.tab_pages[0].name.is_none() {
            self.tab_pages.clear();
        }
        self.popup = Some(Popup::Help("Tab pages".into(), rows));
    }

    /// CTRL-W T: this window into a tab of its own.
    pub(crate) fn win_to_tab(&mut self) {
        if self.tree.ids().len() < 2 {
            return self.say("E1218: one window: it is a tab of its own already");
        }
        let doc = self.cur;
        self.doom_ui("closeWin", "");
        self.tab_new(Some(doc), None);
    }

    /// Each window of the tabs not shown, and each one's current buffer (buffers went or moved).
    pub(crate) fn tabs_each_win(&mut self, mut win: impl FnMut(&mut Win), mut cur: impl FnMut(&mut usize)) {
        for (i, t) in self.tab_pages.iter_mut().enumerate() {
            if i == self.tab {
                continue;
            }
            t.wins.iter_mut().flatten().for_each(&mut win);
            cur(&mut t.cur);
        }
    }

    /// The tab line with tabs (from two on): " 1 name " each, the current one marked.
    pub(crate) fn draw_tab_pages(&self, f: &mut Frame, a: Rect) {
        let mut spans = Vec::new();
        for i in 0..self.tab_count() {
            let st = if i == self.tab { hl("TabLineSel") } else { hl("TabLine") };
            spans.push(Span::styled(format!(" {} {} ", i + 1, self.tab_label(i)), st));
            spans.push(Span::raw(" "));
        }
        f.render_widget(Paragraph::new(Line::from(spans)), a);
    }
}

#[cfg(test)]
mod tests {
    use crate::App;

    #[test]
    fn tabs_keep_their_own_windows() {
        let mut a = App::new(Vec::new());
        a.command("vsplit");
        assert_eq!(a.tree.ids().len(), 2);
        a.command("tabnew");
        assert_eq!((a.tab_count(), a.tab, a.tree.ids().len()), (2, 1, 1), "a new tab: one window");
        a.command("tabprevious");
        assert_eq!((a.tab, a.tree.ids().len()), (0, 2), "the first tab's split is still there");
        a.command("tabnext");
        a.command("tabclose");
        assert_eq!((a.tab_count(), a.tree.ids().len()), (1, 2));
        a.command("tabclose");
        assert_eq!(a.tab_count(), 1, "the last tab stays");
        a.command("tabnew");
        a.command("tabnew");
        a.command("tabmove 0");
        assert_eq!(a.tab, 0);
        a.command("tabonly");
        assert_eq!(a.tab_count(), 1);
        a.tab_undo();
        assert_eq!(a.tab_count(), 2, "tab-undo brings one back");
    }
}
