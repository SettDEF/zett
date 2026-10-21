//! Block selection (CTRL-V): a rectangle of columns over lines, its own keys (d x y c s I A r ~
//! u U > < o O $ J p P :), block registers that put their lines one under another, and an
//! insert typed once that lands on every line of the block when Esc ends it.

use super::*;

impl Editor {
    /// The block: (first line, last line, first column, last column), the last column usize::MAX
    /// when it reaches every line's end ($).
    pub fn block(&self) -> Option<(usize, usize, usize, usize)> {
        if self.mode != Mode::VisualBlock {
            return None;
        }
        let (a, b) = (self.buf.to_pos(self.anchor), self.buf.to_pos(self.cursor));
        let (c1, c2) = (a.col.min(b.col), a.col.max(b.col));
        Some((a.line.min(b.line), a.line.max(b.line), c1, if self.vim.block_eol { usize::MAX } else { c2 }))
    }

    /// The block's part of line `l`: (start, end) characters, clipped to the line.
    fn block_part(&self, l: usize, c1: usize, c2: usize) -> (usize, usize) {
        let (s, len) = (self.line_start(l), self.buf.line_len(l));
        (s + c1.min(len), s + c2.saturating_add(1).min(len))
    }

    pub(crate) fn enter_block(&mut self) {
        if self.mode == Mode::Normal {
            self.anchor = self.cursor;
        }
        self.mode = Mode::VisualBlock;
        self.vim.block_eol = false;
        self.count = None;
    }

    fn leave_block(&mut self) {
        self.vim.last_visual = Some((self.anchor, self.cursor, Mode::VisualBlock));
        self.mode = Mode::Normal;
        self.vim.block_eol = false;
        self.clamp();
        self.want_col = self.cursor().col;
    }

    /// The block's text, as a block register.
    fn block_yank(&mut self, (l1, l2, c1, c2): (usize, usize, usize, usize)) {
        let text: Vec<String> = (l1..=l2).map(|l| {
            let (a, b) = self.block_part(l, c1, c2);
            self.buf.slice(a, b)
        }).collect();
        self.reg = Register { text: text.join("\n"), linewise: false, block: true };
    }

    /// Takes the block's text out of every line (bottom up, so positions above stay).
    fn block_delete(&mut self, b: (usize, usize, usize, usize)) {
        self.block_yank(b);
        let (l1, l2, c1, c2) = b;
        self.begin();
        for l in (l1..=l2).rev() {
            let (a, e) = self.block_part(l, c1, c2);
            if e > a {
                self.rem(a, e);
            }
        }
        self.close();
        self.cursor = self.line_start(l1) + c1.min(self.buf.line_len(l1));
    }

    /// Insert mode at column `col` of the first line; Esc copies what was typed to the others.
    fn block_insert(&mut self, l1: usize, l2: usize, col: usize, pad: bool) {
        let len = self.buf.line_len(l1);
        if pad && col != usize::MAX && len < col {
            let e = self.line_end(l1);
            self.begin();
            self.ins(e, &" ".repeat(col - len));
        }
        let at = self.line_start(l1) + col.min(self.buf.line_len(l1));
        self.leave_block();
        self.start_insert(at);
        self.vim.block_insert = Some((l1, l2, col, at, pad));
    }

    /// Esc after a block insert: the text typed on the first line goes in on the others too.
    pub(crate) fn block_insert_done(&mut self) {
        let Some((l1, l2, col, start, pad)) = self.vim.block_insert.take() else { return };
        if self.cursor <= start || self.cur_line() != l1 {
            return;
        }
        let typed = self.buf.slice(start, self.cursor);
        if typed.contains('\n') {
            return;
        }
        for l in l1 + 1..=l2.min(self.buf.len_lines() - 1) {
            let len = self.buf.line_len(l);
            if col == usize::MAX {
                let at = self.line_end(l);
                self.ins(at, &typed);
                continue;
            }
            if len < col {
                if !pad {
                    // I skips lines too short to reach the block
                    continue;
                }
                let e = self.line_end(l);
                self.ins(e, &" ".repeat(col - len));
            }
            let at = self.line_start(l) + col;
            self.ins(at, &typed);
        }
    }

    /// Puts a block register: its lines go in one under another at the cursor's column.
    pub(crate) fn put_block(&mut self, after: bool) {
        let line = self.cur_line();
        let col = self.cursor().col + usize::from(after && self.buf.line_len(line) > 0);
        let parts: Vec<String> = self.reg.text.split('\n').map(String::from).collect();
        let width = parts.iter().map(|p| p.chars().count()).max().unwrap_or(0);
        self.begin();
        for (i, p) in parts.iter().enumerate() {
            let l = line + i;
            if l >= self.buf.len_lines() {
                let e = self.buf.len_chars();
                self.ins(e, "\n");
            }
            let len = self.buf.line_len(l);
            if len < col {
                let e = self.line_end(l);
                self.ins(e, &" ".repeat(col - len));
            }
            // shorter parts are padded when text follows, so the columns after stay in line
            let follows = self.buf.line_len(l) > col;
            let text = if follows { format!("{p:<width$}") } else { p.clone() };
            let at = self.line_start(l) + col;
            self.ins(at, &text);
        }
        self.close();
        self.cursor = self.line_start(line) + col;
    }

