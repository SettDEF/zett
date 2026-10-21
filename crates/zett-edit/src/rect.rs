//! Emacs's C-x r: rectangles (the columns between the mark and the cursor, on every line between
//! them) and registers. Registers are Vim's: text goes in `"a`, a position is mark `a`, a
//! rectangle is a block register, so `C-x r s a` and `"ap` meet the same text.

use crate::{Editor, Register};

pub const COMMANDS: &[(&str, &str)] = &[
    ("kill-rectangle", "Kill the rectangle between the mark and the cursor"),
    ("delete-rectangle", "Delete the rectangle between the mark and the cursor"),
    ("copy-rectangle-as-kill", "Copy the rectangle, for yank-rectangle"),
    ("yank-rectangle", "Insert the last killed rectangle, its lines one under another at the cursor"),
    ("open-rectangle", "Insert blanks the size of the rectangle, pushing the text right"),
    ("clear-rectangle", "Replace the rectangle with blanks"),
    ("delete-whitespace-rectangle", "Delete the blanks at the rectangle's left edge on each line"),
    ("rectangle-number-lines", "Number the rectangle's lines at its left edge"),
];

/// Commands that read a register name (a key) next: their prompts.
pub const REGISTER_COMMANDS: &[(&str, &str)] = &[
    ("copy-to-register", "Copy the region to a register (with an argument: kill it)"),
    ("insert-register", "Insert a register's text (a rectangle as one)"),
    ("point-to-register", "Remember the cursor's place in a register (Vim's mark)"),
    ("jump-to-register", "Go to the place a register remembers"),
    ("number-to-register", "Put the argument's number in a register"),
    ("increment-register", "Add the argument to a register's number (text: append the region)"),
    ("copy-rectangle-to-register", "Copy the rectangle to a register"),
    ("append-to-register", "Add the region to the end of a register"),
    ("prepend-to-register", "Add the region to the start of a register"),
];

pub fn is_command(name: &str) -> bool {
    COMMANDS.iter().any(|(n, _)| *n == name)
}

impl Editor {
    /// The rectangle: its lines and its columns [c1, c2).
    fn rect_span(&self) -> Option<(usize, usize, usize, usize)> {
        let m = self.mark?;
        let (a, b) = (self.buf.to_pos(m), self.buf.to_pos(self.cursor));
        Some((a.line.min(b.line), a.line.max(b.line), a.col.min(b.col), a.col.max(b.col)))
    }
    /// Line `l`'s columns [c1, c2), blank-padded where the line is shorter.
    fn rect_cut(&self, l: usize, c1: usize, c2: usize) -> String {
        let text: Vec<char> = self.buf.line(l).trim_end_matches('\n').chars().collect();
        (c1..c2).map(|c| text.get(c).copied().unwrap_or(' ')).collect()
    }
    /// Each line's [c1, c2) replaced by what `f` makes of it (one undo step).
    fn rect_edit(&mut self, l1: usize, l2: usize, c1: usize, c2: usize, mut f: impl FnMut(usize, &str) -> Option<String>) {
        self.begin();
        for (k, l) in (l1..=l2).enumerate() {
            let len = self.buf.line_len(l);
            let s = self.line_start(l);
            let (a, b) = (s + c1.min(len), s + c2.min(len));
            let old = self.buf.slice(a, b);
            if let Some(new) = f(k, &old) {
                // a line too short for the left edge is padded to it
                let pad = if new.is_empty() { String::new() } else { " ".repeat(c1.saturating_sub(len)) };
                self.rem(a, b);
                self.ins(a, &format!("{pad}{new}"));
            }
        }
        self.close();
    }
    /// The rectangle's lines put in at the cursor, one under another (lines added at the end).
    pub(crate) fn insert_rect(&mut self, lines: &[String]) {
        let pos = self.cursor();
        self.begin();
        for (k, t) in lines.iter().enumerate() {
            let l = pos.line + k;
            if l >= self.buf.len_lines() {
                let end = self.buf.len_chars();
                self.ins(end, "\n");
            }
            let len = self.buf.line_len(l);
            let at = self.line_start(l) + pos.col.min(len);
            let t = format!("{}{t}", " ".repeat(pos.col.saturating_sub(len)));
            self.ins(at, &t);
            self.cursor = at + t.chars().count();
        }
        self.close();
    }

    /// :string-rectangle (C-x r t): each line's columns replaced by `s`.
    pub fn string_rectangle(&mut self, s: &str) {
        let Some((l1, l2, c1, c2)) = self.rect_span() else { return };
        self.rect_edit(l1, l2, c1, c2, |_, _| Some(s.to_string()));
        self.mark_active = false;
        self.cursor = self.line_start(l2) + c1 + s.chars().count();
    }

