//! Vim's mappings, abbreviations, user commands and autocommands: :map :nmap :vmap :xmap :imap and
//! their noremap / unmap / mapclear forms; :abbreviate :iabbrev :noreabbrev :unabbreviate
//! :abclear; :command (-nargs -range -bang -count, <args> <q-args> <f-args> <bang> <line1>
//! <line2> <count>) :delcommand :comclear; :autocmd :augroup :doautocmd :doautoall.

use super::*;

const NAMES: &[(&str, &str)] = &[
    ("map", "map"), ("nmap", "nm"), ("vmap", "vm"), ("xmap", "xm"), ("imap", "im"), ("noremap", "no"), ("nnoremap", "nn"),
    ("vnoremap", "vn"), ("xnoremap", "xn"), ("inoremap", "ino"), ("unmap", "unm"), ("nunmap", "nun"), ("vunmap", "vu"),
    ("xunmap", "xu"), ("iunmap", "iu"), ("mapclear", "mapc"), ("nmapclear", "nmapc"), ("vmapclear", "vmapc"), ("xmapclear", "xmapc"),
    ("imapclear", "imapc"),
    ("abbreviate", "ab"), ("iabbrev", "ia"), ("noreabbrev", "norea"), ("inoreabbrev", "inorea"), ("unabbreviate", "una"),
    ("iunabbrev", "iuna"), ("abclear", "abc"), ("iabclear", "iabc"),
    ("command", "com"), ("delcommand", "delc"), ("comclear", "comc"),
    ("autocmd", "au"), ("augroup", "aug"), ("doautocmd", "do"), ("doautoall", "doautoa"),
];

/// A typed name as one of these commands (its full name).
pub(crate) fn map_name(typed: &str) -> Option<&'static str> {
    NAMES.iter().find(|(full, short)| full.starts_with(typed) && typed.len() >= short.len()).map(|(f, _)| *f)
}

/// A user command (:command): what it runs and what it takes.
#[derive(Clone, Debug, Default)]
pub(crate) struct UserCmd {
    pub repl: String,
    pub nargs: String,
    pub range: bool,
    pub bang: bool,
}

/// An autocommand: its group, events (lowercase), file pattern, command, and whether it runs once.
#[derive(Clone, Debug, Default)]
pub(crate) struct AutoCmd {
    pub group: String,
    pub events: Vec<String>,
    pub pat: String,
    pub cmd: String,
    pub once: bool,
}

/// The zett event an autocommand event is told by (and the ones zett checks for itself).
pub(crate) fn vim_events(zett_event: &str) -> &'static [&'static str] {
    match zett_event {
        "write" => &["bufwritepost", "filewritepost"],
        "write_pre" => &["bufwritepre", "filewritepre", "bufwrite"],
        "buf_enter" => &["bufenter", "bufwinenter"],
        "buf_read" => &["bufreadpost", "bufread", "filetype"],
        "text_changed" => &["textchanged"],
        "option_set" => &["optionset"],
        "insert_enter" => &["insertenter"],
        "insert_leave" => &["insertleave"],
        "leave" => &["vimleavepre", "vimleave"],
        _ => &[],
    }
}

