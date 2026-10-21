//! Emacs's larger units: balanced expressions (a bracket group, a string, a symbol), lists,
//! defuns (zett's outline sections: definitions and top-level blocks), sentences; moving over,
//! killing, marking and transposing them. No grammar needed, as Emacs's own syntax scanning.

use crate::Editor;

pub const COMMANDS: &[(&str, &str)] = &[
    ("forward-sexp", "Move past the next balanced expression (a bracket group, a string, a symbol)"),
    ("backward-sexp", "Move to the start of the balanced expression before the cursor"),
    ("kill-sexp", "Kill the balanced expression after the cursor"),
    ("backward-kill-sexp", "Kill the balanced expression before the cursor"),
    ("mark-sexp", "Set the mark after the next balanced expression (again: one more)"),
    ("transpose-sexps", "Swap the balanced expressions around the cursor"),
    ("forward-list", "Move past the next bracket group"),
    ("backward-list", "Move to the start of the bracket group before the cursor"),
    ("up-list", "Move forward out of the bracket group the cursor is in"),
    ("backward-up-list", "Move back out of the bracket group the cursor is in, to its opening bracket"),
    ("down-list", "Move forward into the next bracket group"),
    ("beginning-of-defun", "Move to the start of this definition (at its start: the one before)"),
    ("end-of-defun", "Move past the end of this definition (at its end: the next one's)"),
    ("mark-defun", "Select the definition the cursor is in"),
    ("narrow-to-defun", "Show only the definition the cursor is in"),
    ("forward-sentence", "Move to the end of the sentence"),
    ("backward-sentence", "Move to the start of the sentence"),
    ("kill-sentence", "Kill to the end of the sentence"),
    ("backward-kill-sentence", "Kill back to the start of the sentence"),
    ("mark-paragraph", "Select the paragraph"),
    ("insert-parentheses", "Insert () (around the region, or around the next N expressions)"),
    ("move-past-close-and-reindent", "Move past the next closing bracket and start an indented line"),
    ("forward-page", "Move past the next page break (a form feed, ^L)"),
    ("backward-page", "Move to the start of the page"),
    ("mark-page", "Select the page"),
    ("narrow-to-page", "Show only the page"),
    ("prog-fill-reindent-defun", "Fill the comment or paragraph at the cursor; in code, reindent the definition"),
    ("isearch-forward-symbol-at-point", "Search for the symbol at the cursor (C-s goes on)"),
];

pub fn is_command(name: &str) -> bool {
    COMMANDS.iter().any(|(n, _)| *n == name)
}

fn is_open(c: char) -> bool {
    matches!(c, '(' | '[' | '{')
}
fn is_close(c: char) -> bool {
    matches!(c, ')' | ']' | '}')
}
fn is_quote(c: char) -> bool {
    matches!(c, '"' | '`')
}

/// Definitions by their first word (after pub, async, export …): fn, def, class, impl …
fn is_defun(title: &str, level: usize) -> bool {
    const LEAD: &[&str] = &["pub", "pub(crate)", "pub(super)", "async", "unsafe", "export", "default", "static", "public", "private", "protected", "local", "const", "extern"];
    const DEF: &[&str] = &["fn", "def", "function", "func", "class", "impl", "struct", "enum", "trait", "mod", "interface", "type", "macro_rules!", "union", "module", "namespace", "object", "proc", "sub", "defun", "(defun", "(defn", "(define"];
    level == 1 || title.split_whitespace().find(|w| !LEAD.contains(w)).is_some_and(|w| DEF.contains(&w) || DEF.iter().any(|d| w.starts_with(&format!("{d}(")) || w.starts_with(&format!("{d}<"))))
}

impl Editor {
    // ── the scanner: brackets match across strings, a backslash escapes ──

