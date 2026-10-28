//! Tags (ctags files): :tag :tjump :tselect, :tnext :tNext :tprevious :trewind :tfirst :tlast,
//! :pop and :tags (the tag stack), their split (:stag :stjump :stselect) and preview (:ptag …)
//! forms, :ltag (the matches into the location list), and CTRL-] / CTRL-T without a language
//! server. Files from 'tags' ("./tags;,tags": ./ is the file's folder, ; looks upwards), read as
//! 'tagrelative', 'tagcase' and 'taglength' say; 'tagstack' off: jumps leave the stack alone.
use super::*;
use crate::quickfix::QfItem;

/// The commands by Vim's full names and shortest forms.
const NAMES: &[(&str, &str)] = &[
    ("tag", "ta"), ("tags", "tags"), ("tjump", "tj"), ("tselect", "ts"), ("tnext", "tn"), ("tNext", "tN"),
    ("tprevious", "tp"), ("trewind", "tr"), ("tfirst", "tf"), ("tlast", "tl"), ("pop", "po"), ("ppop", "pp"),
    ("stag", "sta"), ("stjump", "stj"), ("stselect", "sts"), ("ptag", "pt"), ("ptjump", "ptj"), ("ptselect", "pts"),
    ("ptnext", "ptn"), ("ptNext", "ptN"), ("ptprevious", "ptp"), ("ptrewind", "ptr"), ("ptfirst", "ptf"), ("ptlast", "ptl"),
    ("ltag", "lt"),
];

/// The command a typed name stands for (a prefix down to the shortest form).
pub(crate) fn tag_name(typed: &str) -> Option<&'static str> {
    NAMES.iter().find(|(full, short)| typed.len() >= short.len() && full.starts_with(typed)).map(|(f, _)| *f)
}

/// One line of a tags file: the name, the file (absolute), where in it, its kind (f, v, s …).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Tag {
    pub name: String,
    pub file: PathBuf,
    /// a line number, or a search pattern as written (/^fn main() {$/)
    pub addr: String,
    pub kind: String,
}

/// A jump made by a tag: the name, its matches and the one gone to, and where it was made from.
#[derive(Clone, Debug)]
pub(crate) struct TagEntry {
    pub name: String,
    pub matches: Vec<Tag>,
    pub idx: usize,
    pub from: (usize, usize, usize),
}

/// The tags files 'tags' names: "./x" in the current file's folder, "x" in the current folder; a
/// trailing ";" (or ";stop") looks for it in each folder above too, up to `stop` or the root.
pub(crate) fn tag_files(spec: &str, file_dir: &Path, cwd: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for item in spec.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        let (name, upward) = match item.split_once(';') {
            Some((n, stop)) => (n, Some(stop)),
            None => (item, None),
        };
        let (base, rest) = match name.strip_prefix("./") {
            Some(r) => (file_dir.to_path_buf(), r),
            None if name.starts_with('/') => (PathBuf::from("/"), name.trim_start_matches('/')),
            None => (cwd.to_path_buf(), name),
        };
        let mut dir = Some(base.as_path());
        while let Some(d) = dir {
            let p = d.join(rest);
            if p.is_file() && !out.contains(&p) {
                out.push(p);
            }
            if upward.is_none() || upward.is_some_and(|s| !s.is_empty() && d == Path::new(s)) {
                break;
            }
            dir = d.parent();
        }
    }
    out
}

/// The tags in a file whose names match: exactly (as 'tagcase' says), only the first `len`
/// characters when 'taglength' is set, or a pattern (`/re`). Paths are made absolute against the
/// file's folder ('tagrelative', the default) or the current folder.
pub(crate) fn read_tags(file: &Path, want: &str, ignore_case: bool, len: usize, relative: bool, cwd: &Path) -> Vec<Tag> {
    let Ok(text) = std::fs::read_to_string(file) else { return Vec::new() };
    let base = if relative { file.parent().map(Path::to_path_buf).unwrap_or_default() } else { cwd.to_path_buf() };
    let re = want.strip_prefix('/').and_then(|p| fancy_regex::Regex::new(&format!("{}{p}", if ignore_case { "(?i)" } else { "" })).ok());
    let cut = |s: &str| -> String {
        let s = if ignore_case { s.to_lowercase() } else { s.to_string() };
        if len > 0 { s.chars().take(len).collect() } else { s }
    };
    let want_cut = cut(want);
    let mut out = Vec::new();
    for line in text.lines().filter(|l| !l.starts_with("!_TAG_")) {
        let mut f = line.splitn(3, '\t');
        let (Some(name), Some(path), Some(rest)) = (f.next(), f.next(), f.next()) else { continue };
        let hit = match &re {
            Some(r) => r.is_match(name).unwrap_or(false),
            None => cut(name) == want_cut,
        };
        if !hit {
            continue;
        }
        // the address, up to ;" (the extension fields after it: the kind first)
        let (addr, ext) = match rest.find(";\"") {
            Some(i) => (&rest[..i], &rest[i + 2..]),
            None => (rest, ""),
        };
        let kind = ext.split('\t').map(str::trim).find(|s| !s.is_empty()).map(|k| k.trim_start_matches("kind:").to_string()).unwrap_or_default();
        out.push(Tag { name: name.to_string(), file: base.join(path), addr: addr.to_string(), kind });
    }
    out
}

