//! The quickfix and location list commands: :make and :grep (in the background, through shell,
//! makeprg / grepprg and errorformat / grepformat), :vimgrep, :cc :cn :cp :cfirst :clast :cnfile
//! :cpfile :cbelow :cabove, :copen :cclose :cwindow :clist :chistory :colder :cnewer, :cexpr
//! :cfile :cbuffer and their get/add forms, :cdo :cfdo; and each one's :l… twin for the window's
//! location list.

use super::*;
use crate::quickfix::{line_of, parse, QfItem, QfList, QfStack};
use std::sync::mpsc;

/// A :make or :grep running: its output comes back on `rx`.
pub(crate) struct QfJob {
    rx: mpsc::Receiver<Result<String, String>>,
    loc: bool,
    add: bool,
    jump: bool,
    title: String,
    efm: String,
}

/// The commands by Vim's names and shortest abbreviations: (name, shortest, kind).
const NAMES: &[(&str, &str)] = &[
    ("cc", "cc"), ("cnext", "cn"), ("cNext", "cN"), ("cprevious", "cp"), ("cnfile", "cnf"), ("cNfile", "cNf"), ("cpfile", "cpf"),
    ("cfirst", "cfir"), ("crewind", "cr"), ("clast", "cla"), ("copen", "cope"), ("cclose", "ccl"), ("cwindow", "cw"), ("clist", "cl"),
    ("colder", "col"), ("cnewer", "cnew"), ("chistory", "chi"), ("cexpr", "cex"), ("cgetexpr", "cgete"), ("caddexpr", "cadde"),
    ("cfile", "cf"), ("cgetfile", "cg"), ("caddfile", "caddf"), ("cbuffer", "cb"), ("cgetbuffer", "cgetb"), ("caddbuffer", "cad"),
    ("cdo", "cdo"), ("cfdo", "cfdo"), ("cbelow", "cbel"), ("cabove", "cabo"), ("cbefore", "cbe"), ("cafter", "caf"), ("cbottom", "cbo"),
    ("make", "mak"), ("grep", "gr"), ("grepadd", "grepa"), ("vimgrep", "vim"), ("vimgrepadd", "vimgrepa"),
    ("ll", "ll"), ("lnext", "lne"), ("lNext", "lN"), ("lprevious", "lp"), ("lnfile", "lnf"), ("lNfile", "lNf"), ("lpfile", "lpf"),
    ("lfirst", "lfir"), ("lrewind", "lr"), ("llast", "lla"), ("lopen", "lop"), ("lclose", "lcl"), ("lwindow", "lw"), ("llist", "lli"),
    ("lolder", "lol"), ("lnewer", "lnew"), ("lhistory", "lhi"), ("lexpr", "lex"), ("lgetexpr", "lgete"), ("laddexpr", "lad"),
    ("lfile", "lf"), ("lgetfile", "lg"), ("laddfile", "laddf"), ("lbuffer", "lb"), ("lgetbuffer", "lgetb"), ("laddbuffer", "laddb"),
    ("ldo", "ldo"), ("lfdo", "lfdo"), ("lbelow", "lbel"), ("labove", "lab"), ("lbefore", "lbe"), ("lafter", "laf"), ("lbottom", "lbo"),
    ("lmake", "lmak"), ("lgrep", "lgr"), ("lgrepadd", "lgrepa"), ("lvimgrep", "lv"), ("lvimgrepadd", "lvimgrepa"),
];

/// A typed name as one of these commands (the quickfix spelling, and true for a location list one).
pub(crate) fn qf_name(typed: &str) -> Option<(&'static str, bool)> {
    let (full, _) = NAMES.iter().find(|(full, short)| full.starts_with(typed) && typed.len() >= short.len())?;
    let loc = full.starts_with('l');
    // the location twin of cc is ll: map every l… name to its c… one
    let quick = if loc {
        match *full {
            "ll" => "cc",
            "lmake" => "make",
            "lgrep" => "grep",
            "lgrepadd" => "grepadd",
            "lvimgrep" => "vimgrep",
            "lvimgrepadd" => "vimgrepadd",
            f => NAMES.iter().map(|(n, _)| *n).find(|n| n.starts_with('c') && n[1..] == f[1..]).unwrap_or(f),
        }
    } else {
        full
    };
    Some((quick, loc))
}

