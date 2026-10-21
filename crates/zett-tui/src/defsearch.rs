//! Vim's searches through a file and the files it includes: :ilist :isearch :ijump :isplit (a
//! word or /pattern/), :dlist :dsearch :djump :dsplit (its definitions, 'define'), :psearch (in the
//! preview window, a split here), :checkpath (the included files and the missing ones).
//! 'include', 'path' and 'suffixesadd' find the files; C and C++ have Vim's #include and #define.

use super::*;
use fancy_regex::Regex;

const NAMES: &[(&str, &str)] = &[
    ("ilist", "il"), ("isearch", "is"), ("ijump", "ij"), ("isplit", "isp"),
    ("dlist", "dli"), ("dsearch", "ds"), ("djump", "dj"), ("dsplit", "dsp"),
    ("psearch", "ps"), ("checkpath", "checkp"),
];

/// A typed name as one of these commands (its full name).
pub(crate) fn def_name(typed: &str) -> Option<&'static str> {
    NAMES.iter().find(|(full, short)| typed.len() >= short.len() && full.starts_with(typed)).map(|(f, _)| *f)
}

/// A line found: its file (None: the buffer itself), line (0-based) and text.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Hit {
    pub file: Option<PathBuf>,
    pub line: usize,
    pub text: String,
}

/// `[count] [/]pattern[/]`: the count and the pattern (a word alone as a whole word).
fn parse_arg(arg: &str) -> (usize, String) {
    let t = arg.trim();
    let (count, rest) = match t.split_once(char::is_whitespace) {
        Some((n, r)) if n.chars().all(|c| c.is_ascii_digit()) => (n.parse().unwrap_or(1), r.trim()),
        _ if t.chars().all(|c| c.is_ascii_digit()) && !t.is_empty() => (t.parse().unwrap_or(1), ""),
        _ => (1, t),
    };
    let pat = match rest.strip_prefix('/') {
        Some(p) => p.strip_suffix('/').unwrap_or(p).to_string(),
        None if rest.is_empty() => String::new(),
        None => format!("\\<{}\\>", regex_escape(rest)),
    };
    (count.max(1), pat)
}

/// 'comments' as (leader, a blank must follow): s1:/*,mb:*,ex:*/,://,b:#,…
fn comment_leaders(opt: &str) -> Vec<(String, bool)> {
    opt.split(',').filter_map(|part| part.split_once(':')).filter(|(_, s)| !s.is_empty()).map(|(flags, s)| (s.to_string(), flags.contains('b'))).collect()
}

/// A line that starts with one of them (after its indent).
fn is_comment(line: &str, leaders: &[(String, bool)]) -> bool {
    let t = line.trim_start();
    leaders.iter().any(|(l, blank)| t.strip_prefix(l.as_str()).is_some_and(|rest| !blank || rest.is_empty() || rest.starts_with(char::is_whitespace)))
}

fn regex_escape(s: &str) -> String {
    s.chars().flat_map(|c| if "\\.+*?()|[]{}^$".contains(c) { vec!['\\', c] } else { vec![c] }).collect()
}

impl App {
    /// 'include' and 'define' for this buffer: the option, else Vim's own for C and C++ (Neovim
    /// leaves both empty and its C file type sets them).
    fn def_patterns(&self) -> (String, String) {
        let c_like = matches!(self.docs[self.cur].syntax.as_str(), "C" | "C++" | "Objective-C" | "Objective-C++");
        let or = |v: String, d: &str| if v.is_empty() && c_like { d.to_string() } else { v };
        (or(self.setting(self.cur, "include"), r"^\s*#\s*include"), or(self.setting(self.cur, "define"), r"^\s*#\s*define"))
    }

    /// An included file by the name after the include: in 'path' ("." the file's folder, "" the
    /// current one), with 'suffixesadd' tried too.
    fn include_file(&self, name: &str, from: Option<&Path>) -> Option<PathBuf> {
        let here = from.and_then(Path::parent).map(Path::to_path_buf).unwrap_or_else(|| self.root.clone());
        let p = Path::new(name);
        let mut dirs: Vec<PathBuf> = Vec::new();
        if p.is_absolute() {
            dirs.push(PathBuf::new());
        }
        for d in self.setting(self.cur, "path").split(',') {
            dirs.push(match d {
                "." => here.clone(),
                "" => self.root.clone(),
                d => PathBuf::from(d.trim_end_matches("/**").trim_end_matches("**")),
            });
        }
        let sua = self.setting(self.cur, "suffixesadd");
        let suffixes: Vec<&str> = std::iter::once("").chain(sua.split(',').filter(|s| !s.is_empty())).collect();
        dirs.iter().flat_map(|d| suffixes.iter().map(move |s| d.join(format!("{name}{s}")))).find(|f| f.is_file())
    }

    /// The file an include line names (its <name> or "name" or the word after the match).
    fn included_name(line: &str, inc: &Regex) -> Option<String> {
        let m = inc.find(line).ok()??;
        let rest = line[m.end()..].trim_start();
        let name: String = match rest.chars().next()? {
            '<' => rest[1..].split('>').next()?.to_string(),
            '"' => rest[1..].split('"').next()?.to_string(),
            _ => rest.chars().take_while(|c| !c.is_whitespace() && !"\"'<>;,()".contains(*c)).collect(),
        };
        (!name.is_empty()).then_some(name)
    }

