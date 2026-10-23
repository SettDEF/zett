//! Emacs's commands that ask before they act: minibuffer prompts (one or more answers, then an Ex
//! line, a numbered editor command or a rectangle's text), and query-replace (M-%, C-M-%) with its
//! keys: y n ! . q, the case kept as Emacs keeps it.

use super::*;
use zett_edit::search::emacs_regex;

/// A query point in a recorded macro (C-x q while recording): playback stops there to ask.
pub(crate) const KMACRO_QUERY: &str = "\u{1}kmacro-query";
/// The macro counter written there (F3 or C-x C-k C-i while recording).
pub(crate) const KMACRO_COUNTER: &str = "\u{1}kmacro-counter";

/// A query-replace in progress: the next match is shown as the region.
pub(crate) struct QueryReplace {
    re: fancy_regex::Regex,
    /// the replacement in the regex crate's form ($1), or None: the literal `to`
    rep: Option<String>,
    from: String,
    to: String,
    /// keep the match's case (the search text was all lower case)
    fold: bool,
    /// the next search starts here; nothing at or past `end` is touched (chars)
    at: usize,
    end: usize,
    /// the match shown now (chars)
    hit: Option<(usize, usize)>,
    pub done: usize,
}


/// An Emacs replacement (\& the match, \N a group, \\ a backslash) in the regex crate's form.
pub(crate) fn emacs_replacement(r: &str) -> String {
    let mut out = String::new();
    let mut it = r.chars();
    while let Some(c) = it.next() {
        match c {
            '\\' => match it.next() {
                Some('&') => out.push_str("${0}"),
                Some(d @ '0'..='9') => out.push_str(&format!("${{{d}}}")),
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some(o) => out.push(o),
                None => out.push('\\'),
            },
            '$' => out.push_str("$$"),
            c => out.push(c),
        }
    }
    out
}

/// The replacement shaped like the match (Emacs's case-replace): FOO → BAR, Foo → Bar.
pub(crate) fn match_case(matched: &str, rep: &str) -> String {
    let letters: Vec<char> = matched.chars().filter(|c| c.is_alphabetic()).collect();
    if letters.len() > 1 && letters.iter().all(|c| c.is_uppercase()) {
        return rep.to_uppercase();
    }
    let words_capitalized = matched.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()).all(|w| w.chars().next().is_some_and(char::is_uppercase));
    if !letters.is_empty() && words_capitalized {
        let mut start = true;
        return rep
            .chars()
            .map(|c| {
                let out = if start { c.to_uppercase().collect::<String>() } else { c.to_string() };
                start = !c.is_alphanumeric();
                out
            })
            .collect();
    }
    rep.to_string()
}

/// `{1}` an answer as typed, `{e1}` with the characters a Vim pattern or replacement treats
/// specially escaped (for a literal :s).
fn fill(template: &str, got: &[String]) -> String {
    let mut out = template.to_string();
    for (i, g) in got.iter().enumerate() {
        let esc: String = g.chars().flat_map(|c| if "\\/.*[]~^$&".contains(c) { vec!['\\', c] } else { vec![c] }).collect();
        out = out.replace(&format!("{{e{}}}", i + 1), &esc).replace(&format!("{{{}}}", i + 1), g);
    }
    out
}

impl App {
    /// Asks in the minibuffer: `spec` is the prompts and then what to do, tab-separated
    /// ("Goto line: \tnum:goto-line {1}"). What to do: `ex:LINE`, `num:COMMAND N`, `rect:TEXT`,
    /// `qr:` / `qrx:` (query-replace, literal or regexp: two answers), `rs:` / `rx:` (all at once).
    pub(crate) fn prompt(&mut self, spec: &str) {
        let mut parts: Vec<String> = spec.split('\t').map(str::to_string).collect();
        let Some(what) = parts.pop() else { return };
        if parts.is_empty() {
            return self.prompt_done(&what, &[]);
        }
        let first = parts.remove(0);
        // "… @commands" or "… @settings": those to choose from
        let (title, items) = self.prompt_items(&first);
        let title = title.trim_end().trim_end_matches(':').to_string();
        self.pick_list(Pick::Prompt(parts, what, Vec::new()), &title, items);
    }

    fn prompt_items(&self, title: &str) -> (String, Vec<Cand>) {
        if let Some(t) = title.strip_suffix(" @commands") {
            return (t.to_string(), self.engine.commands().into_iter().map(|(n, d, _)| Cand::noted(n, d)).collect());
        }
        if let Some(t) = title.strip_suffix(" @settings") {
            return (t.to_string(), crate::settings::all().into_iter().map(|s| Cand::noted(s.name, s.about)).collect());
        }
        for src in ["chars", "emoji", "emoji-recent"] {
            if let Some(t) = title.strip_suffix(&format!(" @{src}")) {
                return (t.to_string(), self.char_items(src));
            }
        }
        if let Some(t) = title.strip_suffix(" @inputmethods") {
            return (t.to_string(), crate::inputmethod::METHODS.iter().map(|(n, about)| Cand::noted(*n, *about)).collect());
        }
        if let Some(t) = title.strip_suffix(" @tabs") {
            return (t.to_string(), self.tab_items());
        }
        for src in ["branches", "tags", "worktrees"] {
            if let Some(t) = title.strip_suffix(&format!(" @{src}")) {
                return (t.to_string(), self.vc_items(src));
            }
        }
        if let Some(t) = title.strip_suffix(" @buffers") {
            return (t.to_string(), self.docs.iter().map(|d| Cand::plain(self.buffer_label(d))).collect());
        }
        if let Some(t) = title.strip_suffix(" @bookmarks") {
            return (t.to_string(), bookmarks().into_iter().map(|(n, p, l)| Cand::noted(n, format!("{p}:{}", l + 1))).collect());
        }
        (title.to_string(), Vec::new())
    }

    /// An answer: the next prompt, or the action.
    pub(crate) fn prompt_answered(&mut self, mut rest: Vec<String>, what: String, mut got: Vec<String>, answer: String) {
        got.push(answer);
        if rest.is_empty() {
            return self.prompt_done(&what, &got);
        }
        let title = fill(&rest.remove(0), &got);
        let title = title.trim_end().trim_end_matches(':').to_string();
        self.pick_list(Pick::Prompt(rest, what, got), &title, Vec::new());
    }

