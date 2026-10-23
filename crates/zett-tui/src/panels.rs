//! Plugin panels at the sides (explorer, undotree, …), and the buffer lent to Lua while it runs.

use super::*;

/// How a highlight group a plugin names is drawn (terminal colours, so the theme follows).
pub(crate) fn group_style(g: &str) -> Style {
    hl(g)
}

impl App {
    /// Runs something in the engine with the current buffer lent to it (`zett.buf`); an edit
    /// a script made is noticed afterwards.
    pub(crate) fn with_lua<R>(&mut self, f: impl FnOnce(&mut Engine) -> R) -> R {
        let cur = self.cur.min(self.docs.len() - 1);
        let path = self.docs[cur].abs_path().map(|p| p.display().to_string()).unwrap_or_default();
        let root = project_of(&self.root).unwrap_or(self.root.clone());
        let root = std::fs::canonicalize(&root).unwrap_or(root);
        // every buffer in its place ("" for one without a file), so a buffer's number is its place
        // here (vim.api's handles), and the windows as last drawn (nvim_list_wins, nvim_win_get_*)
        let open: Vec<String> = self.docs.iter().map(|d| d.abs_path().map(|p| p.display().to_string()).unwrap_or_default()).collect();
        self.engine.set_buffers(open);
        let wins: Vec<(usize, usize, u16, u16, u16, u16)> = self.win_rects.iter().filter_map(|&(id, r)| self.wins.get(id).and_then(|w| w.as_ref()).map(|w| (id + 1, w.doc + 1, r.width, r.height, r.y, r.x))).collect();
        self.engine.set_layout(cur + 1, wins, self.focus + 1);
        let tops = self.win_rects.iter().filter_map(|&(id, _)| self.wins.get(id).and_then(|w| w.as_ref()).map(|w| w.top)).collect();
        self.engine.set_tops(tops);
        // the : line and (when it grew) the command history, for getcmdline / histget
        let line = self.cmdline.clone().map(|l| { let n = l.chars().count(); (l, n) });
        let hist = (self.cmd_hist.len() != self.hist_sent).then(|| self.cmd_hist.clone());
        self.hist_sent = self.cmd_hist.len();
        self.engine.set_cmdline(line, hist);
        // the quickfix list for getqflist, given again only when it changed
        let qkey = self.qf.current().map(|l| (l.title.clone(), l.idx, l.items.len(), self.qf.lists.len(), self.qf.cur));
        if qkey != self.qf_sent {
            let (title, idx, items) = self.qf.current().map(|l| (l.title.clone(), l.idx, l.items.iter().map(|i| (i.file.clone(), i.line, i.col, i.kind, i.nr, i.text.clone(), i.valid)).collect())).unwrap_or_default();
            self.engine.set_qf(title, idx, items);
            self.qf_sent = qkey;
        }
        let rev = self.docs[cur].ed.buf.rev();
        let r = self.engine.lend_buffer(&mut self.docs[cur].ed, &path, &root, f);
        if self.docs[cur].ed.buf.rev() != rev {
            let d = &mut self.docs[cur];
            d.states.clear();
            d.touched(0);
            self.edit_at = Instant::now();
        }
        r
    }

    /// Effects, then a question a script asked goes to the input line.
    pub(crate) fn after_lua(&mut self, fx: Vec<Effect>) {
        self.effects(fx);
        if let Some((prompt, default)) = self.engine.take_input() {
            self.prompt = Some(prompt);
            self.cmdline = Some(default);
        }
    }

    pub(crate) fn emit(&mut self, event: &str) {
        // a file read: its type's ftplugin/ and indent/ (Neovim's `filetype plugin indent on`)
        if event == "buf_read" {
            // plugin/ first, as Neovim loads it before the first file (that file is read before
            // the first frame here)
            if self.flag(self.cur, "loadplugins") {
                self.load_runtime_plugins();
            }
            let ft = self.setting(self.cur, "filetype");
            self.load_ftplugins(&ft);
        }
        // Vim's autocommands for the same moment (mapcmds.rs)
        for e in crate::mapcmds::vim_events(event) {
            self.fire_autocmds(e, None);
        }
        if !self.engine.has_hooks(event) {
            return;
        }
        let path = self.docs[self.cur.min(self.docs.len() - 1)].abs_path().map(|p| p.display().to_string()).unwrap_or_default();
        let fx = self.with_lua(|e| e.emit_path(event, &path));
        self.after_lua(fx);
    }

