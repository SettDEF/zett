//! Emacs's text commands beyond moving and killing: filling, sorting, joining, transposing words and
//! lines, region case, blank lines, dynamic abbreviations (M-/). Each edit is one undo step.

use crate::Editor;

pub const COMMANDS: &[(&str, &str)] = &[
    ("fill-paragraph", "Re-wrap the paragraph at the fill column (list and comment prefixes kept)"),
    ("sort-lines", "Sort the lines of the region (or the paragraph); a negative argument sorts backwards"),
    ("delete-duplicate-lines", "Delete lines that already appeared, in the region (or the buffer)"),
    ("join-line", "Join this line to the one before, with one space"),
    ("transpose-words", "Swap the word before the cursor with the one after"),
    ("transpose-lines", "Swap this line with the one before, moving down"),
    ("delete-blank-lines", "Leave one blank line where several are (none if only one)"),
    ("upcase-region", "Upper-case the region"),
    ("downcase-region", "Lower-case the region"),
    ("mark-word", "Select to the end of the next word (again: one more word)"),
    ("dabbrev-expand", "Complete the word before the cursor from words in the buffer (again: the next one)"),
];

pub fn is_command(name: &str) -> bool {
    COMMANDS.iter().any(|(n, _)| *n == name)
}

/// The prefix a filled line keeps, and the one its continuation lines get: indentation, "- ",
/// "* ", "1. ", "> ", "// ", "# " (a comment, not a Markdown heading when the text is code).
pub fn fill_prefix(line: &str) -> (String, String) {
    let indent: String = line.chars().take_while(|c| c.is_whitespace()).collect();
    let rest = &line[indent.len()..];
    for p in ["// ", "/// ", "# ", "-- ", "; ", "> "] {
        if rest.starts_with(p) {
            let pre = format!("{indent}{p}");
            return (pre.clone(), pre);
        }
    }
    for p in ["- [ ] ", "- [x] ", "- ", "* ", "+ "] {
        if rest.starts_with(p) {
            return (format!("{indent}{p}"), format!("{indent}{}", " ".repeat(p.len())));
        }
    }
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    if !digits.is_empty() && rest[digits.len()..].starts_with(". ") {
        let p = format!("{digits}. ");
        return (format!("{indent}{p}"), format!("{indent}{}", " ".repeat(p.len())));
    }
    (indent.clone(), indent)
}

/// Words wrapped to `width` columns, the first line with `first`, the rest with `rest`.
pub fn fill(words: &[&str], first: &str, rest: &str, width: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = first.to_string();
    let mut empty = true;
    for w in words {
        if !empty && cur.chars().count() + 1 + w.chars().count() > width {
            out.push(std::mem::replace(&mut cur, rest.to_string()));
            empty = true;
        }
        if !empty {
            cur.push(' ');
        }
        cur.push_str(w);
        empty = false;
    }
    out.push(cur);
    out
}

impl Editor {
    /// expandtab: Some(shiftwidth) makes Tab insert spaces to the next stop; None, a tab character.
    pub fn set_soft_tab(&mut self, width: Option<usize>) {
        self.soft_tab = width.map(|w| w.max(1));
    }

    /// Tab: a tab character, or spaces to the next multiple of the shift width.
    pub(crate) fn insert_tab(&mut self) {
        // softtabstop and smarttab: blanks to the next of their stops (vim/options.rs)
        let s = match (self.vim_tab_width(), self.soft_tab) {
            (Some(w), _) | (None, Some(w)) => {
                let col = self.cursor().col;
                " ".repeat(w - col % w)
            }
            (None, None) => "\t".to_string(),
        };
        self.ins(self.cursor, &s);
        self.cursor += s.chars().count();
    }

    /// The fill column (zett.opt.fill_column; 80).
    pub fn set_fill_column(&mut self, n: usize) {
        self.fill_column = n.max(10);
    }

    fn lines_of(&self, a: usize, b: usize) -> (usize, usize) {
        let (la, lb) = (self.buf.to_pos(a).line, self.buf.to_pos(b.saturating_sub(1).max(a)).line);
        (la, lb)
    }

    /// The paragraph around a line: the lines between blank ones.
    fn paragraph(&self, line: usize) -> (usize, usize) {
        let blank = |l: usize| self.buf.line(l).trim().is_empty();
        let n = self.buf.len_lines();
        let (mut a, mut b) = (line, line);
        while a > 0 && !blank(a - 1) {
            a -= 1;
        }
        while b + 1 < n && !blank(b + 1) {
            b += 1;
        }
        (a, b)
    }