    /// Every line with `pat` (with `def`: only definitions of it) in lines a..=b of the buffer and
    /// the files it includes, in Vim's order: an included file where its include line is.
    pub(crate) fn def_hits(&self, a: usize, b: usize, pat: &str, def: bool, comments_too: bool) -> Result<Vec<Hit>, String> {
        let (inc, dfn) = self.def_patterns();
        if def && dfn.is_empty() {
            return Err("E388: Couldn't find definition ('define' is empty)".into());
        }
        let re = crate::ex::vim_regex(pat, false)?;
        let inc = (!inc.is_empty()).then(|| crate::ex::vim_regex(&inc, false)).transpose()?;
        let dfn = (!dfn.is_empty()).then(|| crate::ex::vim_regex(&dfn, false)).transpose()?;
        // a comment line by 'comments' (as Vim tells them), skipped without !
        let leaders = comment_leaders(&self.setting(self.cur, "comments"));
        let comment = |t: &str| is_comment(t, &leaders);
        let mut out = Vec::new();
        let mut seen: Vec<PathBuf> = Vec::new();
        let doc = &self.docs[self.cur];
        let own: Vec<String> = (a..=b.min(doc.ed.buf.len_lines().saturating_sub(1))).map(|l| doc.ed.buf.line(l).trim_end_matches('\n').to_string()).collect();
        let me = doc.abs_path();
        if let Some(p) = &me {
            seen.push(p.clone());
        }
        let mut stack: Vec<(Option<PathBuf>, std::rc::Rc<Vec<String>>, usize, usize)> = vec![(None, std::rc::Rc::new(own), 0, a)];
        while let Some((file, lines, i, base)) = stack.pop() {
            if i >= lines.len() {
                continue;
            }
            let t = &lines[i];
            // the rest of this file after this line, then (on top) what this line includes
            stack.push((file.clone(), std::rc::Rc::clone(&lines), i + 1, base));
            let found = re.is_match(t).unwrap_or(false) && (comments_too || !comment(t)) && (!def || dfn.as_ref().is_some_and(|d| d.find(t).ok().flatten().is_some_and(|m| re.find(&t[m.end()..]).ok().flatten().is_some())));
            if found {
                out.push(Hit { file: file.clone(), line: base + i, text: t.clone() });
            }
            if let Some(name) = inc.as_ref().and_then(|r| Self::included_name(t, r)) {
                let from = file.clone().or_else(|| me.clone());
                if let Some(f) = self.include_file(&name, from.as_deref()) {
                    let f = std::fs::canonicalize(&f).unwrap_or(f);
                    if !seen.contains(&f) && stack.len() < 32 {
                        seen.push(f.clone());
                        let text = std::fs::read_to_string(&f).unwrap_or_default();
                        stack.push((Some(f), std::rc::Rc::new(text.lines().map(str::to_string).collect()), 0, 0));
                    }
                }
            }
        }
        Ok(out)
    }

    /// One of these commands.
    pub(crate) fn def_command(&mut self, name: &str, arg: &str, bang: bool, range: Option<(usize, usize)>) {
        let Some(cmd) = def_name(name) else { return };
        if cmd == "checkpath" {
            return self.checkpath(bang);
        }
        let (count, mut pat) = parse_arg(arg);
        if pat.is_empty() {
            // no pattern: the word under the cursor, as [I and [D take it
            let w = self.word_at_cursor();
            if w.is_empty() {
                return self.say("E349: No identifier under cursor");
            }
            pat = format!("\\<{}\\>", regex_escape(&w));
        }
        let def = cmd.starts_with('d');
        let last = self.docs[self.cur].ed.buf.len_lines().saturating_sub(1);
        let (a, b) = range.unwrap_or((0, last));
        let hits = match self.def_hits(a, b, &pat, def, bang) {
            Ok(h) => h,
            Err(e) => return self.say(e),
        };
        if hits.is_empty() {
            return self.say(if def { "E388: Couldn't find definition" } else { "E389: Couldn't find pattern" });
        }
        let label = |h: &Hit, me: &str| format!("{}:{}", h.file.as_ref().map(|f| f.display().to_string()).unwrap_or_else(|| me.to_string()), h.line + 1);
        let me = self.docs[self.cur].name.clone();
        match cmd {
            "ilist" | "dlist" => {
                let rows = hits.iter().enumerate().map(|(i, h)| (format!("{:>3}: {}", i + 1, label(h, &me)), h.text.trim().to_string())).collect();
                self.popup = Some(Popup::Help(format!("{} · {}", if def { "definitions" } else { "matches" }, hits.len()), rows));
            }
            _ => {
                let Some(h) = hits.get(count - 1).cloned() else {
                    return self.say(format!("E387: Match is on current line · only {} found", hits.len()));
                };
                if cmd == "isearch" || cmd == "dsearch" {
                    return self.say(format!("{}: {}", label(&h, &me), h.text.trim()));
                }
                if matches!(cmd, "isplit" | "dsplit" | "psearch") {
                    self.doom_ui("split", "below");
                }
                if let Some(f) = &h.file {
                    self.open(f);
                }
                self.goto_line(h.line + 1);
            }
        }
    }