impl App {
    /// One of these commands; `range` for :command's <line1> <line2>.
    pub(crate) fn map_command(&mut self, name: &str, arg: &str, bang: bool) {
        let Some(cmd) = map_name(name) else { return };
        match cmd {
            "map" | "nmap" | "vmap" | "xmap" | "imap" | "noremap" | "nnoremap" | "vnoremap" | "xnoremap" | "inoremap" => {
                let remap = !cmd.contains("noremap");
                let modes: &[&str] = match cmd.chars().next().unwrap() {
                    'n' if cmd != "noremap" => &["n"],
                    'v' | 'x' => &["v"],
                    'i' => &["i"],
                    _ if bang => &["i"],
                    _ => &["n", "v"],
                };
                // <buffer> <silent> <nowait> <unique> <special> <script> <expr> in front
                let mut rest = arg.trim();
                while let Some(r) = ["<buffer>", "<silent>", "<nowait>", "<unique>", "<special>", "<script>", "<expr>"].iter().find_map(|o| rest.strip_prefix(o)) {
                    rest = r.trim_start();
                }
                let (lhs, rhs) = match rest.split_once(char::is_whitespace) {
                    Some((l, r)) => (l, r.trim()),
                    None => (rest, ""),
                };
                if rhs.is_empty() {
                    return self.map_list(modes, lhs);
                }
                // a zett command by its name stays a command (zett's way); anything else is keys (Vim's)
                let is_cmd = self.engine.commands().iter().any(|(n, _, _)| *n == rhs);
                for m in modes {
                    if is_cmd {
                        let code = format!("zett.map({m:?}, {lhs:?}, {rhs:?}, {{ desc = {rhs:?} }})");
                        if let Err(e) = self.with_lua(|e| e.eval(&code)) {
                            return self.say(e);
                        }
                    } else {
                        self.engine.map_keys(m, lhs, rhs, remap);
                    }
                }
            }
            "unmap" | "nunmap" | "vunmap" | "xunmap" | "iunmap" => {
                let modes: &[&str] = match cmd { "nunmap" => &["n"], "vunmap" | "xunmap" => &["v"], "iunmap" => &["i"], _ if bang => &["i"], _ => &["n", "v"] };
                let lhs = arg.trim().trim_start_matches("<buffer>").trim();
                let mut any = false;
                for m in modes {
                    any |= self.engine.unmap_keys(m, lhs);
                }
                if !any {
                    self.say("E31: No such mapping");
                }
            }
            "mapclear" | "nmapclear" | "vmapclear" | "xmapclear" | "imapclear" => {
                let modes: &[&str] = match cmd { "nmapclear" => &["n"], "vmapclear" | "xmapclear" => &["v"], "imapclear" => &["i"], _ if bang => &["i"], _ => &["n", "v"] };
                for m in modes {
                    self.engine.clear_maps(m);
                }
            }
            "abbreviate" | "iabbrev" | "noreabbrev" | "inoreabbrev" => {
                let t = arg.trim().trim_start_matches("<buffer>").trim();
                match t.split_once(char::is_whitespace) {
                    Some((lhs, rhs)) => {
                        self.abbrevs.retain(|(l, _)| l != lhs);
                        self.abbrevs.push((lhs.to_string(), rhs.trim().to_string()));
                    }
                    None => {
                        let rows = self.abbrevs.iter().filter(|(l, _)| t.is_empty() || l.starts_with(t)).map(|(l, r)| (format!("i  {l}"), r.clone())).collect();
                        self.popup = Some(Popup::Help("abbreviations".into(), rows));
                    }
                }
            }
            "unabbreviate" | "iunabbrev" => {
                let n = self.abbrevs.len();
                self.abbrevs.retain(|(l, _)| l != arg.trim());
                if self.abbrevs.len() == n {
                    self.say("E24: No such abbreviation");
                }
            }
            "abclear" | "iabclear" => self.abbrevs.clear(),
            "command" => self.user_command(arg, bang),
            "delcommand" => {
                if self.user_cmds.remove(arg.trim()).is_none() {
                    self.say(format!("E184: No such user-defined command: {}", arg.trim()));
                }
            }
            "comclear" => self.user_cmds.clear(),
            "augroup" => {
                let g = arg.trim();
                if bang {
                    self.autocmds.retain(|a| a.group != g);
                } else {
                    if !g.eq_ignore_ascii_case("end") {
                        self.augroups.insert(g.to_string());
                    }
                    self.augroup = if g.eq_ignore_ascii_case("end") { String::new() } else { g.to_string() };
                }
            }
            "autocmd" => self.autocmd(arg, bang),
            "doautocmd" | "doautoall" => {
                let mut t = arg.trim();
                if let Some(r) = t.strip_prefix("<nomodeline>") {
                    t = r.trim_start();
                }
                let (ev, rest) = t.split_once(char::is_whitespace).unwrap_or((t, ""));
                // a group name may come first
                let (ev, rest) = if self.autocmds.iter().any(|a| a.group == ev) { rest.trim().split_once(char::is_whitespace).unwrap_or((rest.trim(), "")) } else { (ev, rest) };
                let target = rest.trim().to_string();
                for e in ev.split(',') {
                    self.fire_autocmds(&e.to_lowercase(), (!target.is_empty()).then_some(target.as_str()));
                }
            }
            _ => {}
        }
    }

