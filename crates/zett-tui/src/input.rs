//! Keys and the mouse: the key map (Vim, Emacs), what a key runs and its effects.

use super::*;

impl App {
    // ── input ──

    pub(crate) fn key(&mut self, k: KeyEvent) {
        if let Some(p) = self.popup.take() {
            // any key closes help (Esc and q too)
            let _ = p;
            return;
        }
        if self.minibuf.is_some() {
            return self.minibuf_key(k);
        }
        if self.travel.is_some() {
            return self.travel_key(k);
        }
        if self.cmdline.is_some() {
            return self.cmdline_key(k);
        }
        if self.panel_key(k) {
            return;
        }
        // the git buffer's own keys (s u x c Tab Enter g q)
        if self.git_key(&k) {
            return;
        }
        let ctrl = k.modifiers.contains(KeyModifiers::CONTROL);
        // the Emacs profile: every key through its keymap. C-s is isearch and C-p the previous
        // line there, so the Vim profile's shortcuts below must not catch them first
        if self.emacs_keys() {
            return self.emacs_key(&k);
        }
        // Vim profile: Ctrl+S saves in any mode, Ctrl+P finds a file
        if ctrl && k.code == KeyCode::Char('s') {
            return self.save(None);
        }
        if ctrl && k.code == KeyCode::Char('p') {
            return self.open_picker();
        }
        let Some(ek) = to_key(&k) else { return };
        let mode = self.docs[self.cur].ed.mode();
        // gc in visual mode comments the selection (the engine only sees normal-mode keys)
        if matches!(mode, Mode::Visual | Mode::VisualLine) && !ctrl {
            if self.vis_g {
                self.vis_g = false;
                if k.code == KeyCode::Char('c') {
                    return self.comment();
                }
                self.edit_key(Key::ch('g'));
            } else if k.code == KeyCode::Char('g') {
                self.vis_g = true;
                return;
            }
        }
        // : in visual mode: the selected lines as the range ('<,'>), as Vim does
        if matches!(mode, Mode::Visual | Mode::VisualLine) && k.code == KeyCode::Char(':') {
            let d = &self.docs[self.cur];
            if let Some((a, b)) = d.ed.selection_range() {
                let (la, lb) = (d.ed.buf.to_pos(a).line, d.ed.buf.to_pos(b.saturating_sub(1).max(a)).line);
                self.vis_range = Some((la.min(lb), la.max(lb)));
            }
            self.edit_key(Key::code(Code::Esc));
            self.cmdline = Some("'<,'>".into());
            self.cmd_sel = 0;
            return;
        }
        // the dashboard's own keys (the SPC tree works there too)
        if self.dash && mode == Mode::Normal && self.keyseq.is_empty() && !ctrl {
            match k.code {
                KeyCode::Char('j') | KeyCode::Down | KeyCode::Tab => {
                    self.dash_sel = (self.dash_sel + 1) % self.dash_len();
                    return;
                }
                KeyCode::Char('k') | KeyCode::Up | KeyCode::BackTab => {
                    self.dash_sel = (self.dash_sel + self.dash_len() - 1) % self.dash_len();
                    return;
                }
                KeyCode::Enter => return self.dash_run(),
                KeyCode::Char('q') => {
                    self.quit = true;
                    return;
                }
                _ => {}
            }
        }
        if mode == Mode::Normal {
            let tok = token(&k);
            if tok.is_empty() {
                return;
            }
            if tok == "<CR>" && self.keyseq.is_empty() && (self.follow_help() || self.follow_cmdwin()) {
                return;
            }
            if tok == "<Esc>" && !self.keyseq.is_empty() {
                self.keyseq.clear();
                self.menu.clear();
                return;
            }
            let mut seq = std::mem::take(&mut self.keyseq);
            seq.push(tok);
            let feed = self.with_lua(|e| e.feed("n", &seq));
            return self.after_feed(feed, seq);
        }
        // the completion popup takes its keys first
        if mode == Mode::Insert && self.comp.is_some() {
            let n = self.comp.as_ref().map(|c| c.shown.len()).unwrap_or(0);
            let c = self.comp.as_mut().unwrap();
            match k.code {
                KeyCode::Tab | KeyCode::Down => return c.sel = (c.sel + 1) % n.max(1),
                KeyCode::Char('n') if ctrl => return c.sel = (c.sel + 1) % n.max(1),
                KeyCode::BackTab | KeyCode::Up => return c.sel = (c.sel + n.max(1) - 1) % n.max(1),
                KeyCode::Char('p') if ctrl => return c.sel = (c.sel + n.max(1) - 1) % n.max(1),
                KeyCode::Enter => return self.comp_accept(),
                KeyCode::Char('y') if ctrl => return self.comp_accept(),
                KeyCode::Esc => self.comp = None,
                _ => {}
            }
        }
        let typed = match k.code {
            KeyCode::Char(c) if !ctrl => Some(c),
            KeyCode::Backspace => None,
            _ => Some('\0'),
        };
        self.edit_key(ek);
        if mode == Mode::Insert {
            self.comp_after_key(typed);
        }
    }