/// The line a tag's address names in `lines` (0-based): its number, or the first line its search
/// pattern matches (a ctags pattern is the whole line, ^ and $ anchored, / and \ escaped).
pub(crate) fn tag_line(addr: &str, lines: &[&str]) -> Option<usize> {
    let a = addr.trim();
    if let Ok(n) = a.parse::<usize>() {
        return Some(n.saturating_sub(1).min(lines.len().saturating_sub(1)));
    }
    let body = a.strip_prefix('/').or_else(|| a.strip_prefix('?'))?;
    let body = body.strip_suffix('/').or_else(|| body.strip_suffix('?')).unwrap_or(body);
    let (start, body) = match body.strip_prefix('^') {
        Some(b) => (true, b),
        None => (false, body),
    };
    let (end, body) = match body.strip_suffix('$') {
        Some(b) if !b.ends_with('\\') => (true, b),
        _ => (false, body),
    };
    let text = body.replace("\\/", "/").replace("\\?", "?").replace("\\\\", "\\");
    lines.iter().position(|l| match (start, end) {
        (true, true) => *l == text,
        (true, false) => l.starts_with(&text),
        (false, true) => l.ends_with(&text),
        (false, false) => l.contains(&text),
    })
}

impl App {
    /// The tags matching `name`, from every file 'tags' names.
    pub(crate) fn find_tags(&self, name: &str) -> Vec<Tag> {
        let cwd = std::env::current_dir().unwrap_or_default();
        let file_dir = self.docs[self.cur].abs_path().and_then(|p| p.parent().map(Path::to_path_buf)).unwrap_or_else(|| cwd.clone());
        // 'tagcase': followic (as 'ignorecase'), followscs (as 'ignorecase' and 'smartcase'), ignore, match, smart
        let upper = name.chars().any(char::is_uppercase);
        let ic = match self.setting(self.cur, "tagcase").as_str() {
            "ignore" => true,
            "match" => false,
            "smart" => !upper,
            "followscs" => self.flag(self.cur, "ignorecase") && !(self.flag(self.cur, "smartcase") && upper),
            _ => self.flag(self.cur, "ignorecase"),
        };
        let (len, relative) = (self.num(self.cur, "taglength"), self.flag(self.cur, "tagrelative"));
        let mut out = Vec::new();
        for f in tag_files(&self.setting(self.cur, "tags"), &file_dir, &cwd) {
            for t in read_tags(&f, name, ic, len, relative, &cwd) {
                if !out.contains(&t) {
                    out.push(t);
                }
            }
        }
        out
    }

    /// Goes to a tag's place; false when its file or line is gone.
    fn goto_tag(&mut self, t: &Tag) -> bool {
        if !t.file.is_file() {
            self.say(format!("E429: File \"{}\" does not exist", t.file.display()));
            return false;
        }
        self.open(&t.file);
        let line = {
            let d = &self.docs[self.cur];
            let text = d.ed.text();
            let lines: Vec<&str> = text.lines().collect();
            tag_line(&t.addr, &lines)
        };
        match line {
            Some(l) => {
                self.goto_line(l + 1);
                true
            }
            None => {
                self.say("E434: Can't find tag pattern");
                false
            }
        }
    }

    fn here(&self) -> (usize, usize, usize) {
        let c = self.docs[self.cur].ed.cursor();
        (self.cur, c.line, c.col)
    }

