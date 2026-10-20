//! Named editing commands, the way Emacs has them: what keys point at in the Emacs profile
//! (and what M-x lists). Modeless: the cursor sits between characters, a mark makes a region,
//! kills go to a kill ring. Every edit is recorded for undo like Vim's.

use crate::{Editor, Mode};

/// Every editing command: name and what it does (M-x, C-h f).
pub const COMMANDS: &[(&str, &str)] = &[
    ("forward-char", "Move one character forward"),
    ("backward-char", "Move one character back"),
    ("next-line", "Move to the next line, keeping the column"),
    ("previous-line", "Move to the previous line, keeping the column"),
    ("beginning-of-line", "Move to the start of the line"),
    ("end-of-line", "Move to the end of the line"),
    ("back-to-indentation", "Move to the first non-blank character of the line"),
    ("forward-word", "Move past the end of the next word"),
    ("backward-word", "Move to the start of the previous word"),
    ("beginning-of-buffer", "Move to the start of the buffer (sets the mark)"),
    ("end-of-buffer", "Move to the end of the buffer (sets the mark)"),
    ("forward-paragraph", "Move past the end of the paragraph"),
    ("backward-paragraph", "Move to the start of the paragraph"),
    ("delete-char", "Delete the character after the cursor"),
    ("delete-backward-char", "Delete the character before the cursor"),
    ("kill-line", "Kill to the end of the line; at its end, the line break"),
    ("kill-whole-line", "Kill the whole line, its line break included"),
    ("kill-word", "Kill to the end of the word"),
    ("backward-kill-word", "Kill back to the start of the word"),
    ("kill-region", "Kill the text between the mark and the cursor"),
    ("copy-region-as-kill", "Copy the region to the kill ring, without deleting it"),
    ("yank", "Insert the last killed text"),
    ("yank-pop", "Right after a yank: replace it with the kill before it"),
    ("set-mark-command", "Set the mark at the cursor (starts a region)"),
    ("exchange-point-and-mark", "Swap the cursor and the mark"),
    ("mark-whole-buffer", "Select the whole buffer"),
    ("keyboard-quit", "Cancel: deactivate the mark"),
    ("newline", "Insert a line break, keeping the indentation"),
    ("open-line", "Insert a line break after the cursor, staying put"),
    ("indent-for-tab-command", "Insert a tab"),
    ("transpose-chars", "Swap the characters around the cursor"),
    ("upcase-word", "Upper-case the word after the cursor"),
    ("downcase-word", "Lower-case the word after the cursor"),
    ("capitalize-word", "Capitalize the word after the cursor"),
    ("delete-horizontal-space", "Delete spaces and tabs around the cursor"),
    ("just-one-space", "Leave exactly one space around the cursor"),
    ("undo", "Undo the last change"),
    ("redo", "Redo the last undone change"),
];

/// Kills that follow each other add to the same kill-ring entry (Emacs's kill-append).
const KILLS: &[&str] = &["kill-line", "kill-whole-line", "kill-word", "backward-kill-word", "kill-region"];

pub fn is_command(name: &str) -> bool {
    COMMANDS.iter().any(|(n, _)| *n == name)
}

pub fn describe(name: &str) -> Option<&'static str> {
    COMMANDS.iter().find(|(n, _)| *n == name).map(|(_, d)| *d)
}