impl App {
    fn qf_stack(&mut self, loc: bool) -> &mut QfStack {
        if loc { self.loclists.entry(self.focus).or_default() } else { &mut self.qf }
    }

    fn qf_title(loc: bool) -> &'static str {
        if loc { "[Location List]" } else { "[Quickfix List]" }
    }

    /// One of these commands; false when `name` is none of them.
    pub(crate) fn qf_command(&mut self, name: &str, arg: &str, bang: bool) -> bool {
        let Some((cmd, loc)) = qf_name(name) else { return false };
        let count = arg.trim().parse::<usize>().ok();
        let n = count.unwrap_or(1).max(1);
        match cmd {
            "make" => {
                let prg = self.setting(self.cur, "makeprg");
                let prg = if prg.is_empty() { "make".to_string() } else { prg };
                let line = if prg.contains("$*") { prg.replace("$*", arg) } else { format!("{prg} {arg}") };
                let efm = self.setting(self.cur, "errorformat");
                self.auto_write();
                self.qf_run(line.trim(), loc, false, !bang, ":make", efm);
            }
            "grep" | "grepadd" => {
                // :grep alone: zett's live search over the project
                if arg.trim().is_empty() {
                    self.doom_ui("grep", "");
                    return true;
                }
                let prg = self.setting(self.cur, "grepprg");
                let prg = if prg.is_empty() { "grep -HIn $* /dev/null".to_string() } else { prg };
                let line = if prg.contains("$*") { prg.replace("$*", arg) } else { format!("{prg} {arg}") };
                let efm = self.setting(self.cur, "grepformat");
                self.auto_write();
                self.qf_run(line.trim(), loc, cmd == "grepadd", !bang, &format!(":grep {arg}"), efm);
            }
            "vimgrep" | "vimgrepadd" => self.vimgrep(arg, loc, cmd == "vimgrepadd"),
            "cc" => {
                let i = match count {
                    Some(k) => k.saturating_sub(1),
                    None => self.qf_stack(loc).current().map_or(0, |l| l.idx),
                };
                self.qf_go(loc, i);
            }
            "cnext" | "cNext" | "cprevious" => self.qf_step(loc, if cmd == "cnext" { n as isize } else { -(n as isize) }),
            "cfirst" | "crewind" => self.qf_go_valid(loc, count.map(|k| k.saturating_sub(1)), true),
            "clast" => self.qf_go_valid(loc, count.map(|k| k.saturating_sub(1)), false),
            "cnfile" | "cNfile" | "cpfile" => self.qf_file_step(loc, cmd == "cnfile"),
            "cbelow" | "cabove" | "cbefore" | "cafter" => self.qf_near(loc, matches!(cmd, "cbelow" | "cafter"), n),
            "copen" => self.qf_open(loc),
            "cclose" => self.qf_close(loc),
            "cwindow" => {
                if self.qf_stack(loc).current().is_some_and(|l| !l.valid().is_empty()) {
                    self.qf_open(loc);
                } else {
                    self.qf_close(loc);
                }
            }
            "cbottom" => {
                if let Some(i) = self.docs.iter().position(|d| d.name == Self::qf_title(loc)) {
                    let last = self.docs[i].ed.buf.len_lines().saturating_sub(1);
                    self.docs[i].ed.set_cursor(Pos { line: last, col: 0 });
                }
            }
            "clist" => {
                let Some(l) = self.qf_stack(loc).current().cloned() else { return self.say_no_list(loc) };
                let rows = l.items.iter().enumerate().filter(|(_, it)| bang || it.valid).map(|(i, it)| (format!("{}{:>3}", if i == l.idx { ">" } else { " " }, i + 1), line_of(it))).collect();
                self.popup = Some(Popup::Help(l.title.clone(), rows));
            }
            "chistory" => {
                let s = self.qf_stack(loc).clone();
                let rows = s.lists.iter().enumerate().map(|(i, l)| (format!("{}{} of {}", if i == s.cur { ">" } else { " " }, i + 1, s.lists.len()), format!("{} errors  {}", l.valid().len(), l.title))).collect();
                self.popup = Some(Popup::Help(if loc { "location lists" } else { "error lists" }.into(), rows));
            }
            "colder" | "cnewer" => {
                let s = self.qf_stack(loc);
                let to = if cmd == "colder" { s.cur.checked_sub(n) } else { Some(s.cur + n).filter(|&t| t < s.lists.len()) };
                match to {
                    Some(t) => {
                        s.cur = t;
                        let (len, l) = (s.lists.len(), s.lists[t].clone());
                        self.say(format!("error list {} of {len}; {} errors  {}", t + 1, l.valid().len(), l.title));
                        self.qf_refresh(loc);
                    }
                    None => self.say(if cmd == "colder" { "E380: At bottom of quickfix stack" } else { "E381: At top of quickfix stack" }),
                }
            }
            "cexpr" | "cgetexpr" | "caddexpr" => {
                let text = match self.with_lua(|e| e.eval(&format!("local v = ({arg}); if type(v) == \"table\" then return table.concat(v, \"\\n\") end; return v"))) {
                    Ok((out, _)) => out,
                    Err(e) => return { self.say(e); true },
                };
                let efm = self.setting(self.cur, "errorformat");
                self.qf_set(loc, parse(&efm, &text), &format!(":{cmd} {arg}"), cmd == "caddexpr", cmd == "cexpr");
            }
            "cfile" | "cgetfile" | "caddfile" => {
                let f = if arg.trim().is_empty() { self.setting(self.cur, "errorfile") } else { arg.trim().to_string() };
                match std::fs::read_to_string(crate::files::expand_env(&f)) {
                    Ok(text) => {
                        let efm = self.setting(self.cur, "errorformat");
                        self.qf_set(loc, parse(&efm, &text), &format!(":{cmd} {f}"), cmd == "caddfile", cmd == "cfile");
                    }
                    Err(e) => self.say(format!("E40: Can't open errorfile {f}: {e}")),
                }
            }
            "cbuffer" | "cgetbuffer" | "caddbuffer" => {
                let i = count.map(|k| k.saturating_sub(1)).unwrap_or(self.cur).min(self.docs.len() - 1);
                let text = self.docs[i].ed.buf.text();
                let efm = self.setting(self.cur, "errorformat");
                self.qf_set(loc, parse(&efm, &text), &format!(":{cmd} {}", self.docs[i].name), cmd == "caddbuffer", cmd == "cbuffer");
            }
            "cdo" | "cfdo" => {
                let Some(l) = self.qf_stack(loc).current().cloned() else { return self.say_no_list(loc) };
                let mut seen = std::collections::HashSet::new();
                let picks: Vec<usize> = l.valid().into_iter().filter(|&i| cmd == "cdo" || seen.insert(l.items[i].file.clone())).collect();
                for i in picks {
                    self.qf_go(loc, i);
                    self.command(arg);
                }
            }
            _ => return false,
        }
        true
    }

    fn say_no_list(&mut self, loc: bool) -> bool {
        self.say(if loc { "E776: No location list" } else { "E42: No Errors" });
        true
    }

    /// Runs a program for :make or :grep in the background, through shell and shellcmdflag.
    fn qf_run(&mut self, line: &str, loc: bool, add: bool, jump: bool, title: &str, efm: String) {
        let line = self.expand_percent(&format!(" {line}")).trim_start().to_string();
        let (sh, flag) = (self.setting(self.cur, "shell"), self.setting(self.cur, "shellcmdflag"));
        let sh = if sh.is_empty() { std::env::var("SHELL").unwrap_or_else(|_| "sh".into()) } else { sh };
        let flag = if flag.is_empty() { "-c".to_string() } else { flag };
        // shellxquote around the whole line, shellquote around the command (both empty on Unix)
        let (xq, q) = (self.setting(self.cur, "shellxquote"), self.setting(self.cur, "shellquote"));
        let full = format!("{xq}{q}{line}{q}{xq}");
        let enc = self.setting(self.cur, "makeencoding");
        let dir = self.root.clone();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let out = std::process::Command::new(&sh).args(flag.split_whitespace()).arg(&full).current_dir(&dir).stdin(std::process::Stdio::null()).output();
            let r = out.map_err(|e| format!("{sh}: {e}")).map(|o| {
                // makeencoding: the program's output as it writes it (latin1, else UTF-8)
                let decode = |b: &[u8]| if enc.eq_ignore_ascii_case("latin1") { b.iter().map(|&c| c as char).collect::<String>() } else { String::from_utf8_lossy(b).to_string() };
                format!("{}{}", decode(&o.stdout), decode(&o.stderr))
            });
            let _ = tx.send(r);
        });
        self.say(format!("{title}: running {line}"));
        self.qf_job = Some(QfJob { rx, loc, add, jump, title: title.to_string(), efm });
    }

    /// A :make or :grep that finished: its list, and the first error (unless with !).
    pub(crate) fn qf_poll(&mut self) {
        let Some(j) = &self.qf_job else { return };
        let r = match j.rx.try_recv() {
            Ok(r) => r,
            Err(mpsc::TryRecvError::Empty) => return,
            Err(_) => Err("the program stopped".into()),
        };
        let j = self.qf_job.take().unwrap();
        match r {
            Ok(out) => {
                let items = parse(&j.efm, &out);
                self.qf_set(j.loc, items, &j.title, j.add, j.jump);
            }
            Err(e) => self.say(e),
        }
    }

    /// A new list (or more on the one in use), the quickfix window shown again, and the first
    /// valid item when `jump`.
    pub(crate) fn qf_set(&mut self, loc: bool, items: Vec<QfItem>, title: &str, add: bool, jump: bool) {
        let max = self.num(self.cur, if loc { "lhistory" } else { "chistory" }).max(1);
        let s = self.qf_stack(loc);
        match s.current_mut().filter(|_| add) {
            Some(l) => l.items.extend(items),
            None => s.push(QfList { title: title.to_string(), items, idx: 0 }, max),
        }
        let l = s.current().cloned().unwrap_or_default();
        self.qf_refresh(loc);
        let valid = l.valid();
        if valid.is_empty() {
            return self.say(format!("{title}: no errors"));
        }
        if jump {
            self.qf_go(loc, valid[0]);
        } else {
            self.say(format!("{title}: {} errors", valid.len()));
        }
    }

    /// A new list from items made here (:helpgrep), and its first item.
    pub(crate) fn qf_from_items(&mut self, loc: bool, items: Vec<QfItem>, title: &str) {
        self.qf_set(loc, items, title, false, true);
    }

    /// To item `i` of the list in use: its file (switchbuf: useopen a window that shows it, split
    /// or vsplit one), line and column.
    pub(crate) fn qf_go(&mut self, loc: bool, i: usize) {
        let Some(l) = self.qf_stack(loc).current().cloned() else { self.say_no_list(loc); return };
        let Some(it) = l.items.get(i).cloned() else { return self.say("E553: No more items") };
        if let Some(c) = self.qf_stack(loc).current_mut() {
            c.idx = i;
        }
        if !it.valid {
            return self.say(format!("({} of {}): {}", i + 1, l.items.len(), it.text));
        }
        // from the list's own window: to the window used before
        if self.docs[self.cur].name == Self::qf_title(loc) {
            if let Some(w) = self.last_win.filter(|w| self.tree.ids().contains(w)) {
                self.focus_win(w);
            }
        }
        // a help page (:helpgrep): the topic, then the line
        if let Some(topic) = it.file.strip_prefix("help:") {
            self.open_help(topic);
            if it.line > 0 {
                self.goto_line(it.line);
            }
            return self.say(format!("({} of {}) {}", i + 1, l.items.len(), it.text));
        }
        if !it.file.is_empty() {
            let p = PathBuf::from(&it.file);
            let p = if p.is_absolute() { p } else { self.root.join(p) };
            let same = self.docs[self.cur].abs_path().is_some_and(|a| a == p || std::fs::canonicalize(&p).ok().as_ref() == Some(&a));
            if !same {
                let sb = self.setting(self.cur, "switchbuf");
                let shown = self.tree.ids().into_iter().find(|&w| self.wins.get(w).and_then(|w| w.as_ref()).is_some_and(|w| self.docs.get(w.doc).and_then(|d| d.abs_path()).is_some_and(|a| a == p)));
                if let Some(w) = shown.filter(|_| sb.split(',').any(|o| o == "useopen" || o == "usetab")) {
                    self.focus_win(w);
                } else {
                    if sb.split(',').any(|o| o == "split") {
                        self.doom_ui("split", "below");
                    } else if sb.split(',').any(|o| o == "vsplit") {
                        self.doom_ui("split", "right");
                    }
                    self.open(&p);
                }
            }
        }
        if it.line > 0 {
            self.goto_line(it.line);
            if it.col > 0 {
                let c = self.docs[self.cur].ed.cursor();
                self.docs[self.cur].ed.set_cursor(Pos { line: c.line, col: it.col - 1 });
            }
        }
        let kind = match it.kind { 'E' => " error:", 'W' => " warning:", _ => "" };
        self.say(format!("({} of {}){kind} {}", i + 1, l.items.len(), it.text.replace('\n', " ")));
    }

    /// :cn :cp: `by` valid items on (back when negative).
    fn qf_step(&mut self, loc: bool, by: isize) {
        let Some(l) = self.qf_stack(loc).current().cloned() else { self.say_no_list(loc); return };
        let v = l.valid();
        let at = v.iter().position(|&i| i >= l.idx).unwrap_or(0) as isize;
        let at = if v.get(at as usize) == Some(&l.idx) { at } else if by > 0 { at - 1 } else { at };
        let to = at + by;
        if to < 0 || to as usize >= v.len() {
            return self.say("E553: No more items");
        }
        self.qf_go(loc, v[to as usize]);
    }

    /// :cfirst :clast (or that number).
    fn qf_go_valid(&mut self, loc: bool, nr: Option<usize>, first: bool) {
        let Some(l) = self.qf_stack(loc).current().cloned() else { self.say_no_list(loc); return };
        let v = l.valid();
        let i = match nr {
            Some(k) => k,
            None if first => v.first().copied().unwrap_or(0),
            None => v.last().copied().unwrap_or(0),
        };
        self.qf_go(loc, i);
    }

    /// :cnfile :cpfile: the first item of the next (or the last of the previous) file.
    fn qf_file_step(&mut self, loc: bool, fwd: bool) {
        let Some(l) = self.qf_stack(loc).current().cloned() else { self.say_no_list(loc); return };
        let file = l.items.get(l.idx).map(|i| i.file.clone()).unwrap_or_default();
        let v = l.valid();
        let to = if fwd { v.iter().copied().find(|&i| i > l.idx && l.items[i].file != file) } else { v.iter().rev().copied().find(|&i| i < l.idx && l.items[i].file != file) };
        match to {
            Some(i) => self.qf_go(loc, i),
            None => self.say("E553: No more items"),
        }
    }

    /// :cbelow :cabove (:cafter :cbefore): the nth item in this file past the cursor.
    fn qf_near(&mut self, loc: bool, down: bool, n: usize) {
        let Some(l) = self.qf_stack(loc).current().cloned() else { self.say_no_list(loc); return };
        let here = self.docs[self.cur].abs_path();
        let c = self.docs[self.cur].ed.cursor();
        let root = self.root.clone();
        let mine: Vec<usize> = l.valid().into_iter().filter(|&i| here.as_ref().is_some_and(|h| *h == root.join(&l.items[i].file) || h.ends_with(&l.items[i].file))).collect();
        let key = |i: usize| (l.items[i].line, l.items[i].col);
        let at = (c.line + 1, c.col + 1);
        let list: Vec<usize> = if down { mine.into_iter().filter(|&i| key(i) > at).collect() } else { mine.into_iter().rev().filter(|&i| key(i) < at).collect() };
        match list.get(n - 1) {
            Some(&i) => self.qf_go(loc, i),
            None => self.say("E553: No more items"),
        }
    }

    /// The list as text, for its window.
    fn qf_text(&mut self, loc: bool) -> String {
        let l = self.qf_stack(loc).current().cloned().unwrap_or_default();
        let mut t = l.items.iter().map(line_of).collect::<Vec<_>>().join("\n");
        t.push('\n');
        t
    }

    /// :copen: the list in a window below (one already open is used), Enter on a line goes there.
    fn qf_open(&mut self, loc: bool) {
        let title = Self::qf_title(loc);
        let text = self.qf_text(loc);
        let shown = self.tree.ids().into_iter().find(|&w| self.wins.get(w).and_then(|w| w.as_ref()).is_some_and(|w| self.docs.get(w.doc).is_some_and(|d| d.name == title)));
        if let Some(w) = shown {
            self.focus_win(w);
            return self.qf_refresh(loc);
        }
        self.last_win = Some(self.focus);
        self.doom_ui("split", "below");
        let i = match self.docs.iter().position(|d| d.name == title) {
            Some(i) => i,
            None => {
                self.docs.push(Doc::new(Editor::from_str(&text), title.to_string(), "Plain Text".into()));
                self.docs.len() - 1
            }
        };
        self.show_doc(i);
        self.qf_refresh(loc);
    }

    /// :cclose: the window showing the list goes.
    fn qf_close(&mut self, loc: bool) {
        let title = Self::qf_title(loc);
        let keep = self.focus;
        if let Some(w) = self.tree.ids().into_iter().find(|&w| self.wins.get(w).and_then(|w| w.as_ref()).is_some_and(|w| self.docs.get(w.doc).is_some_and(|d| d.name == title))) {
            self.focus_win(w);
            self.close_win();
            if keep != w && self.tree.ids().contains(&keep) {
                self.focus_win(keep);
            }
        }
    }

    /// The list's buffer again (after :make, :colder …), its cursor on the item in use.
    fn qf_refresh(&mut self, loc: bool) {
        let title = Self::qf_title(loc);
        let Some(i) = self.docs.iter().position(|d| d.name == title) else { return };
        let text = self.qf_text(loc);
        let idx = self.qf_stack(loc).current().map_or(0, |l| l.idx);
        let d = &mut self.docs[i];
        d.ed = Editor::from_str(&text);
        d.states.clear();
        d.ed.set_cursor(Pos { line: idx, col: 0 });
    }

    /// Enter in the list's window: to that item.
    pub(crate) fn follow_qf(&mut self) -> bool {
        let name = self.docs[self.cur].name.clone();
        let loc = match name.as_str() {
            "[Quickfix List]" => false,
            "[Location List]" => true,
            _ => return false,
        };
        let line = self.docs[self.cur].ed.cursor().line;
        self.qf_go(loc, line);
        true
    }

    /// :vimgrep /pattern/[g][j] files: the files searched here (* ** ? in names), every match
    /// (g) or the first of each line; j: no jump.
    fn vimgrep(&mut self, arg: &str, loc: bool, add: bool) {
        let arg = arg.trim();
        let (pat, flags, files) = match arg.chars().next() {
            Some(d) if !d.is_alphanumeric() => {
                let rest = &arg[d.len_utf8()..];
                let Some(end) = rest.find(d) else { return self.say("E682: Invalid search pattern or delimiter") };
                let tail = &rest[end + d.len_utf8()..];
                let fl: String = tail.chars().take_while(|c| matches!(c, 'g' | 'j' | 'f')).collect();
                (rest[..end].to_string(), fl.clone(), tail[fl.len()..].trim().to_string())
            }
            _ => {
                let (p, f) = arg.split_once(' ').unwrap_or((arg, ""));
                (p.to_string(), String::new(), f.trim().to_string())
            }
        };
        let pat = if pat.is_empty() { self.docs[self.cur].ed.register('/').map(|r| r.0).unwrap_or_default() } else { pat };
        let fold = self.flag(self.cur, "ignorecase") && !(self.flag(self.cur, "smartcase") && pat.chars().any(char::is_uppercase));
        let re = match crate::ex::vim_regex(&pat, fold) {
            Ok(r) => r,
            Err(e) => return self.say(e),
        };
        let cwd = std::env::current_dir().unwrap_or_default();
        let ignore: Vec<String> = self.setting(self.cur, "wildignore").split(',').filter(|s| !s.is_empty()).map(str::to_string).collect();
        let mut paths = Vec::new();
        for f in files.split_whitespace() {
            let f = if f == "%" { self.docs[self.cur].abs_path().map(|p| p.display().to_string()).unwrap_or_default() } else { crate::files::expand_env(f) };
            glob_files(&cwd, &f, &ignore, &mut paths);
        }
        let mut items = Vec::new();
        for p in paths.iter().take(20_000) {
            let Ok(text) = std::fs::read_to_string(p) else { continue };
            let shown = p.strip_prefix(&cwd).unwrap_or(p).display().to_string();
            for (n, l) in text.lines().enumerate() {
                for m in re.find_iter(l).flatten() {
                    let col = l[..m.start()].chars().count() + 1;
                    items.push(QfItem { file: shown.clone(), line: n + 1, col, kind: ' ', nr: 0, text: l.trim().to_string(), valid: true });
                    if !flags.contains('g') {
                        break;
                    }
                }
            }
        }
        if items.is_empty() {
            return self.say(format!("E480: No match: {pat}"));
        }
        self.qf_set(loc, items, &format!(":vimgrep {arg}"), add, !flags.contains('j'));
    }
}

