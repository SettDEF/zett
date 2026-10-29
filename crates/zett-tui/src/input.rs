//! Keys and the mouse: the key map (Vim, Emacs), what a key runs and its effects.

use super::*;

impl App {
    // ── input ──

    pub(crate) fn key(&mut self, k: KeyEvent) {
        // the Kitty keyboard protocol tells C-i from Tab, C-m from Enter and C-[ from Esc; zett
        // means them as one, as every terminal sends them (C-= C-\ C-Enter are what it adds)
        let k = match (k.code, k.modifiers) {
            (KeyCode::Char('i'), KeyModifiers::CONTROL) => KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE),
            (KeyCode::Char('m'), KeyModifiers::CONTROL) => KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            (KeyCode::Char('['), KeyModifiers::CONTROL) => KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
            _ => k,
        };
        self.typed_at = Instant::now();
        self.held = false;
        if let Some(p) = self.popup.take() {
            // any key closes help (Esc and q too)
            let _ = p;
            return;
        }
        // a macro stopped at its query point (C-x q): y goes on, n skips to the next repetition, q stops
        if let Some((rest, reps)) = self.kmacro_paused.take() {
            return self.kmacro_answer(k, rest, reps);
        }
        if self.minibuf.is_some() {
            return self.minibuf_key(k);
        }
        if self.travel.is_some() {
            return self.travel_key(k);
        }
        // M-%: its keys first
        if self.qr.is_some() && self.minibuf.is_none() {
            return self.qr_key(k);
        }
        // a command waiting for its key (C-x r s, M-z, C-q), whichever the profile
        if self.char_for.is_some() && self.cmdline.is_none() {
            return self.emacs_key(&k);
        }
        if self.cmdline.is_some() {
            // :cmap: the line's keys through its mappings first
            if self.noremap == 0 && self.engine.has_maps("c") {
                let tok = token(&k);
                if !tok.is_empty() {
                    let mut seq = std::mem::take(&mut self.keyseq);
                    seq.push(tok);
                    match self.with_lua(|e| e.feed("c", &seq)) {
                        Feed::None => {
                            self.menu.clear();
                            for t in seq {
                                for k in zett_edit::keys::parse(&t) {
                                    self.cmdline_key(key_event(&k));
                                }
                            }
                        }
                        feed => self.after_feed(feed, seq),
                    }
                    return;
                }
            }
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
        if matches!(mode, Mode::Visual | Mode::VisualLine | Mode::VisualBlock) && !ctrl {
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
        if matches!(mode, Mode::Visual | Mode::VisualLine | Mode::VisualBlock) && k.code == KeyCode::Char(':') {
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
        // :vmap / :imap: keys in visual and insert mode go through their mappings first
        if matches!(mode, Mode::Visual | Mode::VisualLine | Mode::VisualBlock | Mode::Insert) && self.noremap == 0 {
            let km = if mode == Mode::Insert { "i" } else { "v" };
            if self.engine.has_maps(km) && !(mode == Mode::Insert && self.comp.is_some()) {
                let tok = token(&k);
                if !tok.is_empty() {
                    let mut seq = std::mem::take(&mut self.keyseq);
                    seq.push(tok);
                    let feed = self.with_lua(|e| e.feed(km, &seq));
                    if !matches!(feed, Feed::None) || seq.len() > 1 {
                        return self.after_feed(feed, seq);
                    }
                }
            }
        }
        if mode == Mode::Normal {
            let tok = token(&k);
            if tok.is_empty() {
                return;
            }
            if tok == "<CR>" && self.keyseq.is_empty() && (self.follow_help() || self.follow_cmdwin() || self.follow_qf()) {
                return;
            }
            // help pages: CTRL-] follows a link, CTRL-T / CTRL-O come back (a tag stack, as Vim's)
            if self.keyseq.is_empty() && self.docs[self.cur].name.starts_with("help: ") {
                if tok == "<C-]>" && self.follow_help() {
                    return;
                }
                if matches!(tok.as_str(), "<C-t>" | "<C-o>") && self.help_back() {
                    return;
                }
            }
            if tok == "<Esc>" && !self.keyseq.is_empty() {
                self.keyseq.clear();
                self.menu.clear();
                return;
            }
            let mut seq = std::mem::take(&mut self.keyseq);
            seq.push(tok);
            // after an operator (d c y …) the keys are its motion: :omap's mappings, not :nmap's
            let km = if self.docs[self.cur].ed.op_pending() { "o" } else { "n" };
            let feed = if self.noremap > 0 || self.docs[self.cur].ed.prefix_pending() { Feed::None } else { self.with_lua(|e| e.feed(km, &seq)) };
            return self.after_feed(feed, seq);
        }
        // the completion popup takes its keys first
        if mode == Mode::Insert && self.comp.is_some() {
            let n = self.comp.as_ref().map(|c| c.shown.len()).unwrap_or(0);
            // keyword completion: CTRL-N / CTRL-P put each choice in (unless completeopt has
            // noinsert), CTRL-Y and Enter take it, CTRL-E brings back what was typed
            if self.comp.as_ref().is_some_and(|c| c.keyword.is_some()) {
                let n = n.max(1);
                let step = match k.code {
                    KeyCode::Char('n') if ctrl => Some(1),
                    KeyCode::Down | KeyCode::Tab => Some(1),
                    KeyCode::Char('p') if ctrl => Some(n - 1),
                    KeyCode::Up | KeyCode::BackTab => Some(n - 1),
                    _ => None,
                };
                if let Some(s) = step {
                    let c = self.comp.as_mut().unwrap();
                    c.sel = (c.sel + s) % n;
                    if !self.setting(self.cur, "completeopt").split(',').any(|o| o.trim() == "noinsert") {
                        self.comp_put();
                    }
                    return;
                }
                match k.code {
                    KeyCode::Char('y') if ctrl => return self.comp_accept(),
                    KeyCode::Enter => return self.comp_accept(),
                    KeyCode::Char('e') if ctrl => return self.comp_cancel(),
                    _ => {}
                }
            }
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
        // CTRL-X: the next CTRL key picks the completion (CTRL-N CTRL-K CTRL-T CTRL-U CTRL-O CTRL-L
        // CTRL-F); any other key leaves the mode and is typed as usual
        if mode == Mode::Insert && std::mem::take(&mut self.ctrlx_pending) {
            self.msg = None;
            if let KeyCode::Char(c) = k.code {
                if ctrl {
                    return self.ctrl_x(c);
                }
            }
        }
        if mode == Mode::Insert && ctrl && self.comp.is_none() && k.code == KeyCode::Char('x') {
            self.ctrlx_pending = true;
            return self.say("-- ^X mode (^N^P ^K ^T ^U ^O ^L ^F)");
        }
        // CTRL-N / CTRL-P with no menu: Vim's keyword completion
        if mode == Mode::Insert && ctrl && self.comp.is_none() && matches!(k.code, KeyCode::Char('n' | 'p')) {
            return self.keyword_complete(k.code == KeyCode::Char('p'));
        }
        // a snippet: Tab / S-Tab to its next / previous stop (Neovim's defaults), a placeholder
        // typed over
        if mode == Mode::Insert && self.snip.is_some() {
            match k.code {
                KeyCode::Tab if self.snippet_jump(1) => return,
                KeyCode::BackTab if self.snippet_jump(-1) => return,
                _ => {}
            }
            if self.snippet_before_key(&k) {
                self.snippet_check();
                return self.comp_after_key(None);
            }
        }
        let typed = match k.code {
            KeyCode::Char(c) if !ctrl => Some(c),
            KeyCode::Backspace => None,
            _ => Some('\0'),
        };
        // insert mode: a key that ends a word expands an abbreviation first
        if mode == Mode::Insert {
            match k.code {
                KeyCode::Char(c) if !ctrl => self.expand_abbrev(c),
                KeyCode::Enter => self.expand_abbrev('\n'),
                KeyCode::Tab => self.expand_abbrev('\t'),
                KeyCode::Esc => self.expand_abbrev('\x1b'),
                _ => {}
            }
        }
        self.edit_key(ek);
        // a character typed: the input method (C-\) may turn what is before the cursor into another
        if self.input_method_on().is_some() && matches!(typed, Some(c) if c != '\0') {
            self.input_method_compose();
        }
        if mode == Mode::Insert {
            self.comp_after_key(typed);
        }
    }

    /// What the engine did with keys: ran a mapping, waits for more, or left them to Vim.
    pub(crate) fn after_feed(&mut self, feed: Feed, seq: Vec<String>) {
        match feed {
            Feed::Ran(fx) => {
                self.menu.clear();
                self.ran_keys = seq.len();
                // C-x z repeats it (not itself)
                if !fx.iter().any(|e| matches!(e, Effect::Ui(n, _) if n == "repeat")) && !fx.is_empty() {
                    self.last_fx = fx.clone();
                }
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
                    let typed = t.chars().count() == 1;
                    for k in zett_edit::keys::parse(&t) {
                        self.edit_key(k);
                    }
                    // a character typed: the input method may turn what is before the cursor into another
                    if typed && self.input_method_on().is_some() {
                        self.input_method_compose();
                    }
                }
            }
        }
    }

    /// The input method (C-\) on what was just typed: a finished sequence becomes its character.
    /// The input method typing goes through: C-\'s, or Vim's way, keymap (one of zett's methods by
    /// name) with iminsert=1 or 2, in insert mode.
    pub(crate) fn input_method_on(&self) -> Option<String> {
        if self.input_method.is_some() {
            return self.input_method.clone();
        }
        let km = self.setting(self.cur, "keymap");
        (self.num(self.cur, "iminsert") > 0 && crate::inputmethod::METHODS.iter().any(|(n, _)| *n == km)).then_some(km)
    }

    pub(crate) fn input_method_compose(&mut self) {
        let Some(method) = self.input_method_on() else { return };
        let d = &self.docs[self.cur];
        // only while typing goes into the text (the Emacs profile, Vim's insert mode)
        if !d.ed.is_emacs() && d.ed.mode() != zett_edit::Mode::Insert {
            return;
        }
        let at = d.ed.buf.to_char(d.ed.cursor());
        let start = at.saturating_sub(32);
        let before: String = d.ed.text().chars().skip(start).take(at - start).collect();
        if let Some((back, put)) = crate::inputmethod::compose(&method, &before) {
            let line = d.ed.cursor().line;
            self.doc().ed.replace_text(at - back, at, &put);
            self.doc().touched(line);
            if self.input_method_once {
                self.input_method_once = false;
                self.input_method = None;
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
        // a buffer seen for the first time: readonly when its file cannot be written, and its modeline
        for i in 0..self.docs.len() {
            if !std::mem::replace(&mut self.docs[i].seen, true) {
                if self.docs[i].ed.buf.path().and_then(|p| std::fs::metadata(p).ok()).is_some_and(|m| m.permissions().readonly()) {
                    self.docs[i].local.insert("readonly".into(), "true".into());
                }
                if self.docs[i].ed.buf.path().is_some() {
                    self.take_disk(i);
                }
                // the file type it was opened as, for :set ft? and to switch from
                let ft = crate::settings::ft_key(&self.docs[i].syntax);
                self.docs[i].local.entry("filetype".into()).or_insert(ft);
                self.apply_modeline(i);
                let keep = self.cur;
                self.cur = i;
                self.emit("buf_read");
                self.cur = keep;
            }
        }
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
        if self.emacs_keys() {
            return "e";
        }
        match self.docs[self.cur].ed.mode() {
            Mode::Insert => "i",
            Mode::Visual | Mode::VisualLine | Mode::VisualBlock => "v",
            _ => "n",
        }
    }

    /// Emacs profile: every key goes through the keymap (chords like C-x C-f wait for more);
    /// an unbound printable key types itself, anything else unbound says so.
    pub(crate) fn emacs_key(&mut self, k: &KeyEvent) {
        // help pages: RET and C-] follow a link, C-t comes back (as Emacs's help buttons and Vim's tags)
        if self.char_for.is_none() && self.keyseq.is_empty() && self.docs[self.cur].name.starts_with("help: ") {
            let tok = token(k);
            if matches!(tok.as_str(), "<CR>" | "<C-]>") && self.follow_help() {
                return;
            }
            if tok == "<C-t>" && self.help_back() {
                return;
            }
        }
        // a command asked for a key (C-x r s, M-z, C-q): this one is it
        if let Some((name, arg)) = self.char_for.take() {
            self.msg = None;
            let ctrl = k.modifiers.contains(KeyModifiers::CONTROL);
            let c = match k.code {
                KeyCode::Char('g') if ctrl && name != "quoted-insert" => return self.say("Quit"),
                // C-q C-a: the control character itself
                KeyCode::Char(c) if ctrl && c.is_ascii_alphabetic() => char::from(c.to_ascii_lowercase() as u8 - b'a' + 1),
                KeyCode::Char(c) => c,
                KeyCode::Tab => '\t',
                KeyCode::Enter => '\n',
                KeyCode::Esc => '\u{1b}',
                _ => return self.say("not a character"),
            };
            // registers that hold a buffer, a file or the windows (C-x r B F w): the front end's
            if let Some(what) = name.strip_prefix("place:") {
                return self.place_to_register(what, c);
            }
            // confirm's question: y saves and closes, n closes without saving, anything else keeps it
            if name == "confirm:close" {
                match c.to_ascii_lowercase() {
                    'y' => {
                        self.save(None);
                        if !self.docs[self.cur].dirty() {
                            self.close(true);
                        }
                    }
                    'n' => self.close(true),
                    _ => {}
                }
                return;
            }
            if name == "jump-to-register" && self.jump_to_place(c) {
                return;
            }
            let before = self.docs[self.cur].ed.cursor().line;
            let kills = self.docs[self.cur].ed.kill_count();
            self.doc().ed.exec_char(&name, c, arg);
            if self.docs[self.cur].ed.kill_count() != kills {
                if let Some(t) = self.docs[self.cur].ed.last_kill() {
                    clipboard::copy(&t);
                }
            }
            self.doc().touched(before);
            self.edit_at = Instant::now();
            return;
        }
        // the completion popup (C-M-i, or after typing) takes its keys first
        if self.comp.is_some() && !self.meta {
            let n = self.comp.as_ref().map(|c| c.shown.len()).unwrap_or(0).max(1);
            let ctrl = k.modifiers.contains(KeyModifiers::CONTROL);
            let c = self.comp.as_mut().unwrap();
            match k.code {
                KeyCode::Tab | KeyCode::Down => return c.sel = (c.sel + 1) % n,
                KeyCode::Char('n') if ctrl => return c.sel = (c.sel + 1) % n,
                KeyCode::BackTab | KeyCode::Up => return c.sel = (c.sel + n - 1) % n,
                KeyCode::Char('p') if ctrl => return c.sel = (c.sel + n - 1) % n,
                KeyCode::Enter => return self.comp_accept(),
                KeyCode::Char('g') if ctrl => {
                    self.comp = None;
                    return;
                }
                KeyCode::Esc => self.comp = None,
                _ => {}
            }
        }
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
        // C-h l: the last keys typed
        self.lossage.push_back(tok.clone());
        if self.lossage.len() > 300 {
            self.lossage.pop_front();
        }
        if let Some(m) = self.kmacro.as_mut() {
            m.push(tok.clone());
        }
        if std::mem::take(&mut self.repeat_again) && tok == "z" {
            return self.repeat_last();
        }
        // C-x e e e: each e plays the macro again
        if std::mem::take(&mut self.kmacro_again) && tok == "e" {
            return self.kmacro_call(1);
        }
        // C-h k: these keys are described, not run
        if self.describing.is_some() {
            return self.describe_key_step(tok);
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
                        // abbrev-mode: a character that ends a word expands the one before it
                        if self.abbrev_mode && !(c.is_alphanumeric() || c == '_') {
                            self.expand_abbrev_here();
                        }
                        let times = self.arg.take().map_or(1, |n| n.unsigned_abs().max(1));
                        let before = self.docs[self.cur].ed.cursor().line;
                        for _ in 0..times {
                            self.doc().ed.self_insert(c);
                        }
                        self.doc().touched(before);
                        self.edit_at = Instant::now();
                        if self.input_method_on().is_some() {
                            self.input_method_compose();
                        }
                        // the language server's completions while typing, as in Vim's insert mode
                        self.comp_after_key(Some(c));
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
                // a Vim mapping's keys, typed in again (through the mappings unless :noremap)
                Effect::Keys(keys, remap) => self.replay(&keys, remap),
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
                Effect::Ui(name, arg) if self.emacs_ui(&name, &arg) => {}
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
                        crate::lsp::apply_config(&mut e);
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
                // it reads a key first (the next one emacs_key sees)
                Effect::Edit(name, n) if zett_edit::Editor::char_prompt(zett_edit::emacs::resolve(&name)).is_some() => {
                    let arg = self.arg.take().or((n > 1).then_some(n as i64));
                    let prompt = zett_edit::Editor::char_prompt(&name).unwrap_or_default();
                    self.char_for = Some((name, arg));
                    self.say(prompt);
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
                    if self.abbrev_mode && name == "newline" {
                        self.expand_abbrev_here();
                    }
                    // M-g g with no argument asks for the number
                    if let (Some(p), None) = (zett_edit::screen::asks_number(zett_edit::emacs::resolve(&name)), arg) {
                        self.prompt(&format!("{p}\tnum:{name} {{1}}"));
                        continue;
                    }
                    self.doc().ed.exec_arg(&name, arg);
                    // messages and window moves the command asked for (C-x =, C-v, C-l)
                    self.editor_requests();
                    // the completion popup: DEL narrows it again, any other command closes it
                    if self.comp.is_some() {
                        if matches!(zett_edit::emacs::resolve(&name), "delete-backward-char") {
                            self.comp_after_key(None);
                        } else {
                            self.comp = None;
                        }
                    }
                    // C-SPC: where the mark went joins the global mark ring (C-x C-@)
                    if name == "set-mark-command" && arg.is_none() {
                        let ed = &self.docs[self.cur].ed;
                        let at = ed.buf.to_char(ed.cursor());
                        let doc = self.docs[self.cur].name.clone();
                        self.global_marks.retain(|(d, _)| *d != doc);
                        self.global_marks.push((doc, at));
                        if self.global_marks.len() > 16 {
                            self.global_marks.remove(0);
                        }
                    }
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
        // a button held (scrolling, selecting) or the wheel turning: the help circles wait
        match m.kind {
            MouseEventKind::Down(_) | MouseEventKind::Drag(_) => {
                self.mouse_held = true;
                self.fx_hover = None;
            }
            MouseEventKind::Up(_) => {
                self.mouse_held = false;
                self.fx_wait(crate::emacs::FX_QUIET);
            }
            // a plain move: no button is down (a release zeit kept, off the pane, ends the hold too)
            MouseEventKind::Moved => self.mouse_held = false,
            MouseEventKind::ScrollDown | MouseEventKind::ScrollUp | MouseEventKind::ScrollLeft | MouseEventKind::ScrollRight => {
                self.fx_wait(crate::emacs::FX_QUIET);
                self.fx_hover = None;
            }
        }
        // the wheel over the completion menu moves through the menu, not the text under it
        if matches!(m.kind, MouseEventKind::ScrollDown | MouseEventKind::ScrollUp) && self.comp.is_some() && self.flag(self.cur, "pumwheel") {
            let over = crate::lsp::COMP_AT.with(|c| c.get()).is_some_and(|r| m.column >= r.x && m.column < r.x + r.width && m.row >= r.y && m.row < r.y + r.height);
            if over {
                if let Some(c) = self.comp.as_mut() {
                    let n = c.shown.len().max(1);
                    c.sel = if m.kind == MouseEventKind::ScrollDown { (c.sel + 1).min(n - 1) } else { c.sel.saturating_sub(1) };
                }
                return;
            }
        }
        self.mouse_at = (m.column, m.row);
        if self.panel_mouse(&m) {
            return;
        }
        // mouse: the modes it works in (n v i c h, a for all; empty: off)
        let mo = self.setting(self.cur, "mouse");
        let mode = if self.cmdline.is_some() {
            'c'
        } else if self.docs[self.cur].name.starts_with("help:") {
            'h'
        } else {
            match self.docs[self.cur].ed.mode() {
                Mode::Insert => 'i',
                Mode::Normal => 'n',
                _ => 'v',
            }
        };
        if !mo.contains('a') && !mo.contains(mode) && !(mode == 'h' && mo.contains('n')) {
            return;
        }
        if let MouseEventKind::Down(_) = m.kind {
            self.engine.focus_panel(None);
        }
        // status lines and the lines between windows: drag to resize, C-middle-click to split
        if self.window_mouse(&m) {
            return;
        }
        // a mapping for this mouse action, as Neovim has them: :map <C-LeftMouse> …, <M-LeftDrag>,
        // <S-ScrollWheelUp>, <2-LeftMouse> (a double click); it replaces what the mouse does here
        if self.noremap == 0 {
            let within = Duration::from_millis(self.num(self.cur, "mousetime") as u64);
            let clicks = match (m.kind, self.click) {
                (MouseEventKind::Down(_), Some((t, c, r, n))) if t.elapsed() <= within && (c, r) == (m.column, m.row) => n + 1,
                _ => 1,
            };
            if let Some(tok) = mouse_token(&m, clicks) {
                let km = match mode {
                    // the Emacs profile's keymap has its mouse keys too (<C-LeftMouse> …)
                    _ if self.emacs_keys() && mode != 'c' => "e",
                    'c' => "c",
                    'i' => "i",
                    'v' => "v",
                    _ if self.docs[self.cur].ed.op_pending() => "o",
                    _ => "n",
                };
                if self.engine.has_maps(km) {
                    self.last_mouse = Some(m);
                    match self.with_lua(|e| e.feed(km, std::slice::from_ref(&tok))) {
                        Feed::None => {}
                        // (a <LeftMouse> in the mapping counts its own clicks, as the mouse does)
                        feed => return self.after_feed(feed, vec![tok]),
                    }
                }
            }
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
        // where the mouse is in the text: line and character column (tabs are wide)
        let pos_at = |app: &App, col: u16, row: u16| -> Option<Pos> {
            if row < a.y || row >= a.y + a.height || col < a.x + app.gutter {
                return None;
            }
            let d = &app.docs[app.cur];
            let line = (d.top + (row - a.y) as usize).min(d.ed.buf.len_lines().saturating_sub(1));
            let want = d.left + (col - a.x - app.gutter) as usize;
            let (mut sc, mut c) = (0, 0);
            for ch in d.ed.buf.line(line).chars().filter(|&ch| ch != '\n') {
                let w = if ch == '\t' { crate::tab_at(sc) } else { 1 };
                if sc + w > want {
                    break;
                }
                sc += w;
                c += 1;
            }
            Some(Pos { line, col: c })
        };
        // mousescroll: lines (ver) and columns (hor) a wheel step moves
        let ms = self.setting(self.cur, "mousescroll");
        let step = |k: &str, def: usize| ms.split(',').find_map(|p| p.strip_prefix(k)?.strip_prefix(':')?.parse().ok()).unwrap_or(def);
        let vim = !self.docs[self.cur].ed.is_emacs();
        let visual = matches!(self.docs[self.cur].ed.mode(), Mode::Visual | Mode::VisualLine | Mode::VisualBlock);
        if m.kind == MouseEventKind::Moved {
            let p = pos_at(self, m.column, m.row);
            return self.help_hover(p, m.row);
        }
        match m.kind {
            MouseEventKind::ScrollDown | MouseEventKind::ScrollUp => {
                let n = step("ver", 3);
                let d = &mut self.docs[self.cur];
                let last = d.ed.buf.len_lines().saturating_sub(1);
                if m.kind == MouseEventKind::ScrollDown { d.top = (d.top + n).min(last) } else { d.top = d.top.saturating_sub(n) }
                // the cursor stays on screen, as in Vim
                let c = d.ed.cursor();
                let bottom = d.top + a.height.saturating_sub(1) as usize;
                if c.line < d.top || c.line > bottom {
                    let line = c.line.clamp(d.top, bottom.min(last));
                    d.ed.set_cursor(Pos { line, col: c.col });
                }
            }
            MouseEventKind::ScrollLeft | MouseEventKind::ScrollRight => {
                let n = step("hor", 6);
                let d = &mut self.docs[self.cur];
                d.left = if m.kind == MouseEventKind::ScrollRight { d.left + n } else { d.left.saturating_sub(n) };
            }
            MouseEventKind::Down(MouseButton::Left) => {
                let Some(p) = pos_at(self, m.column, m.row) else { return };
                // mousetime: clicks this close together on one place count as a double or triple click
                let within = Duration::from_millis(self.num(self.cur, "mousetime") as u64);
                let n = match self.click {
                    Some((t, c, r, n)) if t.elapsed() <= within && c == m.column && r == m.row => n % 3 + 1,
                    _ => 1,
                };
                self.click = Some((Instant::now(), m.column, m.row, n));
                self.drag_from = Some(p);
                let ed = &mut self.docs[self.cur].ed;
                if vim && visual {
                    ed.feed("<Esc>");
                }
                ed.set_cursor(p);
                // a double click selects the word, a triple click the line
                match (vim, n) {
                    (true, 2) => ed.feed("viw"),
                    (true, 3) => ed.feed("V"),
                    _ => {}
                }
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                let (Some(p), Some(from)) = (pos_at(self, m.column, m.row), self.drag_from) else { return };
                let ed = &mut self.docs[self.cur].ed;
                // dragging selects from where the button went down
                if vim && !visual {
                    ed.set_cursor(from);
                    ed.feed("v");
                } else if !vim && ed.region().is_none() {
                    ed.set_cursor(from);
                    ed.exec("set-mark-command", 1);
                }
                ed.set_cursor(p);
            }
            MouseEventKind::Down(MouseButton::Right) => {
                let Some(p) = pos_at(self, m.column, m.row) else { return };
                // mousemodel: extend makes or grows the selection to the click; popup_setpos moves
                // the cursor (zett has no pop-up menu), popup leaves it
                let model = self.setting(self.cur, "mousemodel");
                let ed = &mut self.docs[self.cur].ed;
                match model.as_str() {
                    "extend" if vim => {
                        if !visual {
                            ed.feed("v");
                        }
                        ed.set_cursor(p);
                    }
                    "extend" => {
                        if ed.region().is_none() {
                            ed.exec("set-mark-command", 1);
                        }
                        ed.set_cursor(p);
                    }
                    "popup" => {
                        self.pick(Pick::Menu("PopUp".into()));
                    }
                    // popup_setpos (Neovim's default): the cursor there, then the PopUp menu
                    _ => {
                        ed.set_cursor(p);
                        if !self.menu_items("PopUp").is_empty() {
                            self.pick(Pick::Menu("PopUp".into()));
                        }
                    }
                }
            }
            MouseEventKind::Down(MouseButton::Middle) => {
                let Some(p) = pos_at(self, m.column, m.row) else { return };
                // the selection (the * register, the system's) put where the click was
                let ed = &mut self.docs[self.cur].ed;
                ed.set_cursor(p);
                if vim {
                    ed.feed("\"*P");
                } else {
                    ed.exec("yank", 1);
                }
            }
            _ => {}
        }
    }

}

impl App {
    /// Keys in Vim notation ("dd", ":w<CR>", "<C-w>j") as if typed: the : line, the mappings (unless
    /// `remap` is false) and the editor all see them. Deep enough nesting stops (maxmapdepth).
    pub(crate) fn replay(&mut self, keys: &str, remap: bool) {
        // a mapping with mouse actions in it (<LeftMouse>gd): those are done where the mouse was
        if let Some(at) = self.last_mouse.filter(|_| { let l = keys.to_lowercase(); l.contains("mouse>") || l.contains("drag>") || l.contains("release>") || l.contains("scrollwheel") }) {
            for tok in zett_script::keys(keys) {
                match mouse_key(&tok, &at) {
                    Some(ev) => {
                        // what the mouse does by itself: no mapping again
                        self.noremap += 1;
                        self.mouse(ev);
                        self.noremap -= 1;
                    }
                    None => self.replay(&tok, remap),
                }
            }
            return;
        }
        // noremap: every mapping as if :noremap
        let remap = remap && self.flag(self.cur, "remap");
        let depth = self.num(self.cur, "maxmapdepth").max(1);
        if self.replaying >= depth {
            return self.say("E223: recursive mapping");
        }
        self.replaying += 1;
        if !remap {
            self.noremap += 1;
        }
        for k in zett_edit::keys::parse(keys) {
            self.key(key_event(&k));
        }
        if !remap {
            self.noremap -= 1;
        }
        self.replaying -= 1;
    }
}

/// zett-edit's key as the terminal's key event.
pub(crate) fn key_event(k: &zett_edit::keys::Key) -> KeyEvent {
    use zett_edit::keys::Code as C;
    let code = match k.code {
        C::Char(c) => KeyCode::Char(c),
        C::Esc => KeyCode::Esc,
        C::Enter => KeyCode::Enter,
        C::Backspace => KeyCode::Backspace,
        C::Delete => KeyCode::Delete,
        C::Tab if k.shift => KeyCode::BackTab,
        C::Tab => KeyCode::Tab,
        C::Left => KeyCode::Left,
        C::Right => KeyCode::Right,
        C::Up => KeyCode::Up,
        C::Down => KeyCode::Down,
        C::Home => KeyCode::Home,
        C::End => KeyCode::End,
        C::PageUp => KeyCode::PageUp,
        C::PageDown => KeyCode::PageDown,
        C::Insert => KeyCode::Insert,
        C::F(n) => KeyCode::F(n),
        _ => KeyCode::Null,
    };
    let mut m = KeyModifiers::NONE;
    if k.ctrl {
        m |= KeyModifiers::CONTROL;
    }
    if k.alt {
        m |= KeyModifiers::ALT;
    }
    if k.shift && !matches!(code, KeyCode::BackTab | KeyCode::Char(_)) {
        m |= KeyModifiers::SHIFT;
    }
    KeyEvent::new(code, m)
}

/// A mouse event as Vim names it in mappings: <LeftMouse> <C-LeftDrag> <M-RightRelease>
/// <S-ScrollWheelUp> <2-LeftMouse> (clicks in a row); None for a move.
pub(crate) fn mouse_token(m: &ratatui::crossterm::event::MouseEvent, clicks: u8) -> Option<String> {
    use ratatui::crossterm::event::{KeyModifiers, MouseButton, MouseEventKind};
    let button = |b: MouseButton| match b {
        MouseButton::Left => "Left",
        MouseButton::Right => "Right",
        MouseButton::Middle => "Middle",
    };
    let name = match m.kind {
        MouseEventKind::Down(b) => format!("{}{}Mouse", if clicks > 1 { format!("{}-", clicks.min(4)) } else { String::new() }, button(b)),
        MouseEventKind::Drag(b) => format!("{}Drag", button(b)),
        MouseEventKind::Up(b) => format!("{}Release", button(b)),
        MouseEventKind::ScrollUp => "ScrollWheelUp".into(),
        MouseEventKind::ScrollDown => "ScrollWheelDown".into(),
        MouseEventKind::ScrollLeft => "ScrollWheelLeft".into(),
        MouseEventKind::ScrollRight => "ScrollWheelRight".into(),
        MouseEventKind::Moved => return None,
    };
    let mut mods = String::new();
    if m.modifiers.contains(KeyModifiers::CONTROL) {
        mods.push_str("C-");
    }
    if m.modifiers.contains(KeyModifiers::ALT) {
        mods.push_str("M-");
    }
    if m.modifiers.contains(KeyModifiers::SHIFT) {
        mods.push_str("S-");
    }
    Some(format!("<{mods}{name}>"))
}

/// A mapping's key that is a mouse action (<LeftMouse>, <C-ScrollWheelUp> …): its event, at `at`.
pub(crate) fn mouse_key(tok: &str, at: &ratatui::crossterm::event::MouseEvent) -> Option<ratatui::crossterm::event::MouseEvent> {
    use ratatui::crossterm::event::{KeyModifiers, MouseButton, MouseEventKind};
    let inner = tok.strip_prefix('<')?.strip_suffix('>')?;
    let mut rest = inner;
    let mut mods = KeyModifiers::NONE;
    loop {
        let l = rest.to_lowercase();
        if l.starts_with("c-") {
            mods |= KeyModifiers::CONTROL;
        } else if l.starts_with("m-") || l.starts_with("a-") {
            mods |= KeyModifiers::ALT;
        } else if l.starts_with("s-") {
            mods |= KeyModifiers::SHIFT;
        } else if l.len() > 2 && l.as_bytes()[0].is_ascii_digit() && l.as_bytes()[1] == b'-' {
            // <2-LeftMouse>: the click itself (a double click comes from two of them)
        } else {
            break;
        }
        rest = &rest[2..];
    }
    let l = rest.to_lowercase();
    let b = |s: &str| match s {
        "left" => Some(MouseButton::Left),
        "right" => Some(MouseButton::Right),
        "middle" => Some(MouseButton::Middle),
        _ => None,
    };
    let kind = if let Some(s) = l.strip_suffix("mouse") {
        MouseEventKind::Down(b(s)?)
    } else if let Some(s) = l.strip_suffix("drag") {
        MouseEventKind::Drag(b(s)?)
    } else if let Some(s) = l.strip_suffix("release") {
        MouseEventKind::Up(b(s)?)
    } else {
        match l.as_str() {
            "scrollwheelup" => MouseEventKind::ScrollUp,
            "scrollwheeldown" => MouseEventKind::ScrollDown,
            "scrollwheelleft" => MouseEventKind::ScrollLeft,
            "scrollwheelright" => MouseEventKind::ScrollRight,
            _ => return None,
        }
    };
    Some(ratatui::crossterm::event::MouseEvent { kind, column: at.column, row: at.row, modifiers: mods })
}
