//! Emacs's commands that ask before they act: minibuffer prompts (one or more answers, then an Ex
//! line, a numbered editor command or a rectangle's text), and query-replace (M-%, C-M-%) with its
//! keys: y n ! . q, the case kept as Emacs keeps it.

use super::*;

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

/// Emacs's regexp syntax in the regex crate's: \( \) \| \{ \} group and count, bare ones are
/// themselves; \< \> word edges, \` \' the buffer's ends, \s- a blank.
pub(crate) fn emacs_regex(p: &str) -> String {
    let mut out = String::new();
    let mut it = p.chars().peekable();
    while let Some(c) = it.next() {
        match c {
            '\\' => match it.next() {
                Some(d @ ('(' | ')' | '|' | '{' | '}')) => {
                    // \(?: a shy group
                    if d == '(' && it.peek() == Some(&'?') {
                        it.next();
                        if it.peek() == Some(&':') {
                            it.next();
                        }
                        out.push_str("(?:");
                    } else {
                        out.push(d);
                    }
                }
                Some('<' | '>') => out.push_str(r"\b"),
                Some('`') => out.push_str(r"\A"),
                Some('\'') => out.push_str(r"\z"),
                Some('s') => {
                    it.next();
                    out.push_str(r"\s");
                }
                Some('S') => {
                    it.next();
                    out.push_str(r"\S");
                }
                Some(d @ ('w' | 'W' | 'b' | 'B' | '1'..='9')) => {
                    out.push('\\');
                    out.push(d);
                }
                Some(d) => out.push_str(&fancy_regex::escape(&d.to_string())),
                None => out.push_str(r"\\"),
            },
            '(' | ')' | '|' | '{' | '}' => {
                out.push('\\');
                out.push(c);
            }
            c => out.push(c),
        }
    }
    out
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
        self.pick_list(Pick::Prompt(parts, what, Vec::new()), &title, items);
    }

    fn prompt_items(&self, title: &str) -> (String, Vec<Cand>) {
        if let Some(t) = title.strip_suffix(" @commands") {
            return (t.to_string(), self.engine.commands().into_iter().map(|(n, d, _)| Cand::noted(n, d)).collect());
        }
        if let Some(t) = title.strip_suffix(" @settings") {
            return (t.to_string(), crate::settings::all().into_iter().map(|s| Cand::noted(s.name, s.about)).collect());
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
        self.pick_list(Pick::Prompt(rest, what, got), &title, Vec::new());
    }

    fn prompt_done(&mut self, what: &str, got: &[String]) {
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
            _ => self.qr_finish(),
        }
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
            _ => return false,
        }
        true
    }

    /// The last keyboard macro, `n` times.
    pub(crate) fn kmacro_call(&mut self, n: usize) {
        if self.last_kmacro.is_empty() {
            return self.say("No keyboard macro defined");
        }
        let keys = self.last_kmacro.concat();
        for _ in 0..n {
            for k in zett_edit::keys::parse(&keys) {
                self.key(input::key_event(&k));
            }
        }
        self.kmacro_again = true;
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
        assert_eq!(m.title, "Replace t with: ");
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
}