/// Files for a pattern: a plain name, or * ? in names and ** across folders (from the part
/// without them); .git and what wildignore names are left out.
fn glob_files(cwd: &Path, pat: &str, ignore: &[String], out: &mut Vec<PathBuf>) {
    let full = if Path::new(pat).is_absolute() { PathBuf::from(pat) } else { cwd.join(pat) };
    let s = full.to_string_lossy().to_string();
    if !s.contains(['*', '?']) {
        if full.is_file() {
            out.push(full);
        }
        return;
    }
    let base: String = s.split('/').take_while(|p| !p.contains(['*', '?'])).collect::<Vec<_>>().join("/");
    let mut stack = vec![PathBuf::from(if base.is_empty() { "/".to_string() } else { base })];
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else { continue };
        for e in rd.flatten() {
            let p = e.path();
            let name = e.file_name().to_string_lossy().to_string();
            if name == ".git" || ignore.iter().any(|g| crate::files::glob_match(g, &name)) {
                continue;
            }
            if e.file_type().is_ok_and(|t| t.is_dir()) {
                if s.contains("**") {
                    stack.push(p);
                }
            } else if crate::files::glob_match(&s, &p.to_string_lossy()) {
                out.push(p);
            }
            if out.len() > 50_000 {
                return;
            }
        }
    }
    out.sort();
}

#[cfg(test)]
mod tests {
    #[test]
    fn names_and_their_location_twins() {
        assert_eq!(super::qf_name("cn"), Some(("cnext", false)));
        assert_eq!(super::qf_name("lne"), Some(("cnext", true)));
        assert_eq!(super::qf_name("ll"), Some(("cc", true)));
        assert_eq!(super::qf_name("lop"), Some(("copen", true)));
        assert_eq!(super::qf_name("lv"), Some(("vimgrep", true)));
        assert_eq!(super::qf_name("mak"), Some(("make", false)));
        assert_eq!(super::qf_name("lmake"), Some(("make", true)));
        assert_eq!(super::qf_name("cd"), None, ":cd is not one of them");
        assert_eq!(super::qf_name("le"), None, ":le is :left");
        assert_eq!(super::qf_name("lc"), None, ":lc is :lcd");
    }
}
