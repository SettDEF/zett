//! Emacs's window-relative commands: C-v M-v (a screen, two lines kept), C-l and M-r (middle, top,
//! bottom in turn), C-x < >, and the ones that tell where you are (C-x =, M-=, C-x l) or go to a
//! number (M-g g, M-g c, M-g TAB). The window's size comes from set_screen, as Vim's H M L use it.

use crate::vim::Request;
use crate::Editor;

pub const COMMANDS: &[(&str, &str)] = &[
    ("scroll-up-command", "Show the next screen (two lines stay); with an argument, that many lines"),
    ("scroll-down-command", "Show the previous screen (two lines stay); with an argument, that many lines"),
    ("scroll-up-line", "The text up a line (the argument's lines), the cursor staying on its line"),
    ("scroll-down-line", "The text down a line (the argument's lines), the cursor staying on its line"),
    ("scroll-lock-next-line", "The text up a line, the cursor where it is on the screen (scroll-lock-mode's C-n)"),
    ("set-selective-display", "Hide the lines indented at least the argument's columns (none: show them all again)"),
    ("scroll-lock-previous-line", "The text down a line, the cursor where it is on the screen (scroll-lock-mode's C-p)"),
    ("recenter-top-bottom", "The cursor's line to the middle of the window, then the top, then the bottom"),
    ("move-to-window-line-top-bottom", "The cursor to the window's middle line, then its top, then its bottom"),
    ("scroll-left", "Scroll the text left (a screen, or the argument's columns)"),
    ("scroll-right", "Scroll the text right (a screen, or the argument's columns)"),
    ("what-cursor-position", "The character at the cursor (its codes) and where the cursor is"),
    ("count-words-region", "Lines, words and characters in the region (or the buffer)"),
    ("count-words", "Lines, words and characters in the region (or the buffer)"),
    ("count-lines-page", "Lines in the buffer, before and after the cursor"),
    ("goto-line", "Go to a line (the argument's, or asks)"),
    ("goto-char", "Go to a character position (the argument's, or asks)"),
    ("move-to-column", "Go to a column of this line (the argument's, or asks)"),
];

/// These ask for their number when no argument is given (the front end's prompt).
pub fn asks_number(name: &str) -> Option<&'static str> {
    match name {
        "goto-line" => Some("Goto line: "),
        "goto-char" => Some("Goto char: "),
        "move-to-column" => Some("Move to column: "),
        _ => None,
    }
}

pub fn is_command(name: &str) -> bool {
    COMMANDS.iter().any(|(n, _)| *n == name)
}

