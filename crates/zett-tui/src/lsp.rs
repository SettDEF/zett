//! Language servers in the editor: diagnostics, definitions, hover.

use super::*;
use zett_lsp::{Diag, Event, Location};

impl Doc {
    pub(crate) fn abs_path(&self) -> Option<PathBuf> {
        let p = self.ed.buf.path()?;
        Some(std::fs::canonicalize(p).unwrap_or(p.to_path_buf()))
    }

    pub(crate) fn text(&self) -> String {
        let b = &self.ed.buf;
        b.slice(0, usize::MAX)
    }
}

impl App {
    /// Tells the file's server it is open (starting the server when needed).
    pub(crate) fn lsp_open(&mut self, di: usize) {
        let Some(path) = self.docs[di].abs_path() else { return };
        let root = doom::project_of(&path).unwrap_or_else(|| path.parent().map(Path::to_path_buf).unwrap_or_default());
        let text = self.docs[di].text();
        let rev = self.docs[di].ed.buf.rev();
        if let Some((c, lang)) = self.lsp.for_file(&path, &root) {
            if !c.is_open(&path) {
                c.did_open(&path, lang, &text);
            }
            self.docs[di].lsp = true;
            self.docs[di].lsp_rev = rev;
            self.fire_autocmds("lspattach", None);
        }
    }

    pub(crate) fn client_for(&mut self, di: usize) -> Option<&mut zett_lsp::Client> {
        let path = self.docs[di].abs_path()?;
        let root = doom::project_of(&path).unwrap_or_else(|| path.parent().map(Path::to_path_buf).unwrap_or_default());
        self.lsp.for_file(&path, &root).map(|(c, _)| c)
    }

    /// Each turn of the loop: what servers said, and text they have not seen yet.
    pub(crate) fn lsp_tick(&mut self) {
        for ev in self.lsp.events() {
            self.lsp_event(ev);
        }
        self.comp_tick();
        self.comp_resolve_sel();
        // text changed since the server last saw it, and the typing paused
        if self.edit_at.elapsed() < Duration::from_millis(250) {
            return;
        }
        self.lsp_sync();
    }

    fn lsp_event(&mut self, ev: Event) {
            match ev {
                Event::Diagnostics(path, diags) => {
                    if let Some(d) = self.docs.iter_mut().find(|d| d.abs_path().as_deref() == Some(path.as_path())) {
                        d.diags = diags;
                    }
                }
                Event::Definition(locs) => match locs.first() {
                    Some(l) => self.goto_location(l.clone()),
                    None => self.say("no definition found"),
                },
                Event::Completion(line, items, more) => self.comp_arrived(line, items, more),
                Event::Resolved(it) => self.comp_resolved(it),
                Event::References(locs) => self.show_locations("References", locs),
                Event::Symbols(syms) => self.show_symbols(syms),
                Event::Edits(e) => self.apply_workspace(e),
                Event::Hover(text) => {
                    let rows: Vec<(String, String)> = text.lines().filter(|l| !l.starts_with("```")).map(|l| (String::new(), l.to_string())).collect();
                    if rows.iter().all(|r| r.1.trim().is_empty()) {
                        self.say("nothing to show here");
                    } else {
                        self.popup = Some(Popup::Help("Documentation".into(), rows));
                    }
                }
                Event::Status(s) => self.lsp_status = s,
                Event::Progress { server, token, title, message, percent, done } => {
                    if self.engine.has_hooks("lsp_progress") {
                        let f = [("server", server), ("token", token), ("title", title), ("message", message), ("percent", percent.map(|p| p.to_string()).unwrap_or_default()), ("done", done.to_string())];
                        let fx = self.with_lua(|e| e.emit_fields("lsp_progress", &f));
                        self.after_lua(fx);
                    }
                }
                // the window's (signature help, code actions): the terminal editor does not ask for them yet
                Event::Signature(..) | Event::CodeActions(_) => {}
                Event::Exited(tag) => {
                    // only that client: a restarted server's new one has the same name
                    let had = self.lsp.clients.len();
                    self.lsp.clients.retain(|_, c| c.tag != tag);
                    // the reason (its last stderr line) came as a Status just before; the bare word only without one
                    if self.lsp.clients.len() != had && !self.msg.as_ref().is_some_and(|(m, _)| m.contains("stopped:")) {
                        self.say(format!("{} stopped", tag.split('#').next().unwrap_or(&tag)));
                    }
                }
            }
    }

    /// Sends changed text to the servers.
    pub(crate) fn lsp_sync(&mut self) {
        for di in 0..self.docs.len() {
            let d = &self.docs[di];
            if !d.lsp || d.ed.buf.rev() == d.lsp_rev {
                continue;
            }
            let (Some(path), text, rev) = (d.abs_path(), d.text(), d.ed.buf.rev()) else { continue };
            if let Some(c) = self.client_for(di) {
                c.did_change(&path, &text);
            }
            self.docs[di].lsp_rev = rev;
        }
    }

    pub(crate) fn lsp_saved(&mut self, di: usize) {
        if !self.docs[di].lsp {
            return;
        }
        let Some(path) = self.docs[di].abs_path() else { return };
        // the server sees the saved text before the save itself
        let text = self.docs[di].text();
        let rev = self.docs[di].ed.buf.rev();
        if let Some(c) = self.client_for(di) {
            c.did_change(&path, &text);
            c.did_save(&path);
        }
        self.docs[di].lsp_rev = rev;
    }

    /// The cursor's position in the server's column units.
    pub(crate) fn lsp_pos(&mut self) -> Option<(PathBuf, usize, usize)> {
        let di = self.cur;
        let path = self.docs[di].abs_path()?;
        let c = self.docs[di].ed.cursor();
        let line = self.docs[di].ed.buf.line(c.line);
        let utf16 = self.client_for(di)?.utf16();
        Some((path, c.line, if utf16 { zett_lsp::to_utf16(&line, c.col) } else { c.col }))
    }