    /// Once per turn of the loop: the text changed (undotree follows), another buffer is shown.
    pub(crate) fn watch_buffer(&mut self) {
        let cur = self.cur.min(self.docs.len() - 1);
        let key = (cur, self.docs[cur].name.clone());
        if self.seen_doc.as_ref() != Some(&key) {
            self.seen_doc = Some(key);
            self.seen_rev = self.docs[cur].ed.buf.rev();
            self.emit("buf_enter");
            return;
        }
        let rev = self.docs[cur].ed.buf.rev();
        if rev != self.seen_rev && self.docs[cur].ed.mode() != Mode::Insert {
            self.seen_rev = rev;
            self.emit("text_changed");
        }
    }

    /// A key while a panel has the keyboard: true when the panel (or a mapping) took it.
    pub(crate) fn panel_key(&mut self, k: KeyEvent) -> bool {
        if self.engine.focused_panel().is_none() {
            return false;
        }
        let tok = token(&k);
        if tok.is_empty() {
            return true;
        }
        // a key sequence under way (SPC …) goes on in the keymaps
        if self.keyseq.is_empty() {
            let page = self.panel_rects.first().map_or(20, |(_, r)| r.height as usize);
            if let Some(fx) = self.with_lua(|e| e.panel_key(&tok, page)) {
                self.after_lua(fx);
                return true;
            }
        }
        // anything else: the global keymaps (SPC, :), never Vim's editing
        let mut seq = std::mem::take(&mut self.keyseq);
        seq.push(tok);
        let mode = self.keymode();
        let feed = self.with_lua(|e| e.feed(mode, &seq));
        match feed {
            Feed::None => self.menu.clear(),
            f => self.after_feed(f, seq),
        }
        true
    }

    /// Lays the panels out at the sides of `body` and draws them; what is left is for windows.
    pub(crate) fn draw_panels(&mut self, f: &mut Frame, body: Rect) -> Rect {
        let panels = self.engine.panels();
        self.panel_rects.clear();
        if panels.is_empty() {
            return body;
        }
        let mut rest = body;
        for side in ["left", "right"] {
            let ps: Vec<&PanelView> = panels.iter().filter(|p| p.side == side && !self.sides_hidden).collect();
            if ps.is_empty() || rest.width < 50 {
                continue;
            }
            let w = ps.iter().map(|p| p.width).max().unwrap_or(30).min(rest.width / 2);
            let (col, sep, other) = if side == "left" {
                let [a, s, b] = Layout::horizontal([Constraint::Length(w), Constraint::Length(1), Constraint::Min(20)]).areas(rest);
                (a, s, b)
            } else {
                let [b, s, a] = Layout::horizontal([Constraint::Min(20), Constraint::Length(1), Constraint::Length(w)]).areas(rest);
                (a, s, b)
            };
            rest = other;
            let bar: Vec<Line> = (0..sep.height).map(|_| Line::from(Span::styled("│", hl("WinSeparator")))).collect();
            f.render_widget(Paragraph::new(bar), sep);
            // fixed heights first, the rest shared
            let fixed: u16 = ps.iter().map(|p| p.height.min(col.height / 2)).sum();
            let flex = ps.iter().filter(|p| p.height == 0).count().max(1) as u16;
            let mut y = col.y;
            for (i, p) in ps.iter().enumerate() {
                let left = col.bottom().saturating_sub(y);
                let h = if p.height > 0 && ps.iter().any(|q| q.height == 0) { p.height.min(col.height / 2).min(left) } else if p.height > 0 { p.height.min(left) } else { (col.height.saturating_sub(fixed) / flex).max(3).min(left) };
                let h = if i == ps.len() - 1 { left } else { h };
                let r = Rect { x: col.x, y, width: col.width, height: h };
                y += h;
                self.draw_panel(f, r, p);
                self.panel_rects.push((p.id.clone(), r));
            }
        }
        rest
    }

