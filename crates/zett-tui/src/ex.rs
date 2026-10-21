//! Neovim's `:` commands on ranges of lines (`:%s`, `:g`, `:sort`, `:m`, `:!`…) and the
//! everyday rest (`:e!`, `:earlier 5m`, `:so %`, `:nmap`, `:bufdo`…).

use super::*;
use fancy_regex::Regex;

/// The `:` commands with a range in front: `.` `$` `%` `N` `'<,'>` `+N` `-N`, two with a comma.
pub(crate) fn parse_range(s: &str, cur: usize, last: usize, vis: Option<(usize, usize)>) -> (Option<(usize, usize)>, &str) {
    let s = s.trim_start();
    if let Some(rest) = s.strip_prefix('%') {
        return (Some((0, last)), rest);
    }
    if let Some(rest) = s.strip_prefix("'<,'>") {
        return (Some(vis.unwrap_or((cur, cur))), rest);
    }
    // one address: a number, . or $, then +N / -N
    let addr = |s: &str| -> Option<(usize, usize)> {
        let b = s.as_bytes();
        let mut i = 0;
        let mut base: Option<i64> = None;
        if i < b.len() && b[i] == b'.' {
            base = Some(cur as i64);
            i += 1;
        } else if i < b.len() && b[i] == b'$' {
            base = Some(last as i64);
            i += 1;
        } else {
            let st = i;
            while i < b.len() && b[i].is_ascii_digit() {
                i += 1;
            }
            if i > st {
                base = Some(s[st..i].parse::<i64>().ok()? - 1);
            }
        }
        while i < b.len() && (b[i] == b'+' || b[i] == b'-') {
            let sign = if b[i] == b'+' { 1 } else { -1 };
            i += 1;
            let st = i;
            while i < b.len() && b[i].is_ascii_digit() {
                i += 1;
            }
            let n: i64 = if i > st { s[st..i].parse().ok()? } else { 1 };
            base = Some(base.unwrap_or(cur as i64) + sign * n);
        }
        base.map(|v| (v.clamp(0, last as i64) as usize, i))
    };
    let Some((a, used)) = addr(s) else { return (None, s) };
    let rest = &s[used..];
    if let Some(r2) = rest.strip_prefix(',') {
        if let Some((b, used2)) = addr(r2) {
            return (Some((a.min(b), a.max(b))), &r2[used2..]);
        }
    }
    (Some((a, a)), rest)
}

/// Vim's pattern habits into the regex syntax: \( \) \| \+ \? \< \> and \v.
fn vim_regex(p: &str, icase: bool) -> Result<Regex, String> {
    let mut out = String::new();
    let mut it = p.chars().peekable();
    while let Some(c) = it.next() {
        if c == '\\' {
            match it.next() {
                Some('(') => out.push('('),
                Some(')') => out.push(')'),
                Some('|') => out.push('|'),
                Some('+') => out.push('+'),
                Some('?') => out.push('?'),
                Some('<') | Some('>') => out.push_str("\\b"),
                Some('v') => {}
                Some('c') => out.insert_str(0, "(?i)"),
                Some(o) => {
                    out.push('\\');
                    out.push(o);
                }
                None => out.push_str("\\\\"),
            }
        } else {
            out.push(c);
        }
    }
    if icase {
        out.insert_str(0, "(?i)");
    }
    Regex::new(&out).map_err(|e| format!("pattern: {e}"))
}

/// A replacement in Vim's terms: \1 & \r \n \\ into the regex crate's ($1, $0, newline).
fn vim_replacement(r: &str) -> String {
    let mut out = String::new();
    let mut it = r.chars();
    while let Some(c) = it.next() {
        match c {
            '\\' => match it.next() {
                Some(d @ '0'..='9') => out.push_str(&format!("${{{d}}}")),
                Some('r') | Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('&') => out.push('&'),
                Some(o) => out.push(o),
                None => out.push('\\'),
            },
            '&' => out.push_str("${0}"),
            '$' => out.push_str("$$"),
            c => out.push(c),
        }
    }
    out
}

