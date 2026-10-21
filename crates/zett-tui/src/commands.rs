//! Help, files and the : line: opening, picking, saving, closing, and every : command.

use super::*;

impl App {
    /// A `:` line into the history: once (an older copy goes), at most `history` lines kept.
    pub(crate) fn remember_cmd(&mut self, l: String) {
        self.cmd_hist.retain(|h| *h != l);
        self.cmd_hist.push(l);
        let max = self.num(self.cur, "history").max(1);
        if self.cmd_hist.len() > max {
            let cut = self.cmd_hist.len() - max;
            self.cmd_hist.drain(..cut);
        }
    }

    pub(crate) fn help(&mut self, what: &str) {
        let rows: Vec<(String, String)> = if what == "commands" {
            self.engine.commands().into_iter().map(|(n, d, _)| (n, d)).collect()
        } else {
            let leader = self.engine.leader();
            self.engine.bindings(self.keymode()).into_iter().map(|(k, d)| (k.replace("<leader>", if leader == "<Space>" { "SPC " } else { &leader }), d)).collect()
        };
        self.popup = Some(Popup::Help(if what == "commands" { "Commands".into() } else { "Keys".into() }, rows));
    }

    pub(crate) fn open(&mut self, p: &Path) {
        // already open: show it
        let name = p.display().to_string();
        // by the real path too: "notes/a.md" and "/home/…/notes/a.md" are one buffer
        let real = std::fs::canonicalize(p).ok();
        if let Some(i) = self.docs.iter().position(|d| d.name == name || (real.is_some() && std::fs::canonicalize(&d.name).ok() == real)) {
            self.show_doc(i);
            return;
        }
        if p.is_file() {
            doom::remember(p);
        }
        match Doc::open(p, &self.ss) {
            Ok(d) => {
                // an untouched scratch buffer gives way
                if self.docs.len() == 1 && self.docs[0].name == "[scratch]" && !self.docs[0].dirty() {
                    self.docs.clear();
                }
                self.docs.push(d);
                let i = self.docs.len() - 1;
                self.show_doc(i);
                self.lsp_open(i);
            }
            Err(e) => self.say(format!("{}: {e}", p.display())),
        }
    }

    pub(crate) fn open_picker(&mut self) {
        self.pick(Pick::Files(self.root.clone()));
    }

    pub(crate) fn cmdline_key(&mut self, k: KeyEvent) {
        let cedit = self.setting(self.cur, "cedit");
        let Some(line) = self.cmdline.as_mut() else { return };
        // the commit message for the git buffer
        if self.committing && matches!(k.code, KeyCode::Esc | KeyCode::Enter) {
            let l = std::mem::take(line);
            self.cmdline = None;
            self.prompt = None;
            self.committing = false;
            if k.code == KeyCode::Enter {
                self.git_commit(&l);
            }
            return;
        }
        // the question for the agent: its own prompt, not a Lua one
        if self.asking && matches!(k.code, KeyCode::Esc | KeyCode::Enter) {
            let l = std::mem::take(line);
            self.cmdline = None;
            self.prompt = None;
            self.asking = false;
            if k.code == KeyCode::Enter {
                self.agent_start(&l);
            }
            return;
        }
        match k.code {
            KeyCode::Esc => {
                self.cmdline = None;
                if self.prompt.take().is_some() {
                    let fx = self.with_lua(|e| e.input_done(None));
                    self.after_lua(fx);
                }
            }
            KeyCode::Enter => {
                expand_abbrev(line, &self.cabbrevs);
                let l = std::mem::take(line);
                self.cmdline = None;
                if self.prompt.take().is_some() {
                    let fx = self.with_lua(|e| e.input_done(Some(l)));
                    return self.after_lua(fx);
                }
                if !l.trim().is_empty() && self.prompt.is_none() {
                    self.remember_cmd(l.trim().to_string());
                }
                self.command(l.trim());
            }
            KeyCode::Backspace => {
                if line.pop().is_none() && self.prompt.is_none() {
                    self.cmdline = None;
                }
            }
            // cedit (CTRL-F): the line so far into the command-line window
            KeyCode::Char(c) if k.modifiers.contains(KeyModifiers::CONTROL) && self.prompt.is_none() && cedit_is(&cedit, c) => {
                let l = std::mem::take(line);
                self.cmdline = None;
                self.cmd_window(':', Some(l));
            }
            KeyCode::Tab if self.prompt.is_some() => {}
            KeyCode::Tab => self.complete_cmdline(),
            KeyCode::Down => self.cmd_sel += 1,
            KeyCode::Up => self.cmd_sel = self.cmd_sel.saturating_sub(1),
            KeyCode::Char('n') if k.modifiers.contains(KeyModifiers::CONTROL) => self.cmd_sel += 1,
            KeyCode::Char('p') if k.modifiers.contains(KeyModifiers::CONTROL) => self.cmd_sel = self.cmd_sel.saturating_sub(1),
            KeyCode::Char(c) => {
                // :cabbrev: a word before a key that cannot be part of one
                if !(c.is_alphanumeric() || c == '_') {
                    expand_abbrev(line, &self.cabbrevs);
                }
                line.push(c);
                self.cmd_sel = 0;
            }
            _ => {}
        }
    }