    /// A key in block mode.
    pub(crate) fn block_key(&mut self, k: Key) {
        let Some(b) = self.block() else { return };
        let (l1, l2, c1, c2) = b;
        // r{char}: every character of the block becomes it
        if self.prefix == Some('r') {
            self.prefix = None;
            if let Some(ch) = k.char() {
                self.begin();
                for l in l1..=l2 {
                    let (a, e) = self.block_part(l, c1, c2);
                    if e > a {
                        self.rem(a, e);
                        self.ins(a, &ch.to_string().repeat(e - a));
                    }
                }
                self.close();
                self.cursor = self.line_start(l1) + c1;
                self.leave_block();
            }
            return;
        }
        if k.code == Code::Esc || (k.ctrl && matches!(k.code, Code::Char('c' | 'v' | 'q' | '['))) {
            return self.leave_block();
        }
        let Some(c) = k.char() else { return self.normal_key(k) };
        if self.prefix.is_some() {
            return self.normal_key(k);
        }
        match c {
            'y' | 'Y' => {
                self.block_yank(b);
                self.cursor = self.line_start(l1) + c1.min(self.buf.line_len(l1));
                self.leave_block();
            }
            'd' | 'x' | 'D' | 'X' => {
                let b = if matches!(c, 'D' | 'X') { (l1, l2, c1, usize::MAX) } else { b };
                self.block_delete(b);
                self.leave_block();
            }
            'c' | 's' | 'C' | 'S' => {
                let b = if matches!(c, 'C' | 'S') { (l1, l2, c1, usize::MAX) } else { b };
                self.block_delete(b);
                self.mode = Mode::VisualBlock;
                self.block_insert(l1, l2, c1, false);
            }
            'I' => self.block_insert(l1, l2, c1, false),
            'A' => {
                // after $: at each line's own end (usize::MAX), else after the block, padding short lines
                let col = if c2 == usize::MAX { usize::MAX } else { c2 + 1 };
                self.block_insert(l1, l2, col, true);
            }
            'r' => self.prefix = Some('r'),
            '~' | 'u' | 'U' => {
                self.begin();
                for l in l1..=l2 {
                    let (a, e) = self.block_part(l, c1, c2);
                    let t: String = self.buf.slice(a, e).chars().map(|ch| match c { '~' => super::toggle(ch), 'u' => ch.to_lowercase().next().unwrap_or(ch), _ => ch.to_uppercase().next().unwrap_or(ch) }).collect();
                    if e > a {
                        self.rem(a, e);
                        self.ins(a, &t);
                    }
                }
                self.close();
                self.cursor = self.line_start(l1) + c1;
                self.leave_block();
            }
            '>' | '<' => {
                // shifts what is right of the block's left edge
                let sw = self.soft_tab.unwrap_or(4) * self.take_count();
                self.begin();
                for l in l1..=l2 {
                    let len = self.buf.line_len(l);
                    if len <= c1 {
                        continue;
                    }
                    let at = self.line_start(l) + c1;
                    if c == '>' {
                        self.ins(at, &" ".repeat(sw));
                    } else {
                        let blanks = (0..sw).take_while(|&i| self.char(at + i).is_some_and(|ch| ch == ' ' || ch == '\t')).count();
                        if blanks > 0 {
                            self.rem(at, at + blanks);
                        }
                    }
                }
                self.close();
                self.cursor = self.line_start(l1) + c1;
                self.leave_block();
            }
            'o' => std::mem::swap(&mut self.anchor, &mut self.cursor),
            'O' => {
                // the other corner on the same line
                let (a, cu) = (self.buf.to_pos(self.anchor), self.buf.to_pos(self.cursor));
                self.anchor = self.line_start(a.line) + cu.col.min(self.buf.line_len(a.line));
                self.cursor = self.line_start(cu.line) + a.col.min(self.buf.line_len(cu.line));
            }
            '$' => {
                self.vim.block_eol = true;
                let l = self.cur_line();
                self.cursor = self.line_end(l).saturating_sub(1).max(self.line_start(l));
            }
            'J' => {
                self.leave_block();
                self.cursor = self.line_start(l1);
                self.join((l2 - l1).max(1));
            }
            'p' | 'P' => {
                let put = self.reg.clone();
                self.block_delete(b);
                self.reg = put;
                self.leave_block();
                if self.reg.block {
                    self.put_block(false);
                } else {
                    self.put(false, 1);
                }
            }
            ':' => {
                self.leave_block();
                self.vim.requests.push(Request::CmdLine("'<,'>".into()));
            }
            'v' | 'V' => {
                self.mode = if c == 'v' { Mode::Visual } else { Mode::VisualLine };
            }
            _ => {
                // motions move the corner
                self.normal_key(k);
                if self.mode == Mode::Normal {
                    self.mode = Mode::VisualBlock;
                }
            }
        }
    }
}
