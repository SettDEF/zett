//! What the keys under way add up to: the dot, a register given with "x, the jump list.

use super::*;

impl Editor {
    /// The search line in Vim mode (`/` `?`), macro recording and the dot's keys: before a key.
    /// True when the key was taken here.
    pub(crate) fn vim_before(&mut self, k: Key) -> bool {
        if self.emacs {
            return false;
        }
        if let Some((_, keys)) = self.vim.recording.as_mut() {
            keys.push(k);
        }
        if self.vim.replaying == 0 {
            if self.vim.typing.is_empty() {
                self.vim.typing_from = self.edit_marker();
                self.vim.typing_pos = self.cursor;
                self.vim.lines_before = self.buf.len_lines();
                let l = self.cur_line();
                self.vim.typing_line = (l, self.buf.line(l));
            }
            self.vim.typing.push(k);
        }
        if self.isearch.is_some() {
            self.vim_search_key(k);
            return true;
        }
        false
    }

    /// After a key: a finished command that changed the text becomes the dot; a register given
    /// with `"x` takes what was yanked or deleted.
    pub(crate) fn vim_after(&mut self) {
        if self.emacs {
            return;
        }
        let idle = self.mode == Mode::Normal && self.op.is_none() && self.prefix.is_none() && self.count.is_none() && self.isearch.is_none() && self.vim.use_reg.is_none_or(|_| self.vim.reg_before.is_some());
        if !idle {
            return;
        }
        if std::mem::take(&mut self.vim.reg_armed) {
            return;
        }
        // a register asked for: what the command wrote goes there, a put leaves the unnamed one
        if let (Some(r), Some(before)) = (self.vim.use_reg.take(), self.vim.reg_before.take()) {
            let wrote = self.reg.text != self.vim.regs.get(&r.to_ascii_lowercase()).map(|x| x.text.clone()).unwrap_or_default() || self.vim.typing.iter().any(|k| matches!(k.char(), Some('y' | 'd' | 'c' | 'x' | 's')));
            let written = self.reg.clone();
            if wrote && r != '_' {
                let lower = r.to_ascii_lowercase();
                if r.is_ascii_uppercase() {
                    let e = self.vim.regs.entry(lower).or_default();
                    e.text.push_str(&written.text);
                    e.linewise |= written.linewise;
                } else {
                    self.vim.regs.insert(lower, written.clone());
                }
                if matches!(r, '+' | '*') {
                    if let Some((_, set)) = CLIPBOARD.get() {
                        set(&written.text);
                    }
                }
            } else {
                self.reg = before;
            }
            if r == '_' {
                self.reg = self.vim.regs.get(&'"').cloned().unwrap_or_default();
            }
        }
        let keys = std::mem::take(&mut self.vim.typing);
        if keys.is_empty() || self.vim.replaying > 0 {
            return;
        }
        self.vim_note_jump(&keys);
        let changed = self.edit_marker() != self.vim.typing_from;
        let first = keys[0].char();
        // undo, redo and the dot itself are not changes to repeat
        if changed && !matches!(first, Some('u' | '.')) && !(keys[0].ctrl && first == Some('r')) {
            let first_key = keys.first().and_then(|k| k.char());
            let before = self.vim.lines_before;
            self.vim_report(before);
            self.vim.dot = keys;
            let at = self.cursor;
            if self.vim.changes.last() != Some(&at) {
                self.vim.changes.push(at);
                if self.vim.changes.len() > 100 {
                    self.vim.changes.remove(0);
                }
            }
            self.vim.change_at = self.vim.changes.len();
            // U brings back the line as it was before the changes made on it
            if first_key != Some('U') && self.vim.uline.as_ref().map(|u| u.0) != Some(self.vim.typing_line.0) {
                self.vim.uline = Some(self.vim.typing_line.clone());
            }
            self.vim.marks.insert('.', at);
        }
    }

    /// `"x`: the register for the next command; puts read it, yanks and deletes write it.
    pub(crate) fn vim_use_reg(&mut self, r: char) {
        let lower = r.to_ascii_lowercase();
        self.vim.reg_before = Some(self.reg.clone());
        self.vim.regs.insert('"', self.reg.clone());
        self.vim.use_reg = Some(r);
        self.vim.reg_armed = true;
        if matches!(r, '+' | '*') {
            if let Some((get, _)) = CLIPBOARD.get() {
                if let Some(t) = get() {
                    let linewise = t.ends_with('\n');
                    self.vim.regs.insert(lower, Register { text: t, linewise, block: false });
                }
            }
        }
        self.reg = self.vim.regs.get(&lower).cloned().unwrap_or_default();
    }

    /// Where the history stands: a change since shows as a difference (an open change counts its edits).
    pub(crate) fn edit_marker(&self) -> (usize, usize, usize) {
        (self.undo.nodes.len(), self.undo.cur, self.open.as_ref().map_or(0, |c| c.edits.len()))
    }

    /// The jump list: `dir` -1 back (CTRL-O), +1 forward (CTRL-I / Tab).
    pub(crate) fn vim_jump(&mut self, dir: i32) {
        let n = self.take_count();
        for _ in 0..n {
            let j = &mut self.vim;
            if dir < 0 {
                if j.jump_at == j.jumps.len() {
                    j.jumps.push(self.cursor);
                }
                if j.jump_at == 0 {
                    return;
                }
                j.jump_at -= 1;
            } else {
                if j.jump_at + 1 >= j.jumps.len() {
                    return;
                }
                j.jump_at += 1;
            }
            self.cursor = self.vim.jumps[self.vim.jump_at].min(self.buf.len_chars());
            self.clamp();
            self.want_col = self.cursor().col;
        }
    }

    /// After a jump (G, /, n, %, ', H…): where it came from goes on the jump list.
    pub(crate) fn vim_note_jump(&mut self, keys: &[Key]) {
        // past a count and a register: `50G`, `"a3n`
        let mut rest = keys;
        loop {
            match rest.first().and_then(|k| k.char()) {
                Some(c) if c.is_ascii_digit() => rest = &rest[1..],
                Some('"') if rest.len() > 1 => rest = &rest[2..],
                _ => break,
            }
        }
        let first = rest.first().and_then(|k| k.char());
        let second = rest.get(1).and_then(|k| k.char());
        let jump = matches!(first, Some('G' | '/' | '?' | 'n' | 'N' | '*' | '#' | '%' | '(' | ')' | '{' | '}' | 'H' | 'M' | 'L' | '\'' | '`'))
            || (first == Some('g') && matches!(second, Some('g' | 'd' | 'D' | '*' | '#')));
        let from = self.vim.typing_pos;
        if jump && self.buf.to_pos(from.min(self.buf.len_chars())).line != self.cur_line() {
            let j = &mut self.vim;
            j.jumps.truncate(j.jump_at);
            j.jumps.retain(|p| *p != from);
            j.jumps.push(from);
            if j.jumps.len() > 100 {
                j.jumps.remove(0);
            }
            j.jump_at = j.jumps.len();
            j.marks.insert('\'', from);
        }
    }

}
