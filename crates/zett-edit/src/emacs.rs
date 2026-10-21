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
    ("cycle-spacing", "One space around the cursor; again: none; again: as it was"),
    ("tab-to-tab-stop", "Insert blanks to the next tab stop"),
    ("split-line", "Move the rest of the line down, starting at the cursor's column"),
    ("default-indent-new-line", "A line break continuing the comment (or the list item)"),
    ("append-next-kill", "The next kill adds to the last one"),
    ("indent-rigidly", "Shift the region's lines right by the argument's columns (negative: left; none: a shift width)"),
    ("indent-rigidly-left", "Shift the region's lines one column left"),
    ("indent-rigidly-right", "Shift the region's lines one column right"),
    ("indent-region", "Reindent the region's lines"),
    ("delete-trailing-whitespace", "Delete blanks at the ends of lines (the region's, or the buffer's)"),
    ("delete-region", "Delete the region, without keeping it in the kill ring"),
    ("zap-to-char", "Kill through the next occurrence of a character (reads it)"),
    ("zap-up-to-char", "Kill up to the next occurrence of a character (reads it)"),
    ("quoted-insert", "Insert the next key as it is (a tab, a control character)"),
    // Emacs's names for commands above
    ("left-char", "Move one character back (backward-char)"),
    ("right-char", "Move one character forward (forward-char)"),
    ("left-word", "Move to the start of the previous word (backward-word)"),
    ("right-word", "Move past the end of the next word (forward-word)"),
    ("move-beginning-of-line", "Move to the start of the line (beginning-of-line)"),
    ("move-end-of-line", "Move to the end of the line (end-of-line)"),
    ("kill-ring-save", "Copy the region to the kill ring (copy-region-as-kill)"),
    ("undo-redo", "Redo the last undone change (redo)"),
    ("undo-only", "Undo, never redo (undo)"),
    ("delete-indentation", "Join this line to the one before (join-line)"),
    ("backward-delete-char-untabify", "Delete the character before the cursor (delete-backward-char)"),
    ("newline-and-indent", "A line break, keeping the indentation (newline)"),
    ("fixup-whitespace", "One space around the cursor (just-one-space)"),
    ("keyboard-escape-quit", "Cancel: deactivate the mark (keyboard-quit)"),
];

/// Emacs's other names for zett's commands: (name, the command it runs).
const ALIASES: &[(&str, &str)] = &[
    ("left-char", "backward-char"),
    ("right-char", "forward-char"),
    ("left-word", "backward-word"),
    ("right-word", "forward-word"),
    ("move-beginning-of-line", "beginning-of-line"),
    ("move-end-of-line", "end-of-line"),
    ("kill-ring-save", "copy-region-as-kill"),
    ("undo-redo", "redo"),
    ("undo-only", "undo"),
    ("delete-indentation", "join-line"),
    ("backward-delete-char-untabify", "delete-backward-char"),
    ("newline-and-indent", "newline"),
    ("fixup-whitespace", "just-one-space"),
    ("keyboard-escape-quit", "keyboard-quit"),
];

/// The command an Emacs name stands for (itself when it is no other name).
pub fn resolve(name: &str) -> &str {
    ALIASES.iter().find(|a| a.0 == name).map_or(name, |a| a.1)
}

/// Kills that follow each other add to the same kill-ring entry (Emacs's kill-append).
const KILLS: &[&str] = &["kill-line", "kill-whole-line", "kill-word", "backward-kill-word", "kill-region"];

/// Every named editor command: the editing ones here, the search ones (search.rs) and the
/// numeric argument (prefix.rs, which the front end runs).
pub fn all() -> impl Iterator<Item = &'static (&'static str, &'static str)> {
    COMMANDS.iter().chain(crate::search::COMMANDS.iter()).chain(crate::prefix::COMMANDS.iter()).chain(crate::outline::COMMANDS.iter()).chain(crate::suggest::COMMANDS.iter()).chain(crate::textops::COMMANDS.iter()).chain(crate::balanced::COMMANDS.iter()).chain(crate::rect::COMMANDS.iter()).chain(crate::rect::REGISTER_COMMANDS.iter()).chain(crate::screen::COMMANDS.iter())
}

pub fn is_command(name: &str) -> bool {
    all().any(|(n, _)| *n == name)
}

