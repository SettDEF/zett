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
        self.sync_undodir();
        // already open: show it
        let name = p.display().to_string();
        // by the real path too: "notes/a.md" and "/home/…/notes/a.md" are one buffer
        let real = std::fs::canonicalize(p).ok();
        if let Some(i) = self.docs.iter().position(|d| d.name == name || (real.is_some() && std::fs::canonicalize(&d.name).ok() == real)) {
            self.show_doc(i);
            return;
        }
        // a PDF or a picture: a view of it (viewer.rs), not its bytes as text
        if p.is_file() && crate::viewer::is_view(p) {
            doom::remember(p);
            return self.open_view(p);
        }
        // BufReadPre before a file is read, BufNewFile for one that is not there yet
        let target = p.display().to_string();
        self.fire_autocmds(if p.exists() { "bufreadpre" } else { "bufnewfile" }, Some(&target));
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
        // wildchar / wildcharm: the key that completes (Tab unless set); Tab is then itself
        let tok = crate::token(&k);
        let wild = [self.setting(self.cur, "wildchar"), self.setting(self.cur, "wildcharm")].iter().any(|w| wild_token(w).is_some_and(|t| t.eq_ignore_ascii_case(&tok)));
        let k = match (wild, k.code) {
            (true, _) => KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE),
            (false, KeyCode::Tab) if self.prompt.is_none() => {
                if let Some(l) = self.cmdline.as_mut() {
                    l.push('\t');
                }
                return;
            }
            _ => k,
        };
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
        // CTRL-K {char1} {char2}: a digraph on the line too
        let ctrl = k.modifiers.contains(KeyModifiers::CONTROL);
        if let Some(first) = self.cmd_digraph {
            self.cmd_digraph = None;
            if let KeyCode::Char(c) = k.code {
                if !ctrl {
                    match first {
                        None => self.cmd_digraph = Some(Some(c)),
                        Some(a) => line.push(zett_edit::digraph::lookup(a, c)),
                    }
                }
            }
            return;
        }
        if ctrl && k.code == KeyCode::Char('k') {
            self.cmd_digraph = Some(None);
            return;
        }
        match k.code {
            KeyCode::Esc if self.ex_text.is_some() => {
                self.ex_text = None;
                self.cmdline = None;
            }
            KeyCode::Esc => {
                self.cmdline = None;
                if self.prompt.take().is_some() {
                    let fx = self.with_lua(|e| e.input_done(None));
                    self.after_lua(fx);
                }
            }
            // :append :insert :change: a line of text; "." alone puts them in
            KeyCode::Enter if self.ex_text.is_some() => {
                let l = std::mem::take(line);
                if l != "." {
                    if let Some(t) = self.ex_text.as_mut() {
                        t.2.push(l);
                    }
                    return;
                }
                self.cmdline = None;
                let Some((at, replace, lines)) = self.ex_text.take() else { return };
                match replace {
                    Some((a, b)) => self.replace_lines(a, b, &lines),
                    None if !lines.is_empty() => {
                        let text: String = lines.iter().map(|l| format!("{l}\n")).collect();
                        self.put_lines(at, &text);
                    }
                    None => {}
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
        // Vimscript (:let, :if … :endif, :function … :endfunction, :call): vimscript.rs
        if self.vimscript_line(l) {
            return;
        }
        // cmd | cmd: one after the other (the commands that take | as theirs keep it)
        let parts = bar_split(l);
        if parts.len() > 1 {
            for p in parts {
                self.command(trim_cmd(p.trim_start()));
            }
            return;
        }
        let l = &parts[0];
        // Neovim's commands with ranges and the rest of its everyday ones
        if self.ex(l) {
            return;
        }
        let (cmd, arg) = l.split_once(' ').map(|(a, b)| (a, trim_cmd(b.trim_start()))).unwrap_or((l, ""));
        match cmd {
            "" => {}
            "w" | "w!" => {
                // w!: also over readonly
                self.bang = cmd.ends_with('!');
                self.save((!arg.is_empty()).then(|| PathBuf::from(arg)));
                self.bang = false;
            }
            "q" | "quit" => {
                self.fire_autocmds("quitpre", None);
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
            // ! (Vim: also over a group already linked): the same here, :hi lays over as it is
            "hi" | "highlight" | "hi!" | "highlight!" => self.hi_command(arg),
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
                self.fire_autocmds("quitpre", None);
                self.fire_autocmds("exitpre", None);
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
            // :find NAME: 'findfunc' first, then 'path' (with 'suffixesadd')
            "find" | "fin" if !arg.is_empty() => {
                let name = arg.trim();
                let spec = self.setting(self.cur, "findfunc");
                let from_func = if spec.is_empty() { None } else {
                    self.engine.eval(&format!("vim._zett_findfunc({spec:?}, {name:?})")).ok().map(|r| r.0).filter(|s| !s.is_empty())
                };
                let from = self.docs[self.cur].abs_path();
                match from_func.map(|s| PathBuf::from(s.lines().next().unwrap_or_default())).or_else(|| self.include_file(name, from.as_deref())) {
                    Some(p) => { let p = if p.is_absolute() { p } else { self.root.join(p) }; self.open(&p) }
                    None => self.say(format!("E345: Can't find file \"{name}\" in path")),
                }
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
            "ls" | "buffers" | "ls!" | "buffers!" => {
                // nobuflisted buffers (a plugin's own) show only with :ls!
                let all = arg.starts_with('!') || cmd.ends_with('!');
                let list: Vec<String> = self.docs.iter().enumerate().filter(|(i, _)| all || self.flag(*i, "buflisted")).map(|(i, d)| format!("{}{}{}", if i == self.cur { "%" } else { " " }, d.name, if d.dirty() { " [+]" } else { "" })).collect();
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
        // another file that is there already: only with ! (or writeany)
        if let Some(p) = &to {
            let p = if p.is_absolute() { p.clone() } else { root.join(p) };
            let mine = self.docs[self.cur].abs_path().is_some_and(|m| m == p);
            if p.exists() && !mine && !self.bang && !self.flag(self.cur, "writeany") {
                return self.say("E13: File exists (add ! to override)");
            }
        }
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
        // read as root (C-x x @): written as root too
        if self.docs[i].local.get("sudo").is_some_and(|v| v == "true") {
            self.sudo = Some(true);
            return;
        }
        match self.write_buffer(i) {
            Err(e) if e.contains("Permission denied") => self.say(format!("{e}: M-x sudo-save (or C-x x @ first) writes it as root")),
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
            // confirm: asked instead (input.rs takes the answer)
            if self.flag(self.cur, "confirm") {
                self.char_for = Some(("confirm:close".into(), None));
                return self.say(format!("Save changes to \"{}\"? (y)es, (n)o, (c)ancel", self.docs[self.cur].name));
            }
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

/// A : line cut at its bars into commands, as Vim does: `\|` is a bar of the command's own, and
/// the commands that take the rest of the line (:g :normal :! the maps :command :autocmd :bufdo
/// :windo … :r ! :w !) keep every bar.
pub(crate) fn bar_split(l: &str) -> Vec<String> {
    let t = l.trim_start().trim_start_matches(':');
    // the range first (numbers, . $ % ' marks, +, -, , ;, /pattern/)
    let rest = t.trim_start_matches(|c: char| c.is_ascii_digit() || ".$%,;+-<>'".contains(c) || c.is_ascii_whitespace());
    let name: String = rest.chars().take_while(|c| c.is_ascii_alphabetic()).collect();
    let after = &rest[name.len()..];
    const WHOLE: &[&str] = &[
        "g", "gl", "global", "v", "vg", "vglobal", "norm", "norma", "normal", "com", "comm", "comma", "comman", "command",
        "au", "aut", "auto", "autoc", "autocm", "autocmd", "bufdo", "windo", "tabdo", "argdo", "cdo", "cfdo", "ldo", "lfdo",
        "lua", "luado", "exe", "execute",
    ];
    let maps = name.ends_with("map") || name.ends_with("noremap") || name.ends_with("unmap") || matches!(name.as_str(), "ab" | "abbreviate" | "iab" | "cab" | "no" | "nn" | "ino" | "vn" | "xn" | "cno" | "ono" | "nm" | "im" | "vm" | "xm" | "cm" | "om");
    let shell = rest.starts_with('!') || (matches!(name.as_str(), "r" | "read" | "w" | "write") && after.trim_start().starts_with('!'));
    if WHOLE.contains(&name.as_str()) || maps || shell || !l.contains('|') {
        return vec![l.to_string()];
    }
    let mut out = vec![String::new()];
    let mut it = l.chars().peekable();
    while let Some(c) = it.next() {
        match c {
            '\\' if it.peek() == Some(&'|') => {
                out.last_mut().unwrap().push('|');
                it.next();
            }
            '|' => out.push(String::new()),
            c => out.last_mut().unwrap().push(c),
        }
    }
    // a bar at the end of a command that cannot split again is just a bar: the rest goes on as one
    let mut res: Vec<String> = Vec::new();
    for p in out {
        if p.trim().is_empty() {
            continue;
        }
        res.push(p);
    }
    if res.is_empty() {
        res.push(String::new());
    }
    // the first command decides; the ones after are cut again by their own rules
    if res.len() > 1 {
        let rest = res[1..].join("|");
        let mut first = vec![res[0].clone()];
        first.extend(bar_split(&rest));
        return first;
    }
    res
}

#[cfg(test)]
mod bar_tests {
    use super::bar_split;

    #[test]
    fn bars_split_commands_but_not_theirs() {
        assert_eq!(bar_split("split | resize 4"), vec!["split ", " resize 4"]);
        assert_eq!(bar_split("w | bd"), vec!["w ", " bd"]);
        assert_eq!(bar_split("%s/a\\|b/x/g"), vec!["%s/a|b/x/g"]);
        assert_eq!(bar_split("g/x/s/a/b/ | echo 1"), vec!["g/x/s/a/b/ | echo 1"]);
        assert_eq!(bar_split("!ls | wc -l"), vec!["!ls | wc -l"]);
        assert_eq!(bar_split("nnoremap x :a<CR>|:b<CR>"), vec!["nnoremap x :a<CR>|:b<CR>"]);
        assert_eq!(bar_split("bufdo %s/a/b/g | update"), vec!["bufdo %s/a/b/g | update"]);
        // the second command keeps its own bars
        assert_eq!(bar_split("echo 1 | normal! a|b"), vec!["echo 1 ", " normal! a|b"]);
        assert_eq!(bar_split("set ts=4"), vec!["set ts=4"]);
    }
}

/// A wildchar value (a number as Vim keeps it, 9 for Tab, or key notation) as the key it is.
fn wild_token(w: &str) -> Option<String> {
    match w.trim().parse::<u32>() {
        Ok(0) => None,
        Ok(9) => Some("<Tab>".into()),
        Ok(n) if n < 32 => Some(format!("<C-{}>", (n as u8 + 96) as char)),
        Ok(n) => char::from_u32(n).map(|c| c.to_string()),
        Err(_) if w.starts_with('<') => Some(w.to_string()),
        Err(_) => None,
    }
}

/// A command's end trimmed, but a space after a backslash kept (Vim's `:set stc=%l\ `).
pub(crate) fn trim_cmd(s: &str) -> &str {
    let t = s.trim_end();
    if t.ends_with('\\') && t.len() < s.len() {
        &s[..t.len() + 1]
    } else {
        t
    }
}
