//! Vimscript on the : line and in .vim files: the language's commands go to zett-script's
//! vimscript.lua (translated to Lua); a block typed line by line (:function … :endfunction) is kept
//! until it closes.

use super::*;

/// The language's commands by Vim's full names and shortest forms.
const NAMES: &[(&str, &str)] = &[
    ("let", "let"), ("unlet", "unl"), ("const", "cons"), ("if", "if"), ("elseif", "elsei"), ("else", "el"), ("endif", "en"),
    ("while", "wh"), ("endwhile", "endw"), ("for", "for"), ("endfor", "endfo"), ("break", "brea"), ("continue", "con"),
    ("function", "fu"), ("endfunction", "endf"), ("return", "retu"), ("call", "cal"), ("try", "try"), ("catch", "cat"),
    ("finally", "fina"), ("endtry", "endt"), ("throw", "th"), ("echohl", "echoh"), ("finish", "fini"), ("lockvar", "lockv"),
    ("unlockvar", "unlo"), ("delfunction", "delf"), ("defer", "defe"),
];

/// :mkvimrc and :mkexrc (zett's own here, not the language's)
const OWN: &[(&str, &str)] = &[("mkvimrc", "mkv"), ("mkexrc", "mk")];

/// A typed command word as one of them (its full name).
pub(crate) fn name(word: &str) -> Option<&'static str> {
    let word = word.trim_end_matches('!');
    NAMES.iter().find(|(full, short)| full.starts_with(word) && word.len() >= short.len()).map(|(f, _)| *f)
}

/// How a line changes the depth of open blocks: +1 opens one, -1 closes one.
fn depth_change(line: &str) -> i32 {
    let word: String = line.trim_start().trim_start_matches(':').chars().take_while(|c| c.is_ascii_alphabetic()).collect();
    match name(&word) {
        Some("if" | "while" | "for" | "try") => 1,
        // :function Name(…) defines one; :function alone (or with a name only) lists them
        Some("function") if line.contains('(') => 1,
        Some("endif" | "endwhile" | "endfor" | "endtry" | "endfunction") => -1,
        _ => 0,
    }
}

impl App {
    /// A : line that is Vimscript's (or inside a block being typed): run, or kept for the block's
    /// end. False when the line is for the other commands.
    pub(crate) fn vimscript_line(&mut self, line: &str) -> bool {
        let word: String = line.trim_start().chars().take_while(|c| c.is_ascii_alphabetic()).collect();
        if self.vim_block.is_empty() {
            if let Some(&(full, _)) = OWN.iter().find(|(full, short)| full.starts_with(word.as_str()) && word.len() >= short.len()) {
                let rest = line.trim_start()[word.len()..].to_string();
                let bang = rest.starts_with('!');
                self.mkvimrc(rest.trim_start_matches('!').trim(), bang, full == "mkvimrc");
                return true;
            }
        }
        if self.vim_block.is_empty() && name(&word).is_none() {
            return false;
        }
        self.vim_block.push(line.to_string());
        let depth: i32 = self.vim_block.iter().map(|l| depth_change(l)).sum();
        if depth > 0 {
            self.say(format!("{} … (the block goes on until its :end)", self.vim_block[0].trim()));
            return true;
        }
        let src = std::mem::take(&mut self.vim_block).join("\n");
        self.vimscript_run(&src, None);
        true
    }

    /// :mkvimrc / :mkexrc [file]: the options that differ from their defaults, as :set lines
    /// (.vimrc / .exrc in the project when no file is named; ! writes over one that is there).
    fn mkvimrc(&mut self, file: &str, bang: bool, vim: bool) {
        let file = if file.is_empty() { if vim { ".vimrc" } else { ".exrc" } } else { file };
        let path = if Path::new(file).is_absolute() { PathBuf::from(file) } else { self.root.join(file) };
        if path.exists() && !bang {
            return self.say(format!("E189: \"{file}\" exists (add ! to override)"));
        }
        let mut out = Vec::new();
        if vim {
            out.push("version 6.0".to_string());
        }
        for s in crate::settings::all() {
            let v = self.setting(self.cur, s.name);
            // what the file or the terminal decides (its encoding, its type, 24-bit colour) is not a setting to keep
            const OWN_FILE: &[&str] = &["modified", "filetype", "syntax", "fileencoding", "fileformat", "bomb", "endofline", "endoffile", "termguicolors", "readonly"];
            if v == s.default || OWN_FILE.contains(&s.name) {
                continue;
            }
            out.push(match s.kind {
                crate::settings::Kind::Bool => format!("set {}{}", if matches!(v.as_str(), "true" | "1" | "on") { "" } else { "no" }, s.name),
                _ => format!("set {}={}", s.name, v.replace('\\', "\\\\").replace(' ', "\\ ")),
            });
        }
        let n = out.len() - usize::from(vim);
        match std::fs::write(&path, out.join("\n") + "\n") {
            Ok(()) => self.say(format!("\"{file}\" written: {n} settings")),
            Err(e) => self.say(format!("E190: Cannot open \"{file}\" for writing: {e}")),
        }
    }

    /// Runs Vimscript (a : line, a block, a .vim file's text); `name` keeps its s: variables apart.
    pub(crate) fn vimscript_run(&mut self, src: &str, name: Option<&str>) {
        let code = format!("vim._vimscript.run({src:?}, {})", name.map(|n| format!("{n:?}")).unwrap_or_else(|| "nil".into()));
        match self.with_lua(|e| e.eval(&code)) {
            Ok((_, fx)) => self.after_lua(fx),
            Err(e) => self.say(e),
        }
    }

