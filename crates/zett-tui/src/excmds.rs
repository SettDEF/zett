//! More of Vim's Ex commands: the argument list (:args :next :argdo …), buffers in windows (:sb
//! :ball :badd …), windows (:close :resize :windo :vertical …), tab commands on zett's tabs (its
//! buffers), lines (:print :put :center :left :uniq), marks, jumps and registers, the modifiers
//! (:silent :keepjumps …), spelling and fold commands, :match, :luado, :redir, and small ones.

use super::*;

/// The commands by Vim's full names and shortest forms.
const NAMES: &[(&str, &str)] = &[
    ("args", "ar"), ("argadd", "arga"), ("argdelete", "argd"), ("argedit", "arge"), ("argdedupe", "argded"), ("argdo", "argdo"),
    ("argument", "argu"), ("argglobal", "argg"), ("arglocal", "argl"), ("next", "n"), ("Next", "N"), ("previous", "prev"),
    ("first", "fir"), ("rewind", "rew"), ("last", "la"), ("wnext", "wn"), ("wNext", "wN"), ("wprevious", "wp"),
    ("sargument", "sa"), ("snext", "sn"), ("sNext", "sN"), ("sprevious", "spr"), ("srewind", "sre"), ("sfirst", "sfir"),
    ("slast", "sla"), ("sall", "sal"), ("all", "al"),
    ("badd", "bad"), ("balt", "balt"), ("ball", "ba"), ("sball", "sba"), ("bNext", "bN"), ("bunload", "bun"), ("bwipeout", "bw"),
    ("sbuffer", "sb"), ("sbnext", "sbn"), ("sbNext", "sbN"), ("sbprevious", "sbp"), ("sbfirst", "sbf"), ("sbrewind", "sbr"),
    ("sblast", "sbl"), ("sbmodified", "sbm"),
    ("close", "clo"), ("hide", "hid"), ("unhide", "unh"), ("sunhide", "sun"), ("resize", "res"), ("vertical", "vert"),
    ("horizontal", "hor"), ("leftabove", "lefta"), ("aboveleft", "abo"), ("rightbelow", "rightb"), ("belowright", "bel"),
    ("topleft", "to"), ("windo", "wind"), ("winsize", "wi"), ("sview", "sv"), ("sfind", "sf"), ("drop", "dr"),
    ("pclose", "pc"), ("pedit", "ped"), ("pbuffer", "pb"),
    ("tabnew", "tabnew"), ("tabedit", "tabe"), ("tabfind", "tabf"), ("tabclose", "tabc"), ("tabonly", "tabo"), ("tabnext", "tabn"),
    ("tabNext", "tabN"), ("tabprevious", "tabp"), ("tabfirst", "tabfir"), ("tabrewind", "tabr"), ("tablast", "tabl"),
    ("tabmove", "tabm"), ("tabdo", "tabd"), ("tabs", "tabs"), ("tab", "tab"),
    ("print", "p"), ("number", "nu"), ("#", "#"), ("z", "z"), ("put", "pu"), ("center", "ce"), ("left", "le"), ("mark", "ma"),
    ("marks", "marks"), ("delmarks", "delm"), ("jumps", "ju"), ("clearjumps", "cle"), ("changes", "changes"), ("registers", "reg"),
    ("display", "di"), ("uniq", "uni"), ("~", "~"), ("@", "@"), ("*", "*"),
    ("startinsert", "star"), ("stopinsert", "stopi"), ("startreplace", "startr"), ("startgreplace", "startg"),
    ("redraw", "redr"), ("redrawstatus", "redraws"), ("redrawtabline", "redrawt"), ("sleep", "sl"), ("suspend", "sus"), ("stop", "st"),
    ("cquit", "cq"), ("quitall", "quita"), ("exit", "exi"), ("visual", "vi"), ("view", "vie"), ("ex", "ex"), ("intro", "int"),
    ("mode", "mod"), ("winpos", "winp"), ("ascii", "as"), ("exusage", "exu"), ("viusage", "viu"), ("options", "opt"),
    ("filetype", "filet"), ("setfiletype", "setf"), ("setglobal", "setg"), ("syntax", "sy"), ("history", "his"), ("oldfiles", "ol"),
    ("helpclose", "helpc"), ("helpgrep", "helpg"), ("lhelpgrep", "lh"), ("helptags", "helpt"), ("echon", "echon"),
    ("echomsg", "echom"), ("echoerr", "echoe"), ("eval", "ev"), ("execute", "exe"), ("fclose", "fc"),
    ("silent", "sil"), ("unsilent", "uns"), ("verbose", "verb"), ("keepalt", "keepa"), ("keepmarks", "kee"), ("keepjumps", "keepj"),
    ("keeppatterns", "keepp"), ("lockmarks", "loc"), ("noautocmd", "noa"), ("sandbox", "san"), ("filter", "filt"), ("browse", "bro"),
    ("confirm", "conf"), ("noswapfile", "nos"), ("match", "mat"), ("2match", "2mat"), ("3match", "3mat"), ("k", "k"), ("list", "l"), ("goto", "go"), ("iput", "ip"), ("dl", "dl"),
    ("smagic", "sm"), ("snomagic", "sno"), ("undojoin", "undoj"),
    ("spellgood", "spe"), ("spellwrong", "spellw"), ("spellrare", "spellra"), ("spellundo", "spellu"), ("spellinfo", "spelli"),
    ("spelldump", "spelld"), ("fold", "fo"), ("foldclose", "foldc"), ("foldopen", "foldo"), ("folddoopen", "foldd"),
    ("folddoclosed", "folddoc"), ("luado", "luad"), ("redir", "redi"),
    ("python3", "py3"), ("py3do", "py3d"), ("py3file", "py3f"), ("python", "py"), ("pydo", "pyd"), ("pyfile", "pyf"), ("pythonx", "pyx"),
    ("pyxdo", "pyxd"), ("pyxfile", "pyxf"), ("perl", "pe"), ("perldo", "perld"), ("perlfile", "perlf"), ("ruby", "rub"), ("rubydo", "rubyd"),
    ("rubyfile", "rubyf"),
];