    /// Lines `a..=b` replaced by `new` (one change), the cursor at `cursor`.
    fn replace_lines(&mut self, a: usize, b: usize, new: &[String], cursor: Option<usize>) {
        let from = self.line_start(a);
        let to = self.line_start(b) + self.buf.line_len(b);
        let text = new.join("\n");
        if self.buf.slice(from, to) == text {
            return;
        }
        self.begin();
        self.rem(from, to);
        self.ins(from, &text);
        self.close();
        self.cursor = cursor.unwrap_or(from).min(self.buf.len_chars());
    }

    fn line_text(&self, l: usize) -> String {
        self.buf.line(l).trim_end_matches('\n').to_string()
    }

    /// Runs one of COMMANDS; `n` is the numeric argument (negative reverses where that means something).
    pub(crate) fn textops_exec(&mut self, name: &str, n: i64) -> bool {
        let here = self.cursor().line;
        match name {
            "fill-paragraph" => {
                let (a, b) = self.paragraph(here);
                if self.line_text(a).trim().is_empty() {
                    return true;
                }
                let (first, rest) = fill_prefix(&self.line_text(a));
                let strip = |l: &str| {
                    let t = l.trim_start();
                    let (p, _) = fill_prefix(l);
                    let p = p.trim_start();
                    t.strip_prefix(p).unwrap_or(t).to_string()
                };
                let body: Vec<String> = (a..=b).map(|l| if l == a { self.line_text(l)[first.len().min(self.line_text(l).len())..].to_string() } else { strip(&self.line_text(l)) }).collect();
                let joined = body.join(" ");
                let words: Vec<&str> = joined.split_whitespace().collect();
                let new = fill(&words, &first, &rest, self.fill_column);
                let at = self.line_start(a);
                self.replace_lines(a, b, &new, Some(at));
            }
            "sort-lines" | "delete-duplicate-lines" => {
                let (a, b) = match self.region() {
                    Some((s, e)) => self.lines_of(s, e),
                    None if name == "sort-lines" => self.paragraph(here),
                    None => (0, self.buf.len_lines().saturating_sub(1)),
                };
                let mut lines: Vec<String> = (a..=b).map(|l| self.line_text(l)).collect();
                if name == "sort-lines" {
                    lines.sort();
                    if n < 0 {
                        lines.reverse();
                    }
                } else {
                    let mut seen = std::collections::HashSet::new();
                    lines.retain(|l| seen.insert(l.clone()));
                }
                self.mark_active = false;
                let at = self.line_start(a);
                self.replace_lines(a, b, &lines, Some(at));
            }
            "join-line" => {
                if here == 0 {
                    return true;
                }
                let prev = self.line_text(here - 1).trim_end().to_string();
                let cur = self.line_text(here).trim_start().to_string();
                let joined = if cur.is_empty() || prev.is_empty() { format!("{prev}{cur}") } else { format!("{prev} {cur}") };
                let at = self.line_start(here - 1) + prev.chars().count();
                self.replace_lines(here - 1, here, &[joined], Some(at));
            }
            "transpose-lines" => {
                if here == 0 || here >= self.buf.len_lines() {
                    return true;
                }
                let (x, y) = (self.line_text(here - 1), self.line_text(here));
                self.replace_lines(here - 1, here, &[y, x], None);
                let next = (here + 1).min(self.buf.len_lines().saturating_sub(1));
                self.cursor = self.line_start(next);
            }
            "transpose-words" => {
                let text: Vec<char> = self.buf.text().chars().collect();
                let w = |c: char| c.is_alphanumeric() || c == '_';
                let c = self.cursor.min(text.len());
                // the word before (or under) the cursor, and the next one
                let mut s1 = c;
                while s1 > 0 && !w(text[s1 - 1]) && !(s1 < text.len() && w(text[s1])) {
                    s1 -= 1;
                }
                while s1 > 0 && w(text[s1 - 1]) {
                    s1 -= 1;
                }
                let mut e1 = s1;
                while e1 < text.len() && w(text[e1]) {
                    e1 += 1;
                }
                let mut s2 = e1;
                while s2 < text.len() && !w(text[s2]) {
                    s2 += 1;
                }
                let mut e2 = s2;
                while e2 < text.len() && w(text[e2]) {
                    e2 += 1;
                }
                if e1 == s1 || e2 == s2 {
                    return true;
                }
                let a: String = text[s1..e1].iter().collect();
                let mid: String = text[e1..s2].iter().collect();
                let b: String = text[s2..e2].iter().collect();
                self.begin();
                self.rem(s1, e2);
                self.ins(s1, &format!("{b}{mid}{a}"));
                self.close();
                self.cursor = e2;
            }
            "delete-blank-lines" => {
                let blank = |l: usize| self.buf.line(l).trim().is_empty();
                let total = self.buf.len_lines();
                if !blank(here) {
                    // on a text line: the blank lines right after it go
                    let mut b = here + 1;
                    while b < total && blank(b) {
                        b += 1;
                    }
                    if b > here + 1 {
                        let keep: Vec<String> = vec![self.line_text(here)];
                        let at = self.cursor;
                        self.replace_lines(here, b - 1, &keep, Some(at));
                    }
                    return true;
                }
                let (mut a, mut b) = (here, here);
                while a > 0 && blank(a - 1) {
                    a -= 1;
                }
                while b + 1 < total && blank(b + 1) {
                    b += 1;
                }
                let new = if a == b { Vec::new() } else { vec![String::new()] };
                if new.is_empty() {
                    // one blank line alone: it goes
                    let from = self.line_start(a);
                    let to = if a + 1 < total { self.line_start(a + 1) } else { self.buf.len_chars() };
                    self.begin();
                    self.rem(from, to);
                    self.close();
                    self.cursor = from.min(self.buf.len_chars());
                } else {
                    let at = self.line_start(a);
                    self.replace_lines(a, b, &new, Some(at));
                }
            }
            "upcase-region" | "downcase-region" => {
                let Some((s, e)) = self.mark_span() else { return true };
                let t = self.buf.slice(s, e);
                let t = if name == "upcase-region" { t.to_uppercase() } else { t.to_lowercase() };
                self.begin();
                self.rem(s, e);
                self.ins(s, &t);
                self.close();
                self.mark_active = false;
            }
            "mark-word" => {
                let text: Vec<char> = self.buf.text().chars().collect();
                let w = |c: char| c.is_alphanumeric() || c == '_';
                if !(self.mark_active && self.last_cmd_was_mark_word) {
                    self.set_mark(true);
                    self.mark_word_end = self.cursor;
                }
                // the mark goes to the end of one more word; the cursor stays
                let mut e = self.mark_word_end.min(text.len());
                for _ in 0..n.unsigned_abs().max(1) {
                    while e < text.len() && !w(text[e]) {
                        e += 1;
                    }
                    while e < text.len() && w(text[e]) {
                        e += 1;
                    }
                }
                self.mark_word_end = e;
                self.mark = Some(e);
                self.mark_active = true;
            }
            "dabbrev-expand" => self.dabbrev(),
            _ => return false,
        }
        true
    }