    /// A `:` line: Vim's file commands, then zett's commands and `lua …`.
    pub(crate) fn command(&mut self, l: &str) {
        // Neovim's commands with ranges and the rest of its everyday ones
        if self.ex(l) {
            return;
        }
        let (cmd, arg) = l.split_once(' ').map(|(a, b)| (a, b.trim())).unwrap_or((l, ""));
        match cmd {
            "" => {}
            "w" | "w!" => {
                // w!: also over readonly
                self.bang = cmd.ends_with('!');
                self.save((!arg.is_empty()).then(|| PathBuf::from(arg)));
                self.bang = false;
            }
            "q" | "quit" => {
                if !self.close_win() {
                    self.close(false);
                }
            }
            "compile" => self.compile(arg),
            "h" | "help" => self.open_help(arg),
            "set" | "se" => self.set_command(arg),
            "setlocal" | "setl" => {
                for a in crate::settings::set_args(arg) {
                    if !self.set_core(&a, true) {
                        self.say(format!("{a}: :setlocal takes a core setting (:set lists them)"));
                    }
                }
            }
            "hi" | "highlight" => self.hi_command(arg),
            "rename" => self.lsp_rename(arg),
            "wa" | "wall" => {
                let keep = self.cur;
                for i in 0..self.docs.len() {
                    if self.docs[i].dirty() && self.docs[i].ed.buf.path().is_some() {
                        self.cur = i;
                        self.save(None);
                    }
                }
                self.cur = keep;
            }
            "sp" | "split" | "vs" | "vsplit" => {
                self.doom_ui("split", if cmd.starts_with('v') { "right" } else { "below" });
                // :split FILE: that file in the new window
                if !arg.is_empty() {
                    self.command(&format!("e {arg}"));
                }
            }
            "only" => {
                self.doom_ui("onlyWin", "");
            }
            "q!" | "quit!" => self.close(true),
            "qa" | "qall" => {
                if self.docs.iter().any(|d| d.dirty()) {
                    self.say("unsaved changes: :wqa saves them, :qa! drops them");
                } else {
                    self.quit = true;
                }
            }
            "qa!" | "qall!" => self.quit = true,
            "wq" | "x" => {
                self.save(None);
                if !self.docs[self.cur].dirty() {
                    self.close(false);
                }
            }
            "wqa" | "xa" => {
                for i in 0..self.docs.len() {
                    self.cur = i;
                    if self.docs[i].dirty() {
                        self.save(None);
                    }
                }
                if !self.docs.iter().any(|d| d.dirty()) {
                    self.quit = true;
                }
            }
            "e" | "edit" if !arg.is_empty() => {
                let p = if Path::new(arg).is_absolute() { PathBuf::from(arg) } else { self.root.join(arg) };
                self.open(&p);
            }
            "e" | "edit" | "find" | "files" => self.open_picker(),
            "bn" | "bnext" => self.show_doc((self.cur + 1) % self.docs.len()),
            "bp" | "bprevious" => self.show_doc((self.cur + self.docs.len() - 1) % self.docs.len()),
            "bd" | "bdelete" => self.close(false),
            "bm" | "bmove" => match arg.trim() {
                "" => self.say("usage: :bmove +1 | -1 | N (its place in the tab line)"),
                a => match (a.strip_prefix('+'), a.strip_prefix('-'), a.parse::<i32>()) {
                    (Some(n), _, _) => self.move_doc(n.parse().unwrap_or(1)),
                    (_, Some(n), _) => self.move_doc(-n.parse().unwrap_or(1)),
                    (_, _, Ok(n)) => self.move_doc(n - 1 - self.cur as i32),
                    _ => self.say(format!("not a place: {a}")),
                },
            },
            "bo" | "bonly" | "bufonly" => self.close_others(),
            "ls" | "buffers" => {
                let list: Vec<String> = self.docs.iter().enumerate().map(|(i, d)| format!("{}{}{}", if i == self.cur { "%" } else { " " }, d.name, if d.dirty() { " [+]" } else { "" })).collect();
                self.say(list.join("   "));
            }
            "lua" => match self.with_lua(|e| e.eval(arg)) {
                Ok((out, fx)) => {
                    self.after_lua(fx);
                    if out != "nil" {
                        self.say(out);
                    }
                }
                Err(e) => self.say(e),
            },
            n if n.chars().all(|c| c.is_ascii_digit()) => {
                // :42 goes to line 42
                let line: usize = n.parse().unwrap_or(1);
                let last = self.docs[self.cur].ed.buf.len_lines().saturating_sub(1);
                self.doc().ed.set_cursor(Pos { line: line.saturating_sub(1).min(last), col: 0 });
            }
            other => match self.with_lua(|e| e.run(other)) {
                Ok(fx) => self.after_lua(fx),
                Err(_) => self.say(format!("not a command: {other}")),
            },
        }
    }