fn wordish(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

impl Editor {
    /// Emacs profile: modeless (always inserting); Esc and the Vim keys do nothing special.
    pub fn set_emacs(&mut self, on: bool) {
        self.emacs = on;
        self.close();
        self.mode = if on { Mode::Insert } else { Mode::Normal };
        self.mark = None;
        if !on {
            self.clamp();
        }
    }
    pub fn is_emacs(&self) -> bool {
        self.emacs
    }

    /// The active region (start, end) when the mark is set.
    pub fn region(&self) -> Option<(usize, usize)> {
        self.mark.map(|m| (m.min(self.cursor), m.max(self.cursor)))
    }

    /// Types a character (unbound printable keys). Consecutive ones are one undo step.
    pub fn self_insert(&mut self, c: char) {
        if self.last_cmd != "self-insert-command" || self.open.is_none() {
            self.begin();
        }
        self.ins(self.cursor, &c.to_string());
        self.cursor += 1;
        self.want_col = self.cursor().col;
        self.last_cmd = "self-insert-command".into();
    }

    /// Runs a named command `n` times (a numeric argument). False: no such command.
    pub fn exec(&mut self, name: &str, n: usize) -> bool {
        if !is_command(name) {
            return false;
        }
        let n = n.max(1);
        let last = std::mem::replace(&mut self.last_cmd, name.to_string());
        let appending = KILLS.contains(&name) && KILLS.contains(&last.as_str());
        let vertical = matches!(name, "next-line" | "previous-line");
        // an edit is its own undo step (typing runs are grouped in self_insert)
        let edits = !matches!(
            name,
            "forward-char" | "backward-char" | "next-line" | "previous-line" | "beginning-of-line" | "end-of-line" | "back-to-indentation"
                | "forward-word" | "backward-word" | "beginning-of-buffer" | "end-of-buffer" | "forward-paragraph" | "backward-paragraph"
                | "set-mark-command" | "exchange-point-and-mark" | "mark-whole-buffer" | "keyboard-quit" | "copy-region-as-kill" | "undo" | "redo"
        );
        if edits {
            self.begin();
        }
        let len = self.buf.len_chars();
        for _ in 0..n {
            match name {
                "forward-char" => self.cursor = (self.cursor + 1).min(len),
                "backward-char" => self.cursor = self.cursor.saturating_sub(1),
                "next-line" | "previous-line" => {
                    let line = self.cur_line();
                    let target = if name == "next-line" { (line + 1).min(self.buf.len_lines() - 1) } else { line.saturating_sub(1) };
                    let col = self.want_col.min(self.buf.line_len(target));
                    self.cursor = self.line_start(target) + col;
                }
                "beginning-of-line" => self.cursor = self.line_start(self.cur_line()),
                "end-of-line" => self.cursor = self.line_end(self.cur_line()),
                "back-to-indentation" => self.cursor = self.first_non_blank(self.cur_line()),
                "forward-word" => self.cursor = self.emacs_word_end(self.cursor),
                "backward-word" => self.cursor = self.emacs_word_start(self.cursor),
                "beginning-of-buffer" => {
                    self.mark = Some(self.cursor);
                    self.cursor = 0;
                }
                "end-of-buffer" => {
                    self.mark = Some(self.cursor);
                    self.cursor = len;
                }
                "forward-paragraph" => self.cursor = self.paragraph_edge(true),
                "backward-paragraph" => self.cursor = self.paragraph_edge(false),
                "delete-char" => {
                    if self.cursor < self.buf.len_chars() {
                        self.rem(self.cursor, self.cursor + 1);
                    }
                }
                "delete-backward-char" => {
                    if self.cursor > 0 {
                        self.rem(self.cursor - 1, self.cursor);
                        self.cursor -= 1;
                    }
                }
                "kill-line" => {
                    let end = self.line_end(self.cur_line());
                    let to = if self.cursor == end { (end + 1).min(self.buf.len_chars()) } else { end };
                    self.kill(self.cursor, to, appending, false);
                }
                "kill-whole-line" => {
                    let l = self.cur_line();
                    let (a, b) = (self.line_start(l), (self.line_end(l) + 1).min(self.buf.len_chars()));
                    self.kill(a, b, appending, false);
                }
                "kill-word" => {
                    let to = self.emacs_word_end(self.cursor);
                    self.kill(self.cursor, to, appending, false);
                }
                "backward-kill-word" => {
                    let from = self.emacs_word_start(self.cursor);
                    self.kill(from, self.cursor, appending, true);
                }
                "kill-region" => {
                    if let Some((a, b)) = self.region() {
                        self.kill(a, b, appending, false);
                    }
                    self.mark = None;
                }
                "copy-region-as-kill" => {
                    if let Some((a, b)) = self.region() {
                        let t = self.buf.slice(a, b);
                        self.push_kill(t, false, false);
                    }
                    self.mark = None;
                }
                "yank" => {
                    if let Some(t) = self.kill_ring.last().cloned() {
                        let at = self.cursor;
                        self.ins(at, &t);
                        self.cursor = at + t.chars().count();
                        self.yanked = Some((at, self.cursor, self.kill_ring.len() - 1));
                    }
                }
                "yank-pop" => {
                    if let (Some((a, b, i)), true) = (self.yanked, last == "yank" || last == "yank-pop") {
                        if self.kill_ring.len() > 1 {
                            let i = (i + self.kill_ring.len() - 1) % self.kill_ring.len();
                            let t = self.kill_ring[i].clone();
                            self.rem(a, b);
                            self.ins(a, &t);
                            self.cursor = a + t.chars().count();
                            self.yanked = Some((a, self.cursor, i));
                        }
                    }
                }
                "set-mark-command" => self.mark = Some(self.cursor),
                "exchange-point-and-mark" => {
                    if let Some(m) = self.mark {
                        self.mark = Some(self.cursor);
                        self.cursor = m;
                    }
                }
                "mark-whole-buffer" => {
                    self.mark = Some(len);
                    self.cursor = 0;
                }
                "keyboard-quit" => self.mark = None,
                "newline" => {
                    let l = self.cur_line();
                    let ind: String = self.indent_of(l).chars().take(self.cursor - self.line_start(l)).collect();
                    self.ins(self.cursor, &format!("\n{ind}"));
                    self.cursor += 1 + ind.chars().count();
                }
                "open-line" => self.ins(self.cursor, "\n"),
                "indent-for-tab-command" => {
                    self.ins(self.cursor, "\t");
                    self.cursor += 1;
                }
                "transpose-chars" => {
                    // at the end of a line, the two before; else the ones around the cursor
                    let end = self.line_end(self.cur_line());
                    let c = if self.cursor == end { self.cursor.saturating_sub(1) } else { self.cursor };
                    if c >= 1 && c < self.buf.len_chars() {
                        let pair = self.buf.slice(c - 1, c + 1);
                        let swapped: String = pair.chars().rev().collect();
                        self.rem(c - 1, c + 1);
                        self.ins(c - 1, &swapped);
                        self.cursor = c + 1;
                    }
                }
                "upcase-word" | "downcase-word" | "capitalize-word" => {
                    let to = self.emacs_word_end(self.cursor);
                    let text = self.buf.slice(self.cursor, to);
                    let out = match name {
                        "upcase-word" => text.to_uppercase(),
                        "downcase-word" => text.to_lowercase(),
                        _ => {
                            let mut seen = false;
                            text.chars()
                                .map(|c| {
                                    if wordish(c) && !seen {
                                        seen = true;
                                        c.to_uppercase().collect::<String>()
                                    } else {
                                        c.to_lowercase().collect()
                                    }
                                })
                                .collect()
                        }
                    };
                    self.rem(self.cursor, to);
                    self.ins(self.cursor, &out);
                    self.cursor += out.chars().count();
                }
                "delete-horizontal-space" | "just-one-space" => {
                    let blank = |c: Option<char>| matches!(c, Some(' ') | Some('\t'));
                    let (mut a, mut b) = (self.cursor, self.cursor);
                    while a > 0 && blank(self.char(a - 1)) {
                        a -= 1;
                    }
                    while blank(self.char(b)) {
                        b += 1;
                    }
                    self.rem(a, b);
                    self.cursor = a;
                    if name == "just-one-space" {
                        self.ins(a, " ");
                        self.cursor = a + 1;
                    }
                }
                "undo" => {
                    self.undo_once();
                    self.mode = Mode::Insert;
                }
                "redo" => {
                    self.redo_once();
                    self.mode = Mode::Insert;
                }
                _ => {}
            }
        }
        if edits {
            self.close();
        }
        if !vertical {
            self.want_col = self.cursor().col;
        }
        true
    }

    /// Removes a..b into the kill ring (appended to the last kill when kills follow each other).
    fn kill(&mut self, a: usize, b: usize, append: bool, before: bool) {
        if b <= a {
            return;
        }
        let t = self.rem(a, b);
        self.cursor = a;
        self.push_kill(t, append, before);
    }

    fn push_kill(&mut self, t: String, append: bool, before: bool) {
        match self.kill_ring.last_mut() {
            Some(last) if append => {
                if before {
                    last.insert_str(0, &t);
                } else {
                    last.push_str(&t);
                }
            }
            _ => {
                self.kill_ring.push(t);
                if self.kill_ring.len() > 120 {
                    self.kill_ring.remove(0);
                }
            }
        }
        self.yanked = None;
    }

    /// The last killed text (for the system clipboard).
    pub fn last_kill(&self) -> Option<&str> {
        self.kill_ring.last().map(String::as_str)
    }

    /// Emacs's forward-word: skip to a word, then past it.
    fn emacs_word_end(&self, mut i: usize) -> usize {
        let n = self.buf.len_chars();
        while i < n && !self.char(i).is_some_and(wordish) {
            i += 1;
        }
        while i < n && self.char(i).is_some_and(wordish) {
            i += 1;
        }
        i
    }
    fn emacs_word_start(&self, mut i: usize) -> usize {
        while i > 0 && !self.char(i - 1).is_some_and(wordish) {
            i -= 1;
        }
        while i > 0 && self.char(i - 1).is_some_and(wordish) {
            i -= 1;
        }
        i
    }
    /// The blank line after (or before) the cursor's paragraph, or the buffer's edge.
    fn paragraph_edge(&self, forward: bool) -> usize {
        let blank = |l: usize| self.buf.line(l).trim().is_empty();
        let last = self.buf.len_lines() - 1;
        let mut l = self.cur_line();
        if forward {
            while l < last && blank(l) {
                l += 1;
            }
            while l < last && !blank(l) {
                l += 1;
            }
            if l == last && !blank(l) { self.buf.len_chars() } else { self.line_start(l) }
        } else {
            while l > 0 && blank(l) {
                l -= 1;
            }
            while l > 0 && !blank(l) {
                l -= 1;
            }
            self.line_start(l)
        }
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
    fn motions() {
        let mut e = ed("hello world\nsecond line", 0);
        e.exec("forward-word", 1);
        assert_eq!(e.cursor, 5);
        e.exec("forward-word", 1);
        assert_eq!(e.cursor, 11);
        e.exec("backward-word", 1);
        assert_eq!(e.cursor, 6);
        e.exec("end-of-line", 1);
        assert_eq!(e.cursor, 11);
        e.exec("next-line", 1);
        // column 11 on a 11-char line
        assert_eq!(e.cursor().line, 1);
        assert_eq!(e.cursor().col, 11);
        e.exec("beginning-of-line", 1);
        assert_eq!(e.cursor().col, 0);
    }

    #[test]
    fn kill_line_twice_takes_the_line_break_and_appends() {
        let mut e = ed("one\ntwo\nthree", 0);
        e.exec("kill-line", 1);
        assert_eq!(e.text(), "\ntwo\nthree");
        e.exec("kill-line", 1);
        assert_eq!(e.text(), "two\nthree");
        // both kills are one entry
        assert_eq!(e.last_kill(), Some("one\n"));
        e.exec("next-line", 1);
        e.exec("yank", 1);
        assert_eq!(e.text(), "two\none\nthree");
    }

    #[test]
    fn region_kill_copy_yank_pop() {
        let mut e = ed("alpha beta gamma", 0);
        e.exec("set-mark-command", 1);
        e.exec("forward-word", 1);
        e.exec("copy-region-as-kill", 1);
        assert_eq!(e.text(), "alpha beta gamma");
        e.exec("forward-word", 1); // after "beta"
        e.exec("backward-kill-word", 1); // kills "beta"
        assert_eq!(e.text(), "alpha  gamma");
        e.exec("yank", 1);
        assert_eq!(e.text(), "alpha beta gamma");
        e.exec("yank-pop", 1);
        assert_eq!(e.text(), "alpha alpha gamma");
    }

    #[test]
    fn typing_is_one_undo_step_and_edits_undo() {
        let mut e = ed("x", 1);
        for c in "abc".chars() {
            e.self_insert(c);
        }
        assert_eq!(e.text(), "xabc");
        e.exec("undo", 1);
        assert_eq!(e.text(), "x");
        e.exec("redo", 1);
        assert_eq!(e.text(), "xabc");
    }

    #[test]
    fn case_and_space() {
        let mut e = ed("hello   world", 0);
        e.exec("capitalize-word", 1);
        assert_eq!(e.text(), "Hello   world");
        e.exec("just-one-space", 1);
        assert_eq!(e.text(), "Hello world");
        e.exec("upcase-word", 1);
        assert_eq!(e.text(), "Hello WORLD");
    }

    #[test]
    fn transpose_and_counts() {
        let mut e = ed("ab", 1);
        e.exec("transpose-chars", 1);
        assert_eq!(e.text(), "ba");
        let mut e = ed("abcdef", 0);
        e.exec("forward-char", 3);
        assert_eq!(e.cursor, 3);
        assert!(!e.exec("no-such-command", 1));
    }
}