/// `/pat/rep/flags` with any delimiter: (pattern, replacement, flags).
fn split_sub(s: &str) -> Option<(String, String, String)> {
    let mut chars = s.chars();
    let d = chars.next().filter(|c| !c.is_alphanumeric() && *c != ' ' && *c != '"')?;
    let mut parts = vec![String::new()];
    let mut esc = false;
    for c in chars {
        if esc {
            if c != d {
                parts.last_mut().unwrap().push('\\');
            }
            parts.last_mut().unwrap().push(c);
            esc = false;
        } else if c == '\\' {
            esc = true;
        } else if c == d && parts.len() < 3 {
            parts.push(String::new());
        } else {
            parts.last_mut().unwrap().push(c);
        }
    }
    while parts.len() < 3 {
        parts.push(String::new());
    }
    Some((parts[0].clone(), parts[1].clone(), parts[2].trim().to_string()))
}

impl App {
    /// The last line with text (a final newline does not make another).
    fn last_line(&self) -> usize {
        let b = &self.docs[self.cur].ed.buf;
        let n = b.len_lines();
        if n > 1 && b.line(n - 1).is_empty() { n - 2 } else { n.saturating_sub(1) }
    }

    fn lines_of(&self, a: usize, b: usize) -> Vec<String> {
        (a..=b).map(|l| self.docs[self.cur].ed.buf.line(l)).collect()
    }

    /// Lines a..=b become `new` (none: they go), as one undo step.
    fn set_lines(&mut self, a: usize, b: usize, new: Vec<String>) {
        let last = self.docs[self.cur].ed.buf.len_lines().saturating_sub(1);
        let buf = &self.docs[self.cur].ed.buf;
        let edit = if b < last {
            let mut t = new.join("\n");
            if !new.is_empty() {
                t.push('\n');
            }
            (Pos { line: a, col: 0 }, Pos { line: b + 1, col: 0 }, t)
        } else if a > 0 {
            let t = if new.is_empty() { String::new() } else { format!("\n{}", new.join("\n")) };
            (Pos { line: a - 1, col: buf.line_len(a - 1) }, Pos { line: b, col: buf.line_len(b) }, t)
        } else {
            (Pos { line: 0, col: 0 }, Pos { line: b, col: buf.line_len(b) }, new.join("\n"))
        };
        let d = &mut self.docs[self.cur];
        d.ed.apply_edits(vec![edit]);
        d.states.clear();
        d.touched(0);
        self.edit_at = Instant::now();
    }