impl Editor {
    pub(crate) fn screen_exec(&mut self, name: &str, arg: Option<i64>) -> bool {
        let again = self.last_cmd == name;
        self.last_cmd = name.to_string();
        let (top, height, width) = self.vim.screen;
        let height = height.max(3);
        let last = self.buf.len_lines() - 1;
        let line = self.cur_line();
        match name {
            "scroll-up-command" | "scroll-down-command" => {
                let by = arg.unwrap_or(height.saturating_sub(2).max(1) as i64);
                let by = if name == "scroll-down-command" { -by } else { by };
                if by > 0 && top >= last {
                    self.vim.requests.push(Request::Message("End of buffer".into()));
                    self.cursor = self.buf.len_chars();
                    return true;
                }
                if by < 0 && top == 0 {
                    self.vim.requests.push(Request::Message("Beginning of buffer".into()));
                    self.cursor = 0;
                    return true;
                }
                let t = (top as i64 + by).clamp(0, last as i64) as usize;
                self.vim_top(t);
                // the cursor stays on the screen, at its edge when it fell off
                let to = line.clamp(t, (t + height - 1).min(last));
                if to != line {
                    self.vertical_to(to);
                }
            }
            // by lines: the text moves; the cursor stays on its line (scroll-up/down-line) or, with
            // scroll-lock-mode, where it is on the screen; either way it stays inside the window
            "scroll-up-line" | "scroll-down-line" | "scroll-lock-next-line" | "scroll-lock-previous-line" => {
                let n = arg.unwrap_or(1).max(1) as usize;
                let up = matches!(name, "scroll-up-line" | "scroll-lock-next-line");
                let t = if up { (top + n).min(last) } else { top.saturating_sub(n) };
                self.vim_top(t);
                let want = match (name.starts_with("scroll-lock"), up) {
                    (true, true) => (line + (t - top)).min(last),
                    (true, false) => line.saturating_sub(top - t),
                    (false, _) => line,
                };
                let to = want.clamp(t, (t + height - 1).min(last));
                if to != line {
                    self.vertical_to(to);
                }
            }
            // C-x $: each run of lines indented at least N columns folded under the line above it
            // (Emacs shows that line with "..."); with no argument the folds it made go again
            "set-selective-display" => {
                let made = std::mem::take(&mut self.selective);
                self.vim.folds.retain(|&(a, b, _)| !made.contains(&(a, b)));
                if let Some(n) = arg.filter(|n| *n > 0).map(|n| n as usize) {
                    let tab = self.opt_num("tabstop", 8).max(1) as usize;
                    let indent = |l: usize| -> Option<usize> {
                        let s: String = self.buf.line(l).chars().filter(|c| *c != '\n').collect();
                        if s.trim().is_empty() {
                            return None;
                        }
                        Some(s.chars().take_while(|c| *c == ' ' || *c == '\t').fold(0, |w, c| if c == '\t' { w / tab * tab + tab } else { w + 1 }))
                    };
                    let mut runs = Vec::new();
                    let mut l = 1;
                    while l <= last {
                        // a run: deep lines (blank ones inside it go with it)
                        if indent(l).is_some_and(|w| w >= n) {
                            let start = l;
                            let mut end = l;
                            while end < last && indent(end + 1).map_or(true, |w| w >= n) {
                                end += 1;
                            }
                            while end > start && indent(end).is_none() {
                                end -= 1;
                            }
                            runs.push((start - 1, end));
                            l = end + 1;
                        } else {
                            l += 1;
                        }
                    }
                    for (a, b) in runs {
                        let span = (self.line_start(a), self.line_end(b));
                        self.vim.folds.push((span.0, span.1, true));
                        self.selective.push(span);
                    }
                    self.vim.folds_off = false;
                }
                self.folds_changed();
            }
            "recenter-top-bottom" => {
                self.cycle = if again { (self.cycle + 1) % 3 } else { 0 };
                let row = match arg {
                    Some(n) if n >= 0 => n as usize,
                    Some(n) => height.saturating_sub(n.unsigned_abs() as usize),
                    None => [height / 2, 0, height - 1][self.cycle as usize],
                };
                self.vim_top(line.saturating_sub(row));
            }
            "move-to-window-line-top-bottom" => {
                self.cycle = if again { (self.cycle + 1) % 3 } else { 0 };
                let shown = (last + 1 - top).min(height);
                let row = match arg {
                    Some(n) if n >= 0 => n as usize,
                    Some(n) => shown.saturating_sub(n.unsigned_abs() as usize),
                    None => [shown / 2, 0, shown.saturating_sub(1)][self.cycle as usize],
                };
                let to = (top + row).min(last);
                self.cursor = self.first_non_blank(to);
            }
            "scroll-left" | "scroll-right" => {
                let by = arg.unwrap_or(width.saturating_sub(2).max(1) as i64) as i32;
                self.vim.requests.push(Request::Sideways(if name == "scroll-left" { by } else { -by }));
            }
            "what-cursor-position" => {
                let n = self.buf.len_chars();
                let m = match self.char(self.cursor) {
                    Some(c) => {
                        let shown = match c {
                            '\n' => "C-j".to_string(),
                            '\t' => "TAB".to_string(),
                            ' ' => "SPC".to_string(),
                            c if (c as u32) < 32 => format!("C-{}", char::from(c as u8 + b'a' - 1)),
                            c => c.to_string(),
                        };
                        let u = c as u32;
                        format!("Char: {shown} ({u}, #o{u:o}, #x{u:x}) point={} of {n} ({}%) column={}", self.cursor + 1, self.cursor * 100 / n.max(1), self.cursor().col)
                    }
                    None => format!("point={} of {n} (EOB) column={}", self.cursor + 1, self.cursor().col),
                };
                self.vim.requests.push(Request::Message(m));
            }
            "count-words-region" | "count-words" => {
                let (a, b, what) = match self.region() {
                    Some((a, b)) => (a, b, "Region"),
                    None => (0, self.buf.len_chars(), "Buffer"),
                };
                let t = self.buf.slice(a, b);
                let lines = if t.is_empty() { 0 } else { t.matches('\n').count() + usize::from(!t.ends_with('\n')) };
                let m = format!("{what} has {lines} line{}, {} word{}, and {} character{}.", if lines == 1 { "" } else { "s" }, t.split_whitespace().count(), if t.split_whitespace().count() == 1 { "" } else { "s" }, t.chars().count(), if t.chars().count() == 1 { "" } else { "s" });
                self.vim.requests.push(Request::Message(m));
            }
            "count-lines-page" => {
                // the page between form feeds (the buffer, when it has none)
                let (a, b) = self.page_here();
                let (la, lb) = (self.buf.to_pos(a).line, self.buf.to_pos(b).line);
                let total = lb - la + usize::from(b == self.buf.len_chars());
                self.vim.requests.push(Request::Message(format!("Page has {total} lines ({} + {})", line - la, total - (line - la))));
            }
            "goto-line" | "goto-char" | "move-to-column" => {
                let Some(n) = arg else { return true };
                self.set_mark(false);
                self.mark_active = false;
                match name {
                    "goto-line" => self.cursor = self.line_start((n.max(1) as usize - 1).min(last)),
                    "goto-char" => self.cursor = (n.max(1) as usize - 1).min(self.buf.len_chars()),
                    _ => self.cursor = self.line_start(line) + (n.max(0) as usize).min(self.buf.line_len(line)),
                }
                if !self.view_is_empty() {
                    self.reveal();
                }
            }
            _ => return false,
        }
        self.want_col = self.cursor().col;
        true
    }
}

