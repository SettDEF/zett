//! Neovim's options the editor itself acts on, set by the window from :set (set_opt): indenting,
//! Backspace and Tab stops, joining, wrapping while typing, word characters, number formats, where
//! motions land, what counts as a change worth reporting. EDITOR_OPTIONS lists them, so the
//! settings page marks exactly these as working.

use super::*;

/// The options this editor acts on (settings.rs marks them ✓).
pub const EDITOR_OPTIONS: &[&str] = &[
    "autoindent", "smartindent", "cindent", "copyindent", "preserveindent", "backspace", "softtabstop", "smarttab",
    "shiftround", "joinspaces", "startofline", "whichwrap", "tildeop", "nrformats", "iskeyword", "matchpairs",
    "selection", "report", "modifiable", "textwidth", "wrapmargin", "formatoptions", "quoteescape", "paragraphs",
    "sections", "tabstop", "expandtab", "shiftwidth", "wrapscan", "ignorecase", "smartcase", "wrap", "scroll", "clipboard",
    "foldmethod", "foldenable", "foldlevel", "foldlevelstart", "foldmarker", "foldminlines", "foldnestmax", "foldignore",
    "foldopen", "foldclose", "spelllang", "spellfile", "spellsuggest", "spelloptions", "equalprg", "formatprg", "incsearch",
];

impl Editor {
    /// An option's value, as :set gave it (the window passes them all on, defaults included).
    pub fn set_opt(&mut self, name: &str, value: &str) {
        if name == "iskeyword" {
            self.vim.kw = parse_iskeyword(value);
        }
        // the spell checker is shared: it takes the languages and the word file from here
        match name {
            "spelllang" => crate::spell::set_lang(value),
            "spellfile" => crate::spell::set_file(value),
            _ => {}
        }
        self.vim.opts.insert(name.to_string(), value.to_string());
    }

    pub(crate) fn opt(&self, name: &str) -> Option<&str> {
        self.vim.opts.get(name).map(String::as_str)
    }
    pub(crate) fn opt_on(&self, name: &str, default: bool) -> bool {
        self.opt(name).map_or(default, |v| matches!(v, "true" | "1" | "on" | "yes"))
    }
    pub(crate) fn opt_num(&self, name: &str, default: i64) -> i64 {
        self.opt(name).and_then(|v| v.parse().ok()).unwrap_or(default)
    }
    /// A comma list option has this flag (`backspace` has `eol`, `nrformats` has `hex`).
    pub(crate) fn opt_has(&self, name: &str, flag: &str, default: &str) -> bool {
        self.opt(name).unwrap_or(default).split(',').any(|f| f.trim() == flag)
    }
    /// A flag-letter option has this letter (`formatoptions` has `t`, `whichwrap` has `h`).
    pub(crate) fn opt_letter(&self, name: &str, c: char, default: &str) -> bool {
        self.opt(name).unwrap_or(default).contains(c)
    }

    /// The character class for words (w b e, iw, *): 0 blank, 2 a keyword character, 1 the rest.
    pub(crate) fn kw(&self, c: char) -> u8 {
        if c.is_whitespace() {
            return 0;
        }
        let word = match &self.vim.kw {
            Some(t) if (c as u32) < 256 => t[c as usize],
            _ => c.is_alphanumeric() || c == '_',
        };
        if word { 2 } else { 1 }
    }

    pub(crate) fn tabstop(&self) -> usize {
        self.opt_num("tabstop", 8).max(1) as usize
    }
    pub(crate) fn shiftwidth(&self) -> usize {
        match self.opt_num("shiftwidth", 0) {
            n if n > 0 => n as usize,
            _ => self.soft_tab.unwrap_or_else(|| self.tabstop()),
        }
    }

    /// An indent `width` columns wide: spaces with expandtab, else tabs and spaces.
    pub(crate) fn make_indent(&self, width: usize) -> String {
        if self.soft_tab.is_some() || !self.opt_on("expandtab", true) {
            if self.soft_tab.is_none() {
                let ts = self.tabstop();
                return "\t".repeat(width / ts) + &" ".repeat(width % ts);
            }
        }
        " ".repeat(width)
    }