    fn prompt_done(&mut self, what: &str, got: &[String]) {
        if !what.starts_with("repeat") {
            self.last_prompt = Some((what.to_string(), got.to_vec()));
        }
        let line = fill(what, got);
        let (kind, body) = line.split_once(':').unwrap_or(("ex", line.as_str()));
        match kind {
            "ex" => self.command(body),
            "num" => {
                let (name, n) = body.rsplit_once(' ').unwrap_or((body, ""));
                match n.trim().parse::<i64>() {
                    Ok(n) => {
                        self.doc().ed.exec_arg(name, Some(n));
                        self.editor_requests();
                    }
                    Err(_) => self.say(format!("not a number: {n}")),
                }
            }
            "hl" => self.highlight(body),
            "tab" => self.emacs_tab(body),
            "abbrev-add" => self.abbrev_answer(false, body),
            "abbrev-inverse" => self.abbrev_answer(true, body),
            "char" => self.insert_char(body),
            "emoji" => self.insert_emoji(body),
            "delwins" => {
                self.emacs_ui("deleteWindowsOn", body);
            }
            "search" => {
                let ed = &self.docs[self.cur].ed;
                let at = ed.buf.to_char(ed.cursor());
                let fold = ed.folds_for(body);
                match ed.buf.find(body, at + 1, true, fold) {
                    Some(i) => {
                        let to = ed.buf.to_pos(i + body.chars().count());
                        self.doc().ed.set_cursor(to);
                    }
                    None => self.say(format!("Search failed: \"{body}\"")),
                }
            }
            // the macro counter: set (C-x C-k C-c), added to (C-x C-k C-a), its format (C-x C-k C-f)
            "kmcset" | "kmcadd" => match body.trim().parse::<i64>() {
                Ok(n) if kind == "kmcset" => self.kmacro_counter = n,
                Ok(n) => self.kmacro_counter += n,
                Err(_) => self.say(format!("not a number: {body}")),
            },
            // C-x RET f / C-x RET r: the coding system to save with, or to read the file again in
            "coding" => self.set_coding(body),
            "codingrev" => self.revert_coding(body),
            // C-x RET C-\: the input method, chosen
            "imset" => {
                let m = body.split_whitespace().next().unwrap_or("").to_string();
                if crate::inputmethod::METHODS.iter().any(|(n, _)| *n == m) {
                    self.last_input_method = m.clone();
                    self.input_method = Some(m.clone());
                    self.say(format!("Input method: {m}"));
                } else {
                    self.say(format!("No input method {m}"));
                }
            }
            "kmcfmt" => self.kmacro_format = if body.is_empty() { "%d".into() } else { body.to_string() },
            // C-x C-k n: the last macro under a name, an M-x command of its own
            "kmname" => {
                let name = body.trim();
                if self.last_kmacro.is_empty() {
                    return self.say("No keyboard macro defined");
                }
                if name.is_empty() || name.contains(['"', '\\', '\n']) {
                    return self.say(format!("not a command name: {name}"));
                }
                self.named_kmacros.insert(name.to_string(), self.last_kmacro.clone());
                let code = format!("zett.command(\"{name}\", function() zett.ui(\"kmacroNamed\", \"{name}\") end, {{ desc = \"A keyboard macro (C-x C-k n)\" }})");
                match self.engine.eval(&code) {
                    Ok(_) => self.say(format!("Keyboard macro named {name}: M-x {name}")),
                    Err(e) => self.say(e),
                }
            }
            // C-x C-k b: a key runs the last macro (named for it: kmacro-KEY)
            "kmbind" => {
                let key = body.trim();
                if self.last_kmacro.is_empty() || key.is_empty() {
                    return self.say("No keyboard macro defined");
                }
                let name = format!("kmacro-{}", key.chars().filter(|c| c.is_alphanumeric()).collect::<String>());
                self.named_kmacros.insert(name.clone(), self.last_kmacro.clone());
                let code = format!("zett.command(\"{name}\", function() zett.ui(\"kmacroNamed\", \"{name}\") end, {{ desc = \"A keyboard macro bound to {key}\" }})");
                if let Err(e) = self.engine.eval(&code) {
                    return self.say(e);
                }
                let lhs: String = zett_script::keys(key).concat();
                self.engine.map_keys("e", &lhs, &name, false);
                self.say(format!("Keyboard macro bound to {key}"));
            }
            // M-|: the region through a shell command; its output shown, or with C-u put in its place
            "shreg" => {
                let ed = &self.docs[self.cur].ed;
                let Some((a, b)) = ed.region() else {
                    return self.say("The mark is not set now, so there is no region".to_string());
                };
                let input: String = ed.text().chars().skip(a).take(b - a).collect();
                let replace = self.arg.take().is_some();
                let dir = self.root.clone();
                match self.run_shell(body, Some(&input), &dir) {
                    Ok(out) if replace => {
                        self.doc().ed.replace_text(a, b, &out);
                        self.editor_requests();
                    }
                    Ok(out) => {
                        let lines: Vec<&str> = out.trim_end_matches('\n').lines().collect();
                        match lines.len() {
                            0 => self.say("(Shell command succeeded with no output)".to_string()),
                            1 => self.say(lines[0].to_string()),
                            _ => self.popup = Some(Popup::Help(format!("| {body}"), lines.iter().map(|l| (String::new(), l.to_string())).collect())),
                        }
                    }
                    Err(e) => self.say(e),
                }
            }
            "display" => {
                let Some(i) = self.docs.iter().position(|d| self.buffer_label(d) == body) else { return self.say(format!("No buffer {body}")) };
                self.doom_ui("split", "right");
                self.show_doc(i);
                self.command("wincmd p");
            }
            "bookmark-set" => {
                self.bookmark_set(body);
            }
            "bookmark-jump" => {
                self.bookmark_jump(body);
            }
            "insbuf" => {
                self.emacs_ui("insertBuffer", body);
            }
            "vc" => {
                let (w, a) = body.split_once(' ').unwrap_or((body, ""));
                self.vc(w, a);
            }
            "whereis" => {
                self.emacs_ui("whereIs", body);
            }
            "rect" => {
                let before = self.docs[self.cur].ed.cursor().line;
                self.doc().ed.string_rectangle(body);
                self.doc().touched(before);
            }
            "qr" | "qrx" | "rs" | "rx" => {
                let (Some(from), Some(to)) = (got.first(), got.get(1)) else { return };
                self.query_replace(from, to, matches!(kind, "qrx" | "rx"), matches!(kind, "rs" | "rx"));
            }
            "pqrx" => {
                let (Some(from), Some(to)) = (got.first(), got.get(1)) else { return };
                self.project_query_replace(from, to);
            }
            other => self.say(format!("{other}: no such prompt action")),
        }
    }

    /// Starts M-% (or, `all`, replaces every match at once): from the cursor, in the region if
    /// one is active.
    pub(crate) fn query_replace(&mut self, from: &str, to: &str, regexp: bool, all: bool) {
        if from.is_empty() {
            return;
        }
        // case-fold-search: all lower case matches any case, and the replacement takes the match's
        let fold = !from.chars().any(char::is_uppercase);
        let pat = if regexp { emacs_regex(from) } else { fancy_regex::escape(from).into_owned() };
        let re = match fancy_regex::Regex::new(&format!("{}{pat}", if fold { "(?i)" } else { "" })) {
            Ok(r) => r,
            Err(e) => return self.say(format!("Invalid regexp: {e}")),
        };
        let ed = &self.docs[self.cur].ed;
        let (at, end) = match ed.region() {
            Some((a, b)) => (a, b),
            None => (ed.buf.to_char(ed.cursor()), ed.buf.len_chars()),
        };
        let rep = regexp.then(|| emacs_replacement(to));
        self.qr = Some(QueryReplace { re, rep, from: from.into(), to: to.into(), fold, at, end, hit: None, done: 0 });
        if all {
            self.qr_rest();
        } else {
            self.qr_next();
        }
    }

    /// The next match shown, or the end of it.
    fn qr_next(&mut self) {
        let Some(q) = self.qr.as_mut() else { return };
        let ed = &mut self.docs[self.cur].ed;
        let text = ed.text();
        let byte = |c: usize| text.char_indices().nth(c).map_or(text.len(), |(b, _)| b);
        let hit = q.re.find_from_pos(&text, byte(q.at)).ok().flatten().map(|m| (text[..m.start()].chars().count(), text[..m.end()].chars().count()));
        match hit {
            Some((a, b)) if b <= q.end && (b > a || a < q.end) => {
                q.hit = Some((a, b));
                ed.set_region(a, b);
                let m = format!("Query replacing {} with {}: (y, n, !, ., q; ? for help)", q.from, q.to);
                self.say(m);
            }
            _ => self.qr_file_done(),
        }
    }

    /// This buffer has no match left: done, or (C-x p r) on to the next file that has one.
    fn qr_file_done(&mut self) {
        let done = self.qr.as_ref().map_or(0, |q| q.done);
        if self.project_qr.is_none() {
            return self.qr_finish();
        }
        if let Some(q) = self.qr.take() {
            self.docs[self.cur].ed.exec("keyboard-quit", 1);
            let _ = q;
        }
        let Some((mut files, from, to, total)) = self.project_qr.take() else { return };
        let total = total + done;
        match files.pop_front() {
            Some(next) => {
                self.open(&next);
                self.doc().ed.set_cursor(zett_core::Pos { line: 0, col: 0 });
                self.project_qr = Some((files, from.clone(), to.clone(), total));
                self.query_replace(&from, &to, true, false);
            }
            None => self.say(format!("Replaced {total} occurrence{} in the project", if total == 1 { "" } else { "s" })),
        }
    }

    /// C-x p r: query-replace a regexp in every file of the project that has it, one after another.
    pub(crate) fn project_query_replace(&mut self, from: &str, to: &str) {
        let out = std::process::Command::new("rg").args(["-l", "--sort", "path", "-e", &emacs_regex(from)]).current_dir(&self.root).output();
        let files: std::collections::VecDeque<std::path::PathBuf> = match out {
            Ok(o) => String::from_utf8_lossy(&o.stdout).lines().map(|l| self.root.join(l)).collect(),
            Err(e) => return self.say(format!("rg: {e} (ripgrep finds the files)")),
        };
        if files.is_empty() {
            return self.say(format!("No matches for {from} in the project"));
        }
        self.project_qr = Some((files, from.to_string(), to.to_string(), 0));
        self.qr_file_done();
    }

    /// The match shown, replaced; the search goes on after it.
    fn qr_replace(&mut self) {
        let Some(q) = self.qr.as_mut() else { return };
        let Some((a, b)) = q.hit.take() else { return };
        let ed = &mut self.docs[self.cur].ed;
        let text = ed.text();
        let byte = |c: usize| text.char_indices().nth(c).map_or(text.len(), |(b, _)| b);
        let (ba, bb) = (byte(a), byte(b));
        let matched = &text[ba..bb];
        let new = match &q.rep {
            Some(rep) => match q.re.captures_from_pos(&text, ba) {
                Ok(Some(caps)) => {
                    let mut s = String::new();
                    caps.expand(rep, &mut s);
                    s
                }
                _ => q.to.clone(),
            },
            None => q.to.clone(),
        };
        let new = if q.fold { match_case(matched, &new) } else { new };
        let before = ed.cursor().line;
        ed.replace_text(a, b, &new);
        let grown = new.chars().count() as i64 - (b - a) as i64;
        q.end = (q.end as i64 + grown).max(0) as usize;
        // an empty match moves on a character, or it would match again where it is
        q.at = a + new.chars().count() + usize::from(a == b);
        q.done += 1;
        self.doc().touched(before);
        self.edit_at = Instant::now();
    }