    fn run_shell(cmd: &str, input: Option<&str>, dir: &Path) -> Result<String, String> {
        use std::io::Write;
        let mut c = std::process::Command::new("sh");
        c.args(["-c", cmd]).current_dir(dir).stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped());
        c.stdin(if input.is_some() { std::process::Stdio::piped() } else { std::process::Stdio::null() });
        let mut child = c.spawn().map_err(|e| e.to_string())?;
        if let (Some(t), Some(mut si)) = (input, child.stdin.take()) {
            let t = t.to_string();
            std::thread::spawn(move || {
                let _ = si.write_all(t.as_bytes());
            });
        }
        let o = child.wait_with_output().map_err(|e| e.to_string())?;
        let mut s = String::from_utf8_lossy(&o.stdout).to_string();
        if !o.status.success() {
            let err = String::from_utf8_lossy(&o.stderr).to_string();
            if s.is_empty() {
                return Err(format!("{} (exit {})", err.trim(), o.status.code().unwrap_or(-1)));
            }
            s.push_str(&err);
        }
        Ok(s)
    }

    /// `%` in a command: the current file.
    fn expand_percent(&self, s: &str) -> String {
        let name = self.docs[self.cur].abs_path().map(|p| p.display().to_string()).unwrap_or_default();
        s.replace(" %", &format!(" {name}")).replace("%:p", &name)
    }

    /// A `:` line Neovim would understand; false: not one of these (the caller tries its own).
    pub(crate) fn ex(&mut self, line: &str) -> bool {
        let cur = self.docs[self.cur].ed.cursor().line;
        let last = self.last_line();
        let (range, rest) = parse_range(line, cur, last, self.vis_range);
        let rest = rest.trim_start();
        // a named command (sort-lines, fill-paragraph): not Vim's :sort with "-lines"; it runs by name
        if range.is_none() && rest.split_whitespace().next().is_some_and(|w| w.contains('-') && w.chars().next().is_some_and(|c| c.is_ascii_alphabetic())) {
            return false;
        }
        // a range alone: go to its last line (:42, :$, :.+3)
        if range.is_some() && rest.is_empty() {
            let (_, b) = range.unwrap();
            self.goto_line(b + 1);
            return true;
        }
        // the command's name: letters, or one of the sign commands
        let name_len = if rest.starts_with(['!', '&', '<', '>', '=']) { 1 } else { rest.chars().take_while(|c| c.is_ascii_alphabetic()).count() };
        let (name, mut arg) = rest.split_at(name_len);
        let bang = arg.starts_with('!') && !name.is_empty() && name != "!";
        if bang {
            arg = &arg[1..];
        }
        let arg = arg.trim_start();
        let here = (cur, cur);
        let all = (0, last);
        match name {
            "s" | "su" | "substitute" | "&" => {
                let (a, b) = range.unwrap_or(here);
                let spec = if name == "&" { self.last_sub.clone() } else { split_sub(arg) };
                let Some((pat, rep, flags)) = spec.or_else(|| self.last_sub.clone()) else {
                    self.say("usage: :s/pattern/replacement/[g][i]");
                    return true;
                };
                self.last_sub = Some((pat.clone(), rep.clone(), flags.clone()));
                self.substitute(a, b, &pat, &rep, &flags);
            }
            "g" | "global" | "v" | "vglobal" => {
                let (a, b) = range.unwrap_or(all);
                let invert = bang || name.starts_with('v');
                let Some((pat, then, _)) = split_sub(arg).map(|(p, r, f)| (p, if f.is_empty() { r } else { format!("{r}/{f}") }, ())) else {
                    self.say("usage: :g/pattern/[d | p | s/a/b/ | normal keys]");
                    return true;
                };
                self.global(a, b, &pat, then.trim(), invert);
            }
            "d" | "de" | "delete" => {
                let (a, b) = range.unwrap_or(here);
                let b = arg.parse::<usize>().map(|n| (a + n - 1).min(last)).unwrap_or(b);
                let gone = self.lines_of(a, b).join("\n");
                clipboard::copy(&gone);
                self.set_lines(a, b, vec![]);
                self.say(format!("{} fewer lines", b - a + 1));
            }
            "y" | "ya" | "yank" => {
                let (a, b) = range.unwrap_or(here);
                let text = self.lines_of(a, b).join("\n");
                clipboard::copy(&text);
                self.say(format!("{} lines copied", b - a + 1));
            }
            "m" | "mo" | "move" | "t" | "co" | "copy" => {
                let (a, b) = range.unwrap_or(here);
                let (to, _) = parse_range(arg, cur, last, self.vis_range);
                let to = if arg.trim() == "0" { None } else { to.map(|t| t.1) };
                let block = self.lines_of(a, b);
                let copy = !name.starts_with('m');
                if !copy && to.is_some_and(|t| t >= a && t < b) {
                    self.say("cannot move lines into themselves");
                    return true;
                }
                // the span that changes: from the first touched line to the last
                let after = to.map_or(0, |t| t + 1);
                let (lo, hi) = if copy { (after.min(last), after.min(last)) } else { (a.min(after), b.max(after.saturating_sub(1))) };
                let mut span = self.lines_of(lo, hi.max(lo));
                if copy {
                    if after > last {
                        let mut s = span.clone();
                        s.extend(block);
                        self.set_lines(lo, lo, s);
                    } else {
                        let mut s = block;
                        s.extend(span);
                        self.set_lines(lo, lo, s);
                    }
                } else {
                    // lines lo..=hi, the block taken out and put after `after`
                    let rel_a = a - lo;
                    let n = b - a + 1;
                    let taken: Vec<String> = span.drain(rel_a..rel_a + n).collect();
                    let at = if after > b { after - lo - n } else { after - lo };
                    for (i, l) in taken.into_iter().enumerate() {
                        span.insert(at + i, l);
                    }
                    self.set_lines(lo, hi.max(lo), span);
                }
            }
            "j" | "join" => {
                let (a, b) = range.unwrap_or((cur, (cur + 1).min(last)));
                let b = if a == b { (b + 1).min(last) } else { b };
                let joined = self.lines_of(a, b).iter().enumerate().map(|(i, l)| if i == 0 { l.trim_end().to_string() } else { l.trim().to_string() }).filter(|l| !l.is_empty()).collect::<Vec<_>>().join(if bang { "" } else { " " });
                self.set_lines(a, b, vec![joined]);
            }
            ">" | "<" => {
                let (a, b) = range.unwrap_or(here);
                let steps = 1 + arg.chars().filter(|c| c.to_string() == name).count();
                let new = self
                    .lines_of(a, b)
                    .into_iter()
                    .map(|l| {
                        if l.is_empty() {
                            return l;
                        }
                        if name == ">" {
                            format!("{}{l}", " ".repeat(tab() * steps))
                        } else {
                            let mut s = l.as_str();
                            for _ in 0..tab() * steps {
                                s = s.strip_prefix(' ').or_else(|| s.strip_prefix('\t')).unwrap_or(s);
                            }
                            s.to_string()
                        }
                    })
                    .collect();
                self.set_lines(a, b, new);
            }
            "norm" | "normal" => {
                let (a, b) = range.unwrap_or(here);
                for l in a..=b {
                    let d = &mut self.docs[self.cur];
                    d.ed.set_cursor(Pos { line: l, col: 0 });
                    d.ed.feed(arg);
                    d.ed.feed("<Esc>");
                }
                let d = &mut self.docs[self.cur];
                d.states.clear();
                d.touched(0);
            }
            "sor" | "sort" => {
                let (a, b) = range.unwrap_or(all);
                let mut v = self.lines_of(a, b);
                let (numeric, icase, unique) = (arg.contains('n'), arg.contains('i'), arg.contains('u'));
                if numeric {
                    let num = |s: &str| s.split(|c: char| !c.is_ascii_digit() && c != '-').find(|p| !p.is_empty() && p != &"-").and_then(|p| p.parse::<i64>().ok()).unwrap_or(i64::MIN);
                    v.sort_by_key(|l| num(l));
                } else if icase {
                    v.sort_by_key(|l| l.to_lowercase());
                } else {
                    v.sort();
                }
                if unique {
                    v.dedup_by(|x, y| if icase { x.to_lowercase() == y.to_lowercase() } else { x == y });
                }
                if bang {
                    v.reverse();
                }
                let n = v.len();
                self.set_lines(a, b, v);
                self.say(format!("{n} lines sorted"));
            }
            "ret" | "retab" => {
                let (a, b) = range.unwrap_or(all);
                let w = arg.parse::<usize>().unwrap_or(tab());
                let v = self.lines_of(a, b).into_iter().map(|l| l.replace('\t', &" ".repeat(w))).collect();
                self.set_lines(a, b, v);
            }
            "!" => {
                let cmd = self.expand_percent(&format!(" {arg}")).trim_start().to_string();
                let dir = self.root.clone();
                match range {
                    // :{range}!cmd: the lines through cmd (:%!jq . , :'<,'>!sort)
                    Some((a, b)) => {
                        let input = self.lines_of(a, b).join("\n") + "\n";
                        match Self::run_shell(&cmd, Some(&input), &dir) {
                            Ok(out) => {
                                let v: Vec<String> = out.trim_end_matches('\n').lines().map(String::from).collect();
                                self.set_lines(a, b, v);
                            }
                            Err(e) => self.say(e),
                        }
                    }
                    None => match Self::run_shell(&cmd, None, &dir) {
                        Ok(out) => {
                            let rows: Vec<(String, String)> = out.lines().map(|l| (String::new(), l.to_string())).collect();
                            if rows.is_empty() {
                                self.say(format!("{cmd}: done"));
                            } else {
                                self.popup = Some(Popup::Help(format!("!{cmd}"), rows));
                            }
                        }
                        Err(e) => self.say(e),
                    },
                }
            }
            "r" | "read" => {
                let at = range.map_or(cur, |r| r.1);
                let dir = self.root.clone();
                let text = if let Some(c) = arg.strip_prefix('!') {
                    Self::run_shell(c, None, &dir)
                } else {
                    let p = if Path::new(arg).is_absolute() { PathBuf::from(arg) } else { self.root.join(arg) };
                    std::fs::read_to_string(&p).map_err(|e| format!("{}: {e}", p.display()))
                };
                match text {
                    Ok(t) => {
                        let mut v = vec![self.docs[self.cur].ed.buf.line(at)];
                        v.extend(t.trim_end_matches('\n').lines().map(String::from));
                        self.set_lines(at, at, v);
                    }
                    Err(e) => self.say(e),
                }
            }
            "earlier" | "later" => {
                let d = &mut self.docs[self.cur];
                let states = d.ed.undo_states();
                let at = d.ed.undo_current();
                let back = name == "earlier";
                let target = if let Some(t) = arg.strip_suffix(['s', 'm', 'h', 'd']) {
                    // by time: the state from that long ago
                    let unit = match arg.chars().last() {
                        Some('m') => 60,
                        Some('h') => 3600,
                        Some('d') => 86400,
                        _ => 1,
                    };
                    let span = t.parse::<u64>().unwrap_or(1) * unit;
                    let when = if back { states[at].1.saturating_sub(span) } else { states[at].1 + span };
                    let mut best = 0;
                    for (i, s) in states.iter().enumerate() {
                        if s.1 <= when {
                            best = i;
                        }
                    }
                    best
                } else {
                    let n = arg.parse::<usize>().unwrap_or(1);
                    if back { at.saturating_sub(n) } else { (at + n).min(states.len() - 1) }
                };
                d.ed.undo_goto(target);
                d.states.clear();
                d.touched(0);
                self.say(format!("state {target} of {}", states.len() - 1));
            }
            "u" | "undo" if !arg.is_empty() => {
                let n = arg.parse::<usize>().unwrap_or(0);
                let d = &mut self.docs[self.cur];
                d.ed.undo_goto(n);
                d.states.clear();
                d.touched(0);
            }
            "u" | "undo" => self.edit_key(Key::ch('u')),
            "red" | "redo" => self.edit_key(Key { ctrl: true, ..Key::ch('r') }),
            "undol" | "undolist" => {
                let d = &mut self.docs[self.cur];
                let states = d.ed.undo_states();
                let at = d.ed.undo_current();
                let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
                let rows = states.iter().enumerate().rev().map(|(i, (p, t))| (format!("{}{i}", if i == at { "▶ " } else { "  " }), format!("from {p} · {}s ago", now.saturating_sub(*t)))).collect();
                self.popup = Some(Popup::Help("Undo states (:undo N goes there, :earlier 5m)".into(), rows));
            }
            "mes" | "messages" => {
                let rows = self.history.iter().map(|m| (String::new(), m.clone())).collect();
                self.popup = Some(Popup::Help("Messages".into(), rows));
            }
            "e" | "edit" if bang => {
                let Some(p) = self.docs[self.cur].abs_path() else {
                    self.say("no file to reload");
                    return true;
                };
                // the file as it is on disk, changes dropped (a fresh buffer)
                match Doc::open(&p, &self.ss) {
                    Ok(mut nd) => {
                        let c = self.docs[self.cur].ed.cursor();
                        nd.name = self.docs[self.cur].name.clone();
                        nd.ed.set_cursor(c);
                        let i = self.cur;
                        self.docs[i] = nd;
                        self.lsp_open(i);
                        self.say(format!("\"{}\" reloaded", p.display()));
                    }
                    Err(e) => self.say(e.to_string()),
                }
            }
            "ene" | "enew" => {
                    self.doom_ui("newBuffer", "");
                }
            "new" | "vne" | "vnew" => {
                self.doom_ui("split", if name == "new" { "below" } else { "right" });
                self.doom_ui("newBuffer", "");
            }
            "sav" | "saveas" => {
                if arg.is_empty() {
                    self.say("usage: :saveas FILE");
                } else {
                    self.command(&format!("w {arg}"));
                }
            }
            "up" | "update" => {
                if self.docs[self.cur].dirty() {
                    self.command("w");
                }
            }
            "bf" | "bfirst" | "br" | "brewind" => self.show_doc(0),
            "bl" | "blast" => {
                let n = self.docs.len() - 1;
                self.show_doc(n);
            }
            "b" | "buffer" if arg.trim() == "#" => {
                if let Some(l) = self.last_doc.filter(|l| *l < self.docs.len()) {
                    self.show_doc(l);
                }
            }
            "b" | "buffer" => {
                let i = arg.parse::<usize>().ok().map(|n| n.saturating_sub(1)).or_else(|| self.docs.iter().position(|d| d.name.contains(arg)));
                match i.filter(|i| *i < self.docs.len()) {
                    Some(i) => self.show_doc(i),
                    None => self.say(format!("no buffer {arg} (:ls lists them)")),
                }
            }
            "bufdo" | "bufd" => {
                let start = self.cur;
                for i in 0..self.docs.len() {
                    self.show_doc(i);
                    for part in arg.split(" | ") {
                        self.command(part.trim());
                    }
                }
                self.show_doc(start);
            }
            "pw" | "pwd" => self.say(self.root.display().to_string()),
            "f" | "file" => {
                let d = &self.docs[self.cur];
                let c = d.ed.cursor();
                let n = d.ed.buf.len_lines();
                self.say(format!("\"{}\"{} {n} lines --{}%--", d.name, if d.dirty() { " [Modified]" } else { "" }, (c.line + 1) * 100 / n.max(1)));
            }
            "=" => {
                let n = range.map_or(last + 1, |r| r.1 + 1);
                self.say(n.to_string());
            }
            "noh" | "nohlsearch" => {}
            "ec" | "echo" => match self.with_lua(|e| e.eval(arg)) {
                Ok((out, fx)) => {
                    self.after_lua(fx);
                    self.say(out);
                }
                Err(e) => self.say(e),
            },
            "so" | "source" | "luafile" => {
                let p = if arg.is_empty() || arg == "%" { self.docs[self.cur].abs_path() } else { Some(PathBuf::from(arg)) };
                let Some(p) = p else { return true };
                match std::fs::read_to_string(&p) {
                    Ok(code) => match self.with_lua(|e| e.eval(&code)) {
                        Ok((_, fx)) => {
                            self.after_lua(fx);
                            self.say(format!("sourced {}", p.display()));
                        }
                        Err(e) => self.say(e),
                    },
                    Err(e) => self.say(format!("{}: {e}", p.display())),
                }
            }
            "map" | "nmap" | "nm" | "noremap" | "nnoremap" | "nn" => {
                let Some((lhs, rhs)) = arg.split_once(' ') else {
                    self.popup = None;
                    self.help("keys");
                    return true;
                };
                let rhs = rhs.trim();
                // :nmap <leader>x :sort<CR>  (an ex line) or :nmap <leader>x command-name
                let code = match rhs.strip_prefix(':') {
                    Some(ex) => format!("zett.map('n', {lhs:?}, function() zett.ui('ex', {:?}) end, {{ desc = {:?} }})", ex.trim_end_matches("<CR>").trim_end_matches("<cr>"), rhs),
                    None => format!("zett.map('n', {lhs:?}, {rhs:?}, {{ desc = {rhs:?} }})"),
                };
                if let Err(e) = self.with_lua(|e| e.eval(&code)) {
                    self.say(e);
                } else {
                    self.say(format!("{lhs} → {rhs} (for good: zett.map in init.lua)"));
                }
            }
            "unm" | "unmap" | "nun" | "nunmap" => {
                if let Err(e) = self.with_lua(|e| e.eval(&format!("zett.unmap('n', {arg:?})"))) {
                    self.say(e);
                }
            }
            "com" | "command" => {
                let Some((cname, body)) = arg.split_once(' ') else {
                    self.minibuf = None;
                    self.help("commands");
                    return true;
                };
                let code = format!("zett.command({cname:?}, function() zett.ui('ex', {:?}) end, {{ desc = {:?} }})", body.trim(), body.trim());
                match self.with_lua(|e| e.eval(&code)) {
                    Ok(_) => self.say(format!(":{cname} runs :{}", body.trim())),
                    Err(e) => self.say(e),
                }
            }
            "winc" | "wincmd" => match arg {
                "h" => {
                    self.doom_ui("focusWin", "left");
                }
                "j" => {
                    self.doom_ui("focusWin", "down");
                }
                "k" => {
                    self.doom_ui("focusWin", "up");
                }
                "l" => {
                    self.doom_ui("focusWin", "right");
                }
                "w" => {
                    self.doom_ui("focusWin", "next");
                }
                "v" => {
                    self.doom_ui("split", "right");
                }
                "s" => {
                    self.doom_ui("split", "below");
                }
                "c" | "q" => {
                    self.doom_ui("closeWin", "");
                }
                "o" => {
                    self.doom_ui("onlyWin", "");
                }
                _ => self.say("wincmd h j k l w v s c o"),
            },
            "ter" | "terminal" | "term" => {
                    self.doom_ui("terminal", "");
                }
            "mak" | "make" => {
                    self.doom_ui("compile", "");
                }
            "gr" | "grep" | "vim" | "vimgrep" | "Rg" => {
                self.doom_ui("grep", "");
                if let Some(m) = self.minibuf.as_mut() {
                    m.set_query(arg.trim_matches('/'));
                }
                self.grep_update();
            }
            "Ex" | "Explore" | "Lex" | "Lexplore" | "Neotree" | "NvimTreeToggle" => self.run_named("explorer"),
            "UndotreeToggle" => self.run_named("undotree"),
            "LspInfo" => {
                let rows = self.lsp.clients.values().map(|c| (c.name.clone(), "running".to_string())).collect::<Vec<_>>();
                self.popup = Some(Popup::Help("Language servers".into(), if rows.is_empty() { vec![(String::new(), "none running for these files".into())] } else { rows }));
            }
            "checkhealth" | "che" => self.checkhealth(),
            "ve" | "version" => self.say(format!("zett {} · Lua 5.4 · {} plugins", env!("CARGO_PKG_VERSION"), self.engine.plugins.len())),
            _ => return false,
        }
        true
    }

    fn run_named(&mut self, cmd: &str) {
        match self.with_lua(|e| e.run(cmd)) {
            Ok(fx) => self.after_lua(fx),
            Err(e) => self.say(e),
        }
    }

    fn substitute(&mut self, a: usize, b: usize, pat: &str, rep: &str, flags: &str) {
        let re = match vim_regex(pat, flags.contains('i')) {
            Ok(r) => r,
            Err(e) => return self.say(e),
        };
        let rep = vim_replacement(rep);
        let (mut n, mut lines) = (0, 0);
        let new: Vec<String> = self
            .lines_of(a, b)
            .into_iter()
            .map(|l| {
                let count = re.find_iter(&l).filter(|m| m.is_ok()).count();
                if count == 0 {
                    return l;
                }
                lines += 1;
                if flags.contains('g') {
                    n += count;
                    re.replace_all(&l, rep.as_str()).to_string()
                } else {
                    n += 1;
                    re.replace(&l, rep.as_str()).to_string()
                }
            })
            .collect();
        if n == 0 {
            return self.say(format!("pattern not found: {pat}"));
        }
        // a replacement with newlines makes more lines
        let new: Vec<String> = new.iter().flat_map(|l| l.split('\n').map(String::from).collect::<Vec<_>>()).collect();
        self.set_lines(a, b, new);
        self.say(format!("{n} substitution{} on {lines} line{}", if n == 1 { "" } else { "s" }, if lines == 1 { "" } else { "s" }));
    }

    /// `:g/pat/cmd`: d (delete), p (list), s/…/…/, normal keys; `:v` the lines that do not match.
    fn global(&mut self, a: usize, b: usize, pat: &str, then: &str, invert: bool) {
        let re = match vim_regex(pat, false) {
            Ok(r) => r,
            Err(e) => return self.say(e),
        };
        let hits: Vec<usize> = (a..=b).filter(|&l| re.is_match(&self.docs[self.cur].ed.buf.line(l)).unwrap_or(false) != invert).collect();
        if hits.is_empty() {
            return self.say(format!("pattern not found: {pat}"));
        }
        match then.split_once(' ').map(|(c, r)| (c, r)).unwrap_or((then, "")) {
            ("d" | "delete", _) => {
                let keep: Vec<String> = (a..=b).filter(|l| !hits.contains(l)).map(|l| self.docs[self.cur].ed.buf.line(l)).collect();
                self.set_lines(a, b, keep);
                self.say(format!("{} fewer lines", hits.len()));
            }
            ("norm" | "normal", keys) => {
                for &l in hits.iter().rev() {
                    let d = &mut self.docs[self.cur];
                    d.ed.set_cursor(Pos { line: l, col: 0 });
                    d.ed.feed(keys);
                    d.ed.feed("<Esc>");
                }
                let d = &mut self.docs[self.cur];
                d.states.clear();
                d.touched(0);
            }
            (s, _) if s.starts_with('s') && then.len() > 1 => {
                if let Some((p, r, f)) = split_sub(&then[1..]) {
                    for &l in hits.iter().rev() {
                        self.substitute(l, l, &p, &r, &f);
                    }
                    self.say(format!("{} lines changed", hits.len()));
                }
            }
            _ => {
                let rows = hits.iter().map(|&l| (format!("{}", l + 1), self.docs[self.cur].ed.buf.line(l))).collect();
                self.popup = Some(Popup::Help(format!("g/{pat}/ · {} lines", hits.len()), rows));
            }
        }
    }

    /// `:checkhealth`: what zett runs with, and what is missing.
    fn checkhealth(&mut self) {
        let have = |p: &str| std::process::Command::new("sh").args(["-c", &format!("command -v {p}")]).output().is_ok_and(|o| o.status.success());
        let mut rows = vec![("zett".to_string(), format!("{} · Lua 5.4", env!("CARGO_PKG_VERSION")))];
        rows.push(("config".into(), format!("{} · {} plugins", zett_script::config_dir().join("init.lua").display(), self.engine.plugins.len())));
        rows.push(("plugins".into(), self.engine.plugins.join(", ")));
        for e in &self.engine.errors {
            rows.push(("✖ error".into(), e.clone()));
        }
        for (tool, why) in [("rg", "search the project"), ("git", "branch, explorer marks"), ("wl-copy", "the clipboard"), ("gio", "delete into the trash"), ("zeit", "terminals and compiling"), ("rust-analyzer", "Rust"), ("clangd", "C / C++"), ("typescript-language-server", "TypeScript"), ("pyright-langserver", "Python"), ("lua-language-server", "Lua")] {
            rows.push((format!("{} {tool}", if have(tool) { "✔" } else { "–" }), why.to_string()));
        }
        rows.push(("terminal".into(), std::env::var("TERM_PROGRAM").unwrap_or_else(|_| std::env::var("TERM").unwrap_or_default())));
        self.popup = Some(Popup::Help("checkhealth".into(), rows));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranges() {
        assert_eq!(parse_range("%s/a/b/", 4, 9, None).0, Some((0, 9)));
        assert_eq!(parse_range("3,5d", 0, 9, None), (Some((2, 4)), "d"));
        assert_eq!(parse_range(".,$sort", 4, 9, None).0, Some((4, 9)));
        assert_eq!(parse_range(".+2", 4, 9, None).0, Some((6, 6)));
        assert_eq!(parse_range("'<,'>s/x/y/", 0, 9, Some((1, 3))).0, Some((1, 3)));
        assert_eq!(parse_range("sort", 4, 9, None), (None, "sort"));
    }

    #[test]
    fn vim_patterns_and_replacements() {
        let re = vim_regex(r"\(foo\)\+", false).unwrap();
        assert!(re.is_match("foofoo").unwrap());
        assert_eq!(re.replace_all("a foo b", vim_replacement(r"[\1]").as_str()), "a [foo] b");
        assert_eq!(split_sub("#a/b#c#g"), Some(("a/b".into(), "c".into(), "g".into())));
        assert_eq!(vim_replacement("x&y"), "x${0}y");
    }
}