    /// :echo with a Vimscript expression ("a" . b, &ts, g:x), when it is not Lua.
    pub(crate) fn vimscript_echo(&mut self, expr: &str) -> bool {
        match self.with_lua(|e| e.eval(&format!("vim._vimscript.echo_text({expr:?})"))) {
            Ok((out, fx)) => {
                self.after_lua(fx);
                self.say(out);
                true
            }
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_and_blocks() {
        assert_eq!(name("fu"), Some("function"));
        assert_eq!(name("endf"), Some("endfunction"));
        assert_eq!(name("en"), Some("endif"));
        assert_eq!(name("endfo"), Some("endfor"));
        assert_eq!(name("con"), Some("continue"));
        assert_eq!(name("cons"), Some("const"));
        assert_eq!(name("e"), None);
        assert_eq!(depth_change("function! Foo(a)"), 1);
        assert_eq!(depth_change("function"), 0);
        assert_eq!(depth_change("  endfunction"), -1);
        assert_eq!(depth_change("if x | echo 1 | endif"), 1, "counted by its first word: the bar line runs as one block");
    }
}

impl App {
    /// foldmethod=expr: the levels from foldexpr (vimscript.lua fold_levels) when the text or the
    /// expression changed, for the editor to make its folds from.
    pub(crate) fn expr_folds(&mut self) {
        let i = self.cur;
        if self.setting(i, "foldmethod") != "expr" {
            return;
        }
        let expr = self.setting(i, "foldexpr");
        let key = format!("{}|{expr}", self.docs[i].ed.buf.rev());
        if self.fold_expr_key.get(&i) == Some(&key) {
            return;
        }
        self.fold_expr_key.insert(i, key);
        let n = self.docs[i].ed.buf.len_lines();
        let code = format!("vim._vimscript.fold_levels({expr:?}, {n})");
        let Ok((out, _)) = self.with_lua(|e| e.eval(&code)) else { return };
        let levels = out
            .split(',')
            .map(|p| match p.split_once(':') {
                Some((l, s)) => (l.parse().unwrap_or(0), s.parse().ok()),
                None => (p.parse().unwrap_or(0), None),
            })
            .collect();
        self.docs[i].ed.set_fold_levels(levels);
    }
}

impl App {
    /// foldtext for buffer `di`'s closed folds (line → text), evaluated once per text, expression
    /// and fold; nothing when foldtext is empty (the line itself is drawn, with …).
    pub(crate) fn fold_texts(&mut self, di: usize) {
        let expr = self.setting(di, "foldtext");
        if expr.is_empty() {
            self.fold_text.remove(&di);
            return;
        }
        let d = &self.docs[di];
        let folds: Vec<(usize, usize, usize)> = (0..d.ed.buf.len_lines()).filter_map(|l| d.ed.fold_end(l).map(|e| (l, e, d.ed.fold_mark(l).0.max(1)))).collect();
        let key = format!("{}|{expr}|{folds:?}", d.ed.buf.rev());
        if self.fold_text.get(&di).is_some_and(|(k, _)| *k == key) {
            return;
        }
        let mut texts = std::collections::HashMap::new();
        for (s, e, lv) in folds {
            let code = format!("vim._vimscript.fold_text({expr:?}, {}, {}, {lv})", s + 1, e + 1);
            if let Ok((out, _)) = self.with_lua(|en| en.eval(&code)) {
                texts.insert(s, out);
            }
        }
        self.fold_text.insert(di, (key, texts));
    }
}

impl App {
    /// gq with formatexpr: the expression run with v:lnum (the first line), v:count (how many)
    /// and v:char empty, its changes kept; non-zero: gq's own formatting after all (as Vim).
    pub(crate) fn format_expr(&mut self, l1: usize, l2: usize) {
        let expr = self.setting(self.cur, "formatexpr");
        let code = format!("vim.v.lnum = {}; vim.v.count = {}; vim.v.char = ''; return vim._vimscript.eval_str({expr:?})", l1 + 1, l2 - l1 + 1);
        let before = l1;
        match self.with_lua(|e| e.eval(&code)) {
            Ok((out, fx)) => {
                self.after_lua(fx);
                if out.trim() != "0" && !out.trim().is_empty() {
                    self.docs[self.cur].ed.format_lines(l1, l2);
                    self.docs[self.cur].touched(before);
                }
            }
            Err(e) => self.say(format!("formatexpr: {e}")),
        }
    }
}

impl App {
    /// indentexpr for lines `l1..=l2`: each line's indent as the expression gives it with v:lnum
    /// set (-1: kept as it is), as Vim does for a new line and for =.
    pub(crate) fn indent_expr(&mut self, l1: usize, l2: usize) {
        let expr = self.setting(self.cur, "indentexpr");
        for ln in l1..=l2 {
            let code = format!("vim.v.lnum = {}; return vim._vimscript.eval_str({expr:?})", ln + 1);
            match self.with_lua(|e| e.eval(&code)) {
                Ok((out, fx)) => {
                    self.after_lua(fx);
                    if let Ok(n) = out.trim().parse::<i64>() {
                        if n >= 0 {
                            self.docs[self.cur].ed.set_indent(ln, n as usize);
                        }
                    }
                }
                Err(e) => return self.say(format!("indentexpr: {e}")),
            }
        }
        self.docs[self.cur].touched(l1);
    }
}