    pub(crate) fn save(&mut self, to: Option<PathBuf>) {
        let root = self.root.clone();
        let d = self.doc();
        if let Some(p) = to {
            let p = if p.is_absolute() { p } else { root.join(p) };
            d.name = p.display().to_string();
            d.ed.buf.set_path(p);
        }
        if d.ed.buf.path().is_none() {
            return self.say("no file name: :w name");
        }
        let i = self.cur;
        if self.flag(i, "readonly") && !self.bang {
            return self.say("E45: 'readonly' option is set (add ! to override)");
        }
        self.emit("write_pre");
        // trim_trailing_whitespace, final_newline (settings.rs)
        self.tidy_for_save(i);
        let lines = self.docs[i].ed.buf.len_lines();
        match self.write_buffer(i) {
            Ok(()) => {
                let m = format!("\"{}\" {lines} lines written", self.docs[i].name);
                self.docs[i].mtime = self.disk_time(i);
                self.say(m);
                let i = self.cur;
                if self.flag(i, "undofile") {
                    self.docs[i].keep_history();
                }
                self.lsp_saved(i);
                self.emit("write");
            }
            Err(e) => self.say(e),
        }
    }

    pub(crate) fn close(&mut self, force: bool) {
        if self.docs[self.cur].dirty() && !force {
            return self.say("unsaved changes: :w saves, :q! drops them");
        }
        let gone = self.cur;
        self.docs.remove(gone);
        if self.docs.is_empty() {
            self.quit = true;
            self.docs.push(Doc::scratch());
        }
        self.cur = self.cur.min(self.docs.len() - 1);
        self.doc_removed(gone);
        self.last_doc = None;
    }

}

/// cedit ("\x06" or "<C-F>"): the CTRL key that opens the command-line window from the : line.
fn cedit_is(cedit: &str, c: char) -> bool {
    let want = match cedit {
        "" => return false,
        s if s.chars().count() == 1 => s.chars().next().map(|x| ((x as u8) | 0x60) as char),
        s => s.strip_prefix("<C-").and_then(|r| r.strip_suffix('>')).and_then(|r| r.chars().next()).map(|x| x.to_ascii_lowercase()),
    };
    want == Some(c.to_ascii_lowercase())
}

/// The : line's last word as its abbreviation (:cabbrev), when it is one and stands alone.
pub(crate) fn expand_abbrev(line: &mut String, abbrevs: &[(String, String)]) {
    let word = |c: char| c.is_alphanumeric() || c == '_';
    let start = line.char_indices().rev().take_while(|(_, c)| word(*c)).last().map(|(i, _)| i);
    let Some(start) = start else { return };
    if let Some((_, rhs)) = abbrevs.iter().find(|(l, _)| *l == line[start..]) {
        line.replace_range(start.., rhs);
    }
}