    pub(crate) fn rect_exec(&mut self, name: &str) -> bool {
        self.last_cmd = name.to_string();
        let Some((l1, l2, c1, c2)) = self.rect_span() else { return true };
        match name {
            "kill-rectangle" | "copy-rectangle-as-kill" => {
                let lines: Vec<String> = (l1..=l2).map(|l| self.rect_cut(l, c1, c2)).collect();
                self.ring().rect = lines;
                if name == "kill-rectangle" {
                    self.rect_edit(l1, l2, c1, c2, |_, _| Some(String::new()));
                    self.cursor = self.line_start(l1) + c1.min(self.buf.line_len(l1));
                }
            }
            "delete-rectangle" => {
                self.rect_edit(l1, l2, c1, c2, |_, _| Some(String::new()));
                self.cursor = self.line_start(l1) + c1.min(self.buf.line_len(l1));
            }
            "yank-rectangle" => {
                let lines = self.ring().rect.clone();
                self.insert_rect(&lines);
            }
            "open-rectangle" => {
                let w = c2 - c1;
                self.rect_edit(l1, l2, c1, c1, |_, _| Some(" ".repeat(w)));
                self.cursor = self.line_start(l1) + c1;
            }
            "clear-rectangle" => self.rect_edit(l1, l2, c1, c2, |_, old| Some(" ".repeat(old.chars().count()))),
            "delete-whitespace-rectangle" => {
                self.begin();
                for l in l1..=l2 {
                    let s = self.line_start(l);
                    let len = self.buf.line_len(l);
                    if c1 >= len {
                        continue;
                    }
                    let mut e = s + c1;
                    while self.char(e).is_some_and(|c| c == ' ' || c == '\t') {
                        e += 1;
                    }
                    self.rem(s + c1, e);
                }
                self.close();
            }
            "rectangle-number-lines" => {
                let w = (l2 - l1 + 1).to_string().len();
                self.rect_edit(l1, l2, c1, c1, |k, _| Some(format!("{:>w$} ", k + 1)));
            }
            _ => return false,
        }
        self.mark_active = false;
        self.want_col = self.cursor().col;
        true
    }

    /// The prompt for a command that reads a key next (a register, zap-to-char's character).
    pub fn char_prompt(name: &str) -> Option<&'static str> {
        match name {
            "copy-to-register" => Some("Copy to register: "),
            "insert-register" => Some("Insert register: "),
            "point-to-register" => Some("Point to register: "),
            "jump-to-register" => Some("Jump to register: "),
            "number-to-register" => Some("Number to register: "),
            "increment-register" => Some("Increment register: "),
            "copy-rectangle-to-register" => Some("Copy rectangle to register: "),
            "append-to-register" => Some("Append to register: "),
            "prepend-to-register" => Some("Prepend to register: "),
            "zap-to-char" => Some("Zap to char: "),
            "zap-up-to-char" => Some("Zap up to char: "),
            "quoted-insert" => Some("C-q-"),
            _ => None,
        }
    }

    /// Runs a command that read a key: `c` is the register or the character; `arg` the numeric
    /// argument (None: none given).
    pub fn exec_char(&mut self, name: &str, c: char, arg: Option<i64>) -> bool {
        let n = arg.unwrap_or(1);
        let region = self.mark_span();
        let rev = self.buf.rev();
        match name {
            "zap-to-char" | "zap-up-to-char" => {
                // the nth c ahead (behind with a negative argument), killed through (up to) it
                let back = n < 0;
                let mut at = self.cursor;
                let mut found = None;
                for _ in 0..n.unsigned_abs().max(1) {
                    let hit = if back { (0..at).rev().find(|&i| self.char(i) == Some(c)) } else { (at + 1..self.buf.len_chars()).find(|&i| self.char(i) == Some(c)).or_else(|| (self.char(at) == Some(c) && found.is_none()).then_some(at)) };
                    let Some(h) = hit else { break };
                    found = Some(h);
                    at = h;
                }
                let Some(h) = found else { return true };
                let up = name == "zap-up-to-char";
                let (a, b) = if back { (if up { h + 1 } else { h }, self.cursor) } else { (self.cursor, if up { h } else { h + 1 }) };
                if b > a {
                    self.begin();
                    let t = self.rem(a, b);
                    self.cursor = a;
                    self.ring().push(t, false, back);
                    self.yanked = None;
                    self.close();
                }
            }
            "quoted-insert" => {
                self.begin();
                for _ in 0..n.max(1) {
                    self.ins(self.cursor, &c.to_string());
                    self.cursor += 1;
                }
                self.close();
            }
            "copy-to-register" | "append-to-register" | "prepend-to-register" => {
                let Some((a, b)) = region else { return true };
                let t = self.buf.slice(a, b);
                let old = self.register(c).map(|r| r.0).unwrap_or_default();
                let new = match name {
                    "append-to-register" => format!("{old}{t}"),
                    "prepend-to-register" => format!("{t}{old}"),
                    _ => t,
                };
                self.set_register(c, &new);
                // C-u C-x r s: the region goes too
                if arg.is_some() && name == "copy-to-register" {
                    self.begin();
                    self.rem(a, b);
                    self.cursor = a;
                    self.close();
                }
            }
            "copy-rectangle-to-register" => {
                let Some((l1, l2, c1, c2)) = self.rect_span() else { return true };
                let text = (l1..=l2).map(|l| self.rect_cut(l, c1, c2)).collect::<Vec<_>>().join("\n");
                self.vim.regs.insert(c.to_ascii_lowercase(), Register { text, linewise: false, block: true });
            }
            "insert-register" => {
                let Some(r) = self.vim.regs.get(&c.to_ascii_lowercase()).cloned().or_else(|| self.register(c).map(|(text, linewise)| Register { text, linewise, block: false })) else {
                    return true;
                };
                let at = self.cursor;
                if r.block {
                    let lines: Vec<String> = r.text.split('\n').map(str::to_string).collect();
                    self.insert_rect(&lines);
                    self.cursor = at;
                } else {
                    self.begin();
                    self.ins(at, &r.text);
                    self.close();
                    // as Emacs: the cursor before the text, the mark after it (C-u: the other way)
                    self.mark = Some(at + r.text.chars().count());
                    if arg.is_some() {
                        self.cursor = at + r.text.chars().count();
                        self.mark = Some(at);
                    }
                }
            }
            "point-to-register" => self.set_vim_mark(c, self.cursor),
            "jump-to-register" => {
                let Some(&p) = self.vim.marks.get(&c) else { return true };
                self.set_mark(false);
                self.cursor = p.min(self.buf.len_chars());
            }
            "number-to-register" => self.set_register(c, &arg.unwrap_or(0).to_string()),
            "increment-register" => {
                let old = self.register(c).map(|r| r.0).unwrap_or_default();
                match old.trim().parse::<i64>() {
                    Ok(v) => self.set_register(c, &(v + n).to_string()),
                    Err(_) => {
                        if let Some((a, b)) = region {
                            let t = self.buf.slice(a, b);
                            self.set_register(c, &format!("{old}{t}"));
                        }
                    }
                }
            }
            _ => return false,
        }
        self.last_cmd = name.to_string();
        if self.buf.rev() != rev {
            self.mark_active = false;
        }
        self.want_col = self.cursor().col;
        true
    }
}