    fn symbolish(&self, i: usize) -> bool {
        match self.char(i) {
            Some(c) if c.is_alphanumeric() || c == '_' => true,
            // don't, it's: an apostrophe inside a word is part of it
            Some('\'') => i > 0 && self.char(i - 1).is_some_and(char::is_alphanumeric) && self.char(i + 1).is_some_and(char::is_alphanumeric),
            _ => false,
        }
    }
    fn escaped(&self, i: usize) -> bool {
        let mut n = 0;
        while i > n && self.char(i - n - 1) == Some('\\') {
            n += 1;
        }
        n % 2 == 1
    }
    /// After the string starting at the quote `i`.
    fn string_end(&self, i: usize) -> Option<usize> {
        let q = self.char(i)?;
        let n = self.buf.len_chars();
        let mut j = i + 1;
        while j < n {
            let c = self.char(j)?;
            if c == '\\' {
                j += 2;
                continue;
            }
            if c == q {
                return Some(j + 1);
            }
            j += 1;
        }
        None
    }
    /// The opening quote of the string ending with the quote `i`.
    fn string_start(&self, i: usize) -> Option<usize> {
        let q = self.char(i)?;
        (0..i).rev().find(|&k| self.char(k) == Some(q) && !self.escaped(k))
    }
    /// Blanks and punctuation that start nothing, skipped either way.
    fn inert(&self, i: usize) -> bool {
        self.char(i).is_some_and(|c| !is_open(c) && !is_close(c) && !is_quote(c) && !self.symbolish(i))
    }
    fn skip_fwd(&self, mut i: usize) -> usize {
        let n = self.buf.len_chars();
        while i < n && self.inert(i) {
            i += 1;
        }
        i
    }
    fn skip_back(&self, mut i: usize) -> usize {
        while i > 0 && self.inert(i - 1) {
            i -= 1;
        }
        i
    }
    /// After the bracket matching the opener at `i`.
    fn close_of(&self, i: usize) -> Option<usize> {
        let n = self.buf.len_chars();
        let (mut depth, mut j) = (0usize, i);
        while j < n {
            let c = self.char(j)?;
            if c == '\\' {
                j += 2;
                continue;
            }
            if is_quote(c) {
                j = self.string_end(j)?;
                continue;
            }
            if is_open(c) {
                depth += 1;
            } else if is_close(c) {
                depth -= 1;
                if depth == 0 {
                    return Some(j + 1);
                }
            }
            j += 1;
        }
        None
    }
    /// The opener matching the closing bracket at `i`.
    fn open_of(&self, i: usize) -> Option<usize> {
        let (mut depth, mut j) = (0usize, i + 1);
        while j > 0 {
            j -= 1;
            let c = self.char(j)?;
            if self.escaped(j) {
                continue;
            }
            if is_quote(c) {
                j = self.string_start(j)?;
                continue;
            }
            if is_close(c) {
                depth += 1;
            } else if is_open(c) {
                depth -= 1;
                if depth == 0 {
                    return Some(j);
                }
            }
        }
        None
    }

    /// The end of the expression after `i`; None at a closing bracket or the end.
    pub(crate) fn sexp_end(&self, i: usize) -> Option<usize> {
        let i = self.skip_fwd(i);
        let c = self.char(i)?;
        if is_close(c) {
            return None;
        }
        if is_quote(c) {
            return self.string_end(i);
        }
        if is_open(c) {
            return self.close_of(i);
        }
        let n = self.buf.len_chars();
        let mut j = i;
        while j < n && self.symbolish(j) {
            j += 1;
        }
        Some(j)
    }
    /// The start of the expression before `i`; None at an opening bracket or the start.
    pub(crate) fn sexp_start(&self, i: usize) -> Option<usize> {
        let i = self.skip_back(i);
        if i == 0 {
            return None;
        }
        let c = self.char(i - 1)?;
        if is_open(c) {
            return None;
        }
        if is_quote(c) {
            return self.string_start(i - 1);
        }
        if is_close(c) {
            return self.open_of(i - 1);
        }
        let mut j = i;
        while j > 0 && self.symbolish(j - 1) {
            j -= 1;
        }
        Some(j)
    }
    /// After the bracket closing the group around `i`.
    fn up_end(&self, mut i: usize) -> Option<usize> {
        loop {
            let j = self.skip_fwd(i);
            if is_close(self.char(j)?) {
                return Some(j + 1);
            }
            i = self.sexp_end(j)?;
        }
    }
    /// The bracket opening the group around `i`.
    fn up_start(&self, mut i: usize) -> Option<usize> {
        loop {
            let j = self.skip_back(i);
            if j == 0 {
                return None;
            }
            if is_open(self.char(j - 1)?) {
                return Some(j - 1);
            }
            i = self.sexp_start(j)?;
        }
    }
    /// The next bracket group at this level: (its opener, after its closer).
    fn next_list(&self, mut i: usize) -> Option<(usize, usize)> {
        loop {
            let j = self.skip_fwd(i);
            let c = self.char(j)?;
            if is_close(c) {
                return None;
            }
            if is_open(c) {
                return Some((j, self.close_of(j)?));
            }
            i = self.sexp_end(j)?;
        }
    }
    fn prev_list(&self, mut i: usize) -> Option<usize> {
        loop {
            let j = self.skip_back(i);
            if j == 0 || is_open(self.char(j - 1)?) {
                return None;
            }
            if is_close(self.char(j - 1)?) {
                return self.open_of(j - 1);
            }
            i = self.sexp_start(j)?;
        }
    }