    /// M-/: the word before the cursor completed from the buffer, nearest first (before, then
    /// after); again right away replaces it with the next candidate.
    fn dabbrev(&mut self) {
        let text: Vec<char> = self.buf.text().chars().collect();
        let w = |c: char| c.is_alphanumeric() || c == '_' || c == '-';
        let again = self.dabbrev.as_ref().is_some_and(|d| d.at == self.cursor);
        let (start, prefix, mut tried) = if again {
            let d = self.dabbrev.take().unwrap();
            (d.start, d.prefix, d.tried)
        } else {
            let mut s = self.cursor.min(text.len());
            while s > 0 && w(text[s - 1]) {
                s -= 1;
            }
            let p: String = text[s..self.cursor.min(text.len())].iter().collect();
            if p.is_empty() {
                return;
            }
            (s, p.clone(), vec![p])
        };
        // every word starting with the prefix, by distance from where it was typed
        let mut cands: Vec<(usize, String)> = Vec::new();
        let mut i = 0;
        while i < text.len() {
            if w(text[i]) && (i == 0 || !w(text[i - 1])) {
                let mut e = i;
                while e < text.len() && w(text[e]) {
                    e += 1;
                }
                let word: String = text[i..e].iter().collect();
                if i != start && word.starts_with(&prefix) && word != prefix {
                    cands.push((i.abs_diff(start) + if i > start { 1 } else { 0 }, word));
                }
                i = e;
            } else {
                i += 1;
            }
        }
        cands.sort_by_key(|c| c.0);
        let Some((_, next)) = cands.into_iter().find(|(_, c)| !tried.contains(c)) else {
            // none left: back to what was typed
            let cur: String = text[start..self.cursor.min(text.len())].iter().collect();
            if cur != prefix {
                self.begin();
                self.rem(start, self.cursor);
                self.ins(start, &prefix);
                self.close();
                self.cursor = start + prefix.chars().count();
            }
            return;
        };
        self.begin();
        self.rem(start, self.cursor);
        self.ins(start, &next);
        self.close();
        self.cursor = start + next.chars().count();
        tried.push(next);
        self.dabbrev = Some(Dabbrev { start, prefix, tried, at: self.cursor });
    }
}