    /// :map with only a left side (or none): the mappings that start with it.
    fn map_list(&mut self, modes: &[&str], lhs: &str) {
        let mut rows = Vec::new();
        for m in modes {
            for (k, d) in self.engine.bindings(m) {
                if lhs.is_empty() || k.starts_with(lhs) {
                    rows.push((format!("{m}  {k}"), d));
                }
            }
        }
        if rows.is_empty() {
            return self.say("No mapping found");
        }
        self.popup = Some(Popup::Help("mappings".into(), rows));
    }

    /// :command [-nargs=…] [-range] [-bang] [-count] [-complete=…] Name replacement; alone: the list.
    fn user_command(&mut self, arg: &str, bang: bool) {
        let mut uc = UserCmd { nargs: "0".into(), ..Default::default() };
        let mut rest = arg.trim();
        while let Some(r) = rest.strip_prefix('-') {
            let (opt, tail) = r.split_once(char::is_whitespace).unwrap_or((r, ""));
            match opt.split_once('=') {
                Some(("nargs", v)) => uc.nargs = v.to_string(),
                _ if opt == "range" || opt.starts_with("range=") || opt.starts_with("count") => uc.range = true,
                _ if opt == "bang" => uc.bang = true,
                _ => {}
            }
            rest = tail.trim_start();
        }
        let Some((name, repl)) = rest.split_once(char::is_whitespace) else {
            if rest.is_empty() {
                let rows = self.user_cmds.iter().map(|(n, c)| (format!("{n} ({})", c.nargs), c.repl.clone())).collect();
                return self.popup = Some(Popup::Help("user commands".into(), rows));
            }
            return self.say(format!("E183: User defined commands must start with an uppercase letter: {rest}"));
        };
        if !name.starts_with(|c: char| c.is_ascii_uppercase()) {
            return self.say("E183: User defined commands must start with an uppercase letter");
        }
        if self.user_cmds.contains_key(name) && !bang {
            return self.say(format!("E174: Command already exists: add ! to replace it: {name}"));
        }
        uc.repl = repl.trim().to_string();
        self.user_cmds.insert(name.to_string(), uc);
    }

    /// Runs a user command: <args> <q-args> <f-args> <bang> <line1> <line2> <count> <range> <lt>
    /// put in its replacement.
    pub(crate) fn run_user_command(&mut self, name: &str, arg: &str, bang: bool, range: Option<(usize, usize)>) {
        let Some(uc) = self.user_cmds.get(name).cloned() else { return };
        if bang && !uc.bang {
            return self.say("E477: No ! allowed");
        }
        if uc.nargs == "0" && !arg.trim().is_empty() {
            return self.say("E488: Trailing characters");
        }
        if uc.nargs == "1" && arg.trim().is_empty() || uc.nargs == "+" && arg.trim().is_empty() {
            return self.say("E471: Argument required");
        }
        let cur = self.docs[self.cur].ed.cursor().line;
        let (l1, l2) = range.unwrap_or((cur, cur));
        let fargs = arg.split_whitespace().map(|a| format!("{a:?}")).collect::<Vec<_>>().join(", ");
        let line = uc
            .repl
            .replace("<q-args>", &format!("{:?}", arg.trim()))
            .replace("<f-args>", &fargs)
            .replace("<args>", arg.trim())
            .replace("<bang>", if bang { "!" } else { "" })
            .replace("<line1>", &(l1 + 1).to_string())
            .replace("<line2>", &(l2 + 1).to_string())
            .replace("<count>", &(l2 + 1).to_string())
            .replace("<range>", if range.is_some() { "2" } else { "0" })
            .replace("<lt>", "<");
        self.command(&line);
    }

