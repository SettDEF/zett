//! Plugin panels at the sides (explorer, undotree, …), and the buffer lent to Lua while it runs.

use super::*;

/// How a highlight group a plugin names is drawn (terminal colours, so the theme follows).
pub(crate) fn group_style(g: &str) -> Style {
    let fg = |n: u8| Style::default().fg(Color::Indexed(n));
    match g {
        "Title" => fg(4).add_modifier(Modifier::BOLD),
        "Directory" => fg(4),
        "Current" => fg(3).add_modifier(Modifier::BOLD),
        "Accent" => fg(6),
        "Muted" | "Comment" => fg(8),
        "Added" | "GitAdded" => fg(2),
        "Removed" | "GitDeleted" | "Error" | "GitConflict" => fg(1),
        "Changed" | "GitModified" | "GitRenamed" | "Warn" => fg(3),
        "GitUntracked" => fg(5),
        "Bold" => Style::default().add_modifier(Modifier::BOLD),
        _ => Style::default(),
    }
}

impl App {
    /// Runs something in the engine with the current buffer lent to it (`zett.buf`); an edit
    /// a script made is noticed afterwards.
    pub(crate) fn with_lua<R>(&mut self, f: impl FnOnce(&mut Engine) -> R) -> R {
        let cur = self.cur.min(self.docs.len() - 1);
        let path = self.docs[cur].abs_path().map(|p| p.display().to_string()).unwrap_or_default();
        let root = project_of(&self.root).unwrap_or(self.root.clone());
        let root = std::fs::canonicalize(&root).unwrap_or(root);
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
        let feed = self.with_lua(|e| e.feed("n", &seq));
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
            let ps: Vec<&PanelView> = panels.iter().filter(|p| p.side == side).collect();
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
            let bar: Vec<Line> = (0..sep.height).map(|_| Line::from(Span::styled("│", Style::default().fg(Color::Indexed(8))))).collect();
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
            let st = if p.focused { group_style("Title") } else { group_style("Title").remove_modifier(Modifier::BOLD).fg(Color::Indexed(8)) };
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
                    st = st.bg(Color::Indexed(0)).add_modifier(Modifier::BOLD);
                }
                spans.push(Span::styled(t, st));
            }
            if sel {
                spans[0] = Span::styled(" ", Style::default().bg(Color::Indexed(0)));
                spans.push(Span::styled(" ".repeat((a.width as usize).saturating_sub(used)), Style::default().bg(Color::Indexed(0))));
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
