//! Help, files and the : line: opening, picking, saving, closing, and every : command.

use super::*;

impl App {
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
                let l = std::mem::take(line);
                self.cmdline = None;
                if self.prompt.take().is_some() {
                    let fx = self.with_lua(|e| e.input_done(Some(l)));
                    return self.after_lua(fx);
                }
                if !l.trim().is_empty() && self.prompt.is_none() {
                    self.cmd_hist.push(l.trim().to_string());
                }
                self.command(l.trim());
            }
            KeyCode::Backspace => {
                if line.pop().is_none() && self.prompt.is_none() {
                    self.cmdline = None;
                }
            }
            KeyCode::Tab if self.prompt.is_some() => {}
            KeyCode::Tab => self.complete_cmdline(),
            KeyCode::Down => self.cmd_sel += 1,
            KeyCode::Up => self.cmd_sel = self.cmd_sel.saturating_sub(1),
            KeyCode::Char('n') if k.modifiers.contains(KeyModifiers::CONTROL) => self.cmd_sel += 1,
            KeyCode::Char('p') if k.modifiers.contains(KeyModifiers::CONTROL) => self.cmd_sel = self.cmd_sel.saturating_sub(1),
            KeyCode::Char(c) => {
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
            "w" => self.save((!arg.is_empty()).then(|| PathBuf::from(arg))),
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
            "sp" | "split" => {
                self.doom_ui("split", "below");
            }
            "vs" | "vsplit" => {
                self.doom_ui("split", "right");
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
            "cd" if !arg.is_empty() => {
                let p = PathBuf::from(arg);
                if p.is_dir() {
                    self.root = std::fs::canonicalize(&p).unwrap_or(p);
                    self.say(format!("project: {}", self.root.display()));
                }
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
        // trim_trailing_whitespace, final_newline (settings.rs)
        let i = self.cur;
        self.tidy_for_save(i);
        let d = &mut self.docs[i];
        let lines = d.ed.buf.len_lines();
        match d.ed.buf.save() {
            Ok(()) => {
                let m = format!("\"{}\" {lines} lines written", d.name);
                self.say(m);
                let i = self.cur;
                self.docs[i].keep_history();
                self.lsp_saved(i);
                self.emit("write");
            }
            Err(e) => self.say(format!("could not save: {e}")),
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