    /// `!`: every match left, without asking.
    fn qr_rest(&mut self) {
        loop {
            self.qr_next();
            if self.qr.as_ref().is_none_or(|q| q.hit.is_none()) {
                return;
            }
            self.qr_replace();
        }
    }

    fn qr_finish(&mut self) {
        // stopped (q, RET, C-g): the rest of the project's files too
        self.project_qr = None;
        if let Some(q) = self.qr.take() {
            let ed = &mut self.docs[self.cur].ed;
            ed.exec("keyboard-quit", 1);
            self.say(format!("Replaced {} occurrence{}", q.done, if q.done == 1 { "" } else { "s" }));
        }
    }

    /// A key while query-replace waits: y SPC replace, n DEL skip, ! all, . this one and stop,
    /// q RET stop; any other key stops it and does what it always does.
    pub(crate) fn qr_key(&mut self, k: KeyEvent) {
        let ctrl = k.modifiers.contains(KeyModifiers::CONTROL);
        match k.code {
            KeyCode::Char('y') | KeyCode::Char(' ') if !ctrl => {
                self.qr_replace();
                self.qr_next();
            }
            KeyCode::Char('n') | KeyCode::Backspace | KeyCode::Delete if !ctrl => {
                if let Some(q) = self.qr.as_mut() {
                    if let Some((a, b)) = q.hit.take() {
                        q.at = b + usize::from(a == b);
                    }
                }
                self.qr_next();
            }
            KeyCode::Char('!') => {
                self.qr_replace();
                self.qr_rest();
            }
            KeyCode::Char('.') => {
                self.qr_replace();
                self.qr_finish();
            }
            KeyCode::Char('q') | KeyCode::Enter | KeyCode::Esc => self.qr_finish(),
            KeyCode::Char('g') if ctrl => self.qr_finish(),
            KeyCode::Char('?') => self.say("y or SPC: replace, n or DEL: skip, !: all the rest, .: this one and stop, q or RET: stop"),
            _ => {
                self.qr_finish();
                self.key(k);
            }
        }
    }
}