    /// Jumps to match `idx` of the tag stack's top entry, with Vim's "tag 1 of 3" when there are more.
    fn tag_go(&mut self, idx: usize) {
        let Some(top) = self.tag_stack.last_mut() else { return self.say("E73: Tag stack empty") };
        let n = top.matches.len();
        if n == 0 {
            return;
        }
        top.idx = idx.min(n - 1);
        let (t, i) = (top.matches[top.idx].clone(), top.idx);
        if self.goto_tag(&t) && n > 1 {
            self.say(format!("tag {} of {n}{}", i + 1, if i + 1 == n { " (last)" } else { "" }));
        }
    }

    /// 'tagfunc' set: its matches (None when it is unset or says v:null, so the tags files are read)
    fn tagfunc_tags(&mut self, name: &str) -> Option<Vec<Tag>> {
        let spec = self.setting(self.cur, "tagfunc");
        if spec.is_empty() { return None }
        let q = |s: &str| format!("{s:?}");
        let out = self.engine.eval(&format!("vim._zett_tagfunc({}, {}, 'c')", q(&spec), q(name))).ok()?.0;
        if out == "NONE" { return None }
        let cwd = std::env::current_dir().unwrap_or_default();
        Some(out.lines().filter_map(|l| {
            let mut f = l.split('\t');
            let (n, file, addr, kind) = (f.next()?, f.next()?, f.next()?, f.next().unwrap_or(""));
            let p = PathBuf::from(file);
            Some(Tag { name: n.into(), file: if p.is_absolute() { p } else { cwd.join(p) }, addr: addr.into(), kind: kind.into() })
        }).collect())
    }

    /// A new jump to `name` from here: onto the stack (or in place of its top with 'notagstack').
    fn tag_jump(&mut self, name: &str, pick: bool) {
        let matches = self.tagfunc_tags(name).unwrap_or_else(|| self.find_tags(name));
        if matches.is_empty() {
            return self.say(format!("E426: Tag not found: {name}"));
        }
        let entry = TagEntry { name: name.to_string(), matches, idx: 0, from: self.here() };
        if self.flag(self.cur, "tagstack") || self.tag_stack.is_empty() {
            self.tag_stack.push(entry);
        } else if let Some(top) = self.tag_stack.last_mut() {
            *top = TagEntry { from: top.from, ..entry };
        }
        if pick && self.tag_stack.last().is_some_and(|t| t.matches.len() > 1) {
            self.pick(Pick::Tags(name.to_string()));
        } else {
            self.tag_go(0);
        }
    }

    /// CTRL-] without a language server: the word under the cursor.
    pub(crate) fn tag_at_cursor(&mut self) {
        let d = &self.docs[self.cur];
        let c = d.ed.cursor();
        let line: Vec<char> = d.ed.buf.line(c.line).chars().collect();
        let w = |ch: char| ch.is_alphanumeric() || ch == '_';
        let (mut a, mut b) = (c.col.min(line.len()), c.col.min(line.len()));
        while a > 0 && w(line[a - 1]) {
            a -= 1;
        }
        while b < line.len() && w(line[b]) {
            b += 1;
        }
        if a == b {
            return self.say("E349: No identifier under cursor");
        }
        let word: String = line[a..b].iter().collect();
        self.tag_jump(&word, false);
    }

    /// CTRL-T and :pop: back to where the newest jump was made from.
    pub(crate) fn tag_pop(&mut self, count: usize) -> bool {
        let mut from = None;
        for _ in 0..count.max(1) {
            match self.tag_stack.pop() {
                Some(e) => from = Some(e.from),
                None => break,
            }
        }
        let Some((doc, line, _col)) = from else {
            self.say("E73: Tag stack empty");
            return false;
        };
        if doc < self.docs.len() {
            self.show_doc(doc);
            self.goto_line(line + 1);
        }
        true
    }

    /// The tags picker's entries: "1 f  src/main.rs" with the address as the note.
    pub(crate) fn tag_items(&self) -> Vec<Cand> {
        let cwd = std::env::current_dir().unwrap_or_default();
        self.tag_stack.last().map(|e| e.matches.iter().enumerate().map(|(i, t)| {
            let file = t.file.strip_prefix(&cwd).unwrap_or(&t.file).display().to_string();
            Cand::noted(format!("{} {} {}  {file}", i + 1, if t.kind.is_empty() { " " } else { &t.kind }, t.name), t.addr.clone())
        }).collect()).unwrap_or_default()
    }

    /// Enter in the tags picker: the match its number names.
    pub(crate) fn tag_picked(&mut self, item: &str) {
        if let Some(i) = item.split_whitespace().next().and_then(|n| n.parse::<usize>().ok()) {
            self.tag_go(i.saturating_sub(1));
        }
    }