    /// An indent's width in columns.
    pub(crate) fn indent_width(&self, s: &str) -> usize {
        let ts = self.tabstop();
        s.chars().fold(0, |w, c| if c == '\t' { (w / ts + 1) * ts } else { w + 1 })
    }

    /// The indent a new line next to `line` gets (o, O, Enter): none without autoindent; one step
    /// more after an opening brace and one less before a closing one with smartindent or cindent;
    /// copyindent keeps the characters, else they are made anew.
    pub(crate) fn next_indent(&self, line: usize, below: bool, upto: Option<usize>) -> String {
        if !self.opt_on("autoindent", true) && !self.opt_on("smartindent", false) && !self.opt_on("cindent", false) {
            return String::new();
        }
        let base = self.indent_of(line);
        let base: String = match upto {
            Some(n) => base.chars().take(n).collect(),
            None => base,
        };
        let smart = self.opt_on("smartindent", false) || self.opt_on("cindent", false);
        let mut width = self.indent_width(&base);
        if smart && below {
            let text = self.buf.line(line);
            let t = text.trim_end();
            if t.ends_with('{') || t.ends_with('(') || t.ends_with('[') || t.ends_with(':') && self.opt_on("cindent", false) {
                width += self.shiftwidth();
            }
        }
        if self.opt_on("copyindent", false) && width == self.indent_width(&base) {
            return base;
        }
        self.make_indent(width)
    }

    /// Where a line motion lands (G gg H M L CTRL-D …): the first non-blank with startofline, else
    /// the column kept (Neovim's default).
    pub(crate) fn sol_target(&self, line: usize) -> usize {
        if self.opt_on("startofline", false) {
            return self.first_non_blank(line);
        }
        let (s, len) = (self.line_start(line), self.buf.line_len(line));
        let col = if self.want_col == usize::MAX { len.saturating_sub(1) } else { self.want_col.min(len.saturating_sub(1)) };
        s + col
    }

    /// Backspace in insert mode: not past where typing began or over a line break unless
    /// backspace allows it; with softtabstop, a run of blanks to the previous stop goes at once.
    pub(crate) fn vim_backspace(&mut self, insert_start: usize) {
        if self.cursor == 0 {
            return;
        }
        let line = self.cur_line();
        let at_start = self.cursor == self.line_start(line);
        if at_start && !self.opt_has("backspace", "eol", "indent,eol,start") {
            return;
        }
        if self.cursor <= insert_start && !at_start && !self.opt_has("backspace", "start", "indent,eol,start") {
            return;
        }
        let sts = match self.opt_num("softtabstop", 0) {
            n if n < 0 => self.shiftwidth(),
            n => n as usize,
        };
        let col = self.cursor().col;
        if sts > 0 && col > 0 && self.char(self.cursor - 1) == Some(' ') {
            // back to the previous stop, over blanks only
            let stop = (col - 1) / sts * sts;
            let mut n = 0;
            while col - n > stop && self.char(self.cursor - n - 1) == Some(' ') {
                n += 1;
            }
            let n = n.max(1);
            self.rem(self.cursor - n, self.cursor);
            self.cursor -= n;
            return;
        }
        self.rem(self.cursor - 1, self.cursor);
        self.cursor -= 1;
    }

    /// Tab in insert mode with softtabstop or smarttab: blanks to the next stop.
    pub(crate) fn vim_tab_width(&self) -> Option<usize> {
        let line = self.cur_line();
        let before = &self.buf.line(line)[..];
        let col = self.cursor().col;
        let only_blanks = before.chars().take(col).all(char::is_whitespace);
        if only_blanks && self.opt_on("smarttab", true) && self.opt_num("shiftwidth", 0) > 0 {
            return Some(self.shiftwidth());
        }
        match self.opt_num("softtabstop", 0) {
            n if n > 0 => Some(n as usize),
            n if n < 0 => Some(self.shiftwidth()),
            _ => None,
        }
    }