    fn draw_panel(&mut self, f: &mut Frame, a: Rect, p: &PanelView) {
        if a.height == 0 {
            return;
        }
        let mut lines = Vec::new();
        let mut body_h = a.height as usize;
        if !p.title.is_empty() {
            let st = hl(if p.focused { "Title" } else { "TitleNC" });
            lines.push(Line::from(Span::styled(format!(" {}", p.title), st)));
            body_h -= 1;
        }
        let top = self.panel_tops.entry(p.id.clone()).or_insert(0);
        if p.cursor < *top {
            *top = p.cursor;
        } else if p.cursor >= *top + body_h.max(1) {
            *top = p.cursor + 1 - body_h.max(1);
        }
        let top = *top;
        for (i, l) in p.lines.iter().enumerate().skip(top).take(body_h) {
            let sel = i == p.cursor && p.focused;
            let mut spans = vec![Span::raw(" ")];
            let mut used = 1;
            for (text, g) in l {
                let room = (a.width as usize).saturating_sub(used);
                if room == 0 {
                    break;
                }
                let t: String = text.chars().take(room).collect();
                used += t.chars().count();
                let mut st = group_style(g);
                if sel {
                    st = st.patch(hl("PanelSel"));
                }
                spans.push(Span::styled(t, st));
            }
            if sel {
                spans[0] = Span::styled(" ", hl("PanelSel"));
                spans.push(Span::styled(" ".repeat((a.width as usize).saturating_sub(used)), hl("PanelSel")));
            }
            lines.push(Line::from(spans));
        }
        f.render_widget(Paragraph::new(lines), a);
    }

    /// A mouse event on a panel: true when it was one. A second click on a line is Enter.
    pub(crate) fn panel_mouse(&mut self, m: &ratatui::crossterm::event::MouseEvent) -> bool {
        let Some((id, r)) = self.panel_rects.iter().find(|(_, r)| m.column >= r.x && m.column < r.right() && m.row >= r.y && m.row < r.bottom()).cloned() else {
            return false;
        };
        let titled = self.engine.panels().iter().find(|p| p.id == id).is_some_and(|p| !p.title.is_empty());
        let top = *self.panel_tops.get(&id).unwrap_or(&0);
        match m.kind {
            MouseEventKind::Down(_) => {
                let off = (m.row - r.y) as usize;
                if titled && off == 0 {
                    self.engine.focus_panel(Some(&id));
                    return true;
                }
                let row = top + off - titled as usize;
                let again = self.last_click.as_ref().is_some_and(|(t, i, r)| *i == id && *r == row && t.elapsed() < Duration::from_millis(400));
                let fx = self.with_lua(|e| e.panel_click(&id, row));
                self.after_lua(fx);
                self.last_click = Some((Instant::now(), id.clone(), row));
                if again {
                    self.last_click = None;
                    if let Some(fx) = self.with_lua(|e| e.panel_key("<CR>", 20)) {
                        self.after_lua(fx);
                    }
                }
            }
            MouseEventKind::ScrollDown | MouseEventKind::ScrollUp => {
                let n = self.engine.panels().iter().find(|p| p.id == id).map_or(0, |p| p.lines.len());
                let t = self.panel_tops.entry(id).or_insert(0);
                *t = if m.kind == MouseEventKind::ScrollDown { (*t + 3).min(n.saturating_sub(1)) } else { t.saturating_sub(3) };
            }
            _ => {}
        }
        true
    }

    /// `C-w h` at the left edge (or `l` at the right) moves into a panel there.
    pub(crate) fn focus_panel_at(&mut self, dir: &str) -> bool {
        let side = match dir {
            "left" => "left",
            "right" => "right",
            _ => return false,
        };
        let Some(p) = self.engine.panels().into_iter().find(|p| p.side == side) else { return false };
        self.engine.focus_panel(Some(&p.id));
        true
    }
}

impl App {
    /// Plugin floats in the corners of the editing area: lines right-aligned at the right
    /// corners, only their own cells covered (the text around stays visible).
    pub(crate) fn draw_floats(&mut self, f: &mut Frame, body: Rect) {
        for (_, anchor, lines) in self.engine.floats() {
            let h = (lines.len() as u16).min(body.height.saturating_sub(2));
            let right = anchor.ends_with('E');
            // above the bottom windows' modelines, below the top edge
            let top = if anchor.starts_with('S') { body.bottom().saturating_sub(h + 1) } else { body.y };
            for (i, l) in lines.iter().take(h as usize).enumerate() {
                let w: u16 = l.iter().map(|(t, _)| t.chars().count() as u16).sum::<u16>().min(body.width.saturating_sub(2));
                if w == 0 {
                    continue;
                }
                let x = if right { body.right().saturating_sub(w + 1) } else { body.x + 1 };
                let r = Rect { x, y: top + i as u16, width: w, height: 1 };
                let spans: Vec<Span> = l.iter().map(|(t, g)| Span::styled(t.clone(), hl(g))).collect();
                f.render_widget(Clear, r);
                f.render_widget(Paragraph::new(Line::from(spans)), r);
            }
        }
    }
}