impl App {
    /// The Emacs pieces Lua asks for by name: C-h k / C-h c, C-h w, C-h l, keyboard macros.
    pub(crate) fn emacs_ui(&mut self, name: &str, arg: &str) -> bool {
        match name {
            "describeKey" => {
                self.describing = Some((arg == "brief", Vec::new()));
                self.say(if arg == "brief" { "Describe key briefly: " } else { "Describe key: " });
            }
            "whereIs" => {
                let keys = self.engine.keys_of("e").remove(arg).unwrap_or_default();
                if keys.is_empty() {
                    self.say(format!("{arg} is not on any key"));
                } else {
                    let shown: Vec<String> = keys.iter().map(|k| minibuf::emacs_keys(k)).collect();
                    self.say(format!("{arg} is on {}", shown.join(", ")));
                }
            }
            "lossage" => {
                let rows: Vec<(String, String)> = self.lossage.iter().map(|t| (minibuf::emacs_keys(t), String::new())).collect();
                self.popup = Some(Popup::Help("Recent keys".into(), rows));
            }
            // an editor command from Lua (a step of a command that does several)
            "edit" => self.after_lua(vec![Effect::Edit(arg.to_string(), 1)]),
            "repeat" => self.repeat_last(),
            "mouse" => self.mouse_command(arg),
            "tab" => self.emacs_tab(arg),
            "complete" => self.complete_at_point(),
            "placeRegister" => {
                self.char_for = Some((format!("place:{arg}"), None));
                self.say(match arg { "buffer" => "Buffer to register: ", "file" => "File to register: ", _ => "Window configuration to register: " });
            }
            "clone" => self.clone_or_rename(arg),
            "popGlobalMark" => self.pop_global_mark(),
            "repeatComplex" => self.repeat_complex(),
            "hiLock" => self.hi_lock(arg),
            "abbrevAdd" => self.abbrev_add(arg),
            "abbrevExpand" => {
                if !self.expand_abbrev_here() {
                    self.say("No abbrev here");
                }
            }
            "abbrevPrefix" => self.abbrev_prefix_mark(),
            "abbrevMode" => {
                self.abbrev_mode = !self.abbrev_mode;
                self.say(if self.abbrev_mode { "Abbrev mode enabled" } else { "Abbrev mode disabled" });
            }
            "emojiList" => self.emoji_list(),
            "describeChar" => self.describe_char(),
            "layout" => {
                self.tree.relayout(arg);
                self.win_rects.clear();
            }
            "fitWindow" => {
                // as tall as the buffer (C-x w -); "shrink": only when it is taller (C-x -)
                let lines = self.docs[self.cur].ed.buf.len_lines() + 1;
                let h = self.win_rects.iter().find(|(i, _)| *i == self.focus).map_or(0, |(_, r)| r.height as usize);
                if arg != "shrink" || h > lines {
                    self.vim_window("_", Some(lines));
                }
            }
            "deleteWindowsOn" => {
                let Some(d) = self.docs.iter().position(|d| self.buffer_label(d) == arg) else {
                    self.say(format!("No buffer {arg}"));
                    return true;
                };
                let on: Vec<usize> = self.tree.ids().into_iter().filter(|&w| self.wins.get(w).and_then(|w| w.as_ref()).is_some_and(|w| w.doc == d) || (w == self.focus && self.cur == d)).collect();
                for w in on {
                    if self.tree.ids().len() < 2 {
                        break;
                    }
                    self.focus_win(w);
                    self.doom_ui("closeWin", "");
                }
            }
            "frame" => self.frame_toggle(arg),
            "vc" => {
                let (w, a) = arg.split_once(' ').unwrap_or((arg, ""));
                self.vc(w, a);
            }
            "evalLua" => self.eval_lua(arg),
            "webSearch" => {
                let ed = &self.docs[self.cur].ed;
                let text = match ed.region() {
                    Some((a, b)) => ed.buf.slice(a, b),
                    None => ed.word_at_cursor().unwrap_or_default(),
                };
                if text.trim().is_empty() {
                    self.say("Nothing to search for");
                    return true;
                }
                let q: String = text.split_whitespace().collect::<Vec<_>>().join(" ").chars().map(|c| if c.is_ascii_alphanumeric() { c.to_string() } else { c.to_string().bytes().map(|b| format!("%{b:02X}")).collect() }).collect();
                let _ = std::process::Command::new("xdg-open").arg(format!("https://duckduckgo.com/?q={q}")).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).spawn();
            }
            "highlightSymbol" => match self.docs[self.cur].ed.word_at_cursor() {
                Some(w) => self.highlight(&format!("\\<{w}\\>")),
                None => self.say("No symbol here"),
            },
            "unhighlight" => {
                self.hl_slot = 0;
                self.hl_patterns.clear();
                for m in ["match none", "2match none", "3match none"] {
                    self.command(m);
                }
            }
            "bookmarkSet" => self.bookmark_set(arg),
            "bookmarkJump" => self.bookmark_jump(arg),
            "bookmarkList" => {
                let rows: Vec<(String, String)> = bookmarks().into_iter().map(|(n, p, l)| (n, format!("{p}:{}", l + 1))).collect();
                if rows.is_empty() {
                    self.say("No bookmarks (C-x r m sets one)");
                } else {
                    self.popup = Some(Popup::Help("Bookmarks".into(), rows));
                }
            }
            "insertBuffer" => {
                let Some(i) = self.docs.iter().position(|d| self.buffer_label(d) == arg) else {
                    self.say(format!("No buffer {arg}"));
                    return true;
                };
                let text = self.docs[i].ed.text();
                let at = self.docs[self.cur].ed.buf.to_char(self.docs[self.cur].ed.cursor());
                let before = self.docs[self.cur].ed.cursor().line;
                self.doc().ed.replace_text(at, at, &text);
                self.doc().touched(before);
            }
            "kmacroStart" => {
                if self.kmacro.is_some() {
                    self.say("Already defining a keyboard macro");
                } else {
                    self.kmacro = Some(Vec::new());
                    self.say("Defining keyboard macro...");
                }
            }
            "kmacroEnd" => match self.kmacro.take() {
                Some(mut m) => {
                    // the keys that ended it are not part of it
                    m.truncate(m.len().saturating_sub(self.ran_keys));
                    self.last_kmacro = m;
                    self.say("Keyboard macro defined");
                }
                None => self.say("Not defining a keyboard macro"),
            },
            "kmacroEndAndCall" => {
                if let Some(mut m) = self.kmacro.take() {
                    m.truncate(m.len().saturating_sub(self.ran_keys));
                    self.last_kmacro = m;
                }
                let n = self.arg.take().unwrap_or(1).max(1) as usize;
                self.kmacro_call(n);
                self.kmacro_again = true;
            }
            // F3: start recording, or while recording write the counter (kept in the macro as the
            // counter, not as F3: played back, it writes the counter of then)
            "kmacroStartOrCounter" | "kmacroInsertCounter" => {
                if let Some(m) = self.kmacro.as_mut() {
                    m.truncate(m.len().saturating_sub(self.ran_keys));
                    m.push(KMACRO_COUNTER.to_string());
                    self.kmacro_insert_counter();
                } else if name == "kmacroStartOrCounter" {
                    self.kmacro = Some(Vec::new());
                    self.say("Defining keyboard macro...");
                } else {
                    self.kmacro_insert_counter();
                }
            }
            // C-x q while recording: playback stops here to ask
            "kmacroQuery" => match self.kmacro.as_mut() {
                Some(m) => {
                    m.truncate(m.len().saturating_sub(self.ran_keys));
                    m.push(KMACRO_QUERY.to_string());
                    self.say("Query point set: playback asks here");
                }
                None => self.say("Not defining or executing kbd macro"),
            },
            "kmacroRegionLines" => self.kmacro_region_lines(),
            // C-\: the input method on or off (the last one chosen, TeX at first); C-x \: for one character
            "inputMethodToggle" | "inputMethodOnce" => {
                if self.input_method.is_some() && name == "inputMethodToggle" {
                    self.input_method = None;
                    self.say("Input method off");
                } else {
                    self.input_method = Some(self.last_input_method.clone());
                    self.input_method_once = name == "inputMethodOnce";
                    self.say(format!("Input method: {}{}", self.last_input_method, if self.input_method_once { " (one character)" } else { "" }));
                }
            }
            // C-x x @ / sudo-save: sudo needs the terminal for its password, so the run loop does it
            "changelogEntry" => self.changelog_entry(),
            "changelogUpdate" => self.changelog_update(),
            "sudoRead" => self.sudo = Some(false),
            "sudoWrite" => self.sudo = Some(true),
            // C-h C: the buffer's coding system and what zett can read and write
            "describeCoding" => {
                let i = self.cur;
                let rows = vec![
                    ("buffer".to_string(), self.coding_name(i)),
                    ("fileencoding".to_string(), self.setting(i, "fileencoding")),
                    ("fileformat".to_string(), self.setting(i, "fileformat")),
                    ("bomb".to_string(), self.setting(i, "bomb")),
                    ("keyboard, terminal".to_string(), "utf-8".to_string()),
                    ("can save as".to_string(), "utf-8, utf-8-with-signature, latin-1; each -unix -dos -mac".to_string()),
                ];
                self.popup = Some(Popup::Help("Coding system".into(), rows));
            }
            // C-h I: what the method turns into what
            "inputMethodDescribe" => {
                let m = self.input_method.clone().unwrap_or_else(|| self.last_input_method.clone());
                let rows: Vec<(String, String)> = crate::inputmethod::describe(&m);
                self.popup = Some(Popup::Help(format!("Input method {m}"), rows));
            }
            // a named macro (C-x C-k n), played
            "kmacroNamed" => match self.named_kmacros.get(arg).cloned() {
                Some(m) => {
                    let n = self.arg.take().unwrap_or(1).max(1) as usize;
                    let last = std::mem::replace(&mut self.last_kmacro, m.clone());
                    self.kmacro_run(m, n - 1);
                    self.last_kmacro = last;
                }
                None => self.say(format!("No keyboard macro named {arg}")),
            },
            _ => return false,
        }
        true
    }

    /// The last keyboard macro, `n` times.
    pub(crate) fn kmacro_call(&mut self, n: usize) {
        if self.last_kmacro.is_empty() {
            return self.say("No keyboard macro defined");
        }
        let m = self.last_kmacro.clone();
        self.kmacro_run(m, n.saturating_sub(1));
        self.kmacro_again = true;
    }

    /// Plays `keys`, then the last macro `reps` times more; a query point (C-x q) stops it to ask.
    fn kmacro_run(&mut self, keys: Vec<String>, reps: usize) {
        let mut it = keys.into_iter();
        while let Some(chunk) = it.next() {
            if chunk == KMACRO_COUNTER {
                self.kmacro_insert_counter();
                continue;
            }
            if chunk == KMACRO_QUERY {
                self.kmacro_paused = Some((it.collect(), reps));
                return self.say("Proceed with macro? (y, n, q)");
            }
            for k in zett_edit::keys::parse(&chunk) {
                self.key(input::key_event(&k));
            }
        }
        if reps > 0 {
            let m = self.last_kmacro.clone();
            self.kmacro_run(m, reps - 1);
        }
    }

    /// The answer at a query point: y (or SPC) goes on, n (or DEL) skips the rest of this repetition,
    /// q (or RET, C-g) stops.
    pub(crate) fn kmacro_answer(&mut self, k: KeyEvent, rest: Vec<String>, reps: usize) {
        match k.code {
            KeyCode::Char('y') | KeyCode::Char(' ') => self.kmacro_run(rest, reps),
            KeyCode::Char('n') | KeyCode::Backspace | KeyCode::Delete => {
                if reps > 0 {
                    let m = self.last_kmacro.clone();
                    self.kmacro_run(m, reps - 1);
                }
            }
            KeyCode::Char('q') | KeyCode::Enter | KeyCode::Esc => self.say("Keyboard macro stopped"),
            KeyCode::Char('g') if k.modifiers.contains(KeyModifiers::CONTROL) => self.say("Keyboard macro stopped"),
            // anything else: asked again
            _ => {
                self.kmacro_paused = Some((rest, reps));
                self.say("Proceed with macro? (y, n, q)");
            }
        }
    }

    /// The counter as its format writes it ("%d", "%03d", "%x", "%X", "%o"), then counted on.
    fn kmacro_counter_text(&mut self) -> String {
        let n = self.kmacro_counter;
        let f = self.kmacro_format.clone();
        let text = match f.find('%') {
            Some(i) => {
                let spec: String = f[i + 1..].chars().take_while(|c| c.is_ascii_digit()).collect();
                let kind = f[i + 1 + spec.len()..].chars().next().unwrap_or('d');
                let width: usize = spec.trim_start_matches('0').parse().unwrap_or(0);
                let zero = spec.starts_with('0');
                let body = match kind {
                    'x' => format!("{n:x}"),
                    'X' => format!("{n:X}"),
                    'o' => format!("{n:o}"),
                    _ => n.to_string(),
                };
                let body = if body.len() < width { format!("{}{body}", if zero { "0" } else { " " }.repeat(width - body.len())) } else { body };
                format!("{}{body}{}", &f[..i], &f[i + 1 + spec.len() + kind.len_utf8()..])
            }
            None => f,
        };
        text
    }

    /// The counter written at the cursor (C-x C-k C-i, F3 while recording), then up by the argument (1).
    fn kmacro_insert_counter(&mut self) {
        let by = self.arg.take().unwrap_or(1);
        let text = self.kmacro_counter_text();
        let ed = &self.docs[self.cur].ed;
        let at = ed.buf.to_char(ed.cursor());
        let line = ed.cursor().line;
        self.doc().ed.replace_text(at, at, &text);
        self.doc().touched(line);
        self.kmacro_counter += by;
    }

    /// C-x C-k r: the last macro once on each line of the region, from the line's start.
    fn kmacro_region_lines(&mut self) {
        let ed = &self.docs[self.cur].ed;
        let Some((a, b)) = ed.region() else { return self.say("The mark is not set now, so there is no region") };
        if self.last_kmacro.is_empty() {
            return self.say("No keyboard macro defined");
        }
        let (l1, l2) = (ed.buf.to_pos(a).line, ed.buf.to_pos(b.saturating_sub(1).max(a)).line);
        let m = self.last_kmacro.clone();
        for l in l1..=l2 {
            if l >= self.docs[self.cur].ed.buf.len_lines() {
                break;
            }
            self.doc().ed.set_cursor(zett_core::Pos { line: l, col: 0 });
            self.kmacro_run(m.clone(), 0);
        }
    }

    /// C-h k: a key of the sequence being described; a whole one says what it runs.
    pub(crate) fn describe_key_step(&mut self, tok: String) {
        let Some((brief, mut seq)) = self.describing.take() else { return };
        seq.push(tok);
        let shown = minibuf::emacs_keys(&seq.concat());
        match self.engine.lookup("e", &seq) {
            Err(()) => {
                self.say(format!("Describe key: {shown}-"));
                self.describing = Some((brief, seq));
            }
            Ok(None) => {
                let typed = seq.len() == 1 && seq[0].chars().count() == 1;
                self.say(if typed { format!("{shown} runs the command self-insert-command") } else { format!("{shown} is undefined") });
            }
            Ok(Some((cmd, desc))) if cmd.is_empty() => self.say(format!("{shown} runs {desc}")),
            Ok(Some((cmd, desc))) => {
                if brief {
                    self.say(format!("{shown} runs the command {cmd}"));
                } else {
                    self.open_help(&cmd);
                    self.say(format!("{shown} runs the command {cmd}: {desc}"));
                }
            }
        }
    }
}

/// Bookmarks (C-x r m): name, file, line; kept in zett's state folder, one per line.
fn bookmarks_file() -> std::path::PathBuf {
    doom::state_dir().join("bookmarks")
}
pub(crate) fn bookmarks() -> Vec<(String, String, usize)> {
    std::fs::read_to_string(bookmarks_file())
        .unwrap_or_default()
        .lines()
        .filter_map(|l| {
            let mut p = l.splitn(3, '\t');
            Some((p.next()?.to_string(), p.next()?.to_string(), p.next()?.parse().ok()?))
        })
        .collect()
}