    /// :autocmd [group] {events} {pattern} [++once] [++nested] {command}; with !: removes; alone:
    /// the list.
    fn autocmd(&mut self, arg: &str, bang: bool) {
        let mut parts: Vec<&str> = arg.split_whitespace().collect();
        // a group defined with :augroup may come first
        let group = match parts.first() {
            Some(g) if self.augroups.contains(*g) => parts.remove(0).to_string(),
            _ => self.augroup.clone(),
        };
        let events: Vec<String> = parts.first().map(|e| e.split(',').map(|x| x.to_lowercase()).collect()).unwrap_or_default();
        let pat = parts.get(1).map(|p| p.to_string()).unwrap_or_default();
        if bang {
            // :au! [event] [pattern]: those go (all of the group with neither)
            self.autocmds.retain(|a| !(a.group == group && (events.is_empty() || a.events.iter().any(|e| events.contains(e))) && (pat.is_empty() || a.pat == pat)));
            if parts.len() <= 2 {
                return;
            }
        }
        if parts.len() < 3 {
            let rows = self.autocmds.iter().filter(|a| events.is_empty() || a.events.iter().any(|e| events.contains(e))).map(|a| (format!("{} {}", a.events.join(","), a.pat), a.cmd.clone())).collect();
            return self.popup = Some(Popup::Help("autocommands".into(), rows));
        }
        let mut once = false;
        let mut i = 2;
        while let Some(f) = parts.get(i).filter(|f| f.starts_with("++")) {
            once |= *f == "++once";
            i += 1;
        }
        let cmd = parts[i..].join(" ");
        self.autocmds.push(AutoCmd { group, events, pat, cmd, once });
    }

    /// Runs the autocommands for a Vim event (lowercase): those whose pattern fits the file (or the
    /// file type for FileType, the name for User), each once if ++once.
    pub(crate) fn fire_autocmds(&mut self, event: &str, target: Option<&str>) {
        if self.autocmds.is_empty() || self.au_depth > 10 {
            return;
        }
        let file = self.docs[self.cur].abs_path().map(|p| p.display().to_string()).unwrap_or_default();
        let ft = self.setting(self.cur, "filetype");
        let subject = target.map(str::to_string).unwrap_or_else(|| if event == "filetype" { ft } else { file.clone() });
        let tail = std::path::Path::new(&subject).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        let fits = |pat: &str| pat == "*" || pat.split(',').any(|p| crate::files::glob_match(p, &subject) || crate::files::glob_match(p, &tail) || crate::files::glob_match(&format!("*/{p}"), &subject));
        let run: Vec<(usize, AutoCmd)> = self.autocmds.iter().cloned().enumerate().filter(|(_, a)| a.events.iter().any(|e| e == event) && fits(&a.pat)).collect();
        if run.is_empty() {
            return;
        }
        self.au_depth += 1;
        for (_, a) in &run {
            self.command(&a.cmd);
        }
        self.au_depth -= 1;
        let gone: Vec<usize> = run.iter().filter(|(_, a)| a.once).map(|(i, _)| *i).collect();
        let mut k = 0;
        self.autocmds.retain(|_| {
            let keep = !gone.contains(&k);
            k += 1;
            keep
        });
    }

    /// Insert mode: a word just ended (a key that is not part of one): an abbreviation for it is
    /// put in its place before the key goes on.
    pub(crate) fn expand_abbrev(&mut self, typed: char) {
        if self.abbrevs.is_empty() || typed.is_alphanumeric() || typed == '_' {
            return;
        }
        let d = &self.docs[self.cur];
        let c = d.ed.cursor();
        let line: Vec<char> = d.ed.buf.line(c.line).chars().take(c.col).collect();
        let start = line.iter().rposition(|ch| !(ch.is_alphanumeric() || *ch == '_')).map_or(0, |i| i + 1);
        let word: String = line[start..].iter().collect();
        let Some((_, rhs)) = self.abbrevs.iter().find(|(l, _)| *l == word).cloned() else { return };
        let ed = &mut self.docs[self.cur].ed;
        for _ in 0..word.chars().count() {
            ed.key(zett_edit::keys::Key::code(zett_edit::keys::Code::Backspace));
        }
        for ch in rhs.chars() {
            ed.key(zett_edit::keys::Key::ch(ch));
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn names() {
        assert_eq!(super::map_name("nn"), Some("nnoremap"));
        assert_eq!(super::map_name("ino"), Some("inoremap"));
        assert_eq!(super::map_name("au"), Some("autocmd"));
        assert_eq!(super::map_name("com"), Some("command"));
        assert_eq!(super::map_name("do"), Some("doautocmd"));
        assert_eq!(super::map_name("d"), None, ":d is :delete");
        assert_eq!(super::vim_events("write"), &["bufwritepost", "filewritepost"]);
    }
}
