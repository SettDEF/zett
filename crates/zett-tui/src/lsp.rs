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
                Event::Completion(line, items) => self.comp_arrived(line, items),
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
                Event::Exited(tag) => {
                    // only that client: a restarted server's new one has the same name
                    let had = self.lsp.clients.len();
                    self.lsp.clients.retain(|_, c| c.tag != tag);
                    if self.lsp.clients.len() != had {
                        self.say(format!("{} stopped", tag.split('#').next().unwrap_or(&tag)));
                    }
                }
            }
        }
        self.comp_tick();
        // text changed since the server last saw it, and the typing paused
        if self.edit_at.elapsed() < Duration::from_millis(250) {
            return;
        }
        self.lsp_sync();
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
}

fn ident(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

impl App {
    /// After a key in insert mode: refine the popup, ask for one, or close it.
    pub(crate) fn comp_after_key(&mut self, typed: Option<char>) {
        // keyword completion ends with the next key typed (the word stays as it is)
        if self.comp.as_ref().is_some_and(|c| c.keyword.is_some()) {
            self.comp = None;
            return;
        }
        if !self.docs[self.cur].lsp || self.docs[self.cur].ed.mode() != Mode::Insert {
            self.comp = None;
            return;
        }
        // autocomplete: the menu comes by itself after autocompletedelay ms (else C-n / C-x C-o ask)
        let auto = self.flag(self.cur, "autocomplete");
        let delay = Duration::from_millis(self.setting(self.cur, "autocompletedelay").parse().unwrap_or(120));
        match typed {
            Some(c) if ident(c) => {
                if self.comp.is_some() {
                    self.comp_filter();
                } else if auto {
                    self.comp_due = Some(Instant::now() + delay);
                }
            }
            Some('.') | Some(':') => {
                self.comp = None;
                self.comp_due = auto.then(|| Instant::now() + delay / 2);
            }
            None if self.comp.is_some() => self.comp_filter(),
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
        if let Some(c) = self.client_for(self.cur) {
            c.completion(&path, line, col);
        }
    }

    pub(crate) fn comp_arrived(&mut self, line: usize, items: Vec<zett_lsp::Item>) {
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
        self.comp = Some(Comp { items, shown: Vec::new(), sel: 0, line, start, keyword: None });
        self.comp_filter();
    }

    fn comp_filter(&mut self) {
        let d = &self.docs[self.cur];
        let c = d.ed.cursor();
        let Some(comp) = self.comp.as_mut() else { return };
        if c.line != comp.line || c.col < comp.start {
            self.comp = None;
            return;
        }
        let prefix: String = d.ed.buf.line(c.line).chars().skip(comp.start).take(c.col - comp.start).collect::<String>().to_lowercase();
        let q: Vec<char> = prefix.chars().collect();
        let mut scored: Vec<(bool, String, usize)> = comp
            .items
            .iter()
            .enumerate()
            .filter_map(|(i, it)| {
                let l = it.label.to_lowercase();
                let starts = l.starts_with(&prefix);
                (starts || fuzzy(&q, &l).is_some()).then(|| (!starts, it.sort.clone(), i))
            })
            .collect();
        scored.sort();
        comp.shown = scored.into_iter().take(60).map(|s| s.2).collect();
        comp.sel = comp.sel.min(comp.shown.len().saturating_sub(1));
        if comp.shown.is_empty() {
            self.comp = None;
        }
    }

    /// Enter / Tab on the popup: the typed part becomes the item.
    pub(crate) fn comp_accept(&mut self) {
        let Some(comp) = self.comp.take() else { return };
        let Some(&i) = comp.shown.get(comp.sel) else { return };
        let insert = comp.items[i].insert.clone();
        let typed = self.docs[self.cur].ed.cursor().col.saturating_sub(comp.start);
        for _ in 0..typed {
            self.edit_key(Key::code(Code::Backspace));
        }
        for ch in insert.chars() {
            self.edit_key(if ch == '\n' { Key::code(Code::Enter) } else { Key::ch(ch) });
        }
        self.comp_due = None;
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