impl App {
    /// C-x z: the last command again (z again: once more).
    pub(crate) fn repeat_last(&mut self) {
        let fx = self.last_fx.clone();
        if fx.is_empty() {
            return self.say("No command to repeat");
        }
        self.after_lua(fx);
        self.repeat_again = true;
    }

    /// Lua as Emacs Lisp: C-x C-e the line up to the cursor, C-M-x the definition, C-c C-e the
    /// region or the buffer; C-j (`print`) writes the value in after the line.
    pub(crate) fn eval_lua(&mut self, what: &str) {
        let ed = &self.docs[self.cur].ed;
        let at = ed.buf.to_char(ed.cursor());
        let line_start = ed.buf.to_char(zett_core::Pos::new(ed.cursor().line, 0));
        let code = match what {
            "defun" => ed.defun_range().map(|(a, b)| ed.buf.slice(a, b)),
            "region" => Some(ed.region().map_or_else(|| ed.text(), |(a, b)| ed.buf.slice(a, b))),
            _ => Some(ed.buf.slice(line_start, at)),
        };
        let Some(code) = code.filter(|c| !c.trim().is_empty()) else { return self.say("Nothing to evaluate here") };
        match self.with_lua(|e| e.eval(&code)) {
            Ok((out, fx)) => {
                self.after_lua(fx);
                if what == "print" {
                    let ed = &self.docs[self.cur].ed;
                    let at = ed.buf.to_char(ed.cursor());
                    self.doc().ed.replace_text(at, at, &format!("\n{out}\n"));
                } else {
                    self.say(if out.is_empty() { "nil".to_string() } else { out });
                }
            }
            Err(e) => self.say(e),
        }
    }

    /// M-s h: the pattern highlighted in the next of :match, :2match, :3match (the oldest goes).
    pub(crate) fn highlight(&mut self, pat: &str) {
        self.hl_patterns.retain(|p| p != pat);
        self.hl_patterns.push(pat.to_string());
        self.hl_slot = self.hl_slot % 3 + 1;
        let (cmd, group) = [("match", "Search"), ("2match", "IncSearch"), ("3match", "Visual")][self.hl_slot as usize - 1];
        self.command(&format!("{cmd} {group} /{}/", pat.replace('/', "\\/")));
    }

    /// C-x r m: a bookmark here (by the file's name when none is given).
    pub(crate) fn bookmark_set(&mut self, name: &str) {
        let d = &self.docs[self.cur];
        let Some(path) = d.abs_path() else { return self.say("Only a file can have a bookmark") };
        let name = if name.trim().is_empty() { path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default() } else { name.trim().to_string() };
        let line = d.ed.cursor().line;
        let mut all: Vec<(String, String, usize)> = bookmarks().into_iter().filter(|b| b.0 != name).collect();
        all.push((name.clone(), path.display().to_string(), line));
        let text: String = all.iter().map(|(n, p, l)| format!("{n}\t{p}\t{l}\n")).collect();
        let _ = std::fs::create_dir_all(doom::state_dir());
        match std::fs::write(bookmarks_file(), text) {
            Ok(()) => self.say(format!("Bookmark \"{name}\" set")),
            Err(e) => self.say(format!("could not keep the bookmark: {e}")),
        }
    }

    /// C-x r b: to a bookmark's file and line.
    pub(crate) fn bookmark_jump(&mut self, name: &str) {
        let Some((_, path, line)) = bookmarks().into_iter().find(|b| b.0 == name.trim()) else { return self.say(format!("No bookmark \"{}\"", name.trim())) };
        self.open(std::path::Path::new(&path));
        self.goto_line(line + 1);
    }
}

impl App {
    /// The window whose status line is at (col, row), or whose right edge (the line between it and
    /// the next) is column `col`: (window, on the divider).
    fn border_at(&self, col: u16, row: u16) -> Option<(usize, bool)> {
        let status = self.win_rects.iter().find(|(_, r)| row + 1 == r.bottom() && col >= r.x && col < r.right()).map(|(i, _)| (*i, false));
        let divider = || self.win_rects.iter().find(|(_, r)| col == r.right() && row >= r.y && row < r.bottom()).map(|(i, _)| (*i, true));
        // a single window's status line has nothing to resize against, but splits and selects
        status.or_else(divider)
    }

    /// Emacs's mode-line and divider mouse (the same in the Vim profile): drag a status line or the
    /// line between windows to resize (mouse-drag-mode-line, mouse-drag-vertical-line), click it to
    /// select the window, C-middle-click to split there. True when the event was this.
    pub(crate) fn window_mouse(&mut self, m: &ratatui::crossterm::event::MouseEvent) -> bool {
        use ratatui::crossterm::event::{MouseButton, MouseEventKind};
        match m.kind {
            MouseEventKind::Drag(MouseButton::Left) => {
                let Some((w, vertical, last)) = self.border_drag else { return false };
                let now = if vertical { m.column } else { m.row };
                let d = now as i32 - last as i32;
                if d != 0 {
                    if self.focus != w {
                        self.focus_win(w);
                    }
                    let key = match (vertical, d > 0) {
                        (true, true) => ">",
                        (true, false) => "<",
                        (false, true) => "+",
                        (false, false) => "-",
                    };
                    self.vim_window(key, Some(d.unsigned_abs() as usize));
                    self.border_drag = Some((w, vertical, now));
                }
                true
            }
            MouseEventKind::Up(MouseButton::Left) => self.border_drag.take().is_some(),
            MouseEventKind::Down(MouseButton::Left) => {
                let Some((w, vertical)) = self.border_at(m.column, m.row) else { return false };
                if self.focus != w {
                    self.focus_win(w);
                }
                self.border_drag = Some((w, vertical, if vertical { m.column } else { m.row }));
                true
            }
            MouseEventKind::Down(MouseButton::Middle) if m.modifiers.contains(ratatui::crossterm::event::KeyModifiers::CONTROL) => {
                let Some((w, vertical)) = self.border_at(m.column, m.row) else { return false };
                let Some(r) = self.win_rects.iter().find(|(i, _)| *i == w).map(|(_, r)| *r) else { return false };
                self.focus_win(w);
                if vertical {
                    // on the divider: a window below, from this row
                    self.command("split");
                    self.command(&format!("resize {}", m.row.saturating_sub(r.y).max(1)));
                } else {
                    // on the status line: a window beside, from this column
                    self.command("vsplit");
                    self.command(&format!("vertical resize {}", m.column.saturating_sub(r.x).max(1)));
                }
                true
            }
            _ => false,
        }
    }

    /// Emacs's named mouse commands, for keys bound to them: what the mouse does at the last
    /// mouse event, or (from M-x) what it would.
    pub(crate) fn mouse_command(&mut self, name: &str) {
        use ratatui::crossterm::event::{MouseButton, MouseEventKind};
        let at = self.last_mouse;
        let redo = |app: &mut App, kind: MouseEventKind| {
            if let Some(mut ev) = at {
                ev.kind = kind;
                ev.modifiers = ratatui::crossterm::event::KeyModifiers::NONE;
                app.noremap += 1;
                app.mouse(ev);
                app.noremap -= 1;
            }
        };
        match name {
            "mouse-set-point" | "mouse-drag-region" | "mouse-select-window" | "mouse-drag-mode-line" | "mouse-drag-vertical-line" => redo(self, MouseEventKind::Down(MouseButton::Left)),
            "mouse-set-region" | "mouse-drag-region-rectangle" => redo(self, MouseEventKind::Drag(MouseButton::Left)),
            "mouse-yank-primary" => redo(self, MouseEventKind::Down(MouseButton::Middle)),
            "mwheel-scroll" => redo(self, at.map_or(MouseEventKind::ScrollDown, |e| e.kind)),
            "mouse-save-then-kill" => {
                // the region to the click (copied); the same click again kills it
                let again = self.last_key_cmd.as_deref() == Some("mouse-save-then-kill");
                if again {
                    self.after_lua(vec![Effect::Edit("kill-region".into(), 1)]);
                } else {
                    if self.docs[self.cur].ed.region().is_none() {
                        self.doc().ed.exec("set-mark-command", 1);
                    }
                    redo(self, MouseEventKind::Down(MouseButton::Left));
                    self.after_lua(vec![Effect::Edit("kill-ring-save".into(), 1)]);
                }
            }
            "mouse-split-window-horizontally" | "mouse-split-window-vertically" => {
                if let Some(mut ev) = at {
                    ev.kind = MouseEventKind::Down(MouseButton::Middle);
                    ev.modifiers = ratatui::crossterm::event::KeyModifiers::CONTROL;
                    if !self.window_mouse(&ev) {
                        self.command(if name.ends_with("horizontally") { "vsplit" } else { "split" });
                    }
                } else {
                    self.command(if name.ends_with("horizontally") { "vsplit" } else { "split" });
                }
            }
            _ => self.say(format!("{name}: a mouse command")),
        }
        self.last_key_cmd = Some(name.to_string());
    }

