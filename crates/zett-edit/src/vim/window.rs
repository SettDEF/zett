//! Windows, scrolling and the view: CTRL-W …, CTRL-D U F B E Y, z t z b … (handed to the window as requests).

use super::*;

impl Editor {
    /// Ctrl keys of batch 2: windows, scrolling, jumps, the file, the other buffer.
    pub(crate) fn vim_ctrl2(&mut self, c: char) -> bool {
        // after CTRL-W (or CTRL-W g): a window command
        match self.prefix {
            Some(CW) => {
                self.prefix = None;
                if c == 'c' {
                    // CTRL-W CTRL-C cancels, as in Vim
                    self.count = None;
                    return true;
                }
                let n = self.count.take();
                self.vim.requests.push(Request::Window(format!("C-{c}"), n));
                return true;
            }
            Some(b @ ('[' | ']')) => {
                self.prefix = None;
                return match c {
                    'i' => self.vim_bracket(b, super::brackets::BR_TAB),
                    'd' => self.vim_bracket(b, super::brackets::BR_DEF),
                    _ => false,
                };
            }
            Some(CWG) => {
                self.prefix = None;
                let n = self.count.take();
                self.vim.requests.push(Request::Window(if c == 'i' { "g<Tab>".into() } else { format!("g C-{c}") }, n));
                return true;
            }
            Some(CBS) => {
                // CTRL-\ CTRL-N / CTRL-G: to normal mode (here already)
                self.prefix = None;
                self.count = None;
                return true;
            }
            Some('g') => {
                self.prefix = None;
                match c {
                    ']' => self.vim.requests.push(Request::Definition),
                    'g' => self.vim_word_count(),
                    'h' => {
                        self.anchor = self.cursor;
                        self.mode = Mode::Visual;
                    }
                    _ => return false,
                }
                return true;
            }
            _ => {}
        }
        let (top, height, _) = self.vim.screen;
        let height = height.max(2);
        let last = self.buf.len_lines() - 1;
        let line = self.cur_line();
        match c {
            'w' => self.prefix = Some(CW),
            'v' | 'q' => self.enter_block(),
            '\\' => self.prefix = Some(CBS),
            'c' => {
                self.count = None;
                self.op = None;
            }
            'e' | 'y' => {
                let n = self.take_count();
                let t = if c == 'e' { (top + n).min(last) } else { top.saturating_sub(n) };
                self.vim_top(t);
                // the cursor stays on screen
                let to = line.clamp(t, (t + height - 1).min(last));
                if to != line {
                    self.vertical_to(to);
                }
            }
            'd' | 'u' => {
                // scroll: the lines CTRL-D and CTRL-U move (0: half the window); a count sets it, as in Vim
                if let Some(n) = self.count.take() {
                    self.vim.opts.insert("scroll".into(), n.to_string());
                }
                let n = match self.opt_num("scroll", 0) {
                    s if s > 0 => s as usize,
                    _ => height / 2,
                }
                .max(1);
                let (t, to) = if c == 'd' { ((top + n).min(last.saturating_sub(height / 2).max(top)), (line + n).min(last)) } else { (top.saturating_sub(n), line.saturating_sub(n)) };
                self.vim_top(t);
                self.cursor = self.sol_target(to);
            }
            'f' | 'b' => {
                let n = self.take_count();
                // window: set below the screen's height, CTRL-F/CTRL-B move window - 2 lines (23 is the unset default)
                let w = self.opt_num("window", 0);
                let page = if w > 2 && w != 23 && (w as usize) < height.saturating_sub(1) { w as usize - 2 } else { height.saturating_sub(2).max(1) } * n;
                let t = if c == 'f' { (top + page).min(last) } else { top.saturating_sub(page) };
                self.vim_top(t);
                let to = if c == 'f' { t } else { (t + height - 1).min(last) };
                self.cursor = self.sol_target(to);
            }
            'o' => self.vim_jump(-1),
            'i' => self.vim_jump(1),
            'g' => self.vim.requests.push(Request::FileInfo),
            'l' => self.vim.requests.push(Request::Redraw),
            '^' | '6' => {
                let n = self.count.take();
                self.vim.requests.push(Request::Alternate(n));
            }
            ']' => self.vim.requests.push(Request::Definition),
            't' => self.vim.requests.push(Request::Back),
            'z' => self.vim.requests.push(Request::Suspend),
            _ => return false,
        }
        true
    }