    /// What the engine did with keys: ran a mapping, waits for more, or left them to Vim.
    pub(crate) fn after_feed(&mut self, feed: Feed, seq: Vec<String>) {
        match feed {
            Feed::Ran(fx) => {
                self.menu.clear();
                self.after_lua(fx);
            }
            Feed::Pending(menu) => {
                self.keyseq = seq;
                self.menu = menu;
                self.key_at = Instant::now();
            }
            Feed::None => {
                self.menu.clear();
                for t in seq {
                    for k in zett_edit::keys::parse(&t) {
                        self.edit_key(k);
                    }
                }
            }
        }
    }

    /// Started without a file: what `zett.opt.startup` asks for (see [`Startup`]).
    pub(crate) fn startup(&mut self) {
        match Startup::from_opt(self.engine.opt("startup")) {
            Startup::Scratch => {
                // the scratch buffer's own text, when the config gives one; the buffer stays clean
                if let Some(text) = self.engine.opt("scratch_message").filter(|t| !t.is_empty()) {
                    if let Some(d) = self.docs.iter_mut().find(|d| d.name == "[scratch]") {
                        d.ed = Editor::from_str(&text);
                    }
                } else if self.emacs_keys() {
                    self.say("C-x C-f opens a file · M-x runs a command · C-h d reads the documentation");
                }
            }
            Startup::Dashboard => self.dash = true,
            Startup::Command(cmd) => match self.with_lua(|e| e.run(&cmd)) {
                Ok(fx) => self.after_lua(fx),
                Err(e) => self.say(format!("zett.opt.startup: {e}")),
            },
        }
    }

    /// Every buffer in step with the app: the key profile, and the one kill ring they share.
    /// Runs before each draw, so no place that opens a buffer has to.
    pub(crate) fn sync_profile(&mut self) {
        let emacs = self.emacs;
        // the settings the editor acts on itself: expandtab, shiftwidth, fill_column (settings.rs)
        self.apply_settings();
        for d in self.docs.iter_mut() {
            if d.ed.is_emacs() != emacs {
                d.ed.set_emacs(emacs);
            }
            d.ed.use_kill_ring(&self.kill_ring);
            // the outline (folds, sections) read the file type's way
            d.ed.set_outline_kind(zett_edit::outline::OutlineKind::for_syntax(&d.syntax));
        }
    }

