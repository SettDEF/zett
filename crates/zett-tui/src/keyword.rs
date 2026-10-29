//! Vim's keyword completion (i_CTRL-N, i_CTRL-P): the words starting with what is typed, from the
//! places `complete` names (this buffer nearest first, the other buffers, `dictionary` files), in
//! the popup; `ignorecase` and `infercase` decide the case, `completeopt` how the menu behaves.

use super::*;
use crate::lsp::Comp;

fn ident(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// The words of `text` in order, each with where it starts (a char index).
fn words(text: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut at = 0;
    for (i, c) in text.chars().enumerate() {
        if ident(c) {
            if cur.is_empty() {
                at = i;
            }
            cur.push(c);
        } else if !cur.is_empty() {
            out.push((at, std::mem::take(&mut cur)));
        }
    }
    if !cur.is_empty() {
        out.push((at, cur));
    }
    out
}

/// `word` as completed after `typed` with infercase: the case typed carries on (all capitals
/// stays capitals, all small stays small), the typed part is kept as typed.
pub(crate) fn infer_case(typed: &str, word: &str) -> String {
    let n = typed.chars().count();
    let rest: String = word.chars().skip(n).collect();
    let letters: Vec<char> = typed.chars().filter(|c| c.is_alphabetic()).collect();
    let rest = if !letters.is_empty() && letters.iter().all(|c| c.is_uppercase()) {
        rest.to_uppercase()
    } else if !letters.is_empty() && letters.iter().all(|c| c.is_lowercase()) {
        rest.to_lowercase()
    } else {
        rest
    };
    format!("{typed}{rest}")
}

/// The candidates for `prefix`: this buffer's words around `pos` (after it first for CTRL-N,
/// before it nearest first for CTRL-P), then `others` (name, text) in order; each once, never the
/// word being typed itself.
pub fn candidates(prefix: &str, here: &str, pos: usize, back: bool, others: &[(String, String)], ignorecase: bool) -> Vec<(String, String)> {
    let fold = |s: &str| if ignorecase { s.to_lowercase() } else { s.to_string() };
    let p = fold(prefix);
    let fits = |w: &str| w.chars().count() > prefix.chars().count() && fold(w).starts_with(&p);
    let all = words(here);
    // the word at the cursor is the one being typed: not offered
    let (before, after): (Vec<_>, Vec<_>) = all.into_iter().filter(|(at, w)| !(*at <= pos && pos <= at + w.chars().count())).partition(|(at, _)| *at < pos);
    let mut order: Vec<String> = after.into_iter().chain(before).map(|(_, w)| w).collect();
    if back {
        order.reverse();
    }
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for w in order {
        if fits(&w) && seen.insert(w.clone()) {
            out.push((w, String::new()));
        }
    }
    for (name, text) in others {
        for (_, w) in words(text) {
            if fits(&w) && seen.insert(w.clone()) {
                out.push((w, name.clone()));
            }
        }
        if out.len() > 2000 {
            break;
        }
    }
    out
}

impl App {
    /// The word before the cursor: (its line, where it starts, the word) — `fname` takes file
    /// name characters (/ . - ~) as well.
    fn comp_word(&self, fname: bool) -> (usize, usize, String) {
        let c = self.docs[self.cur].ed.cursor();
        let line: Vec<char> = self.docs[self.cur].ed.buf.line(c.line).chars().collect();
        let col = c.col.min(line.len());
        let mut start = col;
        while start > 0 && (ident(line[start - 1]) || (fname && "/.-~+,#$%".contains(line[start - 1]))) {
            start -= 1;
        }
        (c.line, start, line[start..col].iter().collect())
    }

    /// CTRL-N / CTRL-P in insert mode with no menu open: complete the word before the cursor
    /// from the places `complete` names (`only`: these instead, CTRL-X CTRL-N . and CTRL-X CTRL-K k).
    pub(crate) fn keyword_complete(&mut self, back: bool) {
        self.keyword_complete_in(back, None)
    }

    fn keyword_complete_in(&mut self, back: bool, only: Option<&str>) {
        let i = self.cur;
        let (line, start, prefix) = self.comp_word(false);
        let ic = self.flag(i, "ignorecase");
        let infer = ic && self.flag(i, "infercase");
        let sources = only.map(str::to_string).unwrap_or_else(|| self.setting(i, "complete"));
        let has = |f: &str| sources.split(',').any(|s| s.trim() == f || (f == "k" && s.trim().starts_with('k')));
        let here = self.docs[i].ed.text();
        let col = self.docs[i].ed.cursor().col;
        let pos = self.docs[i].ed.buf.to_char(zett_core::Pos { line, col });
        let mut others: Vec<(String, String)> = Vec::new();
        if has("w") || has("b") || has("u") {
            for (j, d) in self.docs.iter().enumerate() {
                if j != i {
                    let name = Path::new(&d.name).file_name().map_or(d.name.clone(), |n| n.to_string_lossy().to_string());
                    others.push((name, d.ed.text()));
                }
            }
        }
        if has("k") {
            for f in self.setting(i, "dictionary").split(',').map(str::trim).filter(|f| !f.is_empty()) {
                if let Ok(t) = std::fs::read_to_string(f) {
                    others.push(("dict".into(), t));
                }
            }
        }
        let here = if has(".") { here } else { String::new() };
        let found = candidates(&prefix, &here, pos, back, &others, ic);
        let items = found.into_iter().map(|(w, from)| (if infer { infer_case(&prefix, &w) } else { w.clone() }, w, from)).collect();
        self.comp_open(line, start, prefix, items, if only == Some("k") { "Dictionary completion (^K^N^P)" } else { "Keyword completion (^N^P)" });
    }

    /// The menu over `start..cursor` on `line` with `items` (what goes in, its label, where it
    /// is from), as completeopt says: one alone goes straight in (unless menuone), the first goes
    /// in at once (unless noinsert or noselect). "Pattern not found" when there are none.
    fn comp_open(&mut self, line: usize, start: usize, typed: String, items: Vec<(String, String, String)>, what: &str) {
        if items.is_empty() {
            return self.say(format!("-- {what} Pattern not found"));
        }
        let opts = self.setting(self.cur, "completeopt");
        let opt = |f: &str| opts.split(',').any(|s| s.trim() == f);
        let items: Vec<zett_lsp::Item> = items.into_iter().enumerate().map(|(n, (insert, label, detail))| zett_lsp::Item { insert, label, detail, kind: 1, sort: format!("{n:06}"), ..Default::default() }).collect();
        if items.len() == 1 && !opt("menuone") {
            self.comp = Some(Comp { shown: vec![0], items, sel: 0, line, start, keyword: Some(typed), incomplete: false, resolving: None, ycm: false });
            return self.comp_accept();
        }
        let shown = (0..items.len()).collect();
        self.comp = Some(Comp { items, shown, sel: 0, line, start, keyword: Some(typed), incomplete: false, resolving: None, ycm: false });
        if !opt("noinsert") && !opt("noselect") {
            self.comp_put();
        }
    }

    /// CTRL-X then `k` in insert mode: Vim's completion modes.
    pub(crate) fn ctrl_x(&mut self, k: char) {
        match k {
            // this buffer's words only
            'n' | 'p' => self.keyword_complete_in(k == 'p', Some(".")),
            'k' => self.keyword_complete_in(false, Some("k")),
            't' => self.thesaurus_complete(),
            'u' => self.func_complete("completefunc", "User defined completion (^U^N^P)"),
            // omnifunc, else the language server's completion (zett's own; vim.lsp.omnifunc too)
            'o' => {
                let f = self.setting(self.cur, "omnifunc");
                if f.is_empty() || f.contains("vim.lsp.omnifunc") {
                    self.comp_due = Some(Instant::now());
                } else {
                    self.func_complete("omnifunc", "Omni completion (^O^N^P)");
                }
            }
            'l' => self.line_complete(),
            'f' => self.file_complete(),
            _ => {}
        }
    }

    /// completefunc / omnifunc (`opt`): asked where the word starts, then for its matches.
    fn func_complete(&mut self, opt: &str, what: &str) {
        let name = self.setting(self.cur, opt);
        if name.is_empty() {
            return self.say(format!("E764: Option '{opt}' is not set"));
        }
        let c = self.docs[self.cur].ed.cursor();
        let text = self.docs[self.cur].ed.buf.line(c.line);
        let call = |e: &mut Engine, find: u8, base: &str| e.eval(&format!("return vim._vimscript.complete_call({name:?}, {find}, {base:?})")).map(|r| r.0).unwrap_or_default();
        let start_byte: i64 = self.with_lua(|e| call(e, 1, "")).trim().parse().unwrap_or(-3);
        if start_byte < 0 {
            return;
        }
        // the start is in bytes; the menu wants characters
        let start = text.char_indices().take_while(|(b, _)| (*b as i64) < start_byte).count().min(c.col);
        let base: String = text.chars().skip(start).take(c.col - start).collect();
        let out = self.with_lua(|e| call(e, 0, &base));
        let items = out.split('\u{1e}').filter(|l| !l.is_empty()).map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            let word = f.first().copied().unwrap_or("").to_string();
            let label = f.get(1).filter(|a| !a.is_empty()).map_or(word.clone(), |a| a.to_string());
            let menu = [f.get(3).copied().unwrap_or(""), f.get(2).copied().unwrap_or("")].iter().filter(|s| !s.is_empty()).copied().collect::<Vec<_>>().join(" ");
            (word, label, menu)
        }).collect();
        self.comp_open(c.line, start, base, items, what);
    }

    /// CTRL-X CTRL-T: words that mean the same, from the thesaurus files (each line a group) or
    /// thesaurusfunc.
    fn thesaurus_complete(&mut self) {
        if !self.setting(self.cur, "thesaurusfunc").is_empty() {
            return self.func_complete("thesaurusfunc", "Thesaurus completion (^T^N^P)");
        }
        let (line, start, word) = self.comp_word(false);
        let lw = word.to_lowercase();
        let mut items: Vec<(String, String, String)> = Vec::new();
        for f in self.setting(self.cur, "thesaurus").split(',').map(str::trim).filter(|f| !f.is_empty()) {
            let Ok(t) = std::fs::read_to_string(f) else { continue };
            for l in t.lines() {
                let ws: Vec<&str> = l.split(|c: char| !ident(c)).filter(|w| !w.is_empty()).collect();
                if ws.iter().any(|w| w.to_lowercase() == lw) {
                    for w in ws.iter().filter(|w| w.to_lowercase() != lw) {
                        if !items.iter().any(|i| i.0 == *w) {
                            items.push((w.to_string(), w.to_string(), "thesaurus".into()));
                        }
                    }
                }
            }
        }
        self.comp_open(line, start, word, items, "Thesaurus completion (^T^N^P)");
    }

    /// CTRL-X CTRL-L: whole lines (of every buffer) that start as this one does after its indent.
    fn line_complete(&mut self) {
        let c = self.docs[self.cur].ed.cursor();
        let text: String = self.docs[self.cur].ed.buf.line(c.line).chars().take(c.col).collect();
        let indent = text.chars().take_while(|ch| ch.is_whitespace()).count();
        let typed: String = text.chars().skip(indent).collect();
        let mut items: Vec<(String, String, String)> = Vec::new();
        for (j, d) in self.docs.iter().enumerate() {
            for (n, l) in d.ed.text().lines().enumerate() {
                let t = l.trim_start();
                if (j == self.cur && n == c.line) || t.len() <= typed.len() || !t.starts_with(typed.as_str()) {
                    continue;
                }
                if !items.iter().any(|i| i.0 == t) {
                    items.push((t.to_string(), t.to_string(), String::new()));
                }
            }
        }
        self.comp_open(c.line, indent, typed, items, "Whole line completion (^L^N^P)");
    }

    /// CTRL-X CTRL-F: file names, from the folder the typed path names (the working one else).
    fn file_complete(&mut self) {
        let (line, start, typed) = self.comp_word(true);
        let (dir, part) = match typed.rfind('/') {
            Some(p) => (typed[..=p].to_string(), typed[p + 1..].to_string()),
            None => (String::new(), typed.clone()),
        };
        let at = if dir.starts_with('~') { dir.replacen('~', &std::env::var("HOME").unwrap_or_default(), 1) } else if dir.is_empty() { ".".into() } else { dir.clone() };
        let mut names: Vec<String> = std::fs::read_dir(&at).map(|r| r.flatten().map(|e| {
            let n = e.file_name().to_string_lossy().to_string();
            if e.path().is_dir() { format!("{n}/") } else { n }
        }).filter(|n| n.starts_with(&part) && (!n.starts_with('.') || part.starts_with('.'))).collect()).unwrap_or_default();
        names.sort();
        // the whole typed path is replaced: the folder part stays as it was typed
        let items = names.into_iter().map(|n| (format!("{dir}{n}"), n, String::new())).collect();
        self.comp_open(line, start, typed, items, "File name completion (^F^N^P)");
    }

    /// Keyword completion's choice put in place of what is there (cycling with CTRL-N / CTRL-P).
    pub(crate) fn comp_put(&mut self) {
        let Some(comp) = &self.comp else { return };
        let Some(&i) = comp.shown.get(comp.sel) else { return };
        let insert = comp.items[i].insert.clone();
        let typed = self.docs[self.cur].ed.cursor().col.saturating_sub(comp.start);
        for _ in 0..typed {
            self.edit_key(Key::code(Code::Backspace));
        }
        for ch in insert.chars() {
            self.edit_key(Key::ch(ch));
        }
    }

    /// CTRL-E: the menu closes and what was typed comes back.
    pub(crate) fn comp_cancel(&mut self) {
        let Some(comp) = self.comp.take() else { return };
        let Some(typed) = comp.keyword else { return };
        let n = self.docs[self.cur].ed.cursor().col.saturating_sub(comp.start);
        for _ in 0..n {
            self.edit_key(Key::code(Code::Backspace));
        }
        for ch in typed.chars() {
            self.edit_key(Key::ch(ch));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_first_each_way_and_never_the_word_typed() {
        let text = "alpha alpine al beta albatross";
        // typing "al" at the third word (index 13)
        let fwd: Vec<String> = candidates("al", text, 13, false, &[], false).into_iter().map(|c| c.0).collect();
        assert_eq!(fwd, ["albatross", "alpha", "alpine"]);
        let back: Vec<String> = candidates("al", text, 13, true, &[], false).into_iter().map(|c| c.0).collect();
        assert_eq!(back, ["alpine", "alpha", "albatross"]);
        // other buffers after this one, each word once
        let others = vec![("b.txt".to_string(), "alder alpha".to_string())];
        let all = candidates("al", text, 13, false, &others, false);
        assert_eq!(all.last().unwrap(), &("alder".to_string(), "b.txt".to_string()));
    }

    #[test]
    fn ignorecase_and_infercase() {
        assert!(candidates("AL", "Alpha", 99, false, &[], false).is_empty());
        assert_eq!(candidates("AL", "Alpha", 99, false, &[], true)[0].0, "Alpha");
        assert_eq!(infer_case("AL", "Alpha"), "ALPHA");
        assert_eq!(infer_case("al", "ALPHA"), "alpha");
        assert_eq!(infer_case("Al", "aLPHA"), "AlPHA");
    }
}

#[cfg(test)]
mod ctrl_x_tests {
    #[test]
    fn whole_lines_and_file_names() {
        let mut app = crate::App::new(Vec::new());
        app.docs[app.cur].ed = zett_edit::Editor::from_str("foo bar\n  foo baz\nother\n");
        app.docs[app.cur].ed.feed("Gofoo");
        app.ctrl_x('l');
        let c = app.comp.as_ref().expect("a menu of lines");
        let labels: Vec<&str> = c.items.iter().map(|i| i.label.as_str()).collect();
        assert_eq!(labels, ["foo bar", "foo baz"], "lines starting as this one does (after their indent)");
        assert_eq!(app.docs[app.cur].ed.buf.line(3).trim_end(), "foo bar", "the first goes in at once");
    }
}

/// A language's keywords and common built-ins, by file extension (completion with no language server).
pub fn language_keywords(ext: &str) -> &'static [&'static str] {
    match ext {
        "rs" => &["as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where", "while", "Option", "Some", "None", "Result", "Ok", "Err", "String", "Vec", "Box", "println!", "format!", "vec!", "assert_eq!", "impl Default"],
        "js" | "mjs" | "cjs" | "jsx" | "ts" | "tsx" | "qml" => &["async", "await", "break", "case", "catch", "class", "const", "continue", "default", "delete", "do", "else", "export", "extends", "false", "finally", "for", "function", "if", "import", "in", "instanceof", "let", "new", "null", "return", "static", "super", "switch", "this", "throw", "true", "try", "typeof", "undefined", "var", "void", "while", "yield", "interface", "type", "enum", "readonly", "property", "signal", "console", "Promise", "Object", "Array", "JSON", "Math"],
        "py" => &["False", "None", "True", "and", "as", "assert", "async", "await", "break", "class", "continue", "def", "del", "elif", "else", "except", "finally", "for", "from", "global", "if", "import", "in", "is", "lambda", "nonlocal", "not", "or", "pass", "raise", "return", "try", "while", "with", "yield", "print", "range", "len", "self", "__init__"],
        "lua" => &["and", "break", "do", "else", "elseif", "end", "false", "for", "function", "goto", "if", "in", "local", "nil", "not", "or", "repeat", "return", "then", "true", "until", "while", "require", "pairs", "ipairs", "table", "string", "print", "vim"],
        "c" | "h" | "cpp" | "cc" | "hpp" | "cxx" => &["auto", "break", "case", "char", "const", "continue", "default", "do", "double", "else", "enum", "extern", "float", "for", "goto", "if", "inline", "int", "long", "register", "return", "short", "signed", "sizeof", "static", "struct", "switch", "typedef", "union", "unsigned", "void", "volatile", "while", "bool", "true", "false", "nullptr", "class", "namespace", "template", "typename", "public", "private", "protected", "virtual", "include", "define", "printf", "malloc", "free"],
        "go" => &["break", "case", "chan", "const", "continue", "default", "defer", "else", "fallthrough", "for", "func", "go", "goto", "if", "import", "interface", "map", "package", "range", "return", "select", "struct", "switch", "type", "var", "nil", "true", "false", "string", "error", "make", "append"],
        "sh" | "bash" | "zsh" => &["if", "then", "else", "elif", "fi", "for", "while", "until", "do", "done", "case", "esac", "function", "return", "local", "export", "readonly", "echo", "printf", "exit", "source", "shift"],
        "java" | "kt" | "cs" => &["abstract", "boolean", "break", "case", "catch", "class", "const", "continue", "default", "do", "double", "else", "enum", "extends", "final", "finally", "float", "for", "if", "implements", "import", "instanceof", "int", "interface", "long", "new", "null", "package", "private", "protected", "public", "return", "static", "super", "switch", "this", "throw", "throws", "try", "void", "while", "fun", "val", "var", "namespace", "using", "string"],
        "php" => &["function", "echo", "class", "public", "private", "protected", "return", "if", "else", "elseif", "foreach", "as", "while", "for", "new", "null", "true", "false", "array", "namespace", "use"],
        "rb" => &["def", "end", "class", "module", "if", "elsif", "else", "unless", "while", "do", "return", "yield", "nil", "true", "false", "self", "require", "puts"],
        _ => &[],
    }
}