impl App {
    /// Floating panels over the windows: centred (then moved), framed, the title on the frame.
    /// The layer with the keyboard stands out; one without it stays in view, dimmed.
    pub(crate) fn draw_float_panels(&mut self, f: &mut Frame, body: Rect) {
        for p in self.engine.panels().into_iter().filter(|p| p.side == "float") {
            let pct = |n: u16, of: u16| ((of as u32 * n.min(100) as u32) / 100) as u16;
            let w = pct(p.width.max(20), body.width).max(20).min(body.width);
            let h = pct(if p.height == 0 { 70 } else { p.height }, body.height).max(5).min(body.height);
            let x = (((body.width - w) / 2) as i32 + p.dx as i32).clamp(0, (body.width - w) as i32) as u16;
            let y = (((body.height - h) / 2) as i32 + p.dy as i32).clamp(0, (body.height - h) as i32) as u16;
            let mut outer = Rect { x: body.x + x, y: body.y + y, width: w, height: h };
            // placed as Neovim places a float (nvim_open_win): its size in cells inside the frame, at
            // row / col from the editor, the cursor or the focused window, kept on the screen
            if let Some((rel, row, col)) = &p.at {
                let (w, h) = ((p.width + 2).min(body.width), ((if p.height == 0 { p.lines.len().max(1) as u16 } else { p.height }) + 2).min(body.height));
                let (ox, oy) = match rel.as_str() {
                    "cursor" => (self.cursor_xy.0 as i32, self.cursor_xy.1 as i32),
                    "win" => self.win_rects.iter().find(|(id, _)| *id == self.focus).map_or((body.x as i32, body.y as i32), |(_, r)| (r.x as i32, r.y as i32)),
                    _ => (body.x as i32, body.y as i32),
                };
                let x = (ox + col).clamp(body.x as i32, (body.x + body.width - w) as i32) as u16;
                let y = (oy + row).clamp(body.y as i32, (body.y + body.height - h) as i32) as u16;
                outer = Rect { x, y, width: w, height: h };
            }
            crate::layer(f, outer);
            // a Neovim float (nvim_open_win): its own title only, none when it has none
            let title = if p.at.is_some() { if p.title.is_empty() { String::new() } else { format!(" {} ", p.title) } } else if p.focused { format!(" {} ", p.title) } else { format!(" {} · SPC w Tab: up ", p.title) };
            let block = Block::default()
                .borders(Borders::ALL)
                .border_set(crate::display::float_set())
                .border_style(hl(if p.focused { "FloatBorder" } else { "FloatBorderDim" }))
                .title(Span::styled(title, hl(if p.focused { "Title" } else { "TitleNC" })));
            let inner = block.inner(outer);
            f.render_widget(block, outer);
            // the lines as a side panel draws them, without its own title row
            let untitled = PanelView { title: String::new(), ..p.clone() };
            self.draw_panel(f, inner, &untitled);
            self.panel_rects.insert(0, (p.id.clone(), inner));
        }
    }

    /// `SPC w Tab`: the keyboard up to the floating panel, or down to the windows below it.
    pub(crate) fn layer_toggle(&mut self) {
        let floats: Vec<PanelView> = self.engine.panels().into_iter().filter(|p| p.side == "float").collect();
        if floats.is_empty() {
            return self.say("no floating panel (zett.panel.open{ side = \"float\" })");
        }
        match floats.iter().find(|p| p.focused) {
            Some(_) => self.engine.focus_panel(None),
            None => self.engine.focus_panel(Some(&floats[floats.len() - 1].id)),
        }
    }
}

/// The config's files: init.lua and every plugin's init.lua (and its lua/ folder's files).
fn config_files() -> Vec<std::path::PathBuf> {
    let dir = zett_script::config_dir();
    let mut v = vec![dir.join("init.lua")];
    let mut stack = vec![dir.join("lua"), dir.join("plugins")];
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else { continue };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| x == "lua") {
                v.push(p);
            }
        }
    }
    v.sort();
    v
}

impl App {
    /// The config changed on disk (saved here or anywhere): it is loaded again, as a
    /// running Neovim never does on its own.
    pub(crate) fn watch_config(&mut self) {
        if self.config_at.elapsed() < Duration::from_secs(2) {
            return;
        }
        self.config_at = Instant::now();
        let files = config_files();
        let stamp: Vec<Option<std::time::SystemTime>> = files.iter().map(|p| std::fs::metadata(p).and_then(|m| m.modified()).ok()).collect();
        if self.config_stamp.is_empty() {
            self.config_stamp = stamp;
            return;
        }
        if stamp != self.config_stamp {
            self.config_stamp = stamp;
            self.effects(vec![Effect::Ui("reload".into(), String::new())]);
        }
    }
}