/// A typed name as one of these commands (its full name).
pub(crate) fn ex_name(typed: &str) -> Option<&'static str> {
    NAMES.iter().find(|(full, short)| full.starts_with(typed) && typed.len() >= short.len()).map(|(f, _)| *f)
}

impl App {
    /// One of these commands with its range (first and last line, 0-based), argument and !.
    pub(crate) fn ex_more(&mut self, name: &str, arg: &str, bang: bool, range: Option<(usize, usize)>) {
        let Some(cmd) = ex_name(name) else { return };
        let cur_line = self.docs[self.cur].ed.cursor().line;
        let (a, b) = range.unwrap_or((cur_line, cur_line));
        let num = arg.trim().parse::<usize>().ok();
        let n = num.unwrap_or(1).max(1);
        match cmd {
            // ── the argument list ──
            "args" | "next" if !arg.trim().is_empty() => {
                self.args = arg.split_whitespace().map(str::to_string).collect();
                self.arg_idx = 0;
                self.arg_edit(0);
            }
            "args" => {
                let shown: Vec<String> = self.args.iter().enumerate().map(|(i, a)| if i == self.arg_idx { format!("[{a}]") } else { a.clone() }).collect();
                self.say(if shown.is_empty() { "(the argument list is empty)".into() } else { shown.join(" ") });
            }
            "argadd" | "argedit" => {
                let files: Vec<String> = if arg.trim().is_empty() || num.is_some() { self.docs[self.cur].abs_path().map(|p| p.display().to_string()).into_iter().collect() } else { arg.split_whitespace().map(str::to_string).collect() };
                let at = (self.arg_idx + 1).min(self.args.len());
                for (k, f) in files.iter().enumerate() {
                    self.args.insert(at + k, f.clone());
                }
                if cmd == "argedit" {
                    self.arg_edit(at);
                }
            }
            "argdelete" => {
                let pats: Vec<&str> = arg.split_whitespace().collect();
                let cur_file = self.docs[self.cur].abs_path().map(|p| p.display().to_string());
                self.args.retain(|f| if pats.is_empty() { Some(f) != cur_file.as_ref() } else { !pats.iter().any(|p| crate::files::glob_match(p, f)) });
                self.arg_idx = self.arg_idx.min(self.args.len().saturating_sub(1));
            }
            "argdedupe" => {
                let mut seen = std::collections::HashSet::new();
                self.args.retain(|f| seen.insert(f.clone()));
            }
            "argdo" => {
                for i in 0..self.args.len() {
                    self.arg_edit(i);
                    self.command(arg);
                }
            }
            "argument" => self.arg_edit(num.map_or(self.arg_idx, |k| k.saturating_sub(1))),
            "argglobal" | "arglocal" => {
                if !arg.trim().is_empty() {
                    self.args = arg.split_whitespace().map(str::to_string).collect();
                }
            }
            "next" | "wnext" => {
                if cmd == "wnext" {
                    self.command("w");
                }
                self.arg_edit(self.arg_idx + n);
            }
            "Next" | "previous" | "wNext" | "wprevious" => {
                if cmd.starts_with('w') {
                    self.command("w");
                }
                match self.arg_idx.checked_sub(n) {
                    Some(i) => self.arg_edit(i),
                    None => self.say("E164: Cannot go before first file"),
                }
            }
            "first" | "rewind" => self.arg_edit(0),
            "last" => self.arg_edit(self.args.len().saturating_sub(1)),
            "sargument" | "snext" | "sNext" | "sprevious" | "srewind" | "sfirst" | "slast" => {
                let to = match cmd {
                    "sargument" => num.map_or(self.arg_idx, |k| k.saturating_sub(1)),
                    "snext" => self.arg_idx + n,
                    "srewind" | "sfirst" => 0,
                    "slast" => self.args.len().saturating_sub(1),
                    _ => self.arg_idx.saturating_sub(n),
                };
                if to < self.args.len() {
                    self.doom_ui("split", "below");
                }
                self.arg_edit(to);
            }
            "sall" | "all" => {
                self.vim_window("o", None);
                for i in 0..self.args.len().min(num.unwrap_or(usize::MAX)) {
                    if i > 0 {
                        self.doom_ui("split", "below");
                    }
                    self.arg_edit(i);
                }
            }
            // ── buffers ──
            "badd" => {
                let p = self.root.join(arg.trim());
                if !self.docs.iter().any(|d| d.abs_path().as_deref() == Some(p.as_path())) {
                    if let Ok(d) = Doc::open(&p, &self.ss) {
                        self.docs.push(d);
                    }
                }
            }
            "balt" => {
                let keep = self.cur;
                self.command(&format!("e {}", arg.trim()));
                let alt = self.cur;
                self.show_doc(keep);
                self.last_doc = Some(alt);
            }
            "ball" | "sball" | "unhide" | "sunhide" => {
                self.vim_window("o", None);
                let max = num.unwrap_or(usize::MAX);
                let first = self.cur;
                for (k, i) in (0..self.docs.len()).filter(|&i| i != first).take(max.saturating_sub(1)).enumerate() {
                    let _ = k;
                    self.doom_ui("split", "below");
                    self.show_doc(i);
                }
            }
            "bNext" => {
                for _ in 0..n {
                    self.command("bp");
                }
            }
            "bunload" | "bwipeout" => self.command(if bang { "bd!" } else { "bd" }),
            "sbuffer" => {
                self.doom_ui("split", "below");
                if !arg.trim().is_empty() {
                    self.command(&format!("b {}", arg.trim()));
                }
            }
            "sbnext" | "sbNext" | "sbprevious" | "sbfirst" | "sbrewind" | "sblast" | "sbmodified" => {
                self.doom_ui("split", "below");
                let to = match cmd {
                    "sbnext" => (self.cur + n) % self.docs.len(),
                    "sbfirst" | "sbrewind" => 0,
                    "sblast" => self.docs.len() - 1,
                    "sbmodified" => (1..=self.docs.len()).map(|k| (self.cur + k) % self.docs.len()).find(|&i| self.docs[i].dirty()).unwrap_or(self.cur),
                    _ => (self.cur + self.docs.len() - n % self.docs.len()) % self.docs.len(),
                };
                self.show_doc(to);
            }
            // ── windows ──
            "close" | "hide" if arg.trim().is_empty() => {
                if !self.close_win() {
                    self.say("E444: Cannot close last window");
                }
            }
            "hide" => self.command(arg),
            "resize" => {
                let t = arg.trim();
                if let Some(k) = t.strip_prefix('+').and_then(|k| k.parse().ok()) {
                    self.vim_window("+", Some(k));
                } else if let Some(k) = t.strip_prefix('-').and_then(|k| k.parse().ok()) {
                    self.vim_window("-", Some(k));
                } else {
                    self.vim_window("_", num);
                }
            }
            "vertical" => {
                let (first, rest) = arg.trim().split_once(' ').unwrap_or((arg.trim(), ""));
                match first {
                    "sp" | "split" => self.command(&format!("vsplit {rest}")),
                    "new" => self.command(&format!("vnew {rest}")),
                    "res" | "resize" => {
                        let r = rest.trim();
                        if let Some(k) = r.strip_prefix('+').and_then(|k| k.parse().ok()) {
                            self.vim_window(">", Some(k));
                        } else if let Some(k) = r.strip_prefix('-').and_then(|k| k.parse().ok()) {
                            self.vim_window("<", Some(k));
                        } else {
                            self.vim_window("|", r.parse().ok());
                        }
                    }
                    "sb" | "sbuffer" => {
                        self.doom_ui("split", "right");
                        if !rest.trim().is_empty() {
                            self.command(&format!("b {rest}"));
                        }
                    }
                    _ => self.command(arg),
                }
            }
            "leftabove" | "aboveleft" | "rightbelow" | "belowright" | "topleft" => {
                // the split in the command goes before (above, left) or after (below, right)
                let after = matches!(cmd, "rightbelow" | "belowright");
                let i = self.cur;
                let keep: Vec<(String, Option<String>)> = ["splitbelow", "splitright"].iter().map(|k| (k.to_string(), self.docs[i].local.get(*k).cloned())).collect();
                for (k, _) in &keep {
                    self.docs[i].local.insert(k.clone(), after.to_string());
                }
                self.command(arg);
                if let Some(d) = self.docs.get_mut(i) {
                    for (k, v) in keep {
                        match v {
                            Some(v) => d.local.insert(k, v),
                            None => d.local.remove(&k),
                        };
                    }
                }
            }
            "horizontal" | "keepalt" | "keepmarks" | "keepjumps" | "keeppatterns" | "lockmarks" | "noautocmd" | "sandbox" | "browse" | "confirm" | "noswapfile" | "verbose" | "unsilent" | "filter" => {
                // modifiers: the command after them runs (filter's /pattern/ first word is dropped)
                let rest = if cmd == "filter" { arg.trim().splitn(2, ' ').nth(1).unwrap_or("") } else { arg };
                let rest = if cmd == "verbose" { rest.trim_start_matches(|c: char| c.is_ascii_digit()).trim_start() } else { rest };
                self.command(rest.trim());
            }
            "silent" => {
                // its messages are not shown (and with ! neither its errors)
                let keep = self.msg.take();
                self.command(arg.trim());
                self.msg = keep;
            }
            "windo" => {
                for w in self.tree.ids() {
                    self.focus_win(w);
                    self.command(arg);
                }
            }
            "winsize" => self.say("E319: :winsize is not supported in a terminal (use :resize)"),
            "sview" | "sfind" | "pedit" => {
                self.doom_ui("split", "below");
                self.command(&format!("e {}", arg.trim()));
                if cmd == "sview" {
                    let i = self.cur;
                    self.docs[i].local.insert("readonly".into(), "true".into());
                }
            }
            "pbuffer" => {
                self.doom_ui("split", "below");
                self.command(&format!("b {}", arg.trim()));
            }
            "pclose" => {}
            "drop" => {
                let p = self.root.join(arg.trim());
                let shown = self.tree.ids().into_iter().find(|&w| self.wins.get(w).and_then(|w| w.as_ref()).is_some_and(|w| self.docs.get(w.doc).and_then(|d| d.abs_path()).is_some_and(|a| a == p)));
                match shown {
                    Some(w) => self.focus_win(w),
                    None => self.command(&format!("e {}", arg.trim())),
                }
            }
            // ── tab commands: zett's tabs are its buffers ──
            "tabnew" | "tabedit" | "tabfind" | "tab" if cmd != "tab" => {
                if arg.trim().is_empty() {
                    self.doom_ui("newBuffer", "");
                } else {
                    self.command(&format!("e {}", arg.trim()));
                }
            }
            "tab" => self.command(arg),
            "tabclose" => self.command(if bang { "bd!" } else { "bd" }),
            "tabonly" => {
                // the other buffers go, the changed ones stay
                let keep = self.cur;
                let mut i = self.docs.len();
                while i > 0 {
                    i -= 1;
                    if i != keep && !self.docs[i].dirty() {
                        self.show_doc(i);
                        self.close(false);
                    }
                }
            }
            "tabnext" => match num {
                Some(k) => self.show_doc(k.saturating_sub(1)),
                None => self.command("bn"),
            },
            "tabNext" | "tabprevious" => {
                for _ in 0..n {
                    self.command("bp");
                }
            }
            "tabfirst" | "tabrewind" => self.show_doc(0),
            "tablast" => self.show_doc(self.docs.len() - 1),
            "tabmove" => {
                let t = arg.trim();
                let len = self.docs.len();
                let to = if t.is_empty() || t == "$" { len - 1 } else if let Some(k) = t.strip_prefix('+') { (self.cur + k.parse::<usize>().unwrap_or(1)).min(len - 1) } else if let Some(k) = t.strip_prefix('-') { self.cur.saturating_sub(k.parse::<usize>().unwrap_or(1)) } else { t.parse::<usize>().unwrap_or(0).min(len - 1) };
                self.move_doc_to(self.cur, to);
            }
            "tabdo" => {
                for i in 0..self.docs.len() {
                    self.show_doc(i);
                    self.command(arg);
                }
            }
            "tabs" => {
                let rows = self.docs.iter().enumerate().map(|(i, d)| (format!("{}{}", if i == self.cur { ">" } else { " " }, i + 1), format!("{}{}", d.name, if d.dirty() { " +" } else { "" }))).collect();
                self.popup = Some(Popup::Help("Tabs (zett's tabs are its buffers)".into(), rows));
            }
            // ── lines ──
            "print" | "number" | "#" | "z" => {
                let d = &self.docs[self.cur];
                let (s, e) = if cmd == "z" { (b, (b + self.area.height.max(2) as usize - 1).min(d.ed.buf.len_lines().saturating_sub(1))) } else { (a, b) };
                let rows = (s..=e).map(|l| (format!("{:>4}", l + 1), d.ed.buf.line(l).trim_end_matches('\n').to_string())).collect();
                self.popup = Some(Popup::Help(format!("lines {}-{}", s + 1, e + 1), rows));
            }
            "put" => {
                let r = arg.trim().chars().next().unwrap_or('"');
                let Some((text, _)) = self.docs[self.cur].ed.register(r) else { return self.say(format!("E353: Nothing in register {r}")) };
                // :put always puts whole lines: after the line (before it with !)
                let text = if text.ends_with('\n') { text } else { format!("{text}\n") };
                let at = if bang { b } else { b + 1 };
                self.put_lines(at, &text);
            }
            "center" | "left" => {
                let width = num.unwrap_or_else(|| match self.num(self.cur, "fill_column") { 0 => 80, w => w });
                let lines: Vec<String> = (a..=b).map(|l| self.docs[self.cur].ed.buf.line(l).trim_end_matches('\n').to_string()).collect();
                let new: Vec<String> = lines
                    .iter()
                    .map(|t| {
                        let body = t.trim();
                        if body.is_empty() {
                            return String::new();
                        }
                        let pad = if cmd == "center" { width.saturating_sub(body.chars().count()) / 2 } else { num.unwrap_or(0) };
                        format!("{}{body}", " ".repeat(pad))
                    })
                    .collect();
                self.replace_lines(a, b, &new);
            }
            "uniq" => {
                let (a, b) = range.unwrap_or((0, self.docs[self.cur].ed.buf.len_lines().saturating_sub(1)));
                let ic = arg.contains('i');
                let lines: Vec<String> = (a..=b).map(|l| self.docs[self.cur].ed.buf.line(l).trim_end_matches('\n').to_string()).collect();
                let mut out: Vec<String> = Vec::new();
                for l in lines {
                    let same = out.last().is_some_and(|p: &String| if ic { p.to_lowercase() == l.to_lowercase() } else { *p == l });
                    if !same {
                        out.push(l);
                    }
                }
                self.replace_lines(a, b, &out);
            }
            "~" => {
                // the last :s with the last search pattern
                let pat = self.docs[self.cur].ed.register('/').map(|r| r.0).unwrap_or_default();
                match self.last_sub.clone() {
                    Some((_, rep, flags)) => {
                        let fl = if arg.trim().is_empty() { flags } else { arg.trim().to_string() };
                        let l = format!("{},{}s/{}/{}/{}", a + 1, b + 1, pat.replace('/', "\\/"), rep, fl);
                        self.command(&l);
                    }
                    None => self.say("E35: No previous regular expression"),
                }
            }
            "@" | "*" => {
                // a register's lines as : commands (@: the last one, @@ the last register again)
                let r = arg.trim().chars().next().unwrap_or('@');
                if r == ':' {
                    if let Some(l) = self.cmd_hist.last().cloned() {
                        self.command(&l);
                    }
                    return;
                }
                // @ the register the last :@ ran
                let r = if r == '@' { self.last_at.unwrap_or('"') } else { r };
                self.last_at = Some(r);
                let Some((text, _)) = self.docs[self.cur].ed.register(r) else { return self.say(format!("E748: No previously used register {r}")) };
                for l in text.lines().filter(|l| !l.trim().is_empty()) {
                    self.command(l.trim_start_matches(':'));
                }
            }
            // ── marks, jumps, changes, registers ──
            // :list: the lines as :print shows them, a tab as ^I and the end as $
            "list" => {
                let d = &self.docs[self.cur];
                let rows = (a..=b).map(|l| (format!("{:>4}", l + 1), format!("{}$", d.ed.buf.line(l).trim_end_matches('\n').replace('\t', "^I")))).collect();
                self.popup = Some(Popup::Help(format!("lines {}-{}", a + 1, b + 1), rows));
            }
            // :dl: delete, then list the line that is current now
            "dl" => {
                let r = range.map(|(a, b)| format!("{},{}", a + 1, b + 1)).unwrap_or_default();
                self.ex(&format!("{r}delete {}", arg.trim()));
                let l = self.docs[self.cur].ed.cursor().line;
                let text = self.docs[self.cur].ed.buf.line(l).trim_end_matches('\n').replace('\t', "^I");
                self.say(format!("{text}$"));
            }
            // :goto N: to byte N of the buffer (1 the first; a range gives it too)
            "goto" => {
                let n = num.or(range.map(|(_, b)| b + 1)).unwrap_or(1).max(1);
                let text = self.docs[self.cur].ed.buf.text();
                let mut byte = (n - 1).min(text.len().saturating_sub(1));
                while byte > 0 && !text.is_char_boundary(byte) {
                    byte -= 1;
                }
                let at = text[..byte].chars().count();
                let d = &mut self.docs[self.cur];
                let p = d.ed.buf.to_pos(at);
                d.ed.set_cursor(p);
            }
            // :iput: :put with the lines indented as the current one (as ]p)
            "iput" => {
                let r = arg.trim().chars().next().unwrap_or('"');
                let Some((text, _)) = self.docs[self.cur].ed.register(r) else { return self.say(format!("E353: Nothing in register {r}")) };
                let want: String = self.docs[self.cur].ed.buf.line(b).chars().take_while(|c| *c == ' ' || *c == '\t').collect();
                let lines: Vec<&str> = text.trim_end_matches('\n').split('\n').collect();
                let least = lines.iter().filter(|l| !l.trim().is_empty()).map(|l| l.chars().take_while(|c| *c == ' ' || *c == '\t').count()).min().unwrap_or(0);
                let text: String = lines.iter().map(|l| if l.trim().is_empty() { "\n".to_string() } else { format!("{want}{}\n", l.chars().skip(least).collect::<String>()) }).collect();
                let at = if bang { b } else { b + 1 };
                self.put_lines(at, &text);
            }
            // :smagic :snomagic: :s with 'magic' on or off for its pattern (\m, \M)
            "smagic" | "snomagic" => {
                let r = range.map(|(a, b)| format!("{},{}", a + 1, b + 1)).unwrap_or_default();
                let t = arg.trim_start();
                let line = match t.chars().next() {
                    Some(d) if !d.is_alphanumeric() && d != '\\' && d != '"' && d != '|' => format!("{r}s{d}{}{}", if cmd == "snomagic" { "\\M" } else { "\\m" }, &t[d.len_utf8()..]),
                    _ => format!("{r}s {t}"),
                };
                self.ex(&line);
            }
            "undojoin" => {
                if let Err(e) = self.docs[self.cur].ed.undo_join() {
                    self.say(e);
                }
            }
            "mark" | "k" => match arg.trim().chars().next() {
                Some(c) if c.is_ascii_alphabetic() || "'`[]<>".contains(c) => {
                    let at = self.docs[self.cur].ed.buf.to_char(Pos { line: b, col: 0 });
                    self.docs[self.cur].ed.set_vim_mark(c, at);
                }
                _ => self.say("E191: Argument must be a letter or forward/backward quote"),
            },
            "marks" => {
                let want: Vec<char> = arg.chars().filter(|c| !c.is_whitespace()).collect();
                let d = &self.docs[self.cur];
                let rows = d.ed.marks().into_iter().filter(|m| want.is_empty() || want.contains(&m.0)).map(|(c, l, col)| (format!(" {c}"), format!("{:>5} {:>4}  {}", l + 1, col, d.ed.buf.line(l).trim().chars().take(60).collect::<String>()))).collect();
                self.popup = Some(Popup::Help("mark  line  col  text".into(), rows));
            }
            "delmarks" => {
                let spec = if bang { "!".to_string() } else { arg.to_string() };
                self.docs[self.cur].ed.del_marks(&spec);
            }
            "jumps" => {
                let d = &self.docs[self.cur];
                let (list, at) = d.ed.jump_list();
                let rows = list.iter().enumerate().map(|(i, &p)| {
                    let pos = d.ed.buf.to_pos(p.min(d.ed.buf.len_chars()));
                    (format!("{}{:>3}", if i == at { ">" } else { " " }, i.abs_diff(at)), format!("{:>5} {:>4}  {}", pos.line + 1, pos.col, d.ed.buf.line(pos.line).trim().chars().take(60).collect::<String>()))
                }).collect();
                self.popup = Some(Popup::Help("jump line  col  text".into(), rows));
            }
            "clearjumps" => self.docs[self.cur].ed.clear_jumps(),
            "changes" => {
                let d = &self.docs[self.cur];
                let list = d.ed.change_list();
                let rows = list.iter().enumerate().map(|(i, &p)| {
                    let pos = d.ed.buf.to_pos(p.min(d.ed.buf.len_chars()));
                    (format!("{:>4}", list.len() - i), format!("{:>5} {:>4}  {}", pos.line + 1, pos.col, d.ed.buf.line(pos.line).trim().chars().take(60).collect::<String>()))
                }).collect();
                self.popup = Some(Popup::Help("change line  col  text".into(), rows));
            }
            "registers" | "display" => {
                let want: Vec<char> = arg.chars().filter(|c| !c.is_whitespace()).collect();
                let rows = self.docs[self.cur].ed.registers().into_iter().filter(|r| want.is_empty() || want.contains(&r.0)).map(|(c, t, lw)| (format!("{}  \"{c}", if lw { "l" } else { "c" }), t.replace('\n', "^J").chars().take(70).collect())).collect();
                self.popup = Some(Popup::Help("Type Name Content".into(), rows));
            }
            // ── modes ──
            "startinsert" => self.docs[self.cur].ed.feed(if bang { "A" } else { "i" }),
            "stopinsert" => self.docs[self.cur].ed.feed("<Esc>"),
            "startreplace" => self.docs[self.cur].ed.feed(if bang { "$R" } else { "R" }),
            "startgreplace" => self.docs[self.cur].ed.feed("gR"),
            // ── the rest ──
            "redraw" | "redrawstatus" | "redrawtabline" => self.clear = true,
            "sleep" => {
                let t = arg.trim();
                let ms = match t.strip_suffix('m') {
                    Some(v) => v.parse::<u64>().unwrap_or(1000),
                    None => t.parse::<u64>().unwrap_or(1) * 1000,
                };
                std::thread::sleep(Duration::from_millis(ms.min(10_000)));
            }
            "suspend" | "stop" => {
                if !bang {
                    self.auto_write();
                }
                self.suspend = true;
            }
            "cquit" => {
                crate::EXIT_CODE.store(num.unwrap_or(1) as i32, std::sync::atomic::Ordering::Relaxed);
                self.quit = true;
            }
            "quitall" => self.command(if bang { "qa!" } else { "qa" }),
            "exit" => self.command(if bang { "x!" } else { "x" }),
            "visual" | "ex" => self.command(&format!("e{} {}", if bang { "!" } else { "" }, arg.trim())),
            "view" => {
                self.command(&format!("e {}", arg.trim()));
                let i = self.cur;
                self.docs[i].local.insert("readonly".into(), "true".into());
            }
            "intro" => {
                if let Some(i) = self.docs.iter().position(|d| d.name == "[scratch]") {
                    self.show_doc(i);
                } else {
                    self.docs.push(Doc::scratch());
                    let i = self.docs.len() - 1;
                    self.show_doc(i);
                }
                self.dash = true;
            }
            "mode" => {}
            "winpos" => self.say("E188: Obtaining window position not implemented for this platform"),
            "ascii" => self.docs[self.cur].ed.feed("ga"),
            "exusage" => self.open_help("commands"),
            "viusage" => self.open_help("vim-keys"),
            "options" => self.open_help("settings"),
            "filetype" => {
                if arg.trim() == "detect" {
                    let i = self.cur;
                    if let Some(p) = self.docs[i].abs_path() {
                        let name = self.ss.find_syntax_for_file(&p).ok().flatten().map(|s| s.name.clone()).unwrap_or_else(|| "Plain Text".into());
                        self.docs[i].local.insert("filetype".into(), crate::settings::ft_key(&name));
                    }
                }
                self.say("filetype detection:ON  plugin:ON  indent:ON");
            }
            "setfiletype" => {
                let i = self.cur;
                self.docs[i].local.insert("filetype".into(), arg.trim().to_string());
            }
            "setglobal" => {
                // :set for the global value only: this buffer keeps its own
                let i = self.cur;
                let before = self.docs[i].local.clone();
                self.set_command(arg);
                self.docs[i].local = before;
                self.apply_settings();
            }
            "syntax" => match arg.trim() {
                "on" | "enable" | "" => self.set_command("syntax="),
                "off" | "clear" | "reset" => self.set_command("syntax=off"),
                other => self.say(format!("syntax {other}: zett highlights through its themes (:hi changes the colours)")),
            },
            "history" => {
                let rows = self.cmd_hist.iter().enumerate().rev().take(50).map(|(i, l)| (format!("{:>4}", i + 1), l.clone())).collect();
                self.popup = Some(Popup::Help("      # cmd history".into(), rows));
            }
            "oldfiles" => {
                let rows = doom::recent().into_iter().enumerate().take(100).map(|(i, f)| (format!("{:>3}", i + 1), f)).collect();
                self.popup = Some(Popup::Help("Recent files".into(), rows));
            }
            "helpclose" => {
                if let Some(i) = self.docs.iter().position(|d| d.name.starts_with("help:")) {
                    self.docs.remove(i);
                    self.doc_removed(i);
                }
            }
            "helpgrep" | "lhelpgrep" => {
                let Ok(re) = crate::ex::vim_regex(arg.trim(), true) else { return self.say("E866: bad pattern") };
                let mut items = Vec::new();
                for (topic, _, text) in zett_script::docs::TOPICS {
                    for (n, l) in text.lines().enumerate() {
                        if re.is_match(l).unwrap_or(false) {
                            items.push(crate::quickfix::QfItem { file: format!("help:{topic}"), line: n + 1, col: 1, kind: ' ', nr: 0, text: l.trim().to_string(), valid: true });
                        }
                    }
                }
                let loc = cmd == "lhelpgrep";
                let text: String = items.iter().map(crate::quickfix::line_of).collect::<Vec<_>>().join("\n");
                let _ = text;
                self.qf_from_items(loc, items, &format!(":helpgrep {}", arg.trim()));
            }
            "helptags" => self.say("zett's help is built in: every topic is found already"),
            "echon" | "echomsg" | "echoerr" => match self.with_lua(|e| e.eval(arg)) {
                Ok((out, fx)) => {
                    self.after_lua(fx);
                    self.say(if cmd == "echoerr" { format!("E605: {out}") } else { out });
                }
                Err(e) => self.say(e),
            },
            "eval" => match self.with_lua(|e| e.eval(arg)) {
                Ok((_, fx)) => self.after_lua(fx),
                Err(e) => self.say(e),
            },
            "execute" => match self.with_lua(|e| e.eval(&format!("return {arg}"))) {
                Ok((out, fx)) => {
                    self.after_lua(fx);
                    for l in out.split('\n').filter(|l| !l.trim().is_empty()) {
                        self.command(l.trim().trim_start_matches(':'));
                    }
                }
                Err(e) => self.say(e),
            },
            "fclose" => self.popup = None,
            "match" | "2match" | "3match" => {
                let w = (self.focus, if cmd == "match" { 1 } else { cmd.as_bytes()[0] - b'0' });
                let t = arg.trim();
                if t.is_empty() || t == "none" {
                    self.win_match.remove(&w);
                } else if let Some((group, pat)) = t.split_once(' ') {
                    let pat = pat.trim();
                    let d = pat.chars().next().unwrap_or('/');
                    let inner = pat.trim_start_matches(d).trim_end_matches(d).to_string();
                    self.win_match.insert(w, (group.to_string(), inner));
                }
            }
            // ── spelling ──
            "spellgood" | "spellwrong" | "spellundo" | "spellrare" => {
                let w = arg.trim();
                if w.is_empty() {
                    return self.say("E1280: give the word (:spellgood word)");
                }
                let good = cmd != "spellwrong";
                zett_edit::spell::with(|s| s.mark(w, good, bang, cmd == "spellundo"));
                self.say(format!("Word '{w}' {}", if cmd == "spellundo" { "taken off the list" } else if good { "added to the good words" } else { "marked wrong" }));
            }
            "spellinfo" => {
                let lang = self.setting(self.cur, "spelllang");
                let have = zett_edit::spell::with(|s| s.available());
                self.say(format!("spelllang={lang}: {}", if have { "a dictionary is loaded" } else { "no dictionary (sudo pacman -S hunspell-en_us)" }));
            }
            "spelldump" => {
                let words = zett_edit::spell::with(|s| s.words());
                let mut text = words.join("\n");
                text.push('\n');
                self.docs.push(Doc::new(Editor::from_str(&text), "[spelldump]".into(), "Plain Text".into()));
                let i = self.docs.len() - 1;
                self.show_doc(i);
            }
            // ── folds ──
            "fold" => {
                self.goto_line(a + 1);
                self.docs[self.cur].ed.feed(&format!("{}zF", b - a + 1));
            }
            "foldclose" | "foldopen" => {
                for l in a..=b {
                    self.goto_line(l + 1);
                    let k = match (cmd == "foldclose", bang) { (true, false) => "zc", (true, true) => "zC", (false, false) => "zo", _ => "zO" };
                    self.docs[self.cur].ed.feed(k);
                }
            }
            "folddoopen" | "folddoclosed" => {
                let (a, b) = range.unwrap_or((0, self.docs[self.cur].ed.buf.len_lines().saturating_sub(1)));
                let closed = cmd == "folddoclosed";
                let picks: Vec<usize> = (a..=b).filter(|&l| {
                    let d = &self.docs[self.cur].ed;
                    let inside = d.is_hidden(l) || d.is_fold_start(l);
                    inside == closed
                }).collect();
                // from the bottom, so the lines above stay where they are
                for l in picks.into_iter().rev() {
                    self.goto_line(l + 1);
                    self.command(arg);
                }
            }
            "luado" => {
                let (a, b) = range.unwrap_or((0, self.docs[self.cur].ed.buf.len_lines().saturating_sub(1)));
                let mut out = Vec::new();
                for l in a..=b {
                    let line = self.docs[self.cur].ed.buf.line(l).trim_end_matches('\n').to_string();
                    let code = format!("local line, linenr = {line:?}, {}; local r = (function() {arg} end)(); if type(r) == \"string\" then return r end; return line", l + 1);
                    match self.with_lua(|e| e.eval(&code)) {
                        Ok((r, _)) => out.push(r),
                        Err(e) => return self.say(e),
                    }
                }
                self.replace_lines(a, b, &out);
            }
            // the other languages: their own interpreters, if installed (decided 2026-10-21)
            c if c.starts_with("py") || c.starts_with("perl") || c.starts_with("rub") => self.provider(c, arg, range),
            "redir" => {
                let t = arg.trim();
                if t == "END" || t == "end" {
                    if let Some((to, text)) = self.redir.take() {
                        self.redir_write(&to, &text);
                    }
                } else if !t.is_empty() {
                    self.redir = Some((t.to_string(), String::new()));
                }
            }
            _ => {}
        }
    }