    /// `g CTRL-G`: words, characters and lines, and where the cursor is among them.
    pub(crate) fn vim_word_count(&mut self) {
        let text = self.buf.text();
        let before: String = text.chars().take(self.cursor).collect();
        let (w, wb) = (text.split_whitespace().count(), before.split_whitespace().count());
        let m = format!("Col {} of {}; Line {} of {}; Word {wb} of {w}; Char {} of {}", self.cursor().col + 1, self.buf.line_len(self.cur_line()), self.cur_line() + 1, self.buf.len_lines(), self.cursor + 1, self.buf.len_chars());
        self.vim.requests.push(Request::Message(m));
    }

    /// `z` and a key: where the window stands (zt zz zb z<CR> z. z- z+ z^), sideways (zh zl zs ze zH zL).
    pub(crate) fn vim_z(&mut self, c: char) -> bool {
        // z{height}<CR>: digits wait for the Enter
        if let Some(d) = c.to_digit(10) {
            self.vim.z_height = self.vim.z_height * 10 + d as usize;
            self.prefix = Some('z');
            return true;
        }
        if c == '\r' && self.vim.z_height > 0 {
            let h = std::mem::take(&mut self.vim.z_height);
            self.vim.requests.push(Request::Window("_".into(), Some(h)));
        }
        // fold keys keep their count (3zF)
        if !matches!(c, 't' | '\r' | 'z' | '.' | 'b' | '-' | '+' | '^' | 'h' | 'l' | 'H' | 'L' | 's' | 'e') {
            return self.vim_fold(c);
        }
        let (top, height, width) = self.vim.screen;
        let height = height.max(1);
        let last = self.buf.len_lines() - 1;
        let line = self.count.take().map(|n| n.saturating_sub(1).min(last)).unwrap_or(self.cur_line());
        let fnb = matches!(c, '\r' | '.' | '-');
        match c {
            't' | '\r' => self.vim_top(line),
            'z' | '.' => self.vim_top(line.saturating_sub(height / 2)),
            'b' | '-' => self.vim_top((line + 1).saturating_sub(height)),
            '+' => {
                let t = (top + height).min(last);
                self.vim_top(t);
                self.cursor = self.first_non_blank(t);
                return true;
            }
            '^' => {
                let b = top.saturating_sub(1);
                self.vim_top((b + 1).saturating_sub(height));
                self.cursor = self.first_non_blank(b);
                return true;
            }
            'h' | 'l' | 'H' | 'L' => {
                let n = if c.is_uppercase() { width.max(2) / 2 } else { self.take_count() } as i32;
                self.vim.requests.push(Request::Sideways(if c.eq_ignore_ascii_case(&'h') { -n } else { n }));
            }
            's' => self.vim.requests.push(Request::LeftAt(self.cursor().col)),
            'e' => self.vim.requests.push(Request::LeftAt((self.cursor().col + 1).saturating_sub(width.max(1)))),
            _ => return self.vim_fold(c),
        }
        if line != self.cur_line() || fnb {
            self.cursor = if fnb { self.first_non_blank(line) } else { self.line_start(line) + self.cursor().col.min(self.buf.line_len(line).saturating_sub(1)) };
            self.clamp();
        }
        true
    }

    /// The window-command prefixes, for vim_prefixed: CTRL-W x, CTRL-W g x.
    pub(crate) fn vim_window_prefixed(&mut self, p: char, c: char) -> bool {
        match p {
            CW if c == 'g' => self.prefix = Some(CWG),
            CW => {
                let n = self.count.take();
                self.vim.requests.push(Request::Window(c.to_string(), n));
            }
            CWG => {
                let n = self.count.take();
                self.vim.requests.push(Request::Window(format!("g{c}"), n));
            }
            CBS => {}
            _ => return false,
        }
        true
    }

}