    pub(crate) fn lsp_definition(&mut self) {
        if !self.docs[self.cur].lsp {
            return self.say("no language server for this file");
        }
        // the server must see the text as it is now
        self.edit_at = Instant::now() - Duration::from_secs(1);
        self.lsp_tick();
        let Some((path, line, col)) = self.lsp_pos() else { return };
        if let Some(c) = self.client_for(self.cur) {
            c.definition(&path, line, col);
        }
    }

    pub(crate) fn lsp_hover(&mut self) {
        if !self.docs[self.cur].lsp {
            return self.say("no language server for this file");
        }
        self.edit_at = Instant::now() - Duration::from_secs(1);
        self.lsp_tick();
        let Some((path, line, col)) = self.lsp_pos() else { return };
        if let Some(c) = self.client_for(self.cur) {
            c.hover(&path, line, col);
        }
    }

    fn goto_location(&mut self, l: Location) {
        let utf16 = self.client_for(self.cur).map(|c| c.utf16()).unwrap_or(true);
        self.open(&l.path);
        let d = &self.docs[self.cur];
        let line = l.line.min(d.ed.buf.len_lines().saturating_sub(1));
        let text = d.ed.buf.line(line);
        let col = if utf16 { zett_lsp::from_utf16(&text, l.col) } else { l.col };
        self.doc().ed.set_cursor(Pos { line, col });
    }

    /// ]e and [e: the next or previous diagnostic in the buffer.
    pub(crate) fn next_diag(&mut self, forward: bool) {
        let d = &self.docs[self.cur];
        let c = d.ed.cursor();
        let mut ds: Vec<&Diag> = d.diags.iter().collect();
        ds.sort_by_key(|x| (x.line, x.col));
        let target = if forward {
            ds.iter().find(|x| (x.line, x.col) > (c.line, c.col)).or(ds.first())
        } else {
            ds.iter().rev().find(|x| (x.line, x.col) < (c.line, c.col)).or(ds.last())
        };
        match target.map(|x| (x.line, x.col, x.message.clone())) {
            Some((line, col, m)) => {
                self.doc().ed.set_cursor(Pos { line, col });
                self.say(m);
            }
            None => self.say("no errors or warnings here"),
        }
    }

    /// The diagnostic message on the cursor's line, for the bottom line.
    pub(crate) fn line_diag(&self) -> Option<(u8, String)> {
        let d = &self.docs[self.cur];
        let l = d.ed.cursor().line;
        d.diags.iter().filter(|x| x.line == l).min_by_key(|x| x.severity).map(|x| (x.severity, x.message.clone()))
    }
}

/// The colour of a diagnostic's severity.
pub(crate) fn severity_color(s: u8) -> Color {
    let g = match s {
        1 => "DiagnosticError",
        2 => "DiagnosticWarn",
        3 => "DiagnosticInfo",
        _ => "DiagnosticHint",
    };
    hl(g).fg.unwrap_or(Color::Reset)
}

/// The completion popup in insert mode.
pub(crate) struct Comp {
    pub items: Vec<zett_lsp::Item>,
    /// indexes into `items`, filtered by what is typed
    pub shown: Vec<usize>,
    pub sel: usize,
    pub line: usize,
    /// where the word being completed starts
    pub start: usize,
    /// keyword completion (CTRL-N / CTRL-P): what was typed (CTRL-E brings it back); None from a server
    pub keyword: Option<String>,
    /// the server's list is only a start: typing asks again
    pub incomplete: bool,
    /// the item whose documentation was asked for (completionItem/resolve)
    pub resolving: Option<usize>,
}