    /// The tag commands (tag_name says which names are theirs).
    pub(crate) fn tag_command(&mut self, typed: &str, arg: &str, count: Option<usize>) {
        let Some(cmd) = tag_name(typed) else { return };
        let arg = arg.trim();
        let n = count.unwrap_or(1);
        // the split and preview forms: a new window first, then as without the s / p
        let (split, cmd) = if cmd.starts_with("st") || cmd.starts_with("pt") { (true, &cmd[1..]) } else { (false, cmd) };
        if split {
            self.command("split");
        }
        match cmd {
            "tag" if arg.is_empty() => {
                // again to the newest entry's match
                let i = self.tag_stack.last().map(|e| e.idx).unwrap_or(0);
                if self.tag_stack.is_empty() {
                    return self.say("E73: Tag stack empty");
                }
                self.tag_go(i);
            }
            "tag" => self.tag_jump(arg, false),
            "tjump" | "tselect" => {
                let name = if arg.is_empty() { self.tag_stack.last().map(|e| e.name.clone()).unwrap_or_default() } else { arg.to_string() };
                if name.is_empty() {
                    return self.say("E73: Tag stack empty");
                }
                if cmd == "tselect" {
                    let matches = self.find_tags(&name);
                    if matches.is_empty() {
                        return self.say(format!("E426: Tag not found: {name}"));
                    }
                    let from = self.here();
                    self.tag_stack.push(TagEntry { name: name.clone(), matches, idx: 0, from });
                    self.pick(Pick::Tags(name));
                } else {
                    self.tag_jump(&name, true);
                }
            }
            "tnext" | "tNext" | "tprevious" | "trewind" | "tfirst" | "tlast" => {
                let Some(top) = self.tag_stack.last() else { return self.say("E73: Tag stack empty") };
                let (i, len) = (top.idx, top.matches.len());
                let to = match cmd {
                    "tnext" => i + n,
                    "tNext" | "tprevious" => i.saturating_sub(n),
                    "tlast" => len - 1,
                    _ => count.map_or(0, |c| c.saturating_sub(1)),
                };
                if cmd == "tnext" && to >= len {
                    return self.say("E428: Cannot go beyond last matching tag");
                }
                if matches!(cmd, "tNext" | "tprevious") && i == 0 {
                    return self.say("E425: Cannot go before first matching tag");
                }
                self.tag_go(to);
            }
            "pop" => {
                self.tag_pop(n);
            }
            // :ppop: :pop in the preview window (the focus comes back)
            "ppop" => {
                let Some(pw) = self.preview_win.filter(|&w| self.wins.get(w).is_some_and(|x| x.is_some())) else { return self.say("E441: There is no preview window") };
                let back = self.focus;
                self.focus_win(pw);
                self.tag_pop(n);
                self.focus_win(back);
            }
            "tags" => {
                let mut out = String::from("  # TO tag         FROM line  in file/text\n");
                for (i, e) in self.tag_stack.iter().enumerate() {
                    let file = self.docs.get(e.from.0).map(|d| d.name.clone()).unwrap_or_default();
                    out.push_str(&format!("{:>3} {:>2} {:<16} {:>5}  {file}\n", i + 1, e.idx + 1, e.name, e.from.1 + 1));
                }
                out.push('>');
                self.say(out);
            }
            "ltag" => {
                let name = if arg.is_empty() { self.tag_stack.last().map(|e| e.name.clone()).unwrap_or_default() } else { arg.to_string() };
                let matches = self.find_tags(&name);
                if matches.is_empty() {
                    return self.say(format!("E426: Tag not found: {name}"));
                }
                let items: Vec<QfItem> = matches.iter().enumerate().map(|(i, t)| {
                    let line = std::fs::read_to_string(&t.file).ok().and_then(|s| tag_line(&t.addr, &s.lines().collect::<Vec<_>>())).map_or(0, |l| l + 1);
                    QfItem { file: t.file.display().to_string(), line, col: 0, kind: ' ', nr: i + 1, text: format!("{} {}", t.kind, t.name).trim().to_string(), valid: line > 0 }
                }).collect();
                self.qf_set(true, items, &format!("ltag {name}"), false, true);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("zett-tags-{name}-{}", std::process::id()));
        std::fs::create_dir_all(d.join("src/deep")).unwrap();
        d
    }

    #[test]
    fn a_tags_file_is_read_with_its_addresses_and_kinds() {
        let d = dir("read");
        std::fs::write(d.join("tags"), "!_TAG_FILE_FORMAT\t2\t/extended format/\nmain\tsrc/main.rs\t/^fn main() {$/;\"\tf\nMAX\tsrc/lib.rs\t12;\"\tc\nmain\tsrc/other.rs\t3;\"\tf\n").unwrap();
        let t = read_tags(&d.join("tags"), "main", false, 0, true, &d);
        assert_eq!(t.len(), 2);
        assert_eq!((t[0].file.clone(), t[0].addr.as_str(), t[0].kind.as_str()), (d.join("src/main.rs"), "/^fn main() {$/", "f"));
        assert!(read_tags(&d.join("tags"), "max", false, 0, true, &d).is_empty(), "tagcase match");
        assert_eq!(read_tags(&d.join("tags"), "max", true, 0, true, &d).len(), 1, "ignoring case");
        assert_eq!(read_tags(&d.join("tags"), "mai", false, 3, true, &d).len(), 2, "taglength 3: mai = main");
        assert_eq!(read_tags(&d.join("tags"), "/^M", false, 0, true, &d).len(), 1, "a pattern");
        std::fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn the_address_finds_its_line() {
        let lines = ["use x;", "", "fn main() {", "    let a = b/c;", "}"];
        assert_eq!(tag_line("/^fn main() {$/", &lines), Some(2));
        assert_eq!(tag_line("/^    let a = b\\/c;$/", &lines), Some(3), "an escaped slash");
        assert_eq!(tag_line("5", &lines), Some(4));
        assert_eq!(tag_line("/^gone$/", &lines), None);
    }

    #[test]
    fn tags_files_are_found_upwards_with_a_semicolon() {
        let d = dir("up");
        std::fs::write(d.join("tags"), "").unwrap();
        let deep = d.join("src/deep");
        assert!(tag_files("./tags", &deep, &deep).is_empty(), "not in the file's folder");
        assert_eq!(tag_files("./tags;", &deep, &deep), vec![d.join("tags")], "found above");
        assert_eq!(tag_files("tags", &deep, &d), vec![d.join("tags")], "in the current folder");
        std::fs::remove_dir_all(&d).ok();
    }
}

#[cfg(test)]
mod tagfunc_tests {
    #[test]
    fn tagfunc_gives_the_matches() {
        let d = std::env::temp_dir().join(format!("zett-tagfunc-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        let f = d.join("a.rs");
        std::fs::write(&f, "use x;\n\nfn target() {}\n").unwrap();
        let mut app = crate::App::new(Vec::new());
        app.command(&format!("lua function _G.my_tf(p) if p == 'target' then return {{ {{ name = p, filename = {:?}, cmd = '/^fn target/', kind = 'f' }} }} end return vim.NIL end", f.display().to_string()));
        app.command("set tagfunc=v:lua.my_tf");
        app.command("tag target");
        assert_eq!(app.docs[app.cur].abs_path().as_deref(), Some(f.as_path()));
        assert_eq!(app.docs[app.cur].ed.cursor().line, 2);
        // v:null: the tags files, none here
        app.command("tag other");
        assert!(app.msg.as_ref().is_some_and(|(m, _)| m.contains("E426")), "{:?}", app.msg);
        let _ = std::fs::remove_dir_all(&d);
    }
}

#[cfg(test)]
mod findfunc_tests {
    #[test]
    fn find_uses_findfunc_then_path() {
        let d = std::env::temp_dir().join(format!("zett-findfunc-{}", std::process::id()));
        std::fs::create_dir_all(d.join("src")).unwrap();
        let (a, b) = (d.join("src/lib.rs"), d.join("other.txt"));
        std::fs::write(&a, "x\n").unwrap();
        std::fs::write(&b, "y\n").unwrap();
        let mut app = crate::App::new(Vec::new());
        app.root = d.clone();
        app.command("set path=src suffixesadd=.rs");
        app.command("find lib");
        assert_eq!(app.docs[app.cur].abs_path().as_deref(), Some(a.as_path()), "path + suffixesadd");
        app.command(&format!("lua function _G.ff(n) return {{ {:?} }} end", b.display().to_string()));
        app.command("set findfunc=v:lua.ff");
        app.command("find anything");
        assert_eq!(app.docs[app.cur].abs_path().as_deref(), Some(b.as_path()), "findfunc");
        let _ = std::fs::remove_dir_all(&d);
    }
}
