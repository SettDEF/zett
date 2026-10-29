//! The settings as a popup (:Settings, :options, SPC h s, customize): every setting with its value and what it does,
//! zett's own first, then Neovim's. Typing filters; ↑ ↓ move; Space or Enter turns a switch, Enter edits a number
//! or a text in place (Enter sets it, Esc leaves it); Esc closes. A change is :set's at once, and kept for the next
//! start in the config folder's settings.lua (loaded after init.lua; init.lua itself is never written).
use super::*;
use crate::settings::Kind;

#[derive(Default)]
pub(crate) struct SettingsUi {
    pub query: String,
    pub sel: usize,
    pub top: usize,
    /// a value being typed for the chosen setting
    pub editing: Option<String>,
    /// the list's place on screen last frame and where its value column starts (for the mouse)
    pub list: Option<(Rect, u16)>,
    /// the chosen row and since when (its description runs across after a moment, when it does not fit)
    pub shown_sel: usize,
    pub since: Option<Instant>,
    /// the chosen description is longer than its room (the main loop draws again soon while so)
    pub running: bool,
}

impl App {
    pub(crate) fn settings_open(&mut self) {
        self.settings_ui = Some(SettingsUi::default());
    }

    /// The settings the search matches (name, short names or what it does), in their order.
    fn settings_shown(&self) -> Vec<&'static crate::settings::Setting> {
        let q = self.settings_ui.as_ref().map(|s| s.query.to_lowercase()).unwrap_or_default();
        crate::settings::all().into_iter().filter(|s| q.is_empty() || s.name.contains(&q) || s.short.iter().any(|x| *x == q) || s.about.to_lowercase().contains(&q)).collect()
    }

    /// Its keys; true when it took the key (it takes them all while open).
    pub(crate) fn settings_key(&mut self, k: &KeyEvent) -> bool {
        if self.settings_ui.is_none() {
            return false;
        }
        let shown = self.settings_shown();
        let ui = self.settings_ui.as_mut().unwrap();
        // typing a value
        if let Some(v) = ui.editing.as_mut() {
            match k.code {
                KeyCode::Esc => ui.editing = None,
                KeyCode::Backspace => { v.pop(); }
                KeyCode::Char(c) => v.push(c),
                KeyCode::Enter => {
                    let v = ui.editing.take().unwrap_or_default();
                    if let Some(s) = shown.get(ui.sel) {
                        let set = format!("{}={}", s.name, v.replace(' ', "\\ "));
                        self.command(&format!("set {set}"));
                        self.settings_keep(s.name, &set);
                    }
                }
                _ => {}
            }
            return true;
        }
        match k.code {
            KeyCode::Esc => self.settings_ui = None,
            KeyCode::Down | KeyCode::Tab => ui.sel = (ui.sel + 1).min(shown.len().saturating_sub(1)),
            KeyCode::Up | KeyCode::BackTab => ui.sel = ui.sel.saturating_sub(1),
            KeyCode::PageDown => ui.sel = (ui.sel + 12).min(shown.len().saturating_sub(1)),
            KeyCode::PageUp => ui.sel = ui.sel.saturating_sub(12),
            KeyCode::Char('n') if k.modifiers.contains(KeyModifiers::CONTROL) => ui.sel = (ui.sel + 1).min(shown.len().saturating_sub(1)),
            KeyCode::Char('p') if k.modifiers.contains(KeyModifiers::CONTROL) => ui.sel = ui.sel.saturating_sub(1),
            KeyCode::Backspace => {
                ui.query.pop();
                ui.sel = 0;
            }
            KeyCode::Char(' ') | KeyCode::Enter => {
                let Some(s) = shown.get(ui.sel).copied() else { return true };
                if matches!(s.kind, Kind::Bool) {
                    let on = self.flag(self.cur, s.name);
                    let set = format!("{}{}", if on { "no" } else { "" }, s.name);
                    self.command(&format!("set {set}"));
                    self.settings_keep(s.name, &set);
                } else if k.code == KeyCode::Enter {
                    let now = self.setting(self.cur, s.name);
                    if let Some(ui) = self.settings_ui.as_mut() { ui.editing = Some(now) }
                } else if let Some(ui) = self.settings_ui.as_mut() {
                    ui.query.push(' ');
                }
            }
            KeyCode::Char(c) if !k.modifiers.contains(KeyModifiers::CONTROL) => {
                ui.query.push(c);
                ui.sel = 0;
            }
            _ => {}
        }
        true
    }

    /// Drawn over everything: a rounded box in the middle of the screen.
    pub(crate) fn settings_draw(&mut self, f: &mut Frame) {
        if self.settings_ui.is_none() {
            return;
        }
        let shown = self.settings_shown();
        let full = f.area();
        let w = (full.width * 4 / 5).clamp(40.min(full.width), 110);
        let h = (full.height * 4 / 5).max(8.min(full.height));
        let area = Rect { x: full.x + (full.width - w) / 2, y: full.y + (full.height - h) / 2, width: w, height: h };
        f.render_widget(ratatui::widgets::Clear, area);
        let title = format!(" Settings · {} of {} ", shown.len(), crate::settings::all().len());
        let block = ratatui::widgets::Block::bordered().border_set(crate::display::float_set()).border_style(hl("FloatBorder")).title(Span::styled(title, hl("FloatTitle").add_modifier(Modifier::BOLD))).style(hl("NormalFloat"));
        let inner = block.inner(area);
        f.render_widget(block, area);
        if inner.height < 4 {
            return;
        }
        let ui = self.settings_ui.as_ref().unwrap();
        // the search line
        let q = if ui.query.is_empty() { Span::styled("type to search the settings and what they do", hl("Comment")) } else { Span::styled(ui.query.clone(), hl("Normal")) };
        f.render_widget(Paragraph::new(Line::from(vec![Span::styled(" 🔍 ", hl("Special")), q])), Rect { height: 1, ..inner });
        f.render_widget(Paragraph::new(Line::from(Span::styled("─".repeat(inner.width as usize), hl("FloatBorder")))), Rect { y: inner.y + 1, height: 1, ..inner });
        let list = Rect { y: inner.y + 2, height: inner.height - 3, ..inner };
        let (sel, editing) = (ui.sel, ui.editing.clone());
        let top = {
            let ui = self.settings_ui.as_mut().unwrap();
            if ui.sel < ui.top { ui.top = ui.sel }
            if ui.sel >= ui.top + list.height as usize { ui.top = ui.sel + 1 - list.height as usize }
            ui.top
        };
        let name_w = 27usize;
        let val_w = 18usize;
        // the chosen row's description runs across (a second after choosing it) when it does not fit
        let since = {
            let ui = self.settings_ui.as_mut().unwrap();
            if ui.shown_sel != sel || ui.since.is_none() {
                ui.shown_sel = sel;
                ui.since = Some(Instant::now());
            }
            ui.list = Some((list, 2 + name_w as u16));
            ui.since.unwrap_or_else(Instant::now)
        };
        let mut running = false;
        let mut lines = Vec::new();
        for (i, s) in shown.iter().enumerate().skip(top).take(list.height as usize) {
            let chosen = i == sel;
            let value = self.setting(self.cur, s.name);
            let (val, vst) = match s.kind {
                Kind::Bool if value == "true" => ("● on".to_string(), hl("String")),
                Kind::Bool => ("○ off".to_string(), hl("Comment")),
                _ if chosen && editing.is_some() => (format!("{}▏", editing.clone().unwrap_or_default()), hl("Normal").add_modifier(Modifier::UNDERLINED)),
                _ if value.is_empty() => ("—".to_string(), hl("Comment")),
                _ => (value.clone(), hl("Constant")),
            };
            // set to something else than its default: the name says so
            let changed = value != s.default && !(matches!(s.kind, Kind::Bool) && (value == "true") == (s.default == "true"));
            // a space always between the name and its value (a long name is cut with …)
            let name: String = if s.name.chars().count() > name_w - 1 { s.name.chars().take(name_w - 2).collect::<String>() + "…" } else { s.name.to_string() };
            let val: String = val.chars().take(val_w).collect();
            let about_w = (list.width as usize).saturating_sub(name_w + val_w + 6);
            let n = s.about.chars().count();
            let about: String = if chosen && n > about_w && about_w > 4 {
                running = true;
                // 8 characters a second after a second's rest; the text, a gap, the text again (a loop)
                let t = since.elapsed().as_secs_f32() - 1.0;
                let off = if t > 0.0 { (t * 8.0) as usize % (n + 4) } else { 0 };
                s.about.chars().chain("    ".chars()).chain(s.about.chars()).skip(off).take(about_w).collect()
            } else if n > about_w {
                s.about.chars().take(about_w.saturating_sub(1)).collect::<String>() + "…"
            } else {
                s.about.to_string()
            };
            let mut nst = if changed { hl("Title").add_modifier(Modifier::BOLD) } else { hl("Normal") };
            if chosen { nst = nst.add_modifier(Modifier::BOLD) }
            let l = Line::from(vec![
                Span::raw(if chosen { " ▌" } else { "  " }),
                Span::styled(format!("{name:<name_w$}"), nst),
                Span::styled(format!("{val:<val_w$}"), vst),
                Span::raw("  "),
                Span::styled(about, hl("Comment")),
            ]);
            lines.push(if chosen { l.style(hl("CursorLine")) } else { l });
        }
        f.render_widget(Paragraph::new(lines), list);
        if let Some(ui) = self.settings_ui.as_mut() {
            ui.running = running;
        }
        let help = if editing.is_some() { " Enter sets it · Esc leaves it" } else { " ↑↓ or wheel choose · click a value to switch · Enter edits · Esc closes · bold: changed · kept in settings.lua" };
        f.render_widget(Paragraph::new(Line::from(Span::styled(help, hl("Comment")))), Rect { y: inner.y + inner.height - 1, height: 1, ..inner });
    }
}