    /// Edits the argument list's file `i` (as :argument).
    fn arg_edit(&mut self, i: usize) {
        if self.args.is_empty() {
            return self.say("E163: There is only one file to edit");
        }
        if i >= self.args.len() {
            return self.say("E165: Cannot go beyond last file");
        }
        self.arg_idx = i;
        let f = self.args[i].clone();
        self.command(&format!("e {f}"));
        if self.args.len() > 1 {
            self.say(format!("\"{f}\" ({} of {})", i + 1, self.args.len()));
        }
    }

    /// Lines `text` put in before line `at` of the current buffer.
    fn put_lines(&mut self, at: usize, text: &str) {
        let d = &mut self.docs[self.cur];
        let lines = d.ed.buf.len_lines();
        let (pos, text) = if at >= lines {
            let last = lines.saturating_sub(1);
            let end = d.ed.buf.line(last).trim_end_matches('\n').chars().count();
            let ends = d.ed.buf.text().ends_with('\n');
            (if ends { Pos { line: lines, col: 0 } } else { Pos { line: last, col: end } }, if ends { text.to_string() } else { format!("\n{}", text.trim_end_matches('\n')) })
        } else {
            (Pos { line: at, col: 0 }, text.to_string())
        };
        d.ed.apply_edits(vec![(pos, pos, text)]);
        d.states.clear();
        let n = self.docs[self.cur].ed.buf.len_lines();
        self.goto_line((at + 1).min(n));
    }