pub fn describe(name: &str) -> Option<&'static str> {
    all().find(|(n, _)| *n == name).map(|(_, d)| *d)
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
        self.mark_active = false;
        if !on {
            self.clamp();
        }
    }
    pub fn is_emacs(&self) -> bool {
        self.emacs
    }

    /// Shows a..b as the region, the cursor at b (a query-replace match).
    pub fn set_region(&mut self, a: usize, b: usize) {
        self.mark = Some(a.min(self.buf.len_chars()));
        self.cursor = b.min(self.buf.len_chars());
        self.mark_active = true;
    }

    /// a..b becomes `t` (one undo step), the cursor after it.
    pub fn replace_text(&mut self, a: usize, b: usize, t: &str) {
        self.begin();
        self.rem(a, b);
        self.ins(a, t);
        self.close();
        self.cursor = a + t.chars().count();
        self.mark_active = false;
    }

    /// The active region (start, end): set with C-SPC, gone after an edit, M-w or C-g.
    pub fn region(&self) -> Option<(usize, usize)> {
        self.mark_active.then(|| self.mark_span()).flatten()
    }

    /// Point to mark, shown or not (C-w kills it either way, as Emacs does).
    pub(crate) fn mark_span(&self) -> Option<(usize, usize)> {
        self.mark.map(|m| (m.min(self.cursor), m.max(self.cursor)))
    }

    /// Leaves a mark behind before a long jump (M-<, M->), unless a region is being made.
    fn push_mark(&mut self) {
        if !self.mark_active {
            self.set_mark(false);
        }
    }

    /// Sets the mark where the cursor is; `active` shows it as the region.
    pub(crate) fn set_mark(&mut self, active: bool) {
        self.mark = Some(self.cursor);
        self.mark_active = active;
    }

    /// Types a character (unbound printable keys). Consecutive ones are one undo step.
    pub fn self_insert(&mut self, c: char) {
        if self.last_cmd != "self-insert-command" || self.open.is_none() {
            self.begin();
        }
        self.mark_active = false;
        self.ins(self.cursor, &c.to_string());
        self.cursor += 1;
        self.want_col = self.cursor().col;
        self.last_cmd = "self-insert-command".into();
    }

    /// Runs a command with a numeric argument (prefix.rs), as Emacs does: None runs it once;
    /// negative runs its opposite (M-- M-f = M-b); `C-u N C-k` kills N whole lines.
    pub fn exec_arg(&mut self, name: &str, arg: Option<i64>) -> bool {
        let name = resolve(name);
        let n = arg.unwrap_or(1);
        if crate::screen::is_command(name) {
            return self.screen_exec(name, arg);
        }
        // these read the argument's sign themselves
        if crate::balanced::is_command(name) {
            return self.balanced_exec(name, n);
        }
        if name == "indent-rigidly" {
            let cols = arg.unwrap_or(self.shiftwidth().max(1) as i64);
            self.last_cmd = name.to_string();
            self.indent_rigidly(cols);
            return true;
        }
        if name == "kill-line" && arg.is_some() && n > 0 {
            if !is_command(name) {
                return false;
            }
            let last = std::mem::replace(&mut self.last_cmd, name.to_string());
            let appending = KILLS.contains(&last.as_str());
            let l = self.cur_line() + n as usize;
            let to = if l < self.buf.len_lines() { self.line_start(l) } else { self.buf.len_chars() };
            self.begin();
            self.kill(self.cursor, to, appending, false);
            self.mark_active = false;
            self.close();
            self.want_col = self.cursor().col;
            return true;
        }
        match crate::prefix::opposite(name) {
            Some(o) if n < 0 => self.exec(o, n.unsigned_abs() as usize),
            _ => self.exec(name, n.unsigned_abs().max(1) as usize),
        }
    }

    /// Runs a named command `n` times (a numeric argument). False: no such command.
    pub fn exec(&mut self, name: &str, n: usize) -> bool {
        if !is_command(name) {
            return false;
        }
        let name = resolve(name);
        let n = n.max(1);
        if crate::balanced::is_command(name) {
            return self.balanced_exec(name, n as i64);
        }
        if crate::rect::is_command(name) {
            return self.rect_exec(name);
        }
        if crate::screen::is_command(name) {
            return self.screen_exec(name, (n > 1).then_some(n as i64));
        }
        match name {
            "indent-rigidly" | "indent-rigidly-left" | "indent-rigidly-right" => {
                self.last_cmd = name.to_string();
                let by = match name { "indent-rigidly-left" => -(n as i64), "indent-rigidly-right" => n as i64, _ => self.shiftwidth().max(1) as i64 };
                self.indent_rigidly(by);
                return true;
            }
            "indent-region" => {
                self.last_cmd = name.to_string();
                let (l1, l2) = match self.mark_span() {
                    Some((a, b)) if self.mark_active => (self.buf.to_pos(a).line, self.buf.to_pos(b.saturating_sub(1).max(a)).line),
                    _ => (self.cur_line(), self.cur_line()),
                };
                let at = self.cursor;
                self.vim_apply(crate::Op::Reindent, crate::Span::Lines(l1, l2));
                self.cursor = at.min(self.buf.len_chars());
                self.mark_active = false;
                return true;
            }
            "append-next-kill" => {
                // the next kill sees a kill before it, and adds to it
                self.last_cmd = "kill-region".into();
                return true;
            }
            // they read a key first: the front end asks for it (rect.rs char_prompt, exec_char)
            "zap-to-char" | "zap-up-to-char" | "quoted-insert" => return true,
            _ => {}
        }
        if crate::rect::REGISTER_COMMANDS.iter().any(|r| r.0 == name) {
            return true;
        }
        if crate::suggest::is_command(name) {
            self.last_cmd = name.to_string();
            return self.suggest_exec(name);
        }
        if crate::textops::is_command(name) {
            let last = std::mem::replace(&mut self.last_cmd, name.to_string());
            self.last_cmd_was_mark_word = last == "mark-word";
            return self.textops_exec(name, n as i64);
        }
        if crate::outline::is_command(name) {
            self.last_cmd = name.to_string();
            return self.outline_exec(name, n);
        }
        // Tab on a Markdown heading folds it, as Org's TAB
        if name == "indent-for-tab-command" && self.view_kind() == crate::outline::OutlineKind::Markdown && self.sections().iter().any(|s| s.start == self.cur_line()) {
            self.last_cmd = name.to_string();
            return self.outline_exec("outline-cycle", 1);
        }
        let last = std::mem::replace(&mut self.last_cmd, name.to_string());
        // searching moves the cursor, never the text: no undo step, no kill run
        if self.search_command(name) {
            self.want_col = self.cursor().col;
            return true;
        }
        // C-g (or Esc Esc) during a search cancels it, as in Emacs
        if name == "keyboard-quit" && self.isearch.is_some() {
            return self.search_command("isearch-abort");
        }
        let appending = KILLS.contains(&name) && KILLS.contains(&last.as_str());
        let vertical = matches!(name, "next-line" | "previous-line");
        // an edit is its own undo step (typing runs are grouped in self_insert)
        let edits = !matches!(
            name,
            "forward-char" | "backward-char" | "next-line" | "previous-line" | "beginning-of-line" | "end-of-line" | "back-to-indentation"
                | "forward-word" | "backward-word" | "beginning-of-buffer" | "end-of-buffer" | "forward-paragraph" | "backward-paragraph"
                | "set-mark-command" | "exchange-point-and-mark" | "mark-whole-buffer" | "keyboard-quit" | "copy-region-as-kill" | "undo" | "redo"
        );
        let rev = self.buf.rev();
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
                    // folded and narrowed-away lines are stepped over
                    self.refresh_view();
                    let target = self.step_shown(line, 1, name == "previous-line");
                    let col = self.want_col.min(self.buf.line_len(target));
                    self.cursor = self.line_start(target) + col;
                }
                "beginning-of-line" => self.cursor = self.line_start(self.cur_line()),
                "end-of-line" => self.cursor = self.line_end(self.cur_line()),
                "back-to-indentation" => self.cursor = self.first_non_blank(self.cur_line()),
                "forward-word" => self.cursor = self.emacs_word_end(self.cursor),
                "backward-word" => self.cursor = self.emacs_word_start(self.cursor),
                "beginning-of-buffer" => {
                    self.push_mark();
                    self.cursor = 0;
                }
                "end-of-buffer" => {
                    self.push_mark();
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
                    if let Some((a, b)) = self.mark_span() {
                        self.kill(a, b, appending, false);
                    }
                }
                "copy-region-as-kill" => {
                    if let Some((a, b)) = self.mark_span() {
                        let t = self.buf.slice(a, b);
                        self.push_kill(t, false, false);
                    }
                    self.mark_active = false;
                }
                "yank" => {
                    let newest = {
                        let r = self.ring();
                        r.last().map(|t| (t.to_string(), r.len() - 1))
                    };
                    if let Some((t, i)) = newest {
                        let at = self.cursor;
                        self.ins(at, &t);
                        self.cursor = at + t.chars().count();
                        self.yanked = Some((at, self.cursor, i));
                    }
                }
                "yank-pop" => {
                    if let (Some((a, b, i)), true) = (self.yanked, last == "yank" || last == "yank-pop") {
                        let older = {
                            let r = self.ring();
                            let n = r.len();
                            (n > 1).then(|| (i + n - 1) % n).and_then(|i| r.get(i).map(|t| (i, t.to_string())))
                        };
                        if let Some((i, t)) = older {
                            self.rem(a, b);
                            self.ins(a, &t);
                            self.cursor = a + t.chars().count();
                            self.yanked = Some((a, self.cursor, i));
                        }
                    }
                }
                "set-mark-command" => self.set_mark(true),
                "exchange-point-and-mark" => {
                    if let Some(m) = self.mark {
                        self.set_mark(true);
                        self.cursor = m;
                    }
                }
                "mark-whole-buffer" => {
                    self.cursor = len;
                    self.set_mark(true);
                    self.cursor = 0;
                }
                "keyboard-quit" => self.mark_active = false,
                "newline" => {
                    let l = self.cur_line();
                    let ind: String = self.indent_of(l).chars().take(self.cursor - self.line_start(l)).collect();
                    self.ins(self.cursor, &format!("\n{ind}"));
                    self.cursor += 1 + ind.chars().count();
                }
                "open-line" => self.ins(self.cursor, "\n"),
                "indent-for-tab-command" => self.insert_tab(),
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
                "cycle-spacing" => {
                    let blank = |c: Option<char>| matches!(c, Some(' ') | Some('\t'));
                    match self.spacing.take().filter(|_| last == "cycle-spacing") {
                        // second: no space at all
                        Some((at, orig, left, 1)) => {
                            self.rem(at, (at + 1).min(self.buf.len_chars()));
                            self.cursor = at;
                            self.spacing = Some((at, orig, left, 2));
                        }
                        // third: as it was
                        Some((at, orig, left, _)) => {
                            self.ins(at, &orig);
                            self.cursor = at + left;
                        }
                        None => {
                            let (mut a, mut b) = (self.cursor, self.cursor);
                            while a > 0 && blank(self.char(a - 1)) {
                                a -= 1;
                            }
                            while blank(self.char(b)) {
                                b += 1;
                            }
                            let left = self.cursor - a;
                            let orig = self.rem(a, b);
                            self.ins(a, " ");
                            self.cursor = a + 1;
                            self.spacing = Some((a, orig, left, 1));
                        }
                    }
                }
                "tab-to-tab-stop" => self.insert_tab(),
                "split-line" => {
                    let col = self.cursor().col;
                    let mut b = self.cursor;
                    while matches!(self.char(b), Some(' ') | Some('\t')) {
                        b += 1;
                    }
                    self.rem(self.cursor, b);
                    self.ins(self.cursor, &format!("\n{}", " ".repeat(col)));
                }
                "default-indent-new-line" => {
                    let (_, rest) = crate::textops::fill_prefix(self.buf.line(self.cur_line()).trim_end_matches('\n'));
                    let mut a = self.cursor;
                    while a > self.line_start(self.cur_line()) && matches!(self.char(a - 1), Some(' ') | Some('\t')) {
                        a -= 1;
                    }
                    self.rem(a, self.cursor);
                    self.ins(a, &format!("\n{rest}"));
                    self.cursor = a + 1 + rest.chars().count();
                }
                "delete-trailing-whitespace" => {
                    let (l1, l2) = match self.region() {
                        Some((a, b)) => (self.buf.to_pos(a).line, self.buf.to_pos(b).line),
                        None => (0, self.buf.len_lines() - 1),
                    };
                    for l in (l1..=l2).rev() {
                        let line = self.buf.line(l);
                        let body = line.trim_end_matches('\n');
                        let kept = body.trim_end_matches([' ', '\t']).chars().count();
                        let (s, e) = (self.line_start(l) + kept, self.line_start(l) + body.chars().count());
                        if e > s {
                            self.rem(s, e);
                            if self.cursor > s {
                                self.cursor = self.cursor.saturating_sub((e - s).min(self.cursor - s));
                            }
                        }
                    }
                }
                "delete-region" => {
                    if let Some((a, b)) = self.mark_span() {
                        self.rem(a, b);
                        self.cursor = a;
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
        // a change to the text ends the region (transient-mark-mode)
        if self.buf.rev() != rev {
            self.mark_active = false;
        }
        if edits {
            self.close();
        }
        if !vertical {
            self.want_col = self.cursor().col;
        }
        // a motion or edit that lands in a fold opens it
        if !self.view_is_empty() {
            self.reveal();
        }
        true
    }

    /// The region's lines (or this one) shifted `by` columns: right, or left when negative.
    fn indent_rigidly(&mut self, by: i64) {
        let (l1, l2) = match self.mark_span() {
            Some((a, b)) if self.mark_active => (self.buf.to_pos(a).line, self.buf.to_pos(b.saturating_sub(1).max(a)).line),
            _ => (self.cur_line(), self.cur_line()),
        };
        let (cp, mp) = (self.cursor(), self.mark.map(|m| self.buf.to_pos(m)));
        let mut moved = std::collections::HashMap::new();
        self.begin();
        for l in l1..=l2 {
            let ind = self.indent_of(l);
            if self.buf.line_len(l) == ind.chars().count() {
                continue;
            }
            let w = (self.indent_width(&ind) as i64 + by).max(0) as usize;
            let new = self.make_indent(w);
            let s = self.line_start(l);
            self.rem(s, s + ind.chars().count());
            self.ins(s, &new);
            moved.insert(l, new.chars().count() as i64 - ind.chars().count() as i64);
        }
        self.close();
        // the cursor and the mark keep their places in the text
        let at = |e: &Self, p: zett_core::Pos| e.line_start(p.line) + ((p.col as i64 + moved.get(&p.line).copied().unwrap_or(0)).max(0) as usize).min(e.buf.line_len(p.line));
        self.cursor = at(self, cp);
        self.mark = mp.map(|p| at(self, p));
        // the region stays, so the next shift moves the same lines (as Emacs's)
        self.want_col = self.cursor().col;
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
        self.ring().push(t, append, before);
        self.yanked = None;
    }

    pub(crate) fn ring(&self) -> std::sync::MutexGuard<'_, crate::KillRing> {
        self.kill_ring.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Shares a front end's ring, so a kill in one buffer yanks in every other one.
    pub fn use_kill_ring(&mut self, ring: &crate::SharedKillRing) {
        if !std::sync::Arc::ptr_eq(&self.kill_ring, ring) {
            self.kill_ring = ring.clone();
            self.yanked = None;
        }
    }

    /// The last killed text (for the system clipboard).
    pub fn last_kill(&self) -> Option<String> {
        self.ring().last().map(String::from)
    }

    /// Goes up with every kill or copy: when it changed, `last_kill` is new.
    pub fn kill_count(&self) -> u64 {
        self.ring().kill_count()
    }

    /// Text copied outside zett (the system clipboard), offered before a yank: it joins the
    /// kill ring when it is not already the newest kill (Emacs' interprogram paste).
    pub fn offer_paste(&mut self, text: String) {
        if self.ring().offer(text) {
            self.yanked = None;
        }
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
    fn a_shared_ring_yanks_in_another_buffer() {
        let ring = crate::KillRing::shared();
        let mut a = ed("alpha beta", 0);
        let mut b = ed("", 0);
        a.use_kill_ring(&ring);
        b.use_kill_ring(&ring);
        a.exec("kill-word", 1);
        b.exec("yank", 1);
        assert_eq!(b.text(), "alpha");
        // and yank-pop in b walks kills made in a (a motion between them keeps them apart: a run
        // of kills is one entry)
        a.exec("beginning-of-line", 1);
        a.exec("kill-line", 1);
        b.exec("yank", 1);
        assert_eq!(b.text(), "alpha beta");
        b.exec("yank-pop", 1);
        assert_eq!(b.text(), "alphaalpha");
    }

    #[test]
    fn kill_line_twice_takes_the_line_break_and_appends() {
        let mut e = ed("one\ntwo\nthree", 0);
        e.exec("kill-line", 1);
        assert_eq!(e.text(), "\ntwo\nthree");
        e.exec("kill-line", 1);
        assert_eq!(e.text(), "two\nthree");
        // both kills are one entry
        assert_eq!(e.last_kill().as_deref(), Some("one\n"));
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
    fn the_region_is_active_only_while_being_made() {
        let mut e = ed("alpha beta gamma", 6);
        e.exec("end-of-buffer", 1); // leaves a mark at 6, but no region
        assert_eq!(e.region(), None);
        e.exec("kill-region", 1); // C-w still kills point..mark
        assert_eq!(e.text(), "alpha ");
        e.exec("set-mark-command", 1);
        e.exec("backward-word", 1);
        assert_eq!(e.region(), Some((0, 6)));
        e.self_insert('x'); // typing ends it
        assert_eq!(e.region(), None);
        e.exec("mark-whole-buffer", 1);
        assert_eq!(e.region(), Some((0, 7)));
        e.exec("keyboard-quit", 1);
        assert_eq!(e.region(), None);
    }

    #[test]
    fn outside_copies_join_the_kill_ring() {
        let mut e = ed("ab", 2);
        e.exec("backward-kill-word", 1);
        let n = e.kill_count();
        e.offer_paste("ab".into()); // the clipboard holds our own kill: nothing new
        e.offer_paste("zz".into());
        assert_eq!(e.kill_count(), n);
        e.exec("yank", 1);
        e.exec("yank-pop", 1);
        assert_eq!(e.text(), "ab");
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

    #[test]
    fn a_negative_argument_runs_the_opposite() {
        let mut e = ed("one two three", 13);
        e.exec_arg("forward-word", Some(-2));
        assert_eq!(e.cursor, 4, "M-- 2 M-f is two words back");
        let mut e = ed("abcdef", 4);
        e.exec_arg("delete-char", Some(-2));
        assert_eq!(e.text(), "abef");
    }

    #[test]
    fn kill_line_with_an_argument_kills_whole_lines() {
        let mut e = ed("aa\nbb\ncc\ndd", 1);
        e.exec_arg("kill-line", Some(2));
        assert_eq!(e.text(), "acc\ndd", "from the cursor through two line breaks");
        assert_eq!(e.last_kill().as_deref(), Some("a\nbb\n"));
        // no argument: to the end of the line only
        let mut e = ed("aa\nbb", 1);
        e.exec_arg("kill-line", None);
        assert_eq!(e.text(), "a\nbb");
    }

    #[test]
    fn emacs_names_and_spacing() {
        let mut e = ed("ab", 0);
        e.exec("right-char", 1);
        assert_eq!(e.cursor, 1, "right-char is forward-char");
        let mut e = ed("a   b", 2);
        e.exec("cycle-spacing", 1);
        assert_eq!(e.text(), "a b");
        e.exec("cycle-spacing", 1);
        assert_eq!(e.text(), "ab");
        e.exec("cycle-spacing", 1);
        assert_eq!((e.text().as_str(), e.cursor), ("a   b", 2), "the third time: as it was");
        let mut e = ed("abc  def", 3);
        e.exec("split-line", 1);
        assert_eq!(e.text(), "abc\n   def");
        let mut e = ed("x  \ny\t\n", 0);
        e.exec("delete-trailing-whitespace", 1);
        assert_eq!(e.text(), "x\ny\n");
        let mut e = ed("    // note", 11);
        e.exec("default-indent-new-line", 1);
        assert_eq!(e.text(), "    // note\n    // ");
    }

    #[test]
    fn indent_rigidly_and_append_next_kill() {
        let mut e = ed("a\nb", 0);
        e.exec("set-mark-command", 1);
        e.cursor = 3;
        e.exec_arg("indent-rigidly", Some(2));
        assert_eq!(e.text(), "  a\n  b");
        assert_eq!(e.cursor, 7, "the cursor stays after b");
        e.exec_arg("indent-rigidly", Some(-1));
        assert_eq!(e.text(), " a\n b");
        let mut e = ed("one two", 0);
        e.exec("kill-word", 1);
        e.exec("forward-char", 1);
        e.exec("append-next-kill", 1);
        e.exec("kill-word", 1);
        assert_eq!(e.last_kill().as_deref(), Some("onetwo"));
    }
}