#[cfg(test)]
mod tests {
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn search_switch_and_edit() {
        let mut app = crate::App::new(Vec::new());
        app.settings_open();
        let key = |c: KeyCode| KeyEvent::new(c, KeyModifiers::NONE);
        for c in "relativenumber".chars() {
            app.key(key(KeyCode::Char(c)));
        }
        let before = app.flag(app.cur, "relativenumber");
        app.key(key(KeyCode::Char(' ')));
        assert_ne!(app.flag(app.cur, "relativenumber"), before, "Space turns it");
        // a number: Enter, a new value, Enter
        app.key(key(KeyCode::Esc));
        app.settings_open();
        for c in "scrolloff".chars() {
            app.key(key(KeyCode::Char(c)));
        }
        app.key(key(KeyCode::Enter));
        for _ in 0..4 {
            app.key(key(KeyCode::Backspace));
        }
        app.key(key(KeyCode::Char('7')));
        app.key(key(KeyCode::Enter));
        assert_eq!(app.setting(app.cur, "scrolloff"), "7");
        app.key(key(KeyCode::Esc));
        assert!(app.settings_ui.is_none(), "Esc closes");
    }
}

/// settings.lua with one setting set to `value` (its line replaced, or added), the others kept: one
/// `vim.cmd("set …")` line per setting the popup changed.
pub(crate) fn settings_file_with(old: &str, name: &str, set: &str) -> String {
    const HEAD: &str = "-- written by zett's settings popup (:Settings); init.lua runs before this file\n";
    let mine = format!("vim.cmd({:?}) -- {name}", format!("set {set}"));
    let mut lines: Vec<String> = old.lines().filter(|l| !l.starts_with("-- written by")).map(str::to_string).collect();
    match lines.iter().position(|l| l.ends_with(&format!(" -- {name}"))) {
        Some(i) => lines[i] = mine,
        None => lines.push(mine),
    }
    format!("{HEAD}{}\n", lines.join("\n"))
}