/// M-/ in progress: where the word starts, what was typed, what was offered, and where the cursor
/// was left (a press anywhere else starts over).
#[derive(Clone, Debug)]
pub(crate) struct Dabbrev {
    pub start: usize,
    pub prefix: String,
    pub tried: Vec<String>,
    pub at: usize,
}

#[cfg(test)]
mod tests {
    use crate::Editor;

    fn ed(t: &str) -> Editor {
        let mut e = Editor::from_str(t);
        e.set_emacs(true);
        e
    }
    fn run(e: &mut Editor, cmd: &str) {
        assert!(e.exec(cmd, 1), "{cmd} runs");
    }

    #[test]
    fn fill_keeps_prefixes_and_wraps() {
        let mut e = ed("- one two three four five six seven eight nine ten eleven twelve\n  thirteen\n\nnext\n");
        e.set_fill_column(24);
        run(&mut e, "fill-paragraph");
        assert_eq!(e.text(), "- one two three four\n  five six seven eight\n  nine ten eleven twelve\n  thirteen\n\nnext\n", "24 columns fit");
        let mut c = ed("// a b c d e f g h i j k l m n o p\n");
        c.set_fill_column(14);
        run(&mut c, "fill-paragraph");
        assert_eq!(c.text(), "// a b c d e f\n// g h i j k l\n// m n o p\n");
        run(&mut c, "undo");
        assert_eq!(c.text(), "// a b c d e f g h i j k l m n o p\n", "one undo step");
    }

    #[test]
    fn sorts_dedups_joins_and_transposes() {
        let mut e = ed("pear\napple\npear\nfig\n\nz\n");
        run(&mut e, "sort-lines");
        assert_eq!(e.text(), "apple\nfig\npear\npear\n\nz\n");
        run(&mut e, "delete-duplicate-lines");
        assert_eq!(e.text(), "apple\nfig\npear\n\nz\n");
        let mut j = ed("one\n   two\n");
        j.exec("next-line", 1);
        run(&mut j, "join-line");
        assert_eq!(j.text(), "one two\n");
        let mut t = ed("a\nb\nc\n");
        t.exec("next-line", 1);
        run(&mut t, "transpose-lines");
        assert_eq!(t.text(), "b\na\nc\n");
        let mut w = ed("hello brave world");
        w.exec("forward-word", 1);
        run(&mut w, "transpose-words");
        assert_eq!(w.text(), "brave hello world");
    }

    #[test]
    fn blank_lines_case_and_dabbrev() {
        let mut b = ed("a\n\n\n\nb\n");
        b.exec("next-line", 2);
        run(&mut b, "delete-blank-lines");
        assert_eq!(b.text(), "a\n\nb\n");
        run(&mut b, "delete-blank-lines");
        assert_eq!(b.text(), "a\nb\n");
        let mut c = ed("make loud");
        c.exec("set-mark-command", 1);
        c.exec("end-of-line", 1);
        run(&mut c, "upcase-region");
        assert_eq!(c.text(), "MAKE LOUD");
        let mut d = ed("configure config_file\nco");
        d.exec("end-of-buffer", 1);
        run(&mut d, "dabbrev-expand");
        assert_eq!(d.text(), "configure config_file\nconfig_file", "the nearest first");
        run(&mut d, "dabbrev-expand");
        assert_eq!(d.text(), "configure config_file\nconfigure", "again: the next");
        run(&mut d, "dabbrev-expand");
        assert_eq!(d.text(), "configure config_file\nco", "none left: what was typed");
    }
}