    /// Lines a..=b of the current buffer become `new`.
    pub(crate) fn replace_lines(&mut self, a: usize, b: usize, new: &[String]) {
        let d = &mut self.docs[self.cur];
        let end = Pos { line: b, col: d.ed.buf.line(b).trim_end_matches('\n').chars().count() };
        d.ed.apply_edits(vec![(Pos { line: a, col: 0 }, end, new.join("\n"))]);
        d.states.clear();
    }

    /// Moves buffer `from` to place `to` in the tab line; the windows keep showing theirs.
    fn move_doc_to(&mut self, from: usize, to: usize) {
        if from == to || from >= self.docs.len() {
            return;
        }
        let d = self.docs.remove(from);
        self.docs.insert(to, d);
        let map = |i: usize| if i == from { to } else if from < to && i > from && i <= to { i - 1 } else if to < from && i >= to && i < from { i + 1 } else { i };
        for w in self.wins.iter_mut().flatten() {
            w.doc = map(w.doc);
        }
        self.cur = map(self.cur);
        self.last_doc = self.last_doc.map(map);
    }

    /// :redir END: what was said goes to a file (> or >>) or a register (@r, @r>>).
    fn redir_write(&mut self, to: &str, text: &str) {
        if let Some(r) = to.strip_prefix('@') {
            let (c, add) = (r.chars().next().unwrap_or('"'), r.contains(">>"));
            let old = if add { self.docs[self.cur].ed.register(c).map(|r| r.0).unwrap_or_default() } else { String::new() };
            self.docs[self.cur].ed.set_register(c, &format!("{old}{text}"));
        } else {
            let (path, append) = match to.strip_prefix(">>") {
                Some(p) => (p.trim(), true),
                None => (to.trim_start_matches('>').trim_start_matches('!').trim(), false),
            };
            let p = crate::files::expand_env(path);
            let r = if append {
                use std::io::Write;
                std::fs::OpenOptions::new().append(true).create(true).open(&p).and_then(|mut f| f.write_all(text.as_bytes()))
            } else {
                std::fs::write(&p, text)
            };
            if let Err(e) = r {
                self.say(format!("E190: Cannot open \"{p}\" for writing: {e}"));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    fn app_with(text: &str) -> crate::App {
        let mut app = crate::App::new(Vec::new());
        let d = &mut app.docs[app.cur];
        d.ed = zett_edit::Editor::from_str(text);
        app
    }

    fn text(app: &crate::App) -> String {
        app.docs[app.cur].ed.buf.text()
    }

    #[test]
    fn goto_iput_dl_k_and_the_match_numbers() {
        let mut app = app_with("ab\ncdé\nfg\n");
        app.command("goto 5");
        assert_eq!(app.docs[app.cur].ed.cursor(), zett_core::Pos { line: 1, col: 1 }, ":goto 5 is byte 5: 'd'");
        app.command("goto 9");
        assert_eq!(app.docs[app.cur].ed.cursor(), zett_core::Pos { line: 2, col: 0 }, "é is two bytes: f is byte 9");
        app.command("goto 7");
        assert_eq!(app.docs[app.cur].ed.cursor(), zett_core::Pos { line: 1, col: 2 }, "inside é: on é");
        // :iput: the register's lines at the current line's indent
        let mut app = app_with("    x\n");
        app.docs[app.cur].ed.set_register('a', "a\n  b\n");
        app.command("iput a");
        assert_eq!(text(&app), "    x\n    a\n      b\n");
        // :dl deletes and lists the next line
        let mut app = app_with("one\ntwo\n");
        app.command("1dl");
        assert_eq!(text(&app), "two\n");
        // :ka is :mark a
        let mut app = app_with("one\ntwo\n");
        app.command("2ka");
        assert!(app.docs[app.cur].ed.marks().iter().any(|m| m.0 == 'a' && m.1 == 1), "{:?}", app.docs[app.cur].ed.marks());
        // :2match and :3match beside :match
        app.command("2match Error /two/");
        app.command("match Search /one/");
        assert_eq!(app.win_match.len(), 2);
        app.command("2match none");
        assert_eq!(app.win_match.len(), 1);
    }

    #[test]
    fn snomagic_and_undojoin() {
        let mut app = app_with("a.c abc\n");
        app.command("snomagic/a.c/X/g");
        assert_eq!(text(&app), "X abc\n", "nomagic: . is a dot");
        let mut app = app_with("a.c abc\n");
        app.command("smagic/a.c/X/g");
        assert_eq!(text(&app), "X X\n", "magic: . is any character");
        // \M in a pattern on its own
        let mut app = app_with("a*b aab\n");
        app.command("s/\\Ma*b/Y/");
        assert_eq!(text(&app), "Y aab\n");
        // :undojoin: two changes undone as one
        let mut app = app_with("x\n");
        for k in zett_edit::keys::parse("Aa<Esc>") {
            app.key(crate::input::key_event(&k));
        }
        app.command("undojoin");
        for k in zett_edit::keys::parse("Ab<Esc>u") {
            app.key(crate::input::key_event(&k));
        }
        assert_eq!(text(&app), "x\n", "u undid both");
        for k in zett_edit::keys::parse("Ac<Esc>u") {
            app.key(crate::input::key_event(&k));
        }
        app.command("undojoin");
        assert!(app.msg.as_ref().is_some_and(|m| m.0.starts_with("E790")), "{:?}", app.msg);
    }

    #[test]
    fn names_and_shortest_forms() {
        assert_eq!(super::ex_name("n"), Some("next"));
        assert_eq!(super::ex_name("N"), Some("Next"));
        assert_eq!(super::ex_name("sb"), Some("sbuffer"));
        assert_eq!(super::ex_name("clo"), Some("close"));
        assert_eq!(super::ex_name("c"), None);
        assert_eq!(super::ex_name("tabe"), Some("tabedit"));
        assert_eq!(super::ex_name("vert"), Some("vertical"));
        assert_eq!(super::ex_name("sl"), Some("sleep"));
        assert_eq!(super::ex_name("s"), None, ":s is :substitute");
    }
}

impl App {
    /// :python3 :perl :ruby (code), their …do (each line: Python's `line`, Perl's and Ruby's $_;
    /// what comes back replaces the line) and …file (a script); E319 when the interpreter is missing.
    fn provider(&mut self, cmd: &str, arg: &str, range: Option<(usize, usize)>) {
        let (prog, flag) = if cmd.starts_with("py") { ("python3", "-c") } else if cmd.starts_with("perl") { ("perl", "-e") } else { ("ruby", "-e") };
        let have = std::process::Command::new("sh").args(["-c", &format!("command -v {prog}")]).output().is_ok_and(|o| o.status.success());
        if !have {
            return self.say(format!("E319: No \"{prog}\" provider found. Run \":checkhealth vim.provider\""));
        }
        let dir = self.root.clone();
        let run = |args: Vec<String>, input: Option<String>| -> Result<String, String> {
            use std::io::Write;
            let mut c = std::process::Command::new(prog);
            c.args(&args).current_dir(&dir).stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped()).stdin(if input.is_some() { std::process::Stdio::piped() } else { std::process::Stdio::null() });
            let mut child = c.spawn().map_err(|e| e.to_string())?;
            if let (Some(t), Some(mut si)) = (input, child.stdin.take()) {
                std::thread::spawn(move || {
                    let _ = si.write_all(t.as_bytes());
                });
            }
            let o = child.wait_with_output().map_err(|e| e.to_string())?;
            if !o.status.success() {
                return Err(String::from_utf8_lossy(&o.stderr).trim().to_string());
            }
            Ok(String::from_utf8_lossy(&o.stdout).to_string())
        };
        if cmd.ends_with("do") {
            let (a, b) = range.unwrap_or((0, self.docs[self.cur].ed.buf.len_lines().saturating_sub(1)));
            let lines: Vec<String> = (a..=b).map(|l| self.docs[self.cur].ed.buf.line(l).trim_end_matches('\n').to_string()).collect();
            let script = match prog {
                "python3" => format!("import sys\ndef _zett(line, linenr):\n    {arg}\nfor _i, _l in enumerate(sys.stdin.read().split('\\n')):\n    _r = _zett(_l, {} + _i)\n    print(_l if _r is None else _r)", a + 1),
                "perl" => format!("while (<STDIN>) {{ chomp; {arg}; print \"$_\\n\" }}"),
                _ => format!("$stdin.each_line {{ |l| $_ = l.chomp; {arg}; puts $_ }}"),
            };
            match run(vec![flag.into(), script], Some(lines.join("\n"))) {
                Ok(out) => {
                    let new: Vec<String> = out.trim_end_matches('\n').split('\n').map(str::to_string).collect();
                    self.replace_lines(a, b, &new);
                }
                Err(e) => self.say(e),
            }
        } else if cmd.ends_with("file") {
            match run(vec![crate::files::expand_env(arg.trim())], None) {
                Ok(out) => self.say(out.trim_end().to_string()),
                Err(e) => self.say(e),
            }
        } else {
            match run(vec![flag.into(), arg.to_string()], None) {
                Ok(out) => self.say(out.trim_end().to_string()),
                Err(e) => self.say(e),
            }
        }
    }
}