    /// F11 and M-F10: the terminal window full screen or maximized (Hyprland's, when it runs).
    pub(crate) fn frame_toggle(&mut self, how: &str) {
        if std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_none() {
            return self.say("full screen is the terminal's (no Hyprland here)");
        }
        let mode = if how == "maximized" { "1" } else { "0" };
        let _ = std::process::Command::new("hyprctl").args(["dispatch", "fullscreen", mode]).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).spawn();
    }
}

impl App {
    /// Emacs's tab-bar (C-x t) on zett's tab pages: "new", "close", "next" … with an argument
    /// after a space where the command takes one (a name, a place, a buffer).
    pub(crate) fn emacs_tab(&mut self, spec: &str) {
        let (what, arg) = spec.split_once(' ').unwrap_or((spec, ""));
        let n = self.arg.take().unwrap_or(1);
        match what {
            "new" => self.tab_new(None, None),
            "group" => self.tab_group(arg),
            "new-to" => {
                // C-x t N: at a place (the argument; 0 or none: after this one)
                let at = (n > 0).then(|| (n as usize).saturating_sub(1)).filter(|_| n != 1);
                self.tab_new(None, at);
            }
            "duplicate" => {
                // the same windows in a new tab
                let (wins, tree, focus) = (self.wins.clone(), self.tree.clone(), self.focus);
                self.tab_new(None, None);
                self.wins = wins;
                self.tree = tree;
                self.focus_win(focus);
            }
            "close" => {
                let t = self.tab;
                self.tab_close(t);
            }
            "close-other" => self.tab_only(),
            "next" => self.tab_step(n),
            "previous" => self.tab_step(-n),
            "move" => {
                let to = (self.tab as i64 + n).clamp(0, self.tab_count() as i64 - 1) as usize;
                self.tab_move(to);
            }
            "move-to" => self.tab_move((n.max(1) - 1) as usize),
            "rename" => self.tab_rename(arg),
            "switch" => {
                if let Some(i) = (0..self.tab_count()).find(|&i| self.tab_label(i) == arg.trim()) {
                    self.tab_goto(i);
                } else if let Ok(k) = arg.trim().parse::<usize>() {
                    self.tab_goto(k.saturating_sub(1));
                } else {
                    self.say(format!("No tab {}", arg.trim()));
                }
            }
            "undo" => self.tab_undo(),
            other => self.say(format!("tab: no {other}")),
        }
    }

    /// The tabs' labels, for C-x t RET.
    pub(crate) fn tab_items(&self) -> Vec<Cand> {
        (0..self.tab_count()).map(|i| Cand::noted(self.tab_label(i), format!("tab {}", i + 1))).collect()
    }
}

impl App {
    /// An abbrev's expansion: this file type's (C-x a +), else everyone's (C-x a g, :iabbrev).
    pub(crate) fn abbrev_for(&self, word: &str) -> Option<String> {
        let syntax = &self.docs[self.cur].syntax;
        let find = |list: &Vec<(String, String)>| list.iter().find(|(l, _)| l == word).map(|(_, r)| r.clone());
        self.mode_abbrevs.get(syntax).and_then(find).or_else(|| find(&self.abbrevs))
    }

    /// The word before the cursor (from M-'s mark when one was set): (start char, the word).
    fn word_before(&self) -> Option<(usize, String)> {
        let ed = &self.docs[self.cur].ed;
        let at = ed.buf.to_char(ed.cursor());
        let line_start = ed.buf.to_char(zett_core::Pos::new(ed.cursor().line, 0));
        let start = match self.abbrev_prefix.filter(|&p| p >= line_start && p < at) {
            Some(p) => p,
            None => (line_start..at).rev().take_while(|&i| ed.buf.char_at(i).is_some_and(|c| c.is_alphanumeric() || c == '_')).last()?,
        };
        Some((start, ed.buf.slice(start, at)))
    }

    /// C-x ' (and abbrev-mode as you type): the word before the cursor as its expansion.
    pub(crate) fn expand_abbrev_here(&mut self) -> bool {
        let Some((start, word)) = self.word_before() else { return false };
        let word = word.trim_start_matches('-').to_string();
        let Some(rhs) = self.abbrev_for(&word) else { return false };
        let ed = &self.docs[self.cur].ed;
        let at = ed.buf.to_char(ed.cursor());
        // M-': the "-" it left goes, the prefix before it joins the expansion
        let from = match self.abbrev_prefix.take() {
            Some(p) if ed.buf.char_at(p) == Some('-') => p,
            _ => start,
        };
        let before = ed.cursor().line;
        self.doc().ed.replace_text(from, at, &rhs);
        self.doc().touched(before);
        true
    }

    /// C-x a g / C-x a + (with `mode`): the words before the cursor (the argument's count; 0: the
    /// region) become an expansion, its name asked for. C-x a i g / i l: the word before the cursor
    /// is the name, the expansion asked for (and put in at once).
    pub(crate) fn abbrev_add(&mut self, how: &str) {
        let n = self.arg.take().unwrap_or(1);
        let (mode, inverse) = (how.contains("mode"), how.starts_with("inverse"));
        if inverse {
            let Some((_, word)) = self.word_before() else { return self.say("No word before the cursor") };
            self.pending_abbrev = Some((word.clone(), mode));
            return self.prompt(&format!("{} expansion for \"{word}\": \tabbrev-inverse:{{1}}", if mode { "Mode" } else { "Global" }));
        }
        let ed = &self.docs[self.cur].ed;
        let at = ed.buf.to_char(ed.cursor());
        let text = if n == 0 {
            ed.region().map(|(a, b)| ed.buf.slice(a, b)).unwrap_or_default()
        } else {
            // the n words before the cursor
            let before = ed.buf.slice(0, at);
            let mut words = before.split_whitespace().rev().take(n.max(1) as usize).collect::<Vec<_>>();
            words.reverse();
            words.join(" ")
        };
        if text.is_empty() {
            return self.say("No text for an abbrev");
        }
        self.pending_abbrev = Some((text.clone(), mode));
        self.prompt(&format!("{} abbrev for \"{text}\": \tabbrev-add:{{1}}", if mode { "Mode" } else { "Global" }));
    }

    /// The prompt's answer: the abbrev defined (and, inverse, the word expanded).
    pub(crate) fn abbrev_answer(&mut self, inverse: bool, answer: &str) {
        let Some((pending, mode)) = self.pending_abbrev.take() else { return };
        let answer = answer.trim();
        if answer.is_empty() {
            return;
        }
        let (name, rhs) = if inverse { (pending, answer.to_string()) } else { (answer.to_string(), pending) };
        let list = if mode { self.mode_abbrevs.entry(self.docs[self.cur].syntax.clone()).or_default() } else { &mut self.abbrevs };
        list.retain(|(l, _)| *l != name);
        list.push((name.clone(), rhs));
        if inverse {
            self.expand_abbrev_here();
        }
        self.say(format!("{} abbrev \"{name}\" defined", if mode { "Mode" } else { "Global" }));
    }

    /// M-': a "-" here, where the next expansion starts (a prefix before it stays).
    pub(crate) fn abbrev_prefix_mark(&mut self) {
        self.expand_abbrev_here();
        let ed = &self.docs[self.cur].ed;
        let at = ed.buf.to_char(ed.cursor());
        self.doc().ed.replace_text(at, at, "-");
        self.abbrev_prefix = Some(at);
    }
}

impl App {
    /// The mouse over a help page: a `key` or `name` (in backticks) under it gets a big loop
    /// drawn round it by zash, gone again when the mouse leaves it.
    pub(crate) fn help_hover(&mut self, p: Option<zett_core::Pos>, row: u16) {
        if self.fx_quiet() {
            self.fx_hover = None;
            return;
        }
        let d = &self.docs[self.cur];
        let span = p.filter(|_| crate::draw::fx_draws() && d.name.starts_with("help: ") && d.left == 0).and_then(|p| {
            let line: Vec<char> = d.ed.buf.line(p.line).chars().filter(|c| *c != '\n').collect();
            tick_span(&line, p.col).map(|(start, text)| (p.line, start, text))
        });
        self.fx_hover = span.map(|(line, start, text)| crate::draw::FxMark {
            x: self.area.x + self.gutter + start as u16,
            y: row,
            key: format!("hover:{}:{line}:{start}", self.docs[self.cur].name),
            text,
            since: Instant::now(),
            spec: CIRCLE,
            transient: true,
        });
    }
}

impl App {
    /// A button held, or still inside the wait after a scroll, release or key: no circles yet.
    pub(crate) fn fx_quiet(&self) -> bool {
        self.mouse_held || self.fx_quiet_until.is_some_and(|t| Instant::now() < t)
    }

    /// The circles wait at least `d` from now.
    pub(crate) fn fx_wait(&mut self, d: std::time::Duration) {
        let until = Instant::now() + d;
        self.fx_quiet_until = Some(self.fx_quiet_until.map_or(until, |t| t.max(until)));
    }

