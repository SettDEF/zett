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
            let about: String = s.about.chars().take(about_w).collect();
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
        let help = if editing.is_some() { " Enter sets it · Esc leaves it" } else { " ↑↓ choose · Space/Enter switch · Enter edits a value · Esc closes · bold: changed · kept in settings.lua" };
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