#[cfg(test)]
mod tests {
    use crate::Editor;

    fn ed(text: &str, mark: usize, at: usize) -> Editor {
        let mut e = Editor::from_str(text);
        e.set_emacs(true);
        e.cursor = mark;
        e.exec("set-mark-command", 1);
        e.cursor = at;
        e
    }

    #[test]
    fn kill_and_yank_a_rectangle() {
        // columns 1..3 of three lines; the short middle line is padded
        let mut e = ed("abcd\nx\nefgh", 1, 10);
        e.exec("kill-rectangle", 1);
        assert_eq!(e.text(), "ad\nx\neh");
        e.cursor = 0;
        e.exec("yank-rectangle", 1);
        assert_eq!(e.text(), "bcad\n  x\nfgeh");
    }

    #[test]
    fn open_clear_string_number() {
        let mut e = ed("abc\ndef", 1, 6);
        e.exec("open-rectangle", 1);
        assert_eq!(e.text(), "a bc\nd ef");
        let mut e = ed("abc\ndef", 1, 6);
        e.exec("clear-rectangle", 1);
        assert_eq!(e.text(), "a c\nd f");
        let mut e = ed("abc\ndef", 1, 6);
        e.string_rectangle("XY");
        assert_eq!(e.text(), "aXYc\ndXYf");
        let mut e = ed("a\nb", 0, 2);
        e.exec("rectangle-number-lines", 1);
        assert_eq!(e.text(), "1 a\n2 b");
    }

    #[test]
    fn registers_are_vims() {
        let mut e = ed("hello world", 0, 5);
        e.exec_char("copy-to-register", 'a', None);
        assert_eq!(e.register('a').map(|r| r.0).as_deref(), Some("hello"));
        e.cursor = 11;
        e.exec_char("insert-register", 'a', None);
        assert_eq!(e.text(), "hello worldhello");
        e.cursor = 6;
        e.exec_char("point-to-register", 'p', None);
        assert!(e.marks().iter().any(|m| m.0 == 'p' && m.2 == 6), "a Vim mark");
        e.cursor = 0;
        e.exec_char("jump-to-register", 'p', None);
        assert_eq!(e.cursor, 6);
        e.exec_char("number-to-register", 'n', Some(41));
        e.exec_char("increment-register", 'n', None);
        assert_eq!(e.register('n').map(|r| r.0).as_deref(), Some("42"));
        let mut e = ed("ab\ncd", 0, 4);
        e.exec_char("copy-rectangle-to-register", 'r', None);
        e.cursor = 5;
        e.exec_char("insert-register", 'r', None);
        assert_eq!(e.text(), "ab\ncda\n  c", "a rectangle register goes in as one");
    }

    #[test]
    fn zap_and_quote() {
        let mut e = ed("one, two, three", 0, 0);
        e.exec_char("zap-to-char", ',', Some(2));
        assert_eq!(e.text(), " three");
        let mut e = ed("one, two", 0, 0);
        e.exec_char("zap-up-to-char", ',', None);
        assert_eq!(e.text(), ", two");
        let mut e = ed("ab", 0, 1);
        e.exec_char("quoted-insert", '\t', None);
        assert_eq!(e.text(), "a\tb");
    }
}