#[cfg(test)]
mod tests {
    use crate::vim::Request;
    use crate::Editor;

    fn ed(lines: usize) -> Editor {
        let text: String = (0..lines).map(|i| format!("line {i}\n")).collect();
        let mut e = Editor::from_str(&text);
        e.set_emacs(true);
        e.set_screen(0, 10, 80);
        e
    }

    #[test]
    fn a_screen_keeps_two_lines() {
        let mut e = ed(50);
        e.exec_arg("scroll-up-command", None);
        assert!(e.take_requests().contains(&Request::Top(8)));
        assert_eq!(e.cursor().line, 8, "the cursor fell off the top: it moves to the new top");
        e.set_screen(8, 10, 80);
        e.exec_arg("scroll-down-command", None);
        assert!(e.take_requests().contains(&Request::Top(0)));
    }

    #[test]
    fn recenter_cycles_middle_top_bottom() {
        let mut e = ed(50);
        e.exec_arg("goto-line", Some(21));
        assert_eq!(e.cursor().line, 20);
        for want in [15, 20, 11] {
            e.exec_arg("recenter-top-bottom", None);
            assert!(e.take_requests().contains(&Request::Top(want)), "{want}");
        }
        e.set_screen(10, 10, 80);
        e.exec_arg("move-to-window-line-top-bottom", None);
        assert_eq!(e.cursor().line, 15);
        e.exec_arg("move-to-window-line-top-bottom", None);
        assert_eq!(e.cursor().line, 10);
    }

    #[test]
    fn where_and_how_much() {
        let mut e = ed(2);
        e.exec_arg("what-cursor-position", None);
        assert!(matches!(&e.take_requests()[..], [Request::Message(m)] if m.starts_with("Char: l (108, #o154, #x6c) point=1 of 14")));
        e.exec_arg("count-words-region", None);
        assert!(matches!(&e.take_requests()[..], [Request::Message(m)] if m == "Buffer has 2 lines, 4 words, and 14 characters."));
        e.exec_arg("move-to-column", Some(3));
        assert_eq!(e.cursor().col, 3);
    }
}

#[cfg(test)]
mod line_scroll_tests {
    use crate::vim::Request;
    use crate::Editor;

    fn ed() -> Editor {
        let text: String = (0..100).map(|i| format!("line {i}\n")).collect();
        let mut e = Editor::from_str(&text);
        e.set_emacs(true);
        e.set_screen(0, 10, 80);
        e
    }

    #[test]
    fn scroll_lock_keeps_the_cursor_where_it_is_on_screen() {
        let mut e = ed();
        e.exec_arg("goto-line", Some(4));
        e.exec_arg("scroll-lock-next-line", None);
        e.exec_arg("scroll-lock-next-line", None);
        assert_eq!(e.take_requests(), vec![Request::Top(1), Request::Top(2)]);
        assert_eq!(e.cursor().line, 5, "moved with the text: still the 4th row on screen");
        e.exec_arg("scroll-lock-previous-line", None);
        assert_eq!(e.cursor().line, 4);
    }

    #[test]
    fn scroll_up_line_keeps_the_cursor_on_its_line_until_it_would_leave() {
        let mut e = ed();
        e.exec_arg("goto-line", Some(4));
        e.exec_arg("scroll-up-line", Some(2));
        assert_eq!(e.cursor().line, 3, "line 4 is still on screen");
        e.exec_arg("scroll-up-line", Some(3));
        assert_eq!(e.cursor().line, 5, "pushed to the new top");
    }
}

#[cfg(test)]
mod selective_tests {
    use crate::Editor;

    #[test]
    fn selective_display_folds_deep_lines_under_their_parent() {
        let mut e = Editor::from_str("fn a() {\n    let x = 1;\n    if x {\n        deep();\n    }\n}\nfn b() {}\n");
        e.set_emacs(true);
        e.exec_arg("set-selective-display", Some(4));
        // lines 1-4 (the body) folded under line 0; line 5 and after shown
        assert_eq!(e.fold_mark(0).1, Some(true));
        assert_eq!(e.fold_mark(6).0, 0);
        // with no argument: shown again
        e.exec_arg("set-selective-display", None);
        assert!(!e.has_folds());
    }
}