fn ident(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

impl App {
    /// After a key in insert mode: refine the popup, ask for one, or close it.
    pub(crate) fn comp_after_key(&mut self, typed: Option<char>) {
        self.snippet_check();
        // keyword completion ends with the next key typed (the word stays as it is)
        if self.comp.as_ref().is_some_and(|c| c.keyword.is_some()) {
            self.comp = None;
            return;
        }
        if !self.docs[self.cur].lsp || self.docs[self.cur].ed.mode() != Mode::Insert {
            self.comp = None;
            self.comp_due = None;
            return;
        }
        // autocomplete: the menu comes by itself after autocompletedelay ms (else C-n / C-x C-o ask)
        let auto = self.flag(self.cur, "autocomplete");
        let delay = Duration::from_millis(self.setting(self.cur, "autocompletedelay").parse().unwrap_or(120));
        // the server's trigger characters (`.`, `:`, `<`, `"`, `/`, `@` …); `.` and `:` without a list
        let triggers = self.client_for(self.cur).map(|c| c.triggers()).unwrap_or_default();
        let is_trigger = |c: char| if triggers.is_empty() { c == '.' || c == ':' } else { triggers.iter().any(|t| t.ends_with(c)) };
        let before = {
            let d = &self.docs[self.cur];
            let c = d.ed.cursor();
            d.ed.buf.line(c.line).chars().take(c.col).last()
        };
        match typed {
            Some(c) if ident(c) => {
                if self.comp.is_some() {
                    self.comp_filter();
                    // a list that is only a start: ask again (the menu stays until the answer)
                    if self.comp.as_ref().is_some_and(|c| c.incomplete) {
                        self.comp_trig = None;
                        self.comp_due = Some(Instant::now() + delay);
                    }
                } else if auto {
                    self.comp_trig = None;
                    self.comp_due = Some(Instant::now() + delay);
                }
            }
            Some(c) if c != '\0' && is_trigger(c) => {
                self.comp = None;
                self.comp_trig = Some(c);
                self.comp_due = auto.then(|| Instant::now() + delay / 2);
            }
            // Backspace: the menu refined, or back when the word before the cursor is still there
            None => {
                if self.comp.is_some() {
                    self.comp_filter();
                }
                // the whole word deleted: nothing left to complete
                if self.comp.as_ref().is_some_and(|c| self.docs[self.cur].ed.cursor().col <= c.start) {
                    self.comp = None;
                }
                if self.comp.is_none() && auto && before.is_some_and(ident) {
                    self.comp_trig = None;
                    self.comp_due = Some(Instant::now() + delay);
                } else if self.comp.is_none() {
                    self.comp_due = None;
                }
            }
            // anything else (a space, an arrow, Enter): the menu goes
            _ => {
                self.comp = None;
                self.comp_due = None;
            }
        }
    }

    /// Asks the server when the pause after typing is over.
    pub(crate) fn comp_tick(&mut self) {
        if self.comp_due.is_none_or(|t| Instant::now() < t) {
            return;
        }
        self.comp_due = None;
        if self.docs[self.cur].ed.mode() != Mode::Insert {
            return;
        }
        self.edit_at = Instant::now() - Duration::from_secs(1);
        self.lsp_sync();
        let Some((path, line, col)) = self.lsp_pos() else { return };
        let trig = self.comp_trig.take();
        self.comp_doc = self.cur;
        if let Some(c) = self.client_for(self.cur) {
            c.completion(&path, line, col, trig);
        }
    }

    pub(crate) fn comp_arrived(&mut self, line: usize, items: Vec<zett_lsp::Item>, incomplete: bool) {
        // late answers: another buffer, another line, insert mode left
        if self.comp_doc != self.cur {
            return;
        }
        let utf16 = self.client_for(self.cur).map(|c| c.utf16()).unwrap_or(true);
        let d = &self.docs[self.cur];
        let c = d.ed.cursor();
        if d.ed.mode() != Mode::Insert || c.line != line || items.is_empty() {
            return;
        }
        let text: Vec<char> = d.ed.buf.line(c.line).chars().collect();
        let mut start = c.col.min(text.len());
        while start > 0 && ident(text[start - 1]) {
            start -= 1;
        }
        // the server's own range says where the word starts (`@decorator`, `#include <…`)
        let line_text = d.ed.buf.line(c.line);
        if let Some((l, col, ..)) = items.first().and_then(|i| i.range) {
            let col = if utf16 { zett_lsp::from_utf16(&line_text, col) } else { col };
            if l == line && col <= c.col {
                start = col;
            }
        }
        let keep = self.comp.as_ref().and_then(|c| c.shown.get(c.sel).map(|&i| c.items[i].label.clone()));
        self.comp = Some(Comp { items, shown: Vec::new(), sel: 0, line, start, keyword: None, incomplete, resolving: None });
        self.comp_filter();
        // asked again while typing: the same item stays chosen
        if let (Some(label), Some(comp)) = (keep, self.comp.as_mut()) {
            if let Some(p) = comp.shown.iter().position(|&i| comp.items[i].label == label) {
                comp.sel = p;
            }
        }
    }

    fn comp_filter(&mut self) {
        // matching starts the word (as Neovim); completeopt=fuzzy matches anywhere too
        let fuzzy_on = self.setting(self.cur, "completeopt").split(',').any(|o| o.trim() == "fuzzy");
        let d = &self.docs[self.cur];
        let c = d.ed.cursor();
        let Some(comp) = self.comp.as_mut() else { return };
        if c.line != comp.line || c.col < comp.start {
            self.comp = None;
            return;
        }
        let prefix: String = d.ed.buf.line(c.line).chars().skip(comp.start).take(c.col - comp.start).collect::<String>().to_lowercase();
        let q: Vec<char> = prefix.chars().collect();
        // what starts with the typed part first, then fuzzy matches; each in the server's order
        let mut scored: Vec<(bool, String, usize)> = comp
            .items
            .iter()
            .enumerate()
            .filter_map(|(i, it)| {
                let l = if it.filter.is_empty() { it.label.to_lowercase() } else { it.filter.to_lowercase() };
                let starts = l.starts_with(&prefix);
                (starts || (fuzzy_on && fuzzy(&q, &l).is_some())).then(|| (!starts, it.sort.clone(), i))
            })
            .collect();
        scored.sort();
        comp.shown = scored.into_iter().take(60).map(|s| s.2).collect();
        comp.sel = comp.sel.min(comp.shown.len().saturating_sub(1));
        if comp.shown.is_empty() {
            self.comp = None;
        }
    }

    /// The chosen item's documentation (and imports) from the server, asked once per item.
    pub(crate) fn comp_resolve_sel(&mut self) {
        let Some(comp) = &self.comp else { return };
        let Some(&i) = comp.shown.get(comp.sel) else { return };
        if comp.keyword.is_some() || comp.resolving == Some(i) || comp.items[i].resolved {
            return;
        }
        let it = comp.items[i].clone();
        let asked = self.client_for(self.cur).is_some_and(|c| c.resolve(&it));
        if let Some(comp) = self.comp.as_mut() {
            comp.resolving = Some(i);
            if !asked {
                comp.items[i].resolved = true;
            }
        }
    }

    pub(crate) fn comp_resolved(&mut self, it: zett_lsp::Item) {
        let Some(comp) = self.comp.as_mut() else { return };
        if let Some(old) = comp.items.iter_mut().find(|o| o.label == it.label && o.sort == it.sort) {
            if !it.doc.is_empty() {
                old.doc = it.doc;
            }
            if !it.detail.is_empty() {
                old.detail = it.detail;
            }
            if !it.extra.is_empty() {
                old.extra = it.extra;
            }
            old.resolved = true;
        }
    }

    /// Enter / CTRL-Y on the popup: the item in place of what was typed (the server's range),
    /// a snippet's stops to Tab through, and the imports it brings.
    pub(crate) fn comp_accept(&mut self) {
        // its imports may only come with the rest of it: asked for and waited on a moment
        self.comp_resolve_sel();
        if let Some(i) = self.comp.as_ref().and_then(|c| c.resolving.filter(|&i| !c.items[i].resolved)) {
            let end = Instant::now() + Duration::from_millis(300);
            while Instant::now() < end && self.comp.as_ref().is_some_and(|c| !c.items[i].resolved) {
                for ev in self.lsp.events() {
                    self.lsp_event(ev);
                }
                std::thread::sleep(Duration::from_millis(5));
            }
        }
        let Some(comp) = self.comp.take() else { return };
        let Some(&i) = comp.shown.get(comp.sel) else { return };
        let it = comp.items[i].clone();
        self.comp_due = None;
        // keyword completion: the plain word, typed in (so . repeats it)
        if comp.keyword.is_some() {
            let typed = self.docs[self.cur].ed.cursor().col.saturating_sub(comp.start);
            for _ in 0..typed {
                self.edit_key(Key::code(Code::Backspace));
            }
            for ch in it.insert.chars() {
                self.edit_key(Key::ch(ch));
            }
            return;
        }
        let utf16 = self.client_for(self.cur).map(|c| c.utf16()).unwrap_or(true);
        let d = &mut self.docs[self.cur];
        let cur = d.ed.cursor();
        let a = d.ed.buf.to_char(Pos { line: cur.line, col: comp.start.min(cur.col) });
        let b = d.ed.offset();
        if it.snippet {
            self.snippet_insert(a, b, &it.body);
        } else {
            d.ed.replace_text(a, b, &it.insert);
            d.touched(cur.line);
        }
        // the imports, elsewhere in the file: the cursor and the snippet's stops move with them
        if !it.extra.is_empty() {
            let d = &mut self.docs[self.cur];
            let conv = |ed: &zett_edit::Editor, line: usize, col: usize| {
                let t = ed.buf.line(line.min(ed.buf.len_lines().saturating_sub(1)));
                Pos { line, col: if utf16 { zett_lsp::from_utf16(&t, col) } else { col } }
            };
            let edits: Vec<(Pos, Pos, String)> = it.extra.iter().map(|(l1, c1, l2, c2, t)| (conv(&d.ed, *l1, *c1), conv(&d.ed, *l2, *c2), t.clone())).collect();
            d.ed.apply_edits_in(edits, true);
            d.states.clear();
            d.touched(0);
        }
        self.edit_at = Instant::now();
    }

    pub(crate) fn draw_comp(&self, f: &mut Frame) {
        let Some(comp) = &self.comp else { return };
        let (cx, cy) = self.cursor_xy;
        let area = f.area();
        // pumheight: at most that many rows (0: as many as fit, up to 10 here)
        let cap = match self.num(self.cur, "pumheight") {
            0 => 10,
            n => n,
        };
        let rows = comp.shown.len().min(cap).max(1) as u16;
        let kind = |k: u8| match k {
            2 | 3 | 4 => "ƒ",
            5 | 10 => "◆",
            6 => "v",
            7 | 22 => "S",
            8 => "I",
            9 => "▤",
            13 | 20 => "E",
            14 => "k",
            21 => "c",
            _ => "·",
        };
        let labels: Vec<(String, String)> = comp.shown.iter().map(|&i| (comp.items[i].label.clone(), comp.items[i].detail.clone())).collect();
        let lw = labels.iter().map(|l| l.0.chars().count()).max().unwrap_or(4).min(40);
        let dw = labels.iter().map(|l| l.1.chars().count()).max().unwrap_or(0).min(30);
        // pumwidth at least, pummaxwidth at most (0: no limit)
        let (minw, maxw) = (self.num(self.cur, "pumwidth"), self.num(self.cur, "pummaxwidth"));
        let w = (lw + dw + 6).max(minw);
        let w = if maxw > 0 { w.min(maxw) } else { w };
        let w = w.min(area.width as usize) as u16;
        let x = cx.saturating_sub((self.docs[self.cur].ed.cursor().col - comp.start) as u16 + 2).min(area.width.saturating_sub(w));
        // a frame (pumborder) takes a row above and below as well
        let framed = !matches!(self.setting(self.cur, "pumborder").as_str(), "" | "none");
        let below = cy + 1 + rows + if framed { 2 } else { 0 } <= area.bottom().saturating_sub(2);
        let y = if below { cy + 1 } else { cy.saturating_sub(rows) };
        let a = Rect { x, y, width: w, height: rows };
        // pumborder: a frame round the menu (none by default, as Neovim), the items inside it
        let set = match self.setting(self.cur, "pumborder").as_str() {
            "single" => Some(ratatui::symbols::border::PLAIN),
            "double" => Some(ratatui::symbols::border::DOUBLE),
            "rounded" => Some(ratatui::symbols::border::ROUNDED),
            "bold" => Some(ratatui::symbols::border::THICK),
            "solid" | "shadow" => Some(ratatui::symbols::border::Set { top_left: " ", top_right: " ", bottom_left: " ", bottom_right: " ", vertical_left: " ", vertical_right: " ", horizontal_top: " ", horizontal_bottom: " " }),
            _ => None,
        };
        let a = match set {
            Some(set) => {
                // the frame stays off the cursor's line: below it from the next row, above it ending a row higher
                // (above with too little room: fewer rows, as Neovim shortens the menu)
                let h = if below { a.height } else { a.height.min(cy.saturating_sub(2)).max(1) };
                let oy = if below { a.y } else { cy.saturating_sub(h + 2) };
                let outer = Rect { x: a.x.saturating_sub(1), y: oy, width: (a.width + 2).min(area.width), height: (h + 2).min(area.height) }.intersection(area);
                crate::layer(f, outer);
                f.render_widget(ratatui::widgets::Block::default().borders(ratatui::widgets::Borders::ALL).border_set(set).border_style(hl("PmenuBorder").patch(hl("FloatBorder"))).style(hl("Pmenu")), outer);
                Rect { x: outer.x + 1, y: outer.y + 1, width: outer.width.saturating_sub(2), height: outer.height.saturating_sub(2) }
            }
            None => {
                crate::layer(f, a);
                a
            }
        };
        let first = comp.sel.saturating_sub(rows as usize - 1);
        let lines: Vec<Line> = comp
            .shown
            .iter()
            .enumerate()
            .skip(first)
            .take(rows as usize)
            .map(|(n, &i)| {
                let it = &comp.items[i];
                let base = if n == comp.sel { hl("PmenuSel") } else { hl("Pmenu") };
                let label: String = it.label.chars().take(lw).collect();
                let detail: String = it.detail.chars().take(dw).collect();
                Line::from(vec![
                    Span::styled(format!(" {} ", kind(it.kind)), if n == comp.sel { base } else { base.patch(hl("PmenuKind")) }),
                    Span::styled(format!("{label:<lw$} "), base),
                    Span::styled(format!("{detail:>dw$} "), if n == comp.sel { base } else { base.patch(hl("PmenuExtra")) }),
                ])
            })
            .collect();
        f.render_widget(Paragraph::new(lines), a);
        // the chosen item's documentation beside the menu (right, else left), as completeopt=popup
        let Some(it) = comp.shown.get(comp.sel).map(|&i| &comp.items[i]) else { return };
        let mut text: Vec<String> = Vec::new();
        if !it.detail.is_empty() && it.detail.chars().count() > dw {
            text.push(it.detail.clone());
        }
        text.extend(it.doc.lines().filter(|l| !l.starts_with("```")).map(String::from));
        while text.last().is_some_and(|l| l.trim().is_empty()) {
            text.pop();
        }
        if text.is_empty() {
            return;
        }
        let right = area.width.saturating_sub(a.right() + 1);
        let left = a.x.saturating_sub(1);
        let room = right.max(left).min(60);
        if room < 16 {
            return;
        }
        let dh = (text.len() as u16).min(area.height.saturating_sub(2)).min(14).max(1);
        let dx = if right >= left { a.right() + 1 } else { a.x.saturating_sub(room + 1) };
        let dy = a.y.min(area.height.saturating_sub(dh));
        let da = Rect { x: dx, y: dy, width: room, height: dh }.intersection(area);
        crate::layer(f, da);
        let rows: Vec<Line> = text.iter().map(|l| Line::styled(format!(" {l}"), hl("Pmenu"))).collect();
        f.render_widget(Paragraph::new(rows).wrap(ratatui::widgets::Wrap { trim: false }).style(hl("Pmenu")), da);
    }
}

impl App {
    fn lsp_ready(&mut self) -> Option<(PathBuf, usize, usize)> {
        if !self.docs[self.cur].lsp {
            self.say("no language server for this file");
            return None;
        }
        self.edit_at = Instant::now() - Duration::from_secs(1);
        self.lsp_sync();
        self.lsp_pos()
    }

    pub(crate) fn lsp_references(&mut self) {
        let Some((path, line, col)) = self.lsp_ready() else { return };
        if let Some(c) = self.client_for(self.cur) {
            c.references(&path, line, col);
        }
    }

    pub(crate) fn lsp_symbols(&mut self) {
        let Some((path, _, _)) = self.lsp_ready() else { return };
        if let Some(c) = self.client_for(self.cur) {
            c.symbols(&path);
        }
    }

    pub(crate) fn lsp_rename(&mut self, name: &str) {
        if name.trim().is_empty() {
            return;
        }
        let Some((path, line, col)) = self.lsp_ready() else { return };
        if let Some(c) = self.client_for(self.cur) {
            c.rename(&path, line, col, name.trim());
        }
    }

    pub(crate) fn lsp_format(&mut self) {
        let Some((path, _, _)) = self.lsp_ready() else { return };
        if let Some(c) = self.client_for(self.cur) {
            c.format(&path, tab());
        }
    }

    /// vim.lsp.buf.format(): formats and waits (up to `ms`), so a BufWritePre one writes the result.
    pub(crate) fn lsp_format_wait(&mut self, ms: u64) {
        self.lsp_sync();
        let Some((path, _, _)) = self.lsp_ready() else { return };
        let Some(c) = self.client_for(self.cur) else { return };
        c.format(&path, tab());
        let end = Instant::now() + Duration::from_millis(ms);
        while Instant::now() < end {
            for ev in self.lsp.events() {
                let edits = matches!(ev, Event::Edits(_));
                self.lsp_event(ev);
                if edits {
                    return;
                }
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        self.say("format: no answer in time");
    }

    /// The word under the cursor (what a rename starts from).
    pub(crate) fn word_at_cursor(&self) -> String {
        let d = &self.docs[self.cur];
        let c = d.ed.cursor();
        let text: Vec<char> = d.ed.buf.line(c.line).chars().collect();
        let (mut a, mut b) = (c.col.min(text.len()), c.col.min(text.len()));
        while a > 0 && ident(text[a - 1]) {
            a -= 1;
        }
        while b < text.len() && ident(text[b]) {
            b += 1;
        }
        text[a..b].iter().collect()
    }

    pub(crate) fn show_locations(&mut self, title: &str, locs: Vec<zett_lsp::Location>) {
        if locs.is_empty() {
            return self.say("nothing found");
        }
        let root = format!("{}/", self.root.display());
        let rows: Vec<Cand> = locs
            .iter()
            .map(|l| {
                let text = std::fs::read_to_string(&l.path).ok().and_then(|t| t.lines().nth(l.line).map(|s| s.trim().to_string())).unwrap_or_default();
                let p = l.path.display().to_string();
                Cand::plain(format!("{}:{}: {text}", p.strip_prefix(&root).unwrap_or(&p), l.line + 1))
            })
            .collect();
        self.pick_list(Pick::Diagnostics, title, rows);
    }

    pub(crate) fn show_symbols(&mut self, syms: Vec<(String, u8, usize, usize, usize)>) {
        if syms.is_empty() {
            return self.say("no symbols");
        }
        let kind = |k: u8| match k {
            5 => "class",
            6 => "method",
            8 => "field",
            9 => "constructor",
            10 => "enum",
            11 => "interface",
            12 => "function",
            13 => "variable",
            14 => "constant",
            22 => "variant",
            23 => "struct",
            26 => "type",
            2 => "module",
            _ => "",
        };
        let rows = syms.iter().map(|(n, k, line, _, depth)| Cand::noted(format!("{:>5}│ {}{n}", line + 1, "  ".repeat(*depth)), kind(*k))).collect();
        self.pick_list(Pick::Lines, "Symbols", rows);
    }

    /// A rename or a format: open buffers change in place (one undo step each); files
    /// not open yet are opened, changed and left unsaved to look at (:wa saves).
    pub(crate) fn apply_workspace(&mut self, edits: Vec<(PathBuf, Vec<(usize, usize, usize, usize, String)>)>) {
        let utf16 = self.client_for(self.cur).map(|c| c.utf16()).unwrap_or(true);
        if edits.iter().all(|(_, l)| l.is_empty()) {
            return self.say(if self.lsp_status.is_empty() { "nothing to change" } else { "nothing changed: the language server is still loading" });
        }
        let back = self.cur;
        let mut files = 0;
        for (path, list) in edits {
            if list.is_empty() {
                continue;
            }
            let di = match self.docs.iter().position(|d| d.abs_path().as_deref() == Some(path.as_path())) {
                Some(i) => i,
                None => {
                    self.open(&path);
                    self.cur
                }
            };
            let d = &mut self.docs[di];
            let conv = |line: usize, col: usize| {
                let t = d.ed.buf.line(line.min(d.ed.buf.len_lines().saturating_sub(1)));
                Pos { line, col: if utf16 { zett_lsp::from_utf16(&t, col) } else { col } }
            };
            let edits: Vec<(Pos, Pos, String)> = list.into_iter().map(|(l1, c1, l2, c2, t)| (conv(l1, c1), conv(l2, c2), t)).collect();
            let before = d.ed.cursor().line;
            d.ed.apply_edits(edits);
            d.states.clear();
            d.touched(before);
            files += 1;
        }
        self.show_doc(back.min(self.docs.len() - 1));
        self.edit_at = Instant::now();
        if files > 1 {
            self.say(format!("changed {files} files (unsaved: :wa saves them)"));
        }
    }
}

impl App {
    /// Neovim 0.12's :lsp: enable / disable / restart / stop a server by its command name (the
    /// current buffer's without one; :lsp enable alone, all of them), and :lsp alone the running ones.
    pub(crate) fn lsp_command(&mut self, arg: &str) {
        let mut w = arg.split_whitespace();
        let sub = w.next().unwrap_or("");
        let given = w.next().map(str::to_string);
        let here = self.docs[self.cur].abs_path().and_then(|p| zett_lsp::Servers::name_for(&p));
        let name = given.clone().or(here);
        // the buffers a server serves (all with None): their diagnostics go, they connect again
        let reopen = |app: &mut App, name: Option<&str>, connect: bool| {
            for i in 0..app.docs.len() {
                let theirs = app.docs[i].abs_path().and_then(|p| zett_lsp::Servers::name_for(&p));
                if theirs.is_some() && (name.is_none() || theirs.as_deref() == name) {
                    app.docs[i].lsp = false;
                    app.docs[i].diags.clear();
                    if connect {
                        app.lsp_open(i);
                    }
                }
            }
        };
        match sub {
            "" | "status" => {
                let rows: Vec<(String, String)> = self.lsp.running().into_iter().map(|(root, cmd)| (cmd, root.display().to_string())).chain(self.lsp.disabled.iter().map(|d| (d.clone(), "disabled".to_string()))).collect();
                if rows.is_empty() {
                    return self.say("no language servers running");
                }
                self.popup = Some(Popup::Help("language servers".into(), rows));
            }
            "enable" => {
                match &given {
                    Some(n) => {
                        self.lsp.disabled.remove(n);
                    }
                    None => self.lsp.disabled.clear(),
                }
                reopen(self, given.as_deref(), true);
                self.say(format!("enabled {}", given.as_deref().unwrap_or("every server")));
            }
            "disable" | "stop" | "restart" => {
                let Some(n) = name else { return self.say("no language server for this buffer") };
                if sub == "disable" {
                    self.lsp.disabled.insert(n.clone());
                }
                let stopped = self.lsp.stop(Some(&n));
                reopen(self, Some(&n), sub == "restart");
                self.say(match sub {
                    "restart" => format!("restarted {n}"),
                    "disable" => format!("disabled {n}"),
                    _ if stopped == 0 => format!("{n} was not running"),
                    _ => format!("stopped {n}"),
                });
            }
            other => self.say(format!("E475: Invalid argument: {other} (enable, disable, restart, stop)")),
        }
    }
}

#[cfg(test)]
mod lsp_command_tests {
    #[test]
    fn disable_keeps_a_server_from_starting_and_enable_lets_it() {
        let dir = std::env::temp_dir().join(format!("zett-lspcmd-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let f = dir.join("x.rs");
        std::fs::write(&f, "fn main() {}\n").unwrap();
        let mut app = crate::App::new(Vec::new());
        app.command("lsp disable rust-analyzer");
        assert!(app.lsp.disabled.contains("rust-analyzer"));
        app.open(&f);
        assert!(app.client_for(app.cur).is_none(), "disabled: not started for a Rust file");
        assert!(app.lsp.running().is_empty());
        app.lsp.disabled.clear();
        app.command("lsp nonsense");
        assert!(app.msg.as_ref().is_some_and(|m| m.0.starts_with("E475")));
        let _ = std::fs::remove_dir_all(&dir);
    }
}

/// zett.opt.lsp from init.lua to the client: { path = { folders with servers }, servers = {
/// ext = { cmd = { … }, lang = "id" } } } (servers for more file types, or other ones).
pub(crate) fn apply_config(engine: &mut zett_script::Engine) {
    let code = r#"(function()
  local l = zett.opt.lsp or {}
  local home = os.getenv("HOME") or ""
  local out = {}
  for _, d in ipairs(l.path or {}) do out[#out + 1] = "dir\t" .. tostring(d):gsub("^~", home) end
  for ext, s in pairs(l.servers or {}) do out[#out + 1] = "srv\t" .. ext .. "\t" .. (s.lang or ext) .. "\t" .. table.concat(s.cmd or {}, "\31") end
  return table.concat(out, "\30")
end)()"#;
    let Ok((text, _)) = engine.eval(code) else { return };
    let (mut servers, mut dirs) = (Vec::new(), Vec::new());
    for row in text.split('\u{1e}') {
        let f: Vec<&str> = row.split('\t').collect();
        match f.as_slice() {
            ["dir", d] => dirs.push(std::path::PathBuf::from(d)),
            ["srv", ext, lang, cmd] if !cmd.is_empty() => servers.push((ext.to_string(), cmd.split('\u{1f}').map(String::from).collect(), lang.to_string())),
            _ => {}
        }
    }
    zett_lsp::configure(servers, dirs);
}

#[cfg(test)]
mod lsp_live_tests {
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::time::{Duration, Instant};

    // a real rust-analyzer: `cargo test -p zett-tui lsp_live -- --ignored`
    fn project(src: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("zett-lsplive-{}-{}", std::process::id(), src.len()));
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(dir.join("Cargo.toml"), "[package]\nname = \"t\"\nversion = \"0.1.0\"\nedition = \"2021\"\n").unwrap();
        std::fs::write(dir.join("src/main.rs"), src).unwrap();
        dir
    }

    fn wait(app: &mut crate::App, secs: u64, done: impl Fn(&crate::App) -> bool) -> bool {
        let end = Instant::now() + Duration::from_secs(secs);
        while Instant::now() < end {
            app.lsp_tick();
            if done(app) {
                return true;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        false
    }

    #[test]
    #[ignore]
    fn completion_pops_up_while_typing() {
        let dir = project("fn main() {\n    let v: Vec<u8> = Vec::new();\n    \n}\n");
        let mut app = crate::App::new(Vec::new());
        app.open(&dir.join("src/main.rs"));
        // the server indexes first
        std::thread::sleep(Duration::from_secs(8));
        app.lsp_tick();
        for k in [KeyCode::Char('3'), KeyCode::Char('G'), KeyCode::Char('A')] {
            app.key(KeyEvent::new(k, KeyModifiers::NONE));
        }
        for c in "v.pu".chars() {
            app.key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
        }
        let ok = wait(&mut app, 20, |a| a.comp.is_some());
        let shown: Vec<String> = app.comp.as_ref().map(|c| c.shown.iter().map(|&i| c.items[i].label.clone()).collect()).unwrap_or_default();
        let _ = std::fs::remove_dir_all(&dir);
        assert!(ok, "no popup came up");
        assert!(shown.iter().any(|l| l.starts_with("push")), "push not offered: {shown:?}");
    }

    #[test]
    #[ignore]
    fn format_rewrites_the_buffer() {
        let dir = project("fn main(){let x=1;println!(\"{}\",x);}\n");
        let mut app = crate::App::new(Vec::new());
        app.open(&dir.join("src/main.rs"));
        std::thread::sleep(Duration::from_secs(5));
        app.lsp_tick();
        app.lsp_format();
        let ok = wait(&mut app, 20, |a| a.docs[a.cur].text().contains("let x = 1;"));
        let text = app.docs[app.cur].text();
        let _ = std::fs::remove_dir_all(&dir);
        assert!(ok, "not formatted: {text:?}");
    }
}

#[cfg(test)]
mod lsp_format_on_save_tests {
    // Neovim's way: a BufWritePre autocmd calling vim.lsp.buf.format(); the file on disk is formatted
    #[test]
    #[ignore]
    fn buf_write_pre_format_lands_in_the_file() {
        let dir = std::env::temp_dir().join(format!("zett-fmtsave-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(dir.join("Cargo.toml"), "[package]\nname = \"t\"\nversion = \"0.1.0\"\nedition = \"2021\"\n").unwrap();
        let f = dir.join("src/main.rs");
        std::fs::write(&f, "fn main(){let x=1;println!(\"{}\",x);}\n").unwrap();
        let mut app = crate::App::new(Vec::new());
        app.open(&f);
        std::thread::sleep(std::time::Duration::from_secs(5));
        app.lsp_tick();
        app.command("lua vim.api.nvim_create_autocmd('BufWritePre', { callback = function() vim.lsp.buf.format({ timeout_ms = 5000 }) end })");
        app.command("w");
        let disk = std::fs::read_to_string(&f).unwrap();
        let _ = std::fs::remove_dir_all(&dir);
        assert!(disk.contains("let x = 1;"), "written unformatted: {disk:?} (msg {:?})", app.msg);
    }
}

#[cfg(test)]
mod last_file_tests {
    // the config's VimEnter: no files given → the newest recent file under the folder zett started in
    #[test]
    fn vim_enter_opens_the_last_file_of_this_folder() {
        let dir = std::env::temp_dir().join(format!("zett-lastfile-{}", std::process::id()));
        let proj = dir.join("proj");
        std::fs::create_dir_all(&proj).unwrap();
        std::fs::write(proj.join("b.rs"), "fn b() {}\n").unwrap();
        std::fs::write(dir.join("elsewhere.txt"), "x\n").unwrap();
        let mut app = crate::App::new(Vec::new());
        assert_eq!(app.engine.eval("return vim.fn.argc()").map(|r| r.0).as_deref(), Ok("0"));
        std::fs::create_dir_all(dir.join("state/zett")).unwrap();
        std::fs::write(dir.join("state/zett/recent"), format!("{}\n{}\n", dir.join("elsewhere.txt").display(), proj.join("b.rs").display())).unwrap();
        app.command(&format!("lua vim.fn.chdir({:?}); vim.env.XDG_STATE_HOME = nil", proj.display().to_string()));
        app.command(&format!(
            "lua _G.__state = {:?}; vim.api.nvim_create_autocmd('VimEnter', {{ callback = function() \
             if vim.fn.argc() > 0 then return end \
             local cwd = vim.fn.getcwd() .. '/' \
             local f = io.open(_G.__state) local list = {{}} for l in f:lines() do list[#list+1] = l end f:close() \
             for _, p in ipairs(list) do if p:sub(1, #cwd) == cwd then vim.cmd.edit(p) return end end end }})",
            dir.join("state/zett/recent").display().to_string()
        ));
        app.fire_autocmds("vimenter", None);
        let name = app.docs[app.cur].abs_path().map(|p| p.display().to_string()).unwrap_or_default();
        let _ = std::fs::remove_dir_all(&dir);
        assert!(name.ends_with("proj/b.rs"), "opened {name:?} (msg {:?})", app.msg);
    }
}

#[cfg(test)]
mod completion_live_tests {
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::time::{Duration, Instant};

    fn project(src: &str) -> (std::path::PathBuf, crate::App) {
        let dir = std::env::temp_dir().join(format!("zett-complive-{}-{}", std::process::id(), src.len()));
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(dir.join("Cargo.toml"), "[package]\nname = \"t\"\nversion = \"0.1.0\"\nedition = \"2021\"\n").unwrap();
        std::fs::write(dir.join("src/main.rs"), src).unwrap();
        let mut app = crate::App::new(Vec::new());
        app.open(&dir.join("src/main.rs"));
        std::thread::sleep(Duration::from_secs(8));
        app.lsp_tick();
        (dir, app)
    }
    fn keys(app: &mut crate::App, s: &str) {
        for c in s.chars() {
            app.key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
        }
    }
    fn popup_with(app: &mut crate::App, label: &str) -> bool {
        let end = Instant::now() + Duration::from_secs(20);
        while Instant::now() < end {
            app.lsp_tick();
            if let Some(c) = app.comp.as_mut() {
                if let Some(p) = c.shown.iter().position(|&i| c.items[i].label.starts_with(label)) {
                    c.sel = p;
                    return true;
                }
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        false
    }

    #[test]
    #[ignore]
    fn a_method_snippet_is_filled_in_with_tab() {
        let (dir, mut app) = project("fn main() {\n    let mut v: Vec<u8> = Vec::new();\n    \n}\n");
        for k in ['3', 'G', 'A'] {
            app.key(KeyEvent::new(KeyCode::Char(k), KeyModifiers::NONE));
        }
        keys(&mut app, "v.pus");
        assert!(popup_with(&mut app, "push"), "no push offered");
        app.key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        keys(&mut app, "7");
        // rust-analyzer's snippet ends `);$0`: Tab goes past the `;` it puts in itself
        app.key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        keys(&mut app, " ");
        let line = app.docs[app.cur].ed.buf.line(2);
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(line.trim_end(), "    v.push(7);");
        assert!(line.trim_end_matches('\n').ends_with("; "), "Tab did not go past the end: {line:?}");
    }

    #[test]
    #[ignore]
    fn an_item_brings_its_import() {
        let (dir, mut app) = project("fn main() {\n    let m: \n}\n");
        app.key(KeyEvent::new(KeyCode::Char('2'), KeyModifiers::NONE));
        app.key(KeyEvent::new(KeyCode::Char('G'), KeyModifiers::NONE));
        app.key(KeyEvent::new(KeyCode::Char('A'), KeyModifiers::NONE));
        keys(&mut app, "HashMa");
        assert!(popup_with(&mut app, "HashMap"), "no HashMap offered");
        app.key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        // the item is `HashMap<$0>`: the cursor waits between the brackets
        keys(&mut app, "u8, u8");
        let text = app.docs[app.cur].ed.text();
        let _ = std::fs::remove_dir_all(&dir);
        assert!(text.contains("use std::collections::HashMap;"), "no import: {text}");
        assert!(text.contains("let m: HashMap<u8, u8>"), "cursor lost after the import: {text}");
    }
}

#[cfg(test)]
mod completion_tests {
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn item(label: &str) -> zett_lsp::Item {
        zett_lsp::Item { label: label.into(), insert: label.into(), filter: label.into(), sort: label.into(), resolved: true, ..Default::default() }
    }

    #[test]
    fn late_answers_backspace_and_moving_away() {
        let mut app = crate::App::new(Vec::new());
        app.docs[app.cur].ed = zett_edit::Editor::from_str("one\npr\n");
        app.docs[app.cur].lsp = true;
        for k in ['j', 'A'] {
            app.key(KeyEvent::new(KeyCode::Char(k), KeyModifiers::NONE));
        }
        // an answer for another line is dropped
        app.comp_arrived(0, vec![item("print")], false);
        assert!(app.comp.is_none());
        app.comp_arrived(1, vec![item("print"), item("parse"), item("prompt")], false);
        let shown = |app: &crate::App| app.comp.as_ref().map(|c| c.shown.iter().map(|&i| c.items[i].label.clone()).collect::<Vec<_>>()).unwrap_or_default();
        assert_eq!(shown(&app), vec!["print", "prompt"]);
        app.key(KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE));
        assert_eq!(shown(&app), vec!["print"]);
        // Backspace widens it again
        app.key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
        assert_eq!(shown(&app), vec!["print", "prompt"]);
        // back before where the word started: gone; an arrow: gone
        app.key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
        app.key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
        assert!(app.comp.is_none());
        assert!(app.comp_due.is_none() || app.docs[app.cur].ed.cursor().col == 0);
        app.key(KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE));
        app.comp_arrived(1, vec![item("print")], false);
        assert!(app.comp.is_some());
        app.key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE));
        assert!(app.comp.is_none());
    }
}
