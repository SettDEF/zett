//! Neovim's autocommand events that follow from a change of state: each frame compared with the last.

use super::*;

/// What the last frame saw.
#[derive(Default)]
pub(crate) struct Seen {
    started: bool,
    doc: usize,
    win: usize,
    wins: usize,
    tab: usize,
    tabs: usize,
    docs: usize,
    cursor: (usize, usize),
    top: usize,
    mode: String,
    rev: u64,
    modified: bool,
    cmdline: Option<String>,
    rec: Option<char>,
    size: (u16, u16),
    cwd: PathBuf,
    colors: String,
    yank: Option<(String, bool)>,
}

/// mode() letters, as ModeChanged's patterns use them.
pub(crate) fn mode_letter(m: Mode) -> &'static str {
    match m {
        Mode::Normal => "n",
        Mode::Insert => "i",
        Mode::Visual => "v",
        Mode::VisualLine => "V",
        Mode::VisualBlock => "\x16",
    }
}

impl App {
    /// Fires what changed since the last frame (the first frame only takes note).
    pub(crate) fn state_events(&mut self) {
        if self.autocmds.is_empty() {
            self.seen_ev.started = false;
            return;
        }
        let d = &self.docs[self.cur];
        let c = d.ed.cursor();
        let now = Seen {
            started: true,
            doc: self.cur,
            win: self.focus,
            wins: self.wins.iter().filter(|w| w.is_some()).count(),
            tab: self.tab,
            tabs: self.tab_pages.len().max(1),
            docs: self.docs.len(),
            cursor: (c.line, c.col),
            top: d.top,
            mode: if self.cmdline.is_some() { "c".into() } else { mode_letter(d.ed.mode()).into() },
            rev: d.ed.buf.rev(),
            modified: d.dirty(),
            cmdline: self.cmdline.clone(),
            rec: d.ed.recording(),
            size: (self.area.width, self.area.height),
            cwd: std::env::current_dir().unwrap_or_default(),
            colors: self.colors_name.clone(),
            yank: d.ed.register('"'),
        };
        let was = std::mem::replace(&mut self.seen_ev, now);
        if !was.started {
            return;
        }
        let now = &self.seen_ev;
        let mut fire: Vec<(&'static str, Option<String>)> = Vec::new();
        // buffers and windows: leaving one before entering the next
        if now.doc != was.doc || now.win != was.win {
            if now.doc != was.doc {
                fire.push(("bufleave", None));
            }
            if now.win != was.win {
                fire.push(("winleave", None));
                fire.push(("winenter", None));
            }
        }
        if now.wins > was.wins {
            fire.push(("winnew", None));
        } else if now.wins < was.wins {
            fire.push(("winclosed", None));
        }
        if now.tabs > was.tabs {
            fire.push(("tabnew", None));
        } else if now.tabs < was.tabs {
            fire.push(("tabclosed", None));
        }
        if now.tab != was.tab {
            fire.push(("tableave", None));
            fire.push(("tabenter", None));
        }
        if now.docs > was.docs {
            fire.push(("bufadd", None));
            fire.push(("bufnew", None));
        } else if now.docs < was.docs {
            fire.push(("bufdelete", None));
            fire.push(("bufunload", None));
            fire.push(("bufwipeout", None));
        }
        let same_doc = now.doc == was.doc;
        if now.mode != was.mode {
            fire.push(("modechanged", Some(format!("{}:{}", was.mode, now.mode))));
        }
        if same_doc && now.rev != was.rev && now.mode == "i" {
            fire.push(("textchangedi", None));
        }
        if same_doc && now.modified != was.modified {
            fire.push(("bufmodifiedset", None));
        }
        if now.cursor != was.cursor || !same_doc {
            fire.push((if now.mode == "i" { "cursormovedi" } else { "cursormoved" }, None));
        }
        if same_doc && now.top != was.top {
            fire.push(("winscrolled", None));
        }
        match (&was.cmdline, &now.cmdline) {
            (None, Some(_)) => fire.push(("cmdlineenter", Some(":".into()))),
            (Some(_), None) => fire.push(("cmdlineleave", Some(":".into()))),
            (Some(a), Some(b)) if a != b => fire.push(("cmdlinechanged", Some(":".into()))),
            _ => {}
        }
        match (was.rec, now.rec) {
            (None, Some(_)) => fire.push(("recordingenter", None)),
            (Some(_), None) => fire.push(("recordingleave", None)),
            _ => {}
        }
        if now.size != was.size {
            fire.push(("vimresized", None));
            fire.push(("winresized", None));
        }
        if now.cwd != was.cwd {
            fire.push(("dirchanged", Some("global".into())));
        }
        if now.colors != was.colors {
            fire.push(("colorscheme", Some(now.colors.clone())));
        }
        if same_doc && now.yank != was.yank && now.yank.is_some() {
            fire.push(("textyankpost", None));
        }
        let (old, new) = (was.mode.clone(), now.mode.clone());
        for (ev, target) in fire {
            if ev == "modechanged" {
                self.command(&format!("lua vim.v.event = {{ old_mode = {old:?}, new_mode = {new:?} }}"));
            }
            self.fire_autocmds(ev, target.as_deref());
        }
    }
}

#[cfg(test)]
mod tests {
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn state_changes_fire_neovims_events() {
        let dir = std::env::temp_dir().join(format!("zett-events-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let f = dir.join("a.txt");
        std::fs::write(&f, "one\ntwo\nthree\n").unwrap();
        let mut app = crate::App::new(Vec::new());
        app.open(&f);
        app.command("lua _G.seen = {} for _, e in ipairs({ 'CursorMoved', 'CursorMovedI', 'ModeChanged', 'TextChangedI', 'TextYankPost', 'BufModifiedSet', 'CmdlineEnter', 'CmdlineLeave' }) do vim.api.nvim_create_autocmd(e, { callback = function(a) seen[a.event] = (seen[a.event] or 0) + 1 end }) end");
        app.command("lua vim.api.nvim_create_autocmd('ModeChanged', { pattern = 'n:i', callback = function() _G.ni = vim.v.event.old_mode .. '>' .. vim.v.event.new_mode end })");
        app.state_events();
        let press = |app: &mut crate::App, k: KeyCode| {
            app.key(KeyEvent::new(k, KeyModifiers::NONE));
            app.state_events();
        };
        press(&mut app, KeyCode::Char('j'));
        press(&mut app, KeyCode::Char('i'));
        press(&mut app, KeyCode::Char('x'));
        press(&mut app, KeyCode::Esc);
        press(&mut app, KeyCode::Char('y'));
        press(&mut app, KeyCode::Char('y'));
        press(&mut app, KeyCode::Char(':'));
        press(&mut app, KeyCode::Esc);
        let get = |app: &mut crate::App, e: &str| app.engine.eval(&format!("return tostring(seen.{e} or 0)")).map(|r| r.0).unwrap_or_default();
        let counts: Vec<(String, String)> = ["CursorMoved", "CursorMovedI", "ModeChanged", "TextChangedI", "TextYankPost", "BufModifiedSet", "CmdlineEnter", "CmdlineLeave"].iter().map(|e| (e.to_string(), get(&mut app, e))).collect();
        let ni = app.engine.eval("return tostring(_G.ni)").map(|r| r.0).unwrap_or_default();
        let _ = std::fs::remove_dir_all(&dir);
        for (e, n) in &counts {
            assert_ne!(n, "0", "{e} never fired: {counts:?}");
        }
        assert_eq!(ni, "n>i");
    }
}

#[cfg(test)]
mod split_view_tests {
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    // a buffer shown in two windows: each has its own cursor and view, as in Vim
    #[test]
    fn two_windows_on_one_buffer_move_separately() {
        let text: String = (1..=200).map(|n| format!("line {n}\n")).collect();
        let mut app = crate::App::new(Vec::new());
        app.docs[app.cur].ed = zett_edit::Editor::from_str(&text);
        let mut term = ratatui::Terminal::new(ratatui::backend::TestBackend::new(80, 30)).unwrap();
        let mut frame = |app: &mut crate::App| {
            term.draw(|f| app.draw(f)).unwrap();
        };
        let keys = |app: &mut crate::App, s: &str| {
            for c in s.chars() {
                app.key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
            }
        };
        frame(&mut app);
        app.command("split");
        frame(&mut app);
        let new = app.focus;
        keys(&mut app, "150G");
        frame(&mut app);
        app.command("wincmd w");
        frame(&mut app);
        assert_ne!(app.focus, new);
        assert_eq!(app.docs[app.cur].ed.cursor().line, 0, "the first window's cursor followed the other");
        assert_eq!(app.wins[app.focus].as_ref().unwrap().top, 0, "the first window scrolled with the other");
        keys(&mut app, "j");
        frame(&mut app);
        app.command("wincmd w");
        frame(&mut app);
        assert_eq!(app.docs[app.cur].ed.cursor().line, 149, "the second window lost its place");
    }
}