impl App {
    /// A change in the popup kept for the next start: settings.lua in the config folder (never in tests).
    fn settings_keep(&mut self, name: &str, set: &str) {
        if cfg!(test) {
            return;
        }
        let f = zett_script::config_dir().join("settings.lua");
        let old = std::fs::read_to_string(&f).unwrap_or_default();
        let _ = std::fs::create_dir_all(zett_script::config_dir());
        if let Err(e) = std::fs::write(&f, settings_file_with(&old, name, set)) {
            self.say(format!("{}: {e}", f.display()));
        }
    }
}

#[cfg(test)]
mod file_tests {
    #[test]
    fn the_file_keeps_one_line_per_setting() {
        let a = super::settings_file_with("", "scrolloff", "scrolloff=7");
        let b = super::settings_file_with(&a, "wrap", "nowrap");
        let c = super::settings_file_with(&b, "scrolloff", "scrolloff=3");
        assert_eq!(c.lines().filter(|l| l.contains("scrolloff")).count(), 1);
        assert!(c.contains("vim.cmd(\"set scrolloff=3\") -- scrolloff") && c.contains("set nowrap"));
        assert!(c.starts_with("-- written by"), "{c}");
        assert_eq!(c.matches("-- written by").count(), 1);
    }
}

impl App {
    /// The mouse while the popup is open (it takes every event): the wheel moves through the list, a click chooses a
    /// row, a click on a switch's value turns it, a click outside closes the popup. True when it was open.
    pub(crate) fn settings_mouse(&mut self, m: ratatui::crossterm::event::MouseEvent) -> bool {
        use ratatui::crossterm::event::{MouseButton, MouseEventKind};
        let Some(ui) = self.settings_ui.as_ref() else { return false };
        let Some((list, val_x)) = ui.list else { return true };
        let n = self.settings_shown().len();
        let ui = self.settings_ui.as_mut().unwrap();
        match m.kind {
            MouseEventKind::ScrollDown => ui.sel = (ui.sel + 3).min(n.saturating_sub(1)),
            MouseEventKind::ScrollUp => ui.sel = ui.sel.saturating_sub(3),
            MouseEventKind::Down(MouseButton::Left) => {
                let inside = m.column >= list.x && m.column < list.x + list.width && m.row >= list.y && m.row < list.y + list.height;
                if !inside {
                    // above or below the list but in the box (the search line, the help): nothing; outside it: close
                    let outer = Rect { x: list.x.saturating_sub(1), y: list.y.saturating_sub(3), width: list.width + 2, height: list.height + 5 };
                    if !(m.column >= outer.x && m.column < outer.x + outer.width && m.row >= outer.y && m.row < outer.y + outer.height) {
                        self.settings_ui = None;
                    }
                    return true;
                }
                let row = ui.top + (m.row - list.y) as usize;
                if row >= n {
                    return true;
                }
                let again = ui.sel == row;
                ui.sel = row;
                // on the value (or a second click on the row): a switch turns, another value is edited
                let on_value = m.column >= list.x + val_x && m.column < list.x + val_x + 18;
                if on_value || again {
                    self.settings_key(&KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
                }
            }
            _ => {}
        }
        true
    }

    /// The popup's description is running across: the main loop draws again soon.
    pub(crate) fn settings_animating(&self) -> bool {
        self.settings_ui.as_ref().is_some_and(|u| u.running)
    }
}

#[cfg(test)]
mod mouse_tests {
    use ratatui::crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