    /// The scroll has rested: the circle under the mouse again, where the text is now.
    pub(crate) fn fx_rested(&mut self) -> bool {
        let Some(t) = self.fx_quiet_until else { return false };
        if Instant::now() < t {
            return false;
        }
        self.fx_quiet_until = None;
        if !self.mouse_held {
            use ratatui::crossterm::event::{KeyModifiers, MouseEvent, MouseEventKind};
            self.mouse(MouseEvent { kind: MouseEventKind::Moved, column: self.mouse_at.0, row: self.mouse_at.1, modifiers: KeyModifiers::empty() });
        }
        true
    }
}

pub(crate) const FX_QUIET: std::time::Duration = std::time::Duration::from_millis(350);
/// After a key: longer than the keyboard's repeat delay (250 ms here), so a held key never draws.
pub(crate) const FX_KEY_QUIET: std::time::Duration = std::time::Duration::from_millis(300);

/// zash's big hand-drawn loop: round what the mouse or the cursor is on in a help page.
pub(crate) const CIRCLE: &str = "scribble;write=0;color=accent;turns=1.5;wobble=0.1;pad=0.5;draw=450ms;sound=draw;volume=0.45";

/// The `name` in backticks round column `col` of `line`: where its text starts, and the text.
pub(crate) fn tick_span(line: &[char], col: usize) -> Option<(usize, String)> {
    let open = (0..col.min(line.len())).rev().find(|&i| line[i] == '`')?;
    let close = (col..line.len()).find(|&i| line[i] == '`')?;
    // inside one pair: an even number of backticks before the opening one
    let before = line[..open].iter().filter(|c| **c == '`').count();
    (before % 2 == 0 && close > open + 1 && !line[open + 1..close].iter().any(|c| *c == '\t')).then(|| (open + 1, line[open + 1..close].iter().collect()))
}

/// What an Emacs register holds besides text and a place in the buffer (those are Vim's).
#[derive(Clone)]
pub(crate) enum Place {
    Buffer(String),
    File(PathBuf, usize),
    /// the windows: their layout, each one's buffer by name (buffers may move), the focused one
    Windows(crate::doom::WTree, Vec<Option<(Win, String)>>, usize),
}

impl App {
    /// C-x r B / F / w / f: this buffer, its file (and line), the windows, kept in register `r`.
    pub(crate) fn place_to_register(&mut self, what: &str, r: char) {
        let d = &self.docs[self.cur];
        let place = match what {
            "buffer" => Place::Buffer(self.buffer_label(d)),
            "file" => match d.abs_path() {
                Some(p) => Place::File(p, d.ed.cursor().line),
                None => return self.say("This buffer is not a file"),
            },
            _ => {
                let cur = self.cur;
                if let Some(Some(w)) = self.wins.get_mut(self.focus) {
                    w.doc = cur;
                }
                let wins = self.wins.iter().map(|w| w.as_ref().map(|w| (w.clone(), self.docs.get(w.doc).map(|d| d.name.clone()).unwrap_or_default()))).collect();
                Place::Windows(self.tree.clone(), wins, self.focus)
            }
        };
        self.places.insert(r, place);
        self.say(format!("Saved to register {r}"));
    }

    /// C-x r j on one of those registers; false when `r` is not one (the editor's mark then).
    pub(crate) fn jump_to_place(&mut self, r: char) -> bool {
        let Some(place) = self.places.get(&r).cloned() else { return false };
        match place {
            Place::Buffer(name) => match self.docs.iter().position(|d| self.buffer_label(d) == name) {
                Some(i) => self.show_doc(i),
                None => self.say(format!("No buffer {name} any more")),
            },
            Place::File(p, line) => {
                self.open(&p);
                self.goto_line(line + 1);
            }
            Place::Windows(tree, wins, focus) => {
                let wins = wins
                    .into_iter()
                    .map(|w| {
                        w.map(|(mut w, name)| {
                            w.doc = self.docs.iter().position(|d| d.name == name).unwrap_or(0);
                            w
                        })
                    })
                    .collect();
                self.wins = wins;
                self.tree = tree;
                self.win_rects.clear();
                self.focus_win(focus);
            }
        }
        true
    }

    /// C-M-i: the language server's completions here (else the buffer's words, M-/).
    pub(crate) fn complete_at_point(&mut self) {
        if self.docs[self.cur].lsp {
            self.comp_due = Some(Instant::now());
        } else {
            self.after_lua(vec![Effect::Edit("dabbrev-expand".into(), 1)]);
        }
    }

    /// C-x x n: a copy of this buffer as a new one (name<2>); C-x x u: this one renamed so
    /// another of its name can come (a buffer that is not a file).
    pub(crate) fn clone_or_rename(&mut self, how: &str) {
        let base = self.docs[self.cur].name.clone();
        let free = |app: &App| (2..).map(|n| format!("{base}<{n}>")).find(|n| !app.docs.iter().any(|d| d.name == *n)).unwrap_or_default();
        let name = free(self);
        if how == "rename" {
            if self.docs[self.cur].abs_path().is_some() {
                return self.say("A file's buffer keeps its file's name");
            }
            self.docs[self.cur].name = name;
            return;
        }
        let d = &self.docs[self.cur];
        let mut ed = Editor::from_str(&d.ed.text());
        ed.set_emacs(d.ed.is_emacs());
        let syntax = d.syntax.clone();
        self.docs.push(Doc::new(ed, name, syntax));
        let i = self.docs.len() - 1;
        if how == "indirect" {
            self.doom_ui("split", "right");
        }
        self.show_doc(i);
    }

    /// C-x C-@: back to the last mark set in another buffer (the global mark ring).
    pub(crate) fn pop_global_mark(&mut self) {
        let here = self.docs[self.cur].name.clone();
        let Some(k) = self.global_marks.iter().rposition(|(d, _)| *d != here) else { return self.say("No global mark set") };
        let (doc, at) = self.global_marks.remove(k);
        match self.docs.iter().position(|d| d.name == doc) {
            Some(i) => {
                self.show_doc(i);
                let pos = self.docs[i].ed.buf.to_pos(at.min(self.docs[i].ed.buf.len_chars()));
                self.doc().ed.set_cursor(pos);
            }
            None => self.say(format!("{doc} is not open any more")),
        }
    }

    /// C-x M-: the last command that asked something, again with the same answers.
    pub(crate) fn repeat_complex(&mut self) {
        match self.last_prompt.clone() {
            Some((what, got)) => self.prompt_done(&what, &got),
            None => self.say("No command to repeat"),
        }
    }