    /// :checkpath: the files the buffer includes, found or not; ! all of them, deeper too.
    fn checkpath(&mut self, all: bool) {
        let (inc, _) = self.def_patterns();
        let Ok(re) = crate::ex::vim_regex(if inc.is_empty() { "$^" } else { &inc }, false) else { return };
        let me = self.docs[self.cur].abs_path();
        let mut rows: Vec<(String, String)> = Vec::new();
        let mut seen: Vec<PathBuf> = Vec::new();
        let doc = &self.docs[self.cur];
        let mut todo: Vec<(Option<PathBuf>, Vec<String>, usize)> = vec![(me.clone(), (0..doc.ed.buf.len_lines()).map(|l| doc.ed.buf.line(l)).collect(), 0)];
        while let Some((from, lines, depth)) = todo.pop() {
            for l in &lines {
                let Some(name) = Self::included_name(l, &re) else { continue };
                let indent = "  ".repeat(depth);
                match self.include_file(&name, from.as_deref()) {
                    Some(f) => {
                        let f = std::fs::canonicalize(&f).unwrap_or(f);
                        if seen.contains(&f) {
                            if all {
                                rows.push((format!("{indent}{name}"), "(already listed)".into()));
                            }
                            continue;
                        }
                        seen.push(f.clone());
                        if all {
                            rows.push((format!("{indent}{name}"), f.display().to_string()));
                        }
                        if depth < 8 {
                            let text = std::fs::read_to_string(&f).unwrap_or_default();
                            todo.push((Some(f), text.lines().map(str::to_string).collect(), depth + 1));
                        }
                    }
                    None => rows.push((format!("{indent}{name}"), "NOT FOUND".into())),
                }
            }
        }
        if rows.is_empty() {
            return self.say(if all { "No included files" } else { "All included files were found" });
        }
        self.popup = Some(Popup::Help(if all { "included files".into() } else { "included files not found".into() }, rows));
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn names_and_arguments() {
        assert_eq!(super::def_name("il"), Some("ilist"));
        assert_eq!(super::def_name("dsp"), Some("dsplit"));
        assert_eq!(super::def_name("ds"), Some("dsearch"));
        assert_eq!(super::def_name("d"), None);
        assert_eq!(super::parse_arg("2 /fo+/"), (2, "fo+".to_string()));
        assert_eq!(super::parse_arg("foo"), (1, "\\<foo\\>".to_string()));
    }

    #[test]
    fn the_bracket_keys_use_them() {
        let mut app = crate::App::new(Vec::new());
        app.docs[app.cur].ed = zett_edit::Editor::from_str("foo\nbar\nx foo\n");
        let typed = |app: &mut crate::App, keys: &str| {
            for k in zett_edit::keys::parse(keys) {
                app.key(crate::input::key_event(&k));
            }
        };
        typed(&mut app, "[I");
        let rows = |app: &crate::App| match &app.popup { Some(crate::Popup::Help(_, r)) => r.len(), _ => 0 };
        assert_eq!(rows(&app), 2, "[I: every line with the word");
        app.popup = None;
        typed(&mut app, "]I");
        assert_eq!(rows(&app), 1, "]I: from the line below");
        app.popup = None;
        typed(&mut app, "]<C-i>");
        assert_eq!(app.docs[app.cur].ed.cursor().line, 2, "] CTRL-I jumps to it");
    }

    #[test]
    fn through_the_included_files() {
        let dir = std::env::temp_dir().join(format!("zett-defs-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.h"), "#define SIZE 4\nint size_of(void);\n").unwrap();
        let main = dir.join("main.c");
        std::fs::write(&main, "#include \"a.h\"\n#include <missing.h>\n// SIZE in a comment\nint x = SIZE;\n").unwrap();
        let mut app = crate::App::new(vec![main.clone()]);
        let hits = app.def_hits(0, 3, "\\<SIZE\\>", false, false).unwrap();
        let at: Vec<(bool, usize)> = hits.iter().map(|h| (h.file.is_some(), h.line)).collect();
        assert_eq!(at, vec![(true, 0), (false, 3)], "the header's line where it is included, the comment skipped");
        assert_eq!(app.def_hits(0, 3, "\\<SIZE\\>", false, true).unwrap().len(), 3, "! takes comments too");
        let defs = app.def_hits(0, 3, "\\<SIZE\\>", true, false).unwrap();
        assert_eq!(defs.len(), 1, "only the #define");
        app.command("checkpath");
        let rows = match &app.popup { Some(crate::Popup::Help(_, r)) => r.clone(), _ => Vec::new() };
        assert!(rows.iter().any(|(n, s)| n.contains("missing.h") && s == "NOT FOUND"), "{rows:?}");
        app.command("djump SIZE");
        assert!(app.docs[app.cur].name.ends_with("a.h"), "{}", app.docs[app.cur].name);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