    /// The key profile: Emacs unless `zett.opt.keys = "vim"`.
    pub(crate) fn emacs_keys(&self) -> bool {
        self.emacs
    }
    /// The keymap mode keys go through: "e" (Emacs) or Vim's normal-mode "n".
    pub(crate) fn keymode(&self) -> &'static str {
        if self.emacs_keys() { "e" } else { "n" }
    }

    /// Emacs profile: every key goes through the keymap (chords like C-x C-f wait for more);
    /// an unbound printable key types itself, anything else unbound says so.
    pub(crate) fn emacs_key(&mut self, k: &KeyEvent) {
        // Esc, as in Emacs and zsh: Meta on the next key (Esc f = M-f). Esc Esc, or Esc in the
        // middle of a chord, cancels everything (Emacs' keyboard-escape-quit)
        let pending_meta = std::mem::take(&mut self.meta);
        if k.code == KeyCode::Esc && k.modifiers.is_empty() {
            if pending_meta || !self.keyseq.is_empty() {
                self.keyseq.clear();
                self.menu.clear();
                return self.after_lua(vec![Effect::Edit("keyboard-quit".into(), 1)]);
            }
            self.meta = true;
            return self.say("ESC-");
        }
        // the pending Meta joins the key itself, so token() is the one place keys are spelled
        let mut k = *k;
        if pending_meta {
            k.modifiers |= KeyModifiers::ALT;
            self.msg = None; // the "ESC-" prompt is answered
        }
        let tok = token(&k);
        if tok.is_empty() {
            return;
        }
        // C-g drops a question the agent is still answering
        if tok == "<C-g>" && self.agent.take().is_some() {
            return self.say("dropped the question");
        }
        // and stops waiting for running code blocks
        if tok == "<C-g>" && self.notebook_drop() {
            return self.say("stopped waiting for the code");
        }
        // C-g abandons a half-typed chord or argument (and, bound, deactivates the mark)
        if tok == "<C-g>" && (!self.keyseq.is_empty() || self.arg.active()) {
            self.keyseq.clear();
            self.menu.clear();
            self.arg.take();
            return self.say("Quit");
        }
        // typing a numeric argument: digits (and a leading -) are part of it, not text
        if self.arg.active() && self.keyseq.is_empty() {
            let plain = !k.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT);
            match k.code {
                KeyCode::Char(c) if plain && c.is_ascii_digit() => {
                    self.arg.digit(c.to_digit(10).unwrap_or(0));
                    return self.say(self.arg.prompt());
                }
                KeyCode::Char('-') if plain && self.arg.accepts_minus() => {
                    self.arg.minus();
                    return self.say(self.arg.prompt());
                }
                _ => {}
            }
        }
        self.last_key = tok.clone();
        self.engine.set_arg(self.arg.value());
        let mut seq = std::mem::take(&mut self.keyseq);
        seq.push(tok);
        // a search in progress has its own keymap (the isearch plugin's), as Emacs's isearch-mode-map
        let searching = self.docs[self.cur].ed.isearch().is_some();
        match self.with_lua(|e| e.feed(if searching { "isearch" } else { "e" }, &seq)) {
            Feed::None if searching => {
                self.menu.clear();
                let plain = !k.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT);
                match k.code {
                    KeyCode::Char(c) if plain => self.doc().ed.isearch_char(c),
                    // any other key ends the search where it is, then does what it always does
                    _ => {
                        self.doc().ed.exec("isearch-exit", 1);
                        self.emacs_key(&k);
                    }
                }
            }
            Feed::None => {
                self.menu.clear();
                let plain = !k.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT);
                match k.code {
                    KeyCode::Char(c) if plain && seq.len() == 1 => {
                        // C-u 5 a types aaaaa
                        let times = self.arg.take().map_or(1, |n| n.unsigned_abs().max(1));
                        let before = self.docs[self.cur].ed.cursor().line;
                        for _ in 0..times {
                            self.doc().ed.self_insert(c);
                        }
                        self.doc().touched(before);
                        self.edit_at = Instant::now();
                    }
                    _ => {
                        self.arg.take();
                        self.say(format!("{} is undefined", seq.join(" ")))
                    }
                }
            }
            f => {
                // a Lua command used the argument (zett.arg); an editing one takes it in effects()
                if let Feed::Ran(fx) = &f {
                    if !fx.iter().any(|e| matches!(e, Effect::Edit(..))) {
                        self.arg.take();
                    }
                }
                self.after_feed(f, seq)
            }
        }
    }

    pub(crate) fn edit_key(&mut self, k: Key) {
        let before = self.docs[self.cur].ed.cursor().line;
        let rev = self.docs[self.cur].ed.buf.rev();
        self.doc().ed.key(k);
        self.doc().touched(before);
        self.editor_requests();
        if self.docs[self.cur].ed.buf.rev() != rev {
            self.edit_at = Instant::now();
        }
    }

    pub(crate) fn effects(&mut self, fx: Vec<Effect>) {
        for e in fx {
            match e {
                Effect::Notify(m) => {
                    // plugins may show messages too (fidget); a message they send is not sent back
                    if !self.notifying && self.engine.has_hooks("notify") {
                        self.notifying = true;
                        let fx = self.with_lua(|e| e.emit_fields("notify", &[("message", m.clone())]));
                        self.after_lua(fx);
                        self.notifying = false;
                    }
                    self.say(m)
                }
                Effect::Ui(name, arg) if self.doom_ui(&name, &arg) => {}
                Effect::Ui(name, arg) => match name.as_str() {
                    "quickOpen" | "search" => self.open_picker(),
                    "commandLine" => {
                        // a count before : becomes a range (2: → :.,.+1)
                        let n = self.docs[self.cur].ed.take_pending_count();
                        self.cmdline = Some(match n { Some(1) => ".".into(), Some(n) => format!(".,.+{}", n - 1), None => arg });
                    }
                    "help" => self.help(&arg),
                    "reload" => {
                        let mut e = Engine::new(None);
                        e.set_frontend("editor");
                        e.load_config(&zett_script::config_dir());
                        let n = e.plugins.len();
                        self.engine = e;
                        hl::reset();
                        self.take_opts();
                        self.say(format!("config reloaded ({n} plugins)"));
                    }
                    "openConfig" => {
                        let p = zett_script::config_dir().join("init.lua");
                        let _ = std::fs::create_dir_all(zett_script::config_dir());
                        self.open(&p);
                    }
                    "toggleSidebar" | "focusBlock" => {}
                    "openFile" => {
                        let (how, path) = arg.split_once('\t').unwrap_or(("", arg.as_str()));
                        match how {
                            "right" | "below" => {
                                self.engine.focus_panel(None);
                                self.doom_ui("split", how);
                            }
                            _ => {}
                        }
                        self.open(Path::new(path));
                    }
                    "ex" => self.command(&arg),
                    "clipboard" => {
                        if !clipboard::copy(&arg) {
                            self.say("could not copy to the clipboard");
                        }
                    }
                    other => self.say(format!("{other}: only in the window (zett --gui)")),
                },
                // C-u, M-0…M-9, M--: build the argument for the next command
                Effect::Edit(name, _) if zett_edit::prefix::is_command(&name) => {
                    match name.as_str() {
                        "universal-argument" => self.arg.universal(),
                        "negative-argument" => self.arg.minus(),
                        _ => {
                            let d = self.last_key.chars().rev().find_map(|c| c.to_digit(10)).unwrap_or(0);
                            self.arg.digit(d);
                        }
                    }
                    self.say(self.arg.prompt());
                }
                Effect::Edit(name, n) => {
                    let before = self.docs[self.cur].ed.cursor().line;
                    let rev = self.docs[self.cur].ed.buf.rev();
                    let kills = self.docs[self.cur].ed.kill_count();
                    // the kill ring and the system clipboard are one, as in Emacs
                    if name == "yank" {
                        if let Some(t) = clipboard::paste() {
                            self.doc().ed.offer_paste(t);
                        }
                    }
                    // the typed argument, else the count the effect carries
                    let arg = self.arg.take().or((n > 1).then_some(n as i64));
                    if arg.is_some() {
                        self.msg = None; // the "C-u 4-" prompt is answered
                    }
                    self.doc().ed.exec_arg(&name, arg);
                    if self.docs[self.cur].ed.kill_count() != kills {
                        if let Some(t) = self.docs[self.cur].ed.last_kill() {
                            clipboard::copy(&t);
                        }
                    }
                    self.doc().touched(before);
                    if self.docs[self.cur].ed.buf.rev() != rev {
                        self.edit_at = Instant::now();
                    }
                }
                Effect::Open(_) | Effect::Refresh => {}
            }
        }
        for e in std::mem::take(&mut self.engine.errors) {
            self.say(e);
        }
    }

    pub(crate) fn mouse(&mut self, m: ratatui::crossterm::event::MouseEvent) {
        if self.panel_mouse(&m) {
            return;
        }
        if let MouseEventKind::Down(_) = m.kind {
            self.engine.focus_panel(None);
        }
        // a click in another window focuses it first
        if let MouseEventKind::Down(_) = m.kind {
            if let Some(&(id, r)) = self.win_rects.iter().find(|(_, r)| m.column >= r.x && m.column < r.right() && m.row >= r.y && m.row < r.bottom()) {
                if id != self.focus {
                    self.focus_win(id);
                    self.area = Rect { height: r.height.saturating_sub(1), ..r };
                    if let Some(Some(w)) = self.wins.get(id) {
                        self.docs[self.cur].top = w.top;
                        self.docs[self.cur].left = w.left;
                    }
                }
            }
        }
        let a = self.area;
        match m.kind {
            MouseEventKind::ScrollDown | MouseEventKind::ScrollUp => {
                let d = &mut self.docs[self.cur];
                let last = d.ed.buf.len_lines().saturating_sub(1);
                if m.kind == MouseEventKind::ScrollDown { d.top = (d.top + 3).min(last) } else { d.top = d.top.saturating_sub(3) }
                // the cursor stays on screen, as in Vim
                let c = d.ed.cursor();
                let bottom = d.top + a.height.saturating_sub(1) as usize;
                if c.line < d.top || c.line > bottom {
                    let line = c.line.clamp(d.top, bottom.min(last));
                    d.ed.set_cursor(Pos { line, col: c.col });
                }
            }
            MouseEventKind::Down(_) if m.row >= a.y && m.row < a.y + a.height && m.column >= a.x + self.gutter => {
                let d = &mut self.docs[self.cur];
                let line = (d.top + (m.row - a.y) as usize).min(d.ed.buf.len_lines().saturating_sub(1));
                let want = d.left + (m.column - a.x - self.gutter) as usize;
                // screen column back to a character column (tabs are wide)
                let text = d.ed.buf.line(line);
                let mut sc = 0;
                let mut col = 0;
                for ch in text.chars() {
                    let w = if ch == '\t' { tab() } else { 1 };
                    if sc + w > want {
                        break;
                    }
                    sc += w;
                    col += 1;
                }
                d.ed.set_cursor(Pos { line, col });
            }
            _ => {}
        }
    }

}