    // ── defuns: the outline's definitions ──

    /// Definitions as char ranges, in order.
    fn defuns(&self) -> Vec<(usize, usize)> {
        let n = self.buf.len_chars();
        self.sections()
            .into_iter()
            .filter(|s| self.view_kind() == crate::outline::OutlineKind::Markdown || is_defun(s.title.trim(), s.level))
            .map(|s| (self.line_start(s.start), if s.end < self.buf.len_lines() { self.line_start(s.end) } else { n }))
            .collect()
    }
    /// The innermost definition around the cursor.
    /// The symbol at (or just before) the cursor: M-s M-w, C-h f's default.
    pub fn word_at_cursor(&self) -> Option<String> {
        let mut a = self.cursor;
        while a > 0 && self.symbolish(a - 1) {
            a -= 1;
        }
        let mut b = a;
        while self.symbolish(b) {
            b += 1;
        }
        (b > a).then(|| self.buf.slice(a, b))
    }
    /// The definition around the cursor as a char range (C-M-x evaluates it).
    pub fn defun_range(&self) -> Option<(usize, usize)> {
        self.defun_here()
    }
    /// The page around the cursor: after the form feed before it, through the one after it.
    pub(crate) fn page_here(&self) -> (usize, usize) {
        let n = self.buf.len_chars();
        let a = (0..self.cursor).rev().find(|&i| self.char(i) == Some('\x0c')).map_or(0, |i| i + 1);
        let b = (self.cursor..n).find(|&i| self.char(i) == Some('\x0c')).unwrap_or(n);
        (a, b)
    }
    fn defun_here(&self) -> Option<(usize, usize)> {
        self.defuns().into_iter().filter(|&(a, b)| a <= self.cursor && self.cursor < b).min_by_key(|&(a, b)| b - a)
    }

    // ── sentences: . ? ! (and closing quotes or brackets) before a blank; blank lines end them too ──

    fn sentence_end_at(&self, i: usize) -> Option<usize> {
        if !matches!(self.char(i)?, '.' | '?' | '!') {
            return None;
        }
        let mut j = i + 1;
        while self.char(j).is_some_and(|c| matches!(c, '"' | '\'' | ')' | ']' | '”' | '’')) {
            j += 1;
        }
        self.char(j).is_none_or(char::is_whitespace).then_some(j)
    }
    fn para_break(&self, i: usize) -> bool {
        // a line break followed by a blank line
        self.char(i) == Some('\n') && {
            let mut j = i + 1;
            while self.char(j).is_some_and(|c| c == ' ' || c == '\t') {
                j += 1;
            }
            self.char(j) == Some('\n')
        }
    }
    pub(crate) fn sentence_fwd(&self, i: usize) -> usize {
        let n = self.buf.len_chars();
        let mut j = i;
        while j < n && self.char(j).is_some_and(char::is_whitespace) {
            j += 1;
        }
        while j < n {
            if let Some(e) = self.sentence_end_at(j) {
                return e;
            }
            if self.para_break(j) {
                return j;
            }
            j += 1;
        }
        n
    }
    pub(crate) fn sentence_back(&self, i: usize) -> usize {
        let mut j = i;
        while j > 0 && self.char(j - 1).is_some_and(char::is_whitespace) {
            j -= 1;
        }
        while j > 0 {
            let p = j - 1;
            if self.char(p).is_some_and(char::is_whitespace) {
                let mut k = p;
                while k > 0 && self.char(k - 1).is_some_and(|c| matches!(c, '"' | '\'' | ')' | ']' | '”' | '’')) {
                    k -= 1;
                }
                if k > 0 && self.sentence_end_at(k - 1).is_some() {
                    break;
                }
                if self.char(p) == Some('\n') && p > 0 && self.para_break(p - 1) {
                    break;
                }
            }
            j -= 1;
        }
        while self.char(j).is_some_and(char::is_whitespace) && j < i {
            j += 1;
        }
        j
    }