    /// While typing: past textwidth (or the window's width less wrapmargin) the line breaks at its
    /// last blank, when formatoptions has t.
    pub(crate) fn vim_autowrap(&mut self) {
        let tw = match self.opt_num("textwidth", 0) {
            n if n > 0 => n as usize,
            _ => match self.opt_num("wrapmargin", 0) {
                m if m > 0 && self.vim.screen.2 > m as usize => self.vim.screen.2 - m as usize,
                _ => return,
            },
        };
        if !self.opt_letter("formatoptions", 't', "tcqj") {
            return;
        }
        let line = self.cur_line();
        let s = self.line_start(line);
        let text: Vec<char> = self.buf.line(line).chars().collect();
        if self.indent_width(&text.iter().collect::<String>()) <= tw && text.len() <= tw {
            return;
        }
        // the last blank at or before the width, after the indent
        let ind = text.iter().take_while(|c| c.is_whitespace()).count();
        let Some(b) = (ind + 1..=tw.min(text.len().saturating_sub(1))).rev().find(|&i| text[i] == ' ') else { return };
        let indent = self.next_indent(line, true, None);
        let at = s + b;
        let mut end = at;
        while end > s + ind && self.char(end - 1) == Some(' ') {
            end -= 1;
        }
        let cursor_after = self.cursor > at;
        let moved = self.cursor.saturating_sub(at + 1);
        self.rem(end, at + 1);
        self.ins(end, &format!("\n{indent}"));
        if cursor_after {
            self.cursor = end + 1 + indent.chars().count() + moved;
        }
    }

    /// The pairs % jumps between (matchpairs, "(:),{:},[:]").
    pub(crate) fn match_pairs(&self) -> Vec<(char, char)> {
        self.opt("matchpairs").unwrap_or("(:),{:},[:]").split(',').filter_map(|p| {
            let mut c = p.chars();
            let (a, colon, b) = (c.next()?, c.next()?, c.next()?);
            (colon == ':').then_some((a, b))
        }).collect()
    }

    /// A paragraph or section boundary for { } [[ ]]: a blank line, or an nroff macro the option names.
    pub(crate) fn nroff_boundary(&self, line: usize, opt: &str, default: &str) -> bool {
        let t = self.buf.line(line);
        let Some(m) = t.strip_prefix('.') else { return false };
        let name: String = m.chars().take(2).collect();
        let list: Vec<char> = self.opt(opt).unwrap_or(default).chars().collect();
        list.chunks(2).any(|p| p.iter().collect::<String>().trim_end() == name.trim_end())
    }

    /// After a change: "N more lines" / "N fewer lines" past the report threshold.
    pub(crate) fn vim_report(&mut self, lines_before: usize) {
        let now = self.buf.len_lines();
        let diff = now as i64 - lines_before as i64;
        if diff.unsigned_abs() as i64 > self.opt_num("report", 2) {
            let m = if diff > 0 { format!("{diff} more lines") } else { format!("{} fewer lines", -diff) };
            self.vim.requests.push(Request::Message(m));
        }
    }

    /// The buffer may not be changed (modifiable off): edits are refused, once said.
    pub(crate) fn refuses_change(&mut self) -> bool {
        if self.opt_on("modifiable", true) {
            return false;
        }
        if !self.vim.requests.iter().any(|r| matches!(r, Request::Message(m) if m.starts_with("E21"))) {
            self.vim.requests.push(Request::Message("E21: Cannot make changes, 'modifiable' is off".into()));
        }
        true
    }
}

/// iskeyword ("@,48-57,_,192-255") as a table of the first 256 characters.
fn parse_iskeyword(v: &str) -> Option<[bool; 256]> {
    let mut t = [false; 256];
    for part in v.split(',').filter(|p| !p.is_empty()) {
        let (neg, part) = match part.strip_prefix('^') { Some(p) if !p.is_empty() => (true, p), _ => (false, part) };
        let num = |s: &str| s.parse::<usize>().ok().or_else(|| (s.chars().count() == 1).then(|| s.chars().next().unwrap() as usize));
        let range: Vec<usize> = if part == "@" {
            (0..256).filter(|&i| (i as u8 as char).is_alphabetic()).collect()
        } else if part == "@-@" {
            vec!['@' as usize]
        } else if let Some((a, b)) = part.split_once('-').filter(|(a, b)| !a.is_empty() && !b.is_empty()) {
            match (num(a), num(b)) {
                (Some(a), Some(b)) => (a..=b.min(255)).collect(),
                _ => continue,
            }
        } else {
            match num(part) {
                Some(n) if n < 256 => vec![n],
                _ => continue,
            }
        };
        for i in range {
            if i < 256 {
                t[i] = !neg;
            }
        }
    }
    Some(t)
}