    fn screen(app: &mut crate::App, term: &mut ratatui::Terminal<ratatui::backend::TestBackend>) -> Vec<String> {
        term.draw(|f| app.draw(f)).unwrap();
        let b = term.backend().buffer();
        (0..b.area.height).map(|y| (0..b.area.width).map(|x| b[(x, y)].symbol().to_string()).collect::<String>()).collect()
    }

    #[test]
    fn the_mouse_turns_a_switch_and_the_long_description_runs() {
        let mut app = crate::App::new(Vec::new());
        let mut term = ratatui::Terminal::new(ratatui::backend::TestBackend::new(90, 30)).unwrap();
        app.settings_open();
        screen(&mut app, &mut term);
        let (list, val_x) = app.settings_ui.as_ref().unwrap().list.unwrap();
        let at = |col, row| MouseEvent { kind: MouseEventKind::Down(MouseButton::Left), column: col, row, modifiers: KeyModifiers::NONE };
        // the wheel moves the choice
        app.mouse(MouseEvent { kind: MouseEventKind::ScrollDown, column: list.x + 2, row: list.y, modifiers: KeyModifiers::NONE });
        assert_eq!(app.settings_ui.as_ref().unwrap().sel, 3);
        // the first switch in view, clicked on its value, turns
        let shown = app.settings_shown();
        let i = shown.iter().position(|s| matches!(s.kind, crate::settings::Kind::Bool)).unwrap();
        let name = shown[i].name;
        app.settings_ui.as_mut().unwrap().top = i;
        let before = app.flag(app.cur, name);
        app.mouse(at(list.x + val_x + 1, list.y));
        assert_ne!(app.flag(app.cur, name), before, "{name} turned");
        // a long description on the chosen row moves after a moment
        let long = shown.iter().position(|s| s.about.chars().count() > 60).unwrap();
        app.settings_ui.as_mut().unwrap().sel = long;
        let a = screen(&mut app, &mut term).join("\n");
        app.settings_ui.as_mut().unwrap().since = Some(std::time::Instant::now() - std::time::Duration::from_secs(3));
        let b = screen(&mut app, &mut term).join("\n");
        assert!(app.settings_animating());
        assert_ne!(a, b, "the description ran on");
        // a click outside closes it
        app.mouse(at(0, 0));
        assert!(app.settings_ui.is_none());
    }
}