    /// Runs one of COMMANDS `n` times (n < 0: backwards where that means something).
    pub(crate) fn balanced_exec(&mut self, name: &str, n: i64) -> bool {
        let last = std::mem::replace(&mut self.last_cmd, name.to_string());
        let times = n.unsigned_abs().max(1) as usize;
        let back = n < 0;
        let kills = matches!(last.as_str(), "kill-line" | "kill-whole-line" | "kill-word" | "backward-kill-word" | "kill-region" | "kill-sexp" | "backward-kill-sexp" | "kill-sentence" | "backward-kill-sentence");
        let rev = self.buf.rev();
        match name {
            "forward-sexp" | "backward-sexp" | "forward-list" | "backward-list" | "up-list" | "backward-up-list" | "down-list" | "forward-sentence" | "backward-sentence" => {
                let fwd = (name.starts_with("forward") || name == "up-list" || name == "down-list") != back;
                for _ in 0..times {
                    let to = match (name, fwd) {
                        ("forward-sexp" | "backward-sexp", true) => self.sexp_end(self.cursor),
                        ("forward-sexp" | "backward-sexp", false) => self.sexp_start(self.cursor),
                        ("forward-list" | "backward-list", true) => self.next_list(self.cursor).map(|l| l.1),
                        ("forward-list" | "backward-list", false) => self.prev_list(self.cursor),
                        ("up-list" | "backward-up-list", true) => self.up_end(self.cursor),
                        ("up-list" | "backward-up-list", false) => self.up_start(self.cursor),
                        ("down-list", _) => self.next_list(self.cursor).map(|l| l.0 + 1),
                        (_, true) => Some(self.sentence_fwd(self.cursor)),
                        (_, false) => Some(self.sentence_back(self.cursor)),
                    };
                    match to {
                        Some(t) => self.cursor = t,
                        None => break,
                    }
                }
            }
            "kill-sexp" | "backward-kill-sexp" | "kill-sentence" | "backward-kill-sentence" => {
                let fwd = name.starts_with("kill") != back;
                let sexp = name.ends_with("sexp");
                let mut to = self.cursor;
                for _ in 0..times {
                    let t = match (sexp, fwd) {
                        (true, true) => self.sexp_end(to),
                        (true, false) => self.sexp_start(to),
                        (false, true) => Some(self.sentence_fwd(to)),
                        (false, false) => Some(self.sentence_back(to)),
                    };
                    match t {
                        Some(t) => to = t,
                        None => break,
                    }
                }
                let (a, b) = (self.cursor.min(to), self.cursor.max(to));
                if b > a {
                    self.begin();
                    let t = self.rem(a, b);
                    self.cursor = a;
                    self.ring().push(t, kills, !fwd);
                    self.yanked = None;
                    self.close();
                }
            }
            "mark-sexp" => {
                // again right after: one more expression from the mark
                let from = if last == "mark-sexp" && self.mark_active { self.mark.unwrap_or(self.cursor) } else { self.cursor };
                let mut to = from;
                for _ in 0..times {
                    match if back { self.sexp_start(to) } else { self.sexp_end(to) } {
                        Some(t) => to = t,
                        None => break,
                    }
                }
                self.mark = Some(to);
                self.mark_active = true;
            }
            "transpose-sexps" => {
                let (Some(a), Some(c)) = (self.sexp_start(self.cursor), self.sexp_end(self.cursor)) else { return true };
                let (Some(b), Some(d)) = (self.sexp_end(a), self.sexp_start(c)) else { return true };
                if b <= d {
                    let (x, mid, y) = (self.buf.slice(a, b), self.buf.slice(b, d), self.buf.slice(d, c));
                    self.begin();
                    self.rem(a, c);
                    self.ins(a, &format!("{y}{mid}{x}"));
                    self.close();
                    self.cursor = c;
                }
            }
            "beginning-of-defun" | "end-of-defun" => {
                let fwd = (name == "end-of-defun") != back;
                for _ in 0..times {
                    let ds = self.defuns();
                    let to = if name == "beginning-of-defun" && !back {
                        ds.iter().map(|d| d.0).filter(|&a| a < self.cursor).max()
                    } else if name == "beginning-of-defun" {
                        ds.iter().map(|d| d.0).filter(|&a| a > self.cursor).min()
                    } else if fwd {
                        ds.iter().map(|d| d.1).filter(|&b| b > self.cursor).min()
                    } else {
                        ds.iter().map(|d| d.1).filter(|&b| b < self.cursor).max()
                    };
                    match to {
                        Some(t) => {
                            if name == "beginning-of-defun" && self.mark.is_none_or(|_| !self.mark_active) {
                                self.mark = Some(self.cursor);
                            }
                            self.cursor = t;
                        }
                        None => break,
                    }
                }
            }
            "mark-defun" | "narrow-to-defun" => {
                let Some((a, b)) = self.defun_here() else { return true };
                self.mark = Some(b);
                self.cursor = a;
                self.mark_active = true;
                if name == "narrow-to-defun" {
                    self.last_cmd = name.into();
                    self.outline_exec("narrow-to-region", 1);
                    self.mark_active = false;
                }
            }
            "mark-paragraph" => {
                let l = self.cur_line();
                let blank = |l: usize| self.buf.line(l).trim().is_empty();
                let (mut a, mut b) = (l, l);
                let last_line = self.buf.len_lines() - 1;
                while a > 0 && !blank(a - 1) {
                    a -= 1;
                }
                while b < last_line && !blank(b + 1) {
                    b += 1;
                }
                let end = if b < last_line { self.line_start(b + 1) } else { self.buf.len_chars() };
                self.mark = Some(end);
                self.cursor = self.line_start(a);
                self.mark_active = true;
            }
            "insert-parentheses" => {
                let (a, b) = match self.region() {
                    Some(r) => r,
                    None if n != 1 || back => {
                        let mut to = self.skip_fwd(self.cursor);
                        let a = to;
                        for _ in 0..times {
                            if let Some(t) = self.sexp_end(to) {
                                to = t;
                            }
                        }
                        (a, to)
                    }
                    None => (self.cursor, self.cursor),
                };
                self.begin();
                self.ins(b, ")");
                self.ins(a, "(");
                self.close();
                self.mark_active = false;
                self.cursor = a + 1;
            }
            "move-past-close-and-reindent" => {
                let Some(after) = self.up_end(self.cursor) else { return true };
                let close = after - 1;
                self.begin();
                // a closer alone on its line joins the line before
                let l = self.buf.to_pos(close).line;
                let mut at = close;
                if l > 0 && self.first_non_blank(l) == close {
                    let mut s = self.line_start(l) - 1;
                    while s > 0 && self.char(s - 1).is_some_and(char::is_whitespace) {
                        s -= 1;
                    }
                    self.rem(s, close);
                    at = s;
                }
                let line = self.buf.to_pos(at).line;
                let ind = self.indent_of(line);
                self.ins(at + 1, &format!("\n{ind}"));
                self.cursor = at + 2 + ind.chars().count();
                self.close();
            }
            "forward-page" | "backward-page" => {
                let fwd = (name == "forward-page") != back;
                let n = self.buf.len_chars();
                for _ in 0..times {
                    self.cursor = if fwd {
                        (self.cursor..n).find(|&i| self.char(i) == Some('\x0c')).map_or(n, |i| i + 1)
                    } else {
                        // at a page's start: the one before
                        let from = self.cursor.saturating_sub(1);
                        (0..from).rev().find(|&i| self.char(i) == Some('\x0c')).map_or(0, |i| i + 1)
                    };
                }
            }
            "mark-page" | "narrow-to-page" => {
                let (a, b) = self.page_here();
                self.mark = Some(b);
                self.cursor = a;
                self.mark_active = true;
                if name == "narrow-to-page" {
                    self.outline_exec("narrow-to-region", 1);
                    self.mark_active = false;
                }
            }
            "prog-fill-reindent-defun" => {
                // text, a comment or a list item: fill it; code: reindent the definition around it
                let line = self.buf.line(self.cur_line()).trim_end_matches('\n').to_string();
                let lead = crate::textops::fill_prefix(&line).0;
                let code = self.view_kind() == crate::outline::OutlineKind::Code && lead.trim().is_empty() && self.defun_here().is_some();
                if !code {
                    return self.textops_exec("fill-paragraph", 1);
                }
                let (a, b) = self.defun_here().unwrap_or((self.cursor, self.cursor));
                let (l1, l2) = (self.buf.to_pos(a).line, self.buf.to_pos(b.saturating_sub(1).max(a)).line);
                let at = self.cursor();
                self.vim_apply(crate::Op::Reindent, crate::Span::Lines(l1, l2));
                self.cursor = self.line_start(at.line) + at.col.min(self.buf.line_len(at.line));
            }
            "isearch-forward-symbol-at-point" => {
                let mut a = self.cursor;
                while a > 0 && self.symbolish(a - 1) {
                    a -= 1;
                }
                let Some(b) = self.sexp_end(a).filter(|_| self.symbolish(a)) else { return true };
                let word = self.buf.slice(a, b);
                self.cursor = a;
                self.search_command("isearch-forward");
                for c in word.chars() {
                    self.isearch_char(c);
                }
                self.last_cmd = name.into();
                return true;
            }
            _ => return false,
        }
        if self.buf.rev() != rev {
            self.mark_active = false;
        }
        self.want_col = self.cursor().col;
        if !self.view_is_empty() {
            self.reveal();
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use crate::Editor;

    fn ed(text: &str, at: usize) -> Editor {
        let mut e = Editor::from_str(text);
        e.set_emacs(true);
        e.cursor = at;
        e
    }

    #[test]
    fn sexps_over_brackets_strings_and_symbols() {
        let mut e = ed("f(a, \"x)\" [b]) tail", 1);
        e.exec("forward-sexp", 1);
        assert_eq!(e.cursor, 14, "the bracket group, the ) inside the string skipped");
        e.exec("forward-sexp", 1);
        assert_eq!(e.cursor, 19);
        e.exec("backward-sexp", 2);
        assert_eq!(e.cursor, 1);
        e.exec("down-list", 1);
        assert_eq!(e.cursor, 2);
        e.exec("forward-sexp", 2);
        assert_eq!(e.cursor, 9, "a, then the string");
        e.exec("up-list", 1);
        assert_eq!(e.cursor, 14);
        e.cursor = 11;
        e.exec("backward-up-list", 1);
        assert_eq!(e.cursor, 10, "to the [");
        e.exec("backward-up-list", 1);
        assert_eq!(e.cursor, 1);
    }

    #[test]
    fn kill_mark_and_transpose_sexps() {
        let mut e = ed("(one) (two three)", 0);
        e.exec("kill-sexp", 1);
        assert_eq!(e.text(), " (two three)");
        assert_eq!(e.last_kill().as_deref(), Some("(one)"));
        let mut e = ed("alpha beta", 5);
        e.exec("transpose-sexps", 1);
        assert_eq!(e.text(), "beta alpha");
        assert_eq!(e.cursor, 10);
        let mut e = ed("a b c", 0);
        e.exec("mark-sexp", 1);
        e.exec("mark-sexp", 1);
        assert_eq!(e.region(), Some((0, 3)), "twice: two expressions");
    }

    #[test]
    fn sentences() {
        let mut e = ed("One two. Three? Four (five!) six.\n\nNext para", 0);
        e.exec("forward-sentence", 1);
        assert_eq!(e.cursor, 8);
        e.exec("forward-sentence", 1);
        assert_eq!(e.cursor, 15);
        e.exec("forward-sentence", 1);
        assert_eq!(e.cursor, 28, "a closing bracket after the ! is part of the end");
        e.exec("forward-sentence", 1);
        assert_eq!(e.cursor, 33);
        e.exec("backward-sentence", 1);
        assert_eq!(e.cursor, 29);
        e.exec("backward-kill-sentence", 1);
        assert_eq!(e.text(), "One two. Three? six.\n\nNext para", "at a sentence start: the one before");
        let mut e = ed("Para one.\n\nNext para", 13);
        e.exec("backward-sentence", 1);
        assert_eq!(e.cursor, 11, "a blank line ends a sentence");
    }

    #[test]
    fn defuns_are_the_outline_definitions() {
        let src = "fn a() {\n    if x {\n        y();\n    }\n}\n\nfn b() {\n    z();\n}\n";
        let mut e = ed(src, src.find("y()").unwrap());
        e.exec("beginning-of-defun", 1);
        assert_eq!(e.cursor, 0, "the if block is not a definition");
        e.exec("end-of-defun", 1);
        assert_eq!(e.cursor().line, 5);
        e.exec("end-of-defun", 1);
        assert_eq!(e.cursor().line, 9);
        e.exec("beginning-of-defun", 1);
        assert_eq!(e.cursor().line, 6);
        e.cursor = src.find("z()").unwrap();
        e.exec("mark-defun", 1);
        assert_eq!(e.region().map(|(a, b)| e.buf.slice(a, b)), Some("fn b() {\n    z();\n}\n".to_string()));
    }

    #[test]
    fn parentheses_and_paragraphs() {
        let mut e = ed("foo bar", 0);
        e.exec_arg("insert-parentheses", Some(2));
        assert_eq!(e.text(), "(foo bar)");
        let mut e = ed("x", 1);
        e.exec("insert-parentheses", 1);
        assert_eq!((e.text().as_str(), e.cursor), ("x()", 2));
        let mut e = ed("a\nb\n\nc", 2);
        e.exec("mark-paragraph", 1);
        assert_eq!(e.region(), Some((0, 4)));
        let mut e = ed("f(a,\n  b\n  )", 7);
        e.exec("move-past-close-and-reindent", 1);
        assert_eq!(e.text(), "f(a,\n  b)\n  ");
    }

    #[test]
    fn pages_and_m_q_in_code() {
        let mut e = ed("one\n\x0ctwo\nthree\n\x0cfour", 6);
        e.exec("forward-page", 1);
        assert_eq!(e.cursor, 16, "past the next form feed");
        e.exec("backward-page", 1);
        assert_eq!(e.cursor, 5, "the page before");
        e.cursor = 7;
        e.exec("mark-page", 1);
        assert_eq!(e.region().map(|(a, b)| e.buf.slice(a, b)), Some("two\nthree\n".to_string()));
        let src = "fn a() {\nx();\n        y();\n}\n";
        let mut e = ed(src, src.find("y()").unwrap());
        e.exec("prog-fill-reindent-defun", 1);
        assert_eq!(e.text(), "fn a() {\nx();\nx();\n}\n".replace("x();\nx();", "x();\ny();"), "reindented as the line above");
        let mut e = ed("// a long comment\n// more", 3);
        e.set_fill_column(80);
        e.exec("prog-fill-reindent-defun", 1);
        assert_eq!(e.text(), "// a long comment more", "a comment is filled");
    }

    #[test]
    fn symbol_at_point_starts_a_search() {
        let mut e = ed("let foo = 1; foo + foo", 5);
        e.exec("isearch-forward-symbol-at-point", 1);
        assert_eq!(e.isearch().map(|s| s.query().to_string()).as_deref(), Some("foo"));
        e.exec("isearch-repeat-forward", 1);
        assert_eq!(e.cursor, 16);
    }
}