    /// M-s h w: the highlights written into the buffer, as Emacs's hi-lock writes them (in a
    /// comment, at the top); M-s h f: such lines read back and highlighted.
    pub(crate) fn hi_lock(&mut self, how: &str) {
        let cs = self.setting(self.cur, "commentstring");
        let (pre, post) = cs.split_once("%s").map(|(a, b)| (a.to_string(), b.to_string())).unwrap_or(("# ".into(), String::new()));
        if how == "write" {
            if self.hl_patterns.is_empty() {
                return self.say("No highlights to write");
            }
            let pats: Vec<String> = self.hl_patterns.iter().map(|p| format!("(\"{}\")", p.replace('"', "\\\""))).collect();
            let line = format!("{pre}Hi-lock: ({}){post}\n", pats.join(" "));
            self.doc().ed.replace_text(0, 0, &line);
            return;
        }
        let text: Vec<String> = (0..self.docs[self.cur].ed.buf.len_lines().min(10)).map(|l| self.docs[self.cur].ed.buf.line(l).to_string()).collect();
        let mut n = 0;
        for l in text.iter().filter(|l| l.contains("Hi-lock: ")) {
            for p in l.split("(\"").skip(1).filter_map(|s| s.split("\")").next()) {
                self.highlight(&p.replace("\\\"", "\""));
                n += 1;
            }
        }
        self.say(format!("{n} highlight{} found", if n == 1 { "" } else { "s" }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emacs_regexps_and_replacements() {
        assert_eq!(emacs_regex(r"\(foo\|bar\)+ (x)"), r"(foo|bar)+ \(x\)");
        assert_eq!(emacs_regex(r"\<word\>"), r"\bword\b");
        assert_eq!(emacs_replacement(r"<\1> \& $"), "<${1}> ${0} $$");
        assert_eq!(match_case("FOO", "bar"), "BAR");
        assert_eq!(match_case("Foo", "bar baz"), "Bar Baz");
        assert_eq!(match_case("foo", "bar"), "bar");
        assert_eq!(fill("s/{e1}/{e2}/", &["a.b".into(), "x/y".into()]), r"s/a\.b/x\/y/");
    }

    fn app(text: &str) -> App {
        let mut app = App::new(Vec::new());
        app.docs[app.cur].ed = zett_edit::Editor::from_str(text);
        app.docs[app.cur].ed.set_emacs(true);
        app
    }
    fn press(app: &mut App, c: char) {
        app.qr_key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
    }

    #[test]
    fn comment_dwim_adds_at_the_column_goes_to_one_and_begins_on_blank() {
        let mut a = app("let x = 1;\n\n");
        a.comment_dwim();
        let l0: String = a.docs[a.cur].ed.buf.line(0).chars().filter(|c| *c != '\n').collect();
        assert_eq!(l0, format!("let x = 1;{}// ", " ".repeat(22)), "at column 32");
        assert_eq!(a.docs[a.cur].ed.cursor().col, 35);
        // again: to just after the opening, nothing added
        a.docs[a.cur].ed.set_cursor(zett_core::Pos { line: 0, col: 0 });
        a.comment_dwim();
        assert_eq!(a.docs[a.cur].ed.cursor().col, 35);
        // C-x ; at column 12, then on another code line
        a.docs[a.cur].ed.set_cursor(zett_core::Pos { line: 0, col: 12 });
        a.comment_set_column();
        a.docs[a.cur].ed.set_cursor(zett_core::Pos { line: 1, col: 0 });
        a.comment_dwim();
        let l1: String = a.docs[a.cur].ed.buf.line(1).chars().filter(|c| *c != '\n').collect();
        assert_eq!(l1, "// ", "a blank line: begun at its indent");
    }

    #[test]
    fn shell_command_on_region_replaces_it_with_c_u() {
        let mut a = app("hello world");
        a.docs[a.cur].ed.set_region(0, 5);
        a.arg.universal();
        a.prompt_done("shreg:tr a-z A-Z", &[]);
        assert_eq!(a.docs[a.cur].ed.text(), "HELLO world");
        // without C-u the region stays as it was
        a.docs[a.cur].ed.set_region(6, 11);
        a.prompt_done("shreg:tr a-z A-Z", &[]);
        assert_eq!(a.docs[a.cur].ed.text(), "HELLO world");
    }

    #[test]
    fn query_replace_asks_at_each_match_and_keeps_case() {
        let mut a = app("foo Foo FOO foo");
        a.query_replace("foo", "bar", false, false);
        press(&mut a, 'y');
        press(&mut a, 'n');
        press(&mut a, '!');
        assert_eq!(a.docs[a.cur].ed.text(), "bar Foo BAR bar");
        assert!(a.qr.is_none(), "done at the end");
        let mut a = app("a1 b2 c3");
        a.query_replace(r"\([a-z]\)\([0-9]\)", r"\2\1", true, true);
        assert_eq!(a.docs[a.cur].ed.text(), "1a 2b 3c");
        let mut a = app("x x x");
        a.query_replace("x", "yy", false, false);
        press(&mut a, '.');
        assert_eq!(a.docs[a.cur].ed.text(), "yy x x", ". replaces one and stops");
    }

    #[test]
    fn prompts_chain_their_answers() {
        let mut a = app("one\ntwo\nthree\n");
        a.prompt("Goto line: \tnum:goto-line {1}");
        let Some(m) = a.minibuf.take() else { panic!("no minibuffer") };
        a.picked(m.kind, "3".into());
        assert_eq!(a.docs[a.cur].ed.cursor().line, 2);
        a.prompt("Replace: \tReplace {1} with: \trs:");
        let m = a.minibuf.take().unwrap();
        a.picked(m.kind, "t".into());
        let m = a.minibuf.take().unwrap();
        assert_eq!(m.title, "Replace t with", "the minibuffer adds its own colon");
        a.docs[a.cur].ed.exec("beginning-of-buffer", 1);
        a.picked(m.kind, "T".into());
        assert_eq!(a.docs[a.cur].ed.text(), "one\nTwo\nThree\n");
    }

    #[test]
    fn repeat_eval_and_highlights() {
        let mut a = app("abcdef");
        a.after_lua(vec![Effect::Edit("forward-char".into(), 1)]);
        a.last_fx = vec![Effect::Edit("forward-char".into(), 1)];
        a.repeat_last();
        assert_eq!(a.docs[a.cur].ed.cursor().col, 2, "C-x z: once more");
        let mut a = app("1 + 41");
        a.docs[a.cur].ed.exec("end-of-line", 1);
        a.eval_lua("print");
        assert_eq!(a.docs[a.cur].ed.text(), "1 + 41\n42\n", "C-j writes the value in");
        let mut a = app("foo bar foo");
        a.highlight("foo");
        a.highlight("bar");
        assert_eq!(a.hl_slot, 2, "the second goes in :2match");
    }

    #[test]
    fn status_lines_and_dividers_resize_and_split() {
        use ratatui::crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
        let ev = |kind, column, row, modifiers| MouseEvent { kind, column, row, modifiers };
        let mut a = app("text");
        a.command("vsplit");
        let ids = a.tree.ids();
        a.win_rects = vec![(ids[0], ratatui::layout::Rect::new(0, 0, 40, 20)), (ids[1], ratatui::layout::Rect::new(41, 0, 39, 20))];
        assert!(a.window_mouse(&ev(MouseEventKind::Down(MouseButton::Left), 40, 5, KeyModifiers::NONE)), "the divider");
        assert_eq!((a.focus, a.border_drag.map(|b| b.1)), (ids[0], Some(true)));
        assert!(a.window_mouse(&ev(MouseEventKind::Drag(MouseButton::Left), 44, 5, KeyModifiers::NONE)));
        assert_eq!(a.border_drag.map(|b| b.2), Some(44), "dragged 4 columns");
        assert!(a.window_mouse(&ev(MouseEventKind::Up(MouseButton::Left), 44, 5, KeyModifiers::NONE)));
        assert!(!a.window_mouse(&ev(MouseEventKind::Down(MouseButton::Left), 10, 5, KeyModifiers::NONE)), "inside the text: not this");
        let before = a.tree.ids().len();
        assert!(a.window_mouse(&ev(MouseEventKind::Down(MouseButton::Middle), 50, 19, KeyModifiers::CONTROL)), "C-middle on a status line");
        assert_eq!(a.tree.ids().len(), before + 1, "split there");
    }

    #[test]
    fn emacs_tabs() {
        let mut a = app("x");
        a.emacs_tab("new");
        a.emacs_tab("rename notes");
        assert_eq!(a.tab_label(1), "notes");
        a.emacs_tab("previous");
        assert_eq!(a.tab, 0);
        a.emacs_tab("switch notes");
        assert_eq!(a.tab, 1);
        a.emacs_tab("close");
        assert_eq!(a.tab_count(), 1);
        a.emacs_tab("undo");
        assert_eq!((a.tab_count(), a.tab_label(a.tab)), (2, "notes".to_string()), "C-x t u brings it back, name and all");
    }

    #[test]
    fn layouts_turn() {
        use crate::doom::WTree;
        let leaf = |i| Box::new(WTree::Leaf(i));
        // A above B
        let mut t = WTree::Split(false, leaf(0), leaf(1), 0.3);
        t.relayout("clockwise");
        assert!(matches!(&t, WTree::Split(true, a, b, r) if matches!(**a, WTree::Leaf(1)) && matches!(**b, WTree::Leaf(0)) && (*r - 0.7).abs() < 1e-6), "the top goes right: B | A");
        t.relayout("anticlockwise");
        assert!(matches!(&t, WTree::Split(false, a, _, _) if matches!(**a, WTree::Leaf(0))), "and back");
        t.relayout("transpose");
        assert!(matches!(&t, WTree::Split(true, a, _, _) if matches!(**a, WTree::Leaf(0))), "A | B");
        t.relayout("flip-leftright");
        assert!(matches!(&t, WTree::Split(true, a, _, _) if matches!(**a, WTree::Leaf(1))), "B | A");
    }

    #[test]
    fn abbrevs_both_ways() {
        let mut a = app("by the way");
        a.docs[a.cur].ed.exec("end-of-buffer", 1);
        a.abbrev_add("global");
        // "way" is the word before the cursor (one word)
        let m = a.minibuf.take().unwrap();
        a.picked(m.kind, "bw".into());
        assert_eq!(a.abbrev_for("bw").as_deref(), Some("way"));
        let mut a = app("I said btw");
        a.docs[a.cur].ed.exec("end-of-buffer", 1);
        a.abbrev_add("inverse-global");
        let m = a.minibuf.take().unwrap();
        a.picked(m.kind, "by the way".into());
        assert_eq!(a.docs[a.cur].ed.text(), "I said by the way", "the inverse expands at once");
        a.abbrev_mode = true;
        a.docs[a.cur].ed.replace_text(17, 17, " btw");
        assert!(a.expand_abbrev_here());
        assert_eq!(a.docs[a.cur].ed.text(), "I said by the way by the way");
    }

    #[test]
    fn registers_for_windows_and_buffers() {
        let mut a = app("one");
        a.place_to_register("windows", 'w');
        a.command("vsplit");
        a.command("vsplit");
        assert_eq!(a.tree.ids().len(), 3);
        assert!(a.jump_to_place('w'));
        assert_eq!(a.tree.ids().len(), 1, "C-x r j w: the one window again");
        a.place_to_register("buffer", 'b');
        a.doom_ui("newBuffer", "");
        let other = a.cur;
        assert!(a.jump_to_place('b'));
        assert_ne!(a.cur, other, "back to the buffer kept");
        assert!(!a.jump_to_place('z'), "not one of these: the editor's mark then");
    }
}
