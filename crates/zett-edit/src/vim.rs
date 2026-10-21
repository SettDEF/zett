//! Vim's normal-mode keys beyond the core in lib.rs, measured against Neovim's own index
//! (docs/parity/nvim-normal-keys.md, probe.rs): search (`/ ? n N * #`), marks, registers,
//! macros, the dot, more operators (`g~ gu gU g? > < = gq !`), counting (`CTRL-A`), and keys the
//! window layer carries out, handed over as requests (`ZZ`, `K`, `gx`…).

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::keys::{Code, Key};
use crate::{Editor, Mode, Op, Register, Span};

/// What the editor asks the window around it to do: things outside one buffer.
#[derive(Clone, Debug, PartialEq)]
pub enum Request {
    /// an Ex command line, as typed after `:` (`x`, `q!`, `s`, `&&`)
    Ex(String),
    /// open the `:` line with this text in it
    CmdLine(String),
    /// show a message
    Message(String),
    /// open a file or an address (gf, gx)
    Open(String),
    /// look a word up (K)
    Keyword(String),
    /// tabs: +1 next, -1 previous (gt, gT)
    Tab(i32),
    /// a pause of this many milliseconds (gs)
    Sleep(u64),
    /// run the operator function on these characters (g@)
    OperatorFunc(usize, usize),
    /// a window command: what follows CTRL-W (`s`, `v`, `C-w`, `+`, `gf`…) and its count
    Window(String, Option<usize>),
    /// show the buffer from this line at the window's top (scrolling: CTRL-E, CTRL-D, zt, zz…)
    Top(usize),
    /// scroll sideways: by columns (+ right), or so this column is at the left edge
    Sideways(i32),
    LeftAt(usize),
    /// the other buffer (CTRL-^), the definition under the cursor (CTRL-], g]), back from it (CTRL-T)
    Alternate(Option<usize>),
    Definition,
    Back,
    /// the file's name and where the cursor is (CTRL-G), redraw (CTRL-L), suspend (CTRL-Z)
    FileInfo,
    Redraw,
    Suspend,
    /// the command-line window (q: q/ q?)
    CmdWindow(char),
}

/// The system clipboard, for `"+` and `"*`: the window sets it (the editor itself reads no
/// clipboard).
pub static CLIPBOARD: OnceLock<(fn() -> Option<String>, fn(&str))> = OnceLock::new();

#[derive(Default, Debug)]
pub(crate) struct VimState {
    pub marks: HashMap<char, usize>,
    pub regs: HashMap<char, Register>,
    /// `"x` given for the next command, and the unnamed register it stood in for
    pub use_reg: Option<char>,
    /// the register was just chosen: the command that uses it comes next
    reg_armed: bool,
    reg_before: Option<Register>,
    /// recording a macro into this register, the keys so far
    pub recording: Option<(char, Vec<Key>)>,
    pub macros: HashMap<char, Vec<Key>>,
    last_macro: Option<char>,
    replaying: u32,
    /// the last change as keys (for `.`), and the keys of the command under way
    pub dot: Vec<Key>,
    typing: Vec<Key>,
    typing_from: (usize, usize, usize),
    /// the last search went backward (`?`), matched whole words (`*`)
    pub search_back: bool,
    pub search_word: bool,
    /// searches wrap around the end (the wrapscan setting)
    pub wrapscan: bool,
    /// `R`: typing replaces
    pub replace: bool,
    /// where insert mode was left (`gi`), the last visual selection (`gv`)
    pub last_insert: Option<usize>,
    pub last_visual: Option<(usize, usize, Mode)>,
    /// where changes happened, newest last (`g;` `g,`), and where in the list `g;` stands
    pub changes: Vec<usize>,
    change_at: usize,
    /// the window's lines around the buffer (top, height) and width: H M L, gm
    pub screen: (usize, usize, usize),
    pub requests: Vec<Request>,
    /// the jump list (CTRL-O, CTRL-I): places before jumps, and where in it we stand
    pub jumps: Vec<usize>,
    jump_at: usize,
    /// where the command under way started (a jump records it)
    typing_pos: usize,
}

impl VimState {
    /// Positions after `n` characters removed at `a` move back; ones inside go to `a`.
    pub fn removed(&mut self, a: usize, n: usize) {
        let mv = |p: &mut usize| {
            if *p > a {
                *p = p.saturating_sub(n).max(a);
            }
        };
        self.marks.values_mut().for_each(mv);
        self.changes.iter_mut().for_each(mv);
        self.jumps.iter_mut().for_each(mv);
        if let Some(p) = self.last_insert.as_mut() {
            mv(p);
        }
    }

    /// Positions after an edit at `at` move by `delta` (marks stay on their text).
    pub fn shift(&mut self, at: usize, delta: isize) {
        let mv = |p: &mut usize| {
            if *p >= at {
                *p = (*p as isize + delta).max(at as isize) as usize;
            }
        };
        self.marks.values_mut().for_each(mv);
        self.changes.iter_mut().for_each(mv);
        self.jumps.iter_mut().for_each(mv);
        if let Some(p) = self.last_insert.as_mut() {
            mv(p);
        }
    }
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

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
            self.vim.dot = keys;
            let at = self.cursor;
            if self.vim.changes.last() != Some(&at) {
                self.vim.changes.push(at);
                if self.vim.changes.len() > 100 {
                    self.vim.changes.remove(0);
                }
            }
            self.vim.change_at = self.vim.changes.len();
            self.vim.marks.insert('.', at);
        }
    }

    /// `"x`: the register for the next command; puts read it, yanks and deletes write it.
    fn vim_use_reg(&mut self, r: char) {
        let lower = r.to_ascii_lowercase();
        self.vim.reg_before = Some(self.reg.clone());
        self.vim.regs.insert('"', self.reg.clone());
        self.vim.use_reg = Some(r);
        self.vim.reg_armed = true;
        if matches!(r, '+' | '*') {
            if let Some((get, _)) = CLIPBOARD.get() {
                if let Some(t) = get() {
                    let linewise = t.ends_with('\n');
                    self.vim.regs.insert(lower, Register { text: t, linewise });
                }
            }
        }
        self.reg = self.vim.regs.get(&lower).cloned().unwrap_or_default();
    }

    // ── search ──

    fn vim_search_start(&mut self, forward: bool) {
        self.isearch_start_vim(forward);
    }

    fn vim_search_key(&mut self, k: Key) {
        match k.code {
            Code::Enter => {
                let Some(s) = self.isearch.take() else { return };
                let q = if s.query().is_empty() { self.last_search.clone() } else { s.query().to_string() };
                let (origin, forward) = (s.origin(), s.forward());
                self.cursor = origin;
                if q.is_empty() {
                    return;
                }
                self.last_search = q;
                self.vim.search_back = !forward;
                self.vim.search_word = false;
                self.vim_next(forward);
            }
            Code::Esc => {
                if let Some(s) = self.isearch.take() {
                    self.cursor = s.origin();
                }
                self.op = None;
                self.count = None;
            }
            Code::Backspace => {
                if self.isearch.as_ref().is_some_and(|s| s.query().is_empty()) {
                    let o = self.isearch.take().map(|s| s.origin()).unwrap_or(self.cursor);
                    self.cursor = o;
                    self.op = None;
                } else {
                    self.exec("isearch-delete-char", 1);
                }
            }
            _ => {
                if let Some(c) = k.char().filter(|_| !k.ctrl && !k.alt) {
                    self.isearch_char(c);
                }
            }
        }
    }

    /// The next match of the last search (`n`; `forward` false: `N`), as a motion.
    fn vim_next(&mut self, forward: bool) {
        let q = self.last_search.clone();
        if q.is_empty() {
            self.count = None;
            self.op = None;
            return self.vim.requests.push(Request::Message("E35: No previous regular expression".into()));
        }
        let n = self.take_count();
        let fold = self.folds_for(&q);
        let len = self.buf.len_chars();
        let mut at = self.cursor;
        for _ in 0..n {
            let mut from = if forward { at + 1 } else { at };
            let found = loop {
                let hit = self.buf.find(&q, from.min(len), forward, fold);
                match hit {
                    Some(i) if self.vim.search_word && !self.whole_word_at(i, q.chars().count()) => {
                        from = if forward { i + 1 } else { i };
                    }
                    other => break other,
                }
            };
            let found = found.or_else(|| {
                if !self.vim.wrapscan {
                    return None;
                }
                let mut from = if forward { 0 } else { len };
                loop {
                    match self.buf.find(&q, from.min(len), forward, fold) {
                        Some(i) if self.vim.search_word && !self.whole_word_at(i, q.chars().count()) => from = if forward { i + 1 } else { i },
                        other => break other,
                    }
                }
            });
            match found {
                Some(i) => at = i,
                None => {
                    self.op = None;
                    return self.vim.requests.push(Request::Message(format!("E486: Pattern not found: {q}")));
                }
            }
        }
        self.vim.marks.insert('\'', self.cursor);
        self.motion(Span::Chars(self.cursor.min(at), self.cursor.max(at)), Some(at));
    }

    fn whole_word_at(&self, i: usize, n: usize) -> bool {
        let before = i.checked_sub(1).and_then(|p| self.char(p)).is_none_or(|c| !is_word(c));
        let after = self.char(i + n).is_none_or(|c| !is_word(c));
        before && after
    }

    /// The word under (or after) the cursor: (start, end).
    fn word_here(&self) -> Option<(usize, usize)> {
        let line = self.cur_line();
        let (s, e) = (self.line_start(line), self.line_end(line));
        let mut i = self.cursor;
        while i < e && !self.char(i).is_some_and(is_word) {
            i += 1;
        }
        if i >= e {
            return None;
        }
        let mut a = i;
        while a > s && self.char(a - 1).is_some_and(is_word) {
            a -= 1;
        }
        let mut b = i;
        while b < e && self.char(b).is_some_and(is_word) {
            b += 1;
        }
        Some((a, b))
    }

    /// `*` `#` (whole word), `g*` `g#` (any match).
    fn vim_star(&mut self, forward: bool, whole: bool) {
        let Some((a, b)) = self.word_here() else {
            self.count = None;
            return self.vim.requests.push(Request::Message("E348: No string under cursor".into()));
        };
        self.last_search = self.buf.slice(a, b);
        self.vim.search_word = whole;
        self.vim.search_back = !forward;
        // from the word's start, so `#` skips this one
        self.cursor = a;
        self.vim_next(forward);
    }

    // ── keys ──

    /// A Ctrl key in normal mode; false when it is not one of these.
    pub(crate) fn vim_ctrl(&mut self, c: char) -> bool {
        if self.vim_ctrl2(c) {
            return true;
        }
        match c {
            'a' | 'x' => {
                let n = self.take_count() as i64;
                self.vim_add(if c == 'a' { n } else { -n });
            }
            'n' | 'j' => self.normal_key(Key::ch('j')),
            'p' => self.normal_key(Key::ch('k')),
            'h' => self.vim_back_char(),
            'm' => self.vim_line_motion(1),
            _ => return false,
        }
        true
    }

    fn vim_back_char(&mut self) {
        // <BS>: left, onto the line above at a line start
        let n = self.take_count();
        let to = self.cursor.saturating_sub(n);
        self.motion(Span::Chars(to, self.cursor), Some(to));
    }

    /// `+` `<CR>` (down) and `-` (up) to the first non-blank, `_` (count-1 lines down).
    fn vim_line_motion(&mut self, dir: i64) {
        let n = self.take_count() as i64;
        let line = self.cur_line() as i64;
        let last = self.buf.len_lines() as i64 - 1;
        let to = if dir == 0 { (line + n - 1).min(last) } else { (line + dir * n).clamp(0, last) } as usize;
        self.motion(Span::Lines(self.cur_line(), to), Some(self.first_non_blank(to)));
    }

    /// CTRL-A / CTRL-X: the number at or after the cursor on this line, plus `by`.
    fn vim_add(&mut self, by: i64) {
        let line = self.cur_line();
        let (s, e) = (self.line_start(line), self.line_end(line));
        let text: Vec<char> = self.buf.slice(s, e).chars().collect();
        let col = self.cursor - s;
        // the number: a run of digits touching or after the cursor (0x… hex, a - before it)
        let mut i = col.min(text.len());
        while i > 0 && text.get(i).is_some_and(|c| c.is_ascii_hexdigit()) && text[i - 1].is_ascii_hexdigit() {
            i -= 1;
        }
        let start_digit = (i..text.len()).find(|&j| text[j].is_ascii_digit());
        let Some(d) = start_digit else { return };
        let hex = d + 1 < text.len() && text[d] == '0' && matches!(text[d + 1], 'x' | 'X');
        let (num_start, mut end) = if hex { (d + 2, d + 2) } else { (d, d) };
        while end < text.len() && (if hex { text[end].is_ascii_hexdigit() } else { text[end].is_ascii_digit() }) {
            end += 1;
        }
        let digits: String = text[num_start..end].iter().collect();
        let (from, new) = if hex {
            let v = i64::from_str_radix(&digits, 16).unwrap_or(0).wrapping_add(by);
            let upper = digits.chars().any(|c| c.is_ascii_uppercase());
            let s = if upper { format!("{:0w$X}", v.max(0), w = digits.len()) } else { format!("{:0w$x}", v.max(0), w = digits.len()) };
            (num_start, s)
        } else {
            let neg = d > 0 && text[d - 1] == '-';
            let v: i64 = digits.parse::<i64>().unwrap_or(0) * if neg { -1 } else { 1 };
            let v = v.saturating_add(by);
            (if neg { d - 1 } else { d }, v.to_string())
        };
        self.begin();
        self.rem(s + from, s + end);
        self.ins(s + from, &new);
        self.close();
        self.cursor = s + from + new.chars().count() - 1;
    }

    /// One key in normal mode that lib.rs does not know; false when it is not one of these either.
    pub(crate) fn vim_more(&mut self, c: char) -> bool {
        let line = self.cur_line();
        match c {
            '/' | '?' => self.vim_search_start(c == '/'),
            'n' => {
                let back = self.vim.search_back;
                self.vim_next(!back);
            }
            'N' => {
                let back = self.vim.search_back;
                self.vim_next(back);
            }
            '*' | '#' => self.vim_star(c == '*', true),
            'm' | '\'' | '`' | '"' | 'q' | '@' | 'Z' | 'z' | '[' | ']' => {
                // q alone stops a recording
                if c == 'q' && self.vim.recording.is_some() {
                    let (r, mut keys) = self.vim.recording.take().unwrap();
                    keys.pop();
                    let text = keys.iter().map(|k| k.notation()).collect::<String>();
                    self.vim.regs.insert(r, Register { text, linewise: false });
                    self.vim.macros.insert(r, keys);
                    return true;
                }
                self.prefix = Some(c);
            }
            '.' => {
                let n = self.count.take();
                let keys = self.vim.dot.clone();
                self.vim.replaying += 1;
                for _ in 0..n.unwrap_or(1) {
                    // a count given to . replaces the one the change had
                    for k in &keys {
                        self.key(*k);
                    }
                }
                self.vim.replaying -= 1;
            }
            '~' => {
                let n = self.take_count();
                let e = (self.cursor + n).min(self.line_end(line));
                if e > self.cursor {
                    let t: String = self.buf.slice(self.cursor, e).chars().map(toggle).collect();
                    self.begin();
                    let at = self.cursor;
                    self.rem(at, e);
                    self.ins(at, &t);
                    self.close();
                    self.cursor = e;
                    self.clamp();
                }
            }
            's' => {
                let n = self.take_count();
                let e = (self.cursor + n).min(self.line_end(line));
                self.apply(Op::Change, Span::Chars(self.cursor, e));
            }
            'S' => {
                let n = self.take_count();
                self.apply(Op::Change, Span::Lines(line, (line + n - 1).min(self.buf.len_lines() - 1)));
            }
            'Y' => {
                let n = self.take_count();
                self.apply(Op::Yank, Span::Lines(line, (line + n - 1).min(self.buf.len_lines() - 1)));
            }
            'R' => {
                self.start_insert(self.cursor);
                self.vim.replace = true;
            }
            '+' => self.vim_line_motion(1),
            '-' => self.vim_line_motion(-1),
            '_' => self.vim_line_motion(0),
            '|' => {
                let n = self.take_count();
                let to = (self.line_start(line) + n - 1).min(self.line_end(line));
                self.motion(Span::Chars(self.cursor.min(to), self.cursor.max(to)), Some(to));
            }
            ' ' => self.normal_key(Key::ch('l')),
            '>' | '<' | '=' | '!' => {
                let op = match c { '>' => Op::ShiftRight, '<' => Op::ShiftLeft, '=' => Op::Reindent, _ => Op::Filter };
                self.vim_op(op);
            }
            '(' | ')' => self.vim_sentence(c == ')'),
            '{' | '}' => self.vim_paragraph(c == '}'),
            '%' => self.vim_percent(),
            'H' | 'M' | 'L' => {
                let (top, height, _) = self.vim.screen;
                let last = self.buf.len_lines() - 1;
                let bottom = (top + height.max(1) - 1).min(last);
                let n = self.count.take().unwrap_or(1).saturating_sub(1);
                let to = match c { 'H' => (top + n).min(bottom), 'L' => bottom.saturating_sub(n).max(top), _ => (top + bottom) / 2 };
                self.motion(Span::Lines(line, to), Some(self.first_non_blank(to)));
            }
            'K' => {
                if let Some((a, b)) = self.word_here() {
                    self.vim.requests.push(Request::Keyword(self.buf.slice(a, b)));
                }
            }
            '&' => self.vim.requests.push(Request::Ex("s".into())),
            'Q' => self.vim.requests.push(Request::CmdLine(String::new())),
            _ => return false,
        }
        true
    }

    /// An operator key: twice (or with the same g-key) means the lines (`>>`, `g~~`, `gUU`).
    fn vim_op(&mut self, op: Op) {
        if self.op == Some(op) {
            let n = self.take_count();
            let line = self.cur_line();
            let to = (line + n - 1).min(self.buf.len_lines() - 1);
            return self.apply(op, Span::Lines(line, to));
        }
        self.op = Some(op);
        self.op_count = self.count.take();
    }

    /// The second key after a prefix lib.rs does not handle; false when not one of these.
    pub(crate) fn vim_prefixed(&mut self, p: char, c: char) -> bool {
        match (p, c) {
            ('m', c) if c.is_ascii_alphabetic() || matches!(c, '\'' | '`' | '[' | ']' | '<' | '>') => {
                self.vim.marks.insert(c, self.cursor);
            }
            ('\'' | '`', m) => {
                let m = if m == '`' { '\'' } else { m };
                let at = match m {
                    '<' => self.vim.last_visual.map(|(a, b, _)| a.min(b)),
                    '>' => self.vim.last_visual.map(|(a, b, _)| a.max(b)),
                    _ => self.vim.marks.get(&m).copied(),
                };
                let Some(at) = at.map(|a| a.min(self.buf.len_chars())) else {
                    self.op = None;
                    self.count = None;
                    self.vim.requests.push(Request::Message("E20: Mark not set".into()));
                    return true;
                };
                // the jump back to here
                self.vim.marks.insert('\'', self.cursor);
                let line = self.cur_line();
                if p == '\'' {
                    let to = self.buf.to_pos(at).line;
                    self.motion(Span::Lines(line, to), Some(self.first_non_blank(to)));
                } else {
                    self.motion(Span::Chars(self.cursor.min(at), self.cursor.max(at)), Some(at));
                }
            }
            ('"', r) if r.is_ascii_alphanumeric() || matches!(r, '"' | '-' | '_' | '+' | '*' | '.' | ':' | '/') => {
                self.vim_use_reg(r);
            }
            ('q', r) if r.is_ascii_alphanumeric() || r == '"' => {
                self.vim.recording = Some((r.to_ascii_lowercase(), Vec::new()));
            }
            ('@', r) => {
                let r = if r == '@' { self.vim.last_macro.unwrap_or('q') } else { r.to_ascii_lowercase() };
                if r == ':' {
                    self.vim.requests.push(Request::Ex("@:".into()));
                    return true;
                }
                self.vim.last_macro = Some(r);
                let keys = self.vim.macros.get(&r).cloned().unwrap_or_else(|| crate::keys::parse(&self.vim.regs.get(&r).map(|x| x.text.clone()).unwrap_or_default()));
                let n = self.take_count();
                if self.vim.replaying > 20 {
                    return true;
                }
                self.vim.replaying += 1;
                for _ in 0..n {
                    for k in &keys {
                        self.key(*k);
                    }
                }
                self.vim.replaying -= 1;
            }
            ('z', c) => return self.vim_z(c),
            ('q', w @ (':' | '/' | '?')) => self.vim.requests.push(Request::CmdWindow(w)),
            (CW | CWG | CBS, c) => return self.vim_window_prefixed(p, c),
            ('Z', 'Z') => self.vim.requests.push(Request::Ex("x".into())),
            ('Z', 'Q') => self.vim.requests.push(Request::Ex("q!".into())),
            ('g', _) => return self.vim_g(c),
            _ => return false,
        }
        true
    }

    /// `g` and a key.
    fn vim_g(&mut self, c: char) -> bool {
        let line = self.cur_line();
        match c {
            '~' => self.vim_op(Op::Toggle),
            'u' => self.vim_op(Op::Lower),
            'U' => self.vim_op(Op::Upper),
            '?' => self.vim_op(Op::Rot13),
            'q' | 'w' => self.vim_op(Op::Format),
            '@' => self.vim_op(Op::Func),
            'J' => {
                let n = self.take_count().max(2) - 1;
                self.begin();
                for _ in 0..n {
                    let l = self.cur_line();
                    if l + 1 >= self.buf.len_lines() {
                        break;
                    }
                    let e = self.line_end(l);
                    self.rem(e, e + 1);
                    self.cursor = e;
                }
                self.close();
                self.clamp();
            }
            'i' => {
                let at = self.vim.last_insert.unwrap_or(self.cursor).min(self.buf.len_chars());
                self.start_insert(at);
            }
            'I' => {
                let s = self.line_start(line);
                self.start_insert(s);
            }
            'v' => {
                if let Some((a, b, m)) = self.vim.last_visual {
                    let n = self.buf.len_chars();
                    self.anchor = a.min(n);
                    self.cursor = b.min(n);
                    self.mode = m;
                }
            }
            ';' | ',' => {
                let n = self.vim.changes.len();
                if n == 0 {
                    self.vim.requests.push(Request::Message("E664: changelist is empty".into()));
                    return true;
                }
                let i = if c == ';' { self.vim.change_at.saturating_sub(1) } else { (self.vim.change_at + 1).min(n - 1) };
                self.vim.change_at = i;
                self.cursor = self.vim.changes[i].min(self.buf.len_chars());
                self.clamp();
            }
            'e' | 'E' => {
                // back to the end of the previous word
                let n = self.take_count();
                let mut i = self.cursor;
                for _ in 0..n {
                    let mut j = i;
                    let cls = |e: &Self, k: usize| e.char(k).map(|ch| if ch.is_whitespace() { 0 } else if c == 'E' || is_word(ch) { 1 } else { 2 }).unwrap_or(0);
                    let start = cls(self, j);
                    while j > 0 && cls(self, j) == start && start != 0 {
                        j -= 1;
                    }
                    while j > 0 && cls(self, j) == 0 {
                        j -= 1;
                    }
                    i = j;
                }
                self.motion(Span::Chars(i, self.cursor + 1), Some(i));
            }
            '_' => {
                let n = self.take_count();
                let to = (line + n - 1).min(self.buf.len_lines() - 1);
                let (s, mut e) = (self.line_start(to), self.line_end(to));
                while e > s && self.char(e - 1).is_some_and(char::is_whitespace) {
                    e -= 1;
                }
                let at = e.saturating_sub(1).max(s);
                self.motion(Span::Chars(self.cursor.min(at), self.cursor.max(at) + 1), Some(at));
            }
            '0' | '^' | '$' | 'm' | 'M' => {
                let (s, e) = (self.line_start(line), self.line_end(line));
                let at = match c {
                    '0' => s,
                    '^' => self.first_non_blank(line),
                    '$' => e.saturating_sub(1).max(s),
                    'm' => (s + self.vim.screen.2.max(2) / 2).min(e),
                    _ => s + (e - s) / 2,
                };
                self.motion(Span::Chars(self.cursor.min(at), self.cursor.max(at)), Some(at));
            }
            'o' => {
                // to byte N (1 by default)
                let n = self.count.take().unwrap_or(1).saturating_sub(1);
                let text = self.buf.text();
                let at = text.char_indices().take_while(|(b, _)| *b <= n).count().saturating_sub(1);
                self.motion(Span::Chars(self.cursor.min(at), self.cursor.max(at)), Some(at));
            }
            'p' | 'P' => {
                let n = self.take_count();
                self.put(c == 'p', n);
                // after what was put
                let len = self.reg.text.chars().count() * n;
                if self.reg.linewise {
                    let l = (self.cur_line() + self.reg.text.matches('\n').count() * n).min(self.buf.len_lines() - 1);
                    self.cursor = self.line_start(l);
                } else {
                    self.cursor = (self.cursor + 1).min(self.buf.len_chars()).max(self.cursor + len.min(1));
                }
            }
            '*' | '#' => self.vim_star(c == '*', false),
            'n' | 'N' => {
                // select the next match
                let back = self.vim.search_back;
                let fwd = if c == 'n' { !back } else { back };
                self.vim_next(fwd);
                let n = self.last_search.chars().count();
                if n > 0 {
                    self.anchor = self.cursor;
                    self.cursor = (self.cursor + n - 1).min(self.buf.len_chars());
                    self.mode = Mode::Visual;
                }
            }
            'a' | '8' => {
                if let Some(ch) = self.char(self.cursor) {
                    let m = if c == 'a' { format!("<{ch}> {}, Hex {:02x}, Oct {:o}", ch as u32, ch as u32, ch as u32) } else { ch.to_string().bytes().map(|b| format!("{b:02x}")).collect::<Vec<_>>().join(" ") };
                    self.vim.requests.push(Request::Message(m));
                }
            }
            's' => {
                let n = self.take_count();
                self.vim.requests.push(Request::Sleep(n as u64 * 1000));
            }
            '&' => self.vim.requests.push(Request::Ex("%s//~/&".into())),
            '<' => self.vim.requests.push(Request::Ex("messages".into())),
            'Q' => self.vim.requests.push(Request::CmdLine(String::new())),
            't' | 'T' => {
                let n = self.count.take();
                self.vim.requests.push(Request::Tab(match (c, n) { ('t', Some(n)) => 1000 + n as i32, ('t', None) => 1, _ => -(n.unwrap_or(1) as i32) }));
            }
            'x' | 'f' | 'F' => {
                // the address or file name under the cursor
                let (s, e) = (self.line_start(line), self.line_end(line));
                let ok = |ch: char| !ch.is_whitespace() && !matches!(ch, '"' | '\'' | '<' | '>' | '(' | ')' | '[' | ']' | '`');
                let (mut a, mut b) = (self.cursor, self.cursor);
                while a > s && self.char(a - 1).is_some_and(ok) {
                    a -= 1;
                }
                while b < e && self.char(b).is_some_and(ok) {
                    b += 1;
                }
                if b > a {
                    self.vim.requests.push(Request::Open(self.buf.slice(a, b)));
                }
            }
            'd' | 'D' => {
                // the word's first use (in the file: gD; Vim's gd looks from the function start)
                let Some((a, b)) = self.word_here() else { return true };
                let w = self.buf.slice(a, b);
                let mut from = 0;
                while let Some(i) = self.buf.find(&w, from, true, false) {
                    if self.whole_word_at(i, w.chars().count()) {
                        self.vim.marks.insert('\'', self.cursor);
                        self.last_search = w.clone();
                        self.vim.search_word = true;
                        self.cursor = i;
                        self.clamp();
                        break;
                    }
                    from = i + 1;
                }
            }
            'h' | 'H' | 'V' => {
                // select mode: zett's visual
                self.anchor = self.cursor;
                self.mode = if c == 'H' { Mode::VisualLine } else { Mode::Visual };
            }
            'R' | 'r' => {
                if c == 'R' {
                    self.start_insert(self.cursor);
                    self.vim.replace = true;
                } else {
                    self.prefix = Some('r');
                }
            }
            _ => return false,
        }
        true
    }

    /// The new operators on what a motion covers.
    pub(crate) fn vim_apply(&mut self, op: Op, span: Span) -> bool {
        if !matches!(op, Op::Toggle | Op::Lower | Op::Upper | Op::Rot13 | Op::ShiftRight | Op::ShiftLeft | Op::Reindent | Op::Format | Op::Filter | Op::Func) {
            return false;
        }
        self.op = None;
        self.count = None;
        let (a, b, lines) = match span {
            Span::Chars(a, b) => (a.min(b), a.max(b), None),
            Span::Lines(l1, l2) => {
                let (l1, l2) = (l1.min(l2), l1.max(l2));
                (self.line_start(l1), self.line_end(l2), Some((l1, l2)))
            }
        };
        let (l1, l2) = lines.unwrap_or_else(|| (self.buf.to_pos(a).line, self.buf.to_pos(b.saturating_sub(1).max(a)).line));
        match op {
            Op::Toggle | Op::Lower | Op::Upper | Op::Rot13 => {
                let t: String = self.buf.slice(a, b).chars().map(|ch| match op { Op::Toggle => toggle(ch), Op::Lower => ch.to_lowercase().next().unwrap_or(ch), Op::Upper => ch.to_uppercase().next().unwrap_or(ch), _ => rot13(ch) }).collect();
                self.replace_range(a, b, &t);
                self.cursor = a;
            }
            Op::ShiftRight | Op::ShiftLeft | Op::Reindent => {
                let sw = self.soft_tab.unwrap_or(4);
                self.begin();
                for l in l1..=l2 {
                    let s = self.line_start(l);
                    let ind = self.indent_of(l);
                    let width = ind.chars().map(|c| if c == '\t' { sw } else { 1 }).sum::<usize>();
                    let blank = self.buf.line_len(l) == ind.chars().count();
                    let new = match op {
                        Op::ShiftRight if !blank => width + sw,
                        Op::ShiftRight => width,
                        Op::ShiftLeft => width.saturating_sub(sw) / sw * sw + if width % sw != 0 && width > sw { 0 } else { 0 },
                        // = : the indent of the nearest non-blank line above (Vim's default with no indentexpr)
                        _ => (0..l).rev().find(|&k| self.buf.line_len(k) > self.indent_of(k).chars().count()).map(|k| self.indent_of(k).chars().map(|c| if c == '\t' { sw } else { 1 }).sum()).unwrap_or(0),
                    };
                    let n = ind.chars().count();
                    self.rem(s, s + n);
                    self.ins(s, &" ".repeat(new));
                }
                self.close();
                self.cursor = self.first_non_blank(l1);
            }
            Op::Format => {
                let width = self.fill_column.max(10);
                let text = self.buf.slice(self.line_start(l1), self.line_end(l2));
                let mut out: Vec<String> = Vec::new();
                for para in text.split("\n\n") {
                    let first = para.lines().next().unwrap_or("");
                    let (p1, rest) = crate::textops::fill_prefix(first);
                    let words: Vec<&str> = para.split_whitespace().collect();
                    out.push(crate::textops::fill(&words, &p1, &rest, width).join("\n"));
                }
                let (s, e) = (self.line_start(l1), self.line_end(l2));
                self.replace_range(s, e, &out.join("\n\n"));
                self.cursor = self.first_non_blank(l1.min(self.buf.len_lines() - 1));
            }
            Op::Filter => self.vim.requests.push(Request::CmdLine(if l1 == l2 { ".!".into() } else { format!(".,.+{}!", l2 - l1) })),
            Op::Func => self.vim.requests.push(Request::OperatorFunc(a, b)),
            _ => {}
        }
        self.clamp();
        self.want_col = self.cursor().col;
        true
    }

    /// Replaces [a, b) with `t` as one change.
    fn replace_range(&mut self, a: usize, b: usize, t: &str) {
        if self.buf.slice(a, b) == t {
            return;
        }
        self.begin();
        self.rem(a, b);
        self.ins(a, t);
        self.close();
    }

    // ── sentences, paragraphs, brackets ──

    fn vim_paragraph(&mut self, forward: bool) {
        let n = self.take_count();
        let last = self.buf.len_lines() - 1;
        let mut l = self.cur_line();
        let blank = |e: &Self, l: usize| e.buf.line_len(l) == 0;
        for _ in 0..n {
            if forward {
                while l < last && blank(self, l) {
                    l += 1;
                }
                while l < last && !blank(self, l) {
                    l += 1;
                }
            } else {
                while l > 0 && blank(self, l) {
                    l -= 1;
                }
                while l > 0 && !blank(self, l) {
                    l -= 1;
                }
            }
        }
        let to = if forward && l == last && !blank(self, l) { self.line_end(l) } else { self.line_start(l) };
        self.motion(Span::Chars(self.cursor.min(to), self.cursor.max(to)), Some(to));
    }

    fn vim_sentence(&mut self, forward: bool) {
        let n = self.take_count();
        let len = self.buf.len_chars();
        let mut i = self.cursor;
        let ends = |e: &Self, k: usize| e.char(k).is_some_and(|c| matches!(c, '.' | '!' | '?')) && e.char(k + 1).is_none_or(|c| c.is_whitespace());
        for _ in 0..n {
            if forward {
                let mut k = i;
                while k < len && !ends(self, k) && !(self.char(k) == Some('\n') && self.char(k + 1) == Some('\n')) {
                    k += 1;
                }
                k += 1;
                while k < len && self.char(k).is_some_and(char::is_whitespace) {
                    k += 1;
                }
                i = k.min(len);
            } else {
                let mut k = i.saturating_sub(1);
                while k > 0 && self.char(k).is_some_and(char::is_whitespace) {
                    k -= 1;
                }
                while k > 0 && !ends(self, k - 1) && !(self.char(k - 1) == Some('\n') && self.char(k.saturating_sub(2)) == Some('\n')) {
                    k -= 1;
                }
                i = k;
            }
        }
        self.motion(Span::Chars(self.cursor.min(i), self.cursor.max(i)), Some(i));
    }

    /// `%`: the bracket matching the one at or after the cursor; with a count, that percentage of the file.
    fn vim_percent(&mut self) {
        if let Some(n) = self.count.take() {
            let last = self.buf.len_lines() - 1;
            let to = ((n.min(100) * (last + 1)).div_ceil(100)).saturating_sub(1).min(last);
            let line = self.cur_line();
            return self.motion(Span::Lines(line, to), Some(self.first_non_blank(to)));
        }
        let line = self.cur_line();
        let e = self.line_end(line);
        let pairs = [('(', ')'), ('[', ']'), ('{', '}')];
        let Some((at, ch)) = (self.cursor..e).find_map(|i| self.char(i).filter(|c| pairs.iter().any(|(a, b)| a == c || b == c)).map(|c| (i, c))) else { return };
        let (open, close, fwd) = pairs.iter().find_map(|&(a, b)| if ch == a { Some((a, b, true)) } else if ch == b { Some((a, b, false)) } else { None }).unwrap();
        let mut depth = 0i32;
        let mut i = at as isize;
        let len = self.buf.len_chars() as isize;
        while i >= 0 && i < len {
            let c = self.char(i as usize).unwrap_or(' ');
            if c == open {
                depth += if fwd { 1 } else { -1 };
            } else if c == close {
                depth += if fwd { -1 } else { 1 };
            }
            if depth == 0 {
                let to = i as usize;
                self.vim.marks.insert('\'', self.cursor);
                return self.motion(Span::Chars(self.cursor.min(to), self.cursor.max(to) + 1), Some(to));
            }
            i += if fwd { 1 } else { -1 };
        }
    }

    /// Typing in `R` mode replaces the character under the cursor (not the line break).
    pub(crate) fn vim_replace_char(&mut self) {
        if self.vim.replace && self.char(self.cursor).is_some_and(|c| c != '\n') {
            let at = self.cursor;
            self.rem(at, at + 1);
        }
    }

    /// What the window should do, since the last time it asked.
    pub fn take_requests(&mut self) -> Vec<Request> {
        std::mem::take(&mut self.vim.requests)
    }
    /// Lines shown (top, height) and the window's width: for H M L and gm.
    pub fn set_screen(&mut self, top: usize, height: usize, width: usize) {
        self.vim.screen = (top, height, width);
    }
    /// The wrapscan setting: searches go on from the other end.
    pub fn set_wrapscan(&mut self, on: bool) {
        self.vim.wrapscan = on;
    }
    /// A register's text (`"a` …), for :registers and the window.
    pub fn register(&self, r: char) -> Option<(String, bool)> {
        if r == '"' {
            return Some((self.reg.text.clone(), self.reg.linewise));
        }
        self.vim.regs.get(&r).map(|x| (x.text.clone(), x.linewise))
    }
    /// The macro being recorded, for the mode line ("recording @q").
    pub fn recording(&self) -> Option<char> {
        self.vim.recording.as_ref().map(|r| r.0)
    }
}

fn toggle(c: char) -> char {
    if c.is_uppercase() { c.to_lowercase().next().unwrap_or(c) } else { c.to_uppercase().next().unwrap_or(c) }
}

fn rot13(c: char) -> char {
    match c {
        'a'..='z' => (((c as u8 - b'a' + 13) % 26) + b'a') as char,
        'A'..='Z' => (((c as u8 - b'A' + 13) % 26) + b'A') as char,
        _ => c,
    }
}

#[cfg(test)]
mod tests {
    use crate::Editor;

    fn ed(t: &str) -> Editor {
        let mut e = Editor::from_str(t);
        e.set_wrapscan(true);
        e
    }

    #[test]
    fn search_with_slash_n_and_star() {
        let mut e = ed("one two one three one");
        e.feed("/one<CR>");
        assert_eq!(e.cursor, 8);
        e.feed("n");
        assert_eq!(e.cursor, 18);
        e.feed("n");
        assert_eq!(e.cursor, 0, "wraps around");
        e.feed("N");
        assert_eq!(e.cursor, 18);
        e.feed("?two<CR>");
        assert_eq!(e.cursor, 4);
        let mut e = ed("foo foobar foo");
        e.feed("*");
        assert_eq!(e.cursor, 11, "* skips foobar: whole words");
        e.feed("d/bar<CR>");
        // from 11 nothing matches forward but it wraps to 7: the motion deletes backward to it
        assert!(e.text().len() < 14);
    }

    #[test]
    fn marks_registers_macros_and_dot() {
        let mut e = ed("alpha\nbeta\ngamma\n");
        e.feed("majj'a");
        assert_eq!(e.cursor().line, 0);
        e.feed("\"ayy");
        e.feed("j\"byy");
        e.feed("G\"ap");
        assert!(e.text().ends_with("gamma\nalpha\n"), "{:?}", e.text());
        e.feed("\"Ayy");
        assert_eq!(e.register('a').unwrap().0, "alpha\nalpha\n", "upper case appends");
        let mut e = ed("1\n2\n3\n");
        e.feed("qqA!<Esc>jq");
        e.feed("@q");
        assert_eq!(e.text(), "1!\n2!\n3\n");
        e.feed("@@");
        assert_eq!(e.text(), "1!\n2!\n3!\n");
        let mut e = ed("a b c d");
        e.feed("dw");
        e.feed("..");
        assert_eq!(e.text(), "d");
    }

    #[test]
    fn counting_case_shifts_and_lines() {
        let mut e = ed("x = 41 and 0x0f");
        e.feed("<C-a>");
        assert_eq!(e.text(), "x = 42 and 0x0f");
        e.feed("w5<C-x>");
        assert!(e.text().contains("0x0a"), "{:?}", e.text());
        let mut e = ed("Hello World");
        e.feed("g~w");
        assert_eq!(e.text(), "hELLO World");
        e.feed("gUU");
        assert_eq!(e.text(), "HELLO WORLD");
        e.feed("g??");
        assert_eq!(e.text(), "URYYB JBEYQ");
        let mut e = ed("a\nb\n");
        e.feed(">>j>>");
        assert_eq!(e.text(), "    a\n    b\n");
        e.feed("<<");
        assert_eq!(e.text(), "    a\nb\n");
        e.feed("~");
        assert_eq!(e.text(), "    a\nB\n");
    }

    #[test]
    fn brackets_paragraphs_and_replace_mode() {
        let mut e = ed("f(a, (b))");
        e.feed("%");
        assert_eq!(e.cursor, 8);
        e.feed("%");
        assert_eq!(e.cursor, 1);
        let mut e = ed("a\nb\n\nc\n");
        e.feed("}");
        assert_eq!(e.cursor().line, 2);
        let mut e = ed("abcd");
        e.feed("RXY<Esc>");
        assert_eq!(e.text(), "XYcd");
        let mut e = ed("a b");
        e.feed("ZZ");
        assert_eq!(e.take_requests(), vec![super::Request::Ex("x".into())]);
    }
}

impl Editor {
    /// With an operator pending, its own key again means its lines (`g~~` `guu` `gUU` `g??` `>>`).
    pub(crate) fn vim_double(&mut self, c: char) -> bool {
        let same = matches!((self.op, c), (Some(Op::Toggle), '~') | (Some(Op::Lower), 'u') | (Some(Op::Upper), 'U') | (Some(Op::Rot13), '?') | (Some(Op::Format), 'q' | 'w') | (Some(Op::ShiftRight), '>') | (Some(Op::ShiftLeft), '<') | (Some(Op::Reindent), '=') | (Some(Op::Filter), '!'));
        if same {
            let op = self.op.unwrap();
            self.vim_op(op);
        }
        same
    }

    /// For the parity probe: the Vim state a key can change (not the keys being typed).
    pub(crate) fn vim_probe(&self) -> String {
        let v = &self.vim;
        let mut marks: Vec<_> = v.marks.iter().collect();
        marks.sort();
        let mut regs: Vec<_> = v.regs.iter().map(|(k, r)| (*k, r.text.clone())).collect();
        regs.sort();
        format!("{marks:?}{regs:?}{:?}{:?}{:?}{:?}{}{}", v.use_reg, v.recording.as_ref().map(|r| r.0), v.requests, v.last_visual, v.replace, v.search_back)
    }
}

impl Editor {
    /// Where the history stands: a change since shows as a difference (an open change counts its edits).
    fn edit_marker(&self) -> (usize, usize, usize) {
        (self.undo.nodes.len(), self.undo.cur, self.open.as_ref().map_or(0, |c| c.edits.len()))
    }
}

/// Prefixes beyond the visible keys: after CTRL-W, after CTRL-W g, after CTRL-\.
pub(crate) const CW: char = '\u{17}';
const CWG: char = '\u{1e}';
const CBS: char = '\u{1c}';

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
                let n = self.count.take().unwrap_or(height / 2).max(1);
                let (t, to) = if c == 'd' { ((top + n).min(last.saturating_sub(height / 2).max(top)), (line + n).min(last)) } else { (top.saturating_sub(n), line.saturating_sub(n)) };
                self.vim_top(t);
                self.cursor = self.first_non_blank(to);
            }
            'f' | 'b' => {
                let n = self.take_count();
                let page = height.saturating_sub(2).max(1) * n;
                let t = if c == 'f' { (top + page).min(last) } else { top.saturating_sub(page) };
                self.vim_top(t);
                let to = if c == 'f' { t } else { (t + height - 1).min(last) };
                self.cursor = self.first_non_blank(to);
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
    fn vim_word_count(&mut self) {
        let text = self.buf.text();
        let before: String = text.chars().take(self.cursor).collect();
        let (w, wb) = (text.split_whitespace().count(), before.split_whitespace().count());
        let m = format!("Col {} of {}; Line {} of {}; Word {wb} of {w}; Char {} of {}", self.cursor().col + 1, self.buf.line_len(self.cur_line()), self.cur_line() + 1, self.buf.len_lines(), self.cursor + 1, self.buf.len_chars());
        self.vim.requests.push(Request::Message(m));
    }

    /// The jump list: `dir` -1 back (CTRL-O), +1 forward (CTRL-I / Tab).
    fn vim_jump(&mut self, dir: i32) {
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

    /// `z` and a key: where the window stands (zt zz zb z<CR> z. z- z+ z^), sideways (zh zl zs ze zH zL).
    pub(crate) fn vim_z(&mut self, c: char) -> bool {
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
            _ => return false,
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

#[cfg(test)]
mod batch2 {
    use super::Request;
    use crate::Editor;

    fn lines(n: usize) -> Editor {
        let t: String = (1..=n).map(|i| format!("line {i}\n")).collect();
        let mut e = Editor::from_str(&t);
        e.set_screen(0, 10, 40);
        e
    }

    #[test]
    fn windows_scrolling_and_the_view() {
        let mut e = lines(100);
        e.feed("<C-w>v2<C-w>+<C-w><C-w><C-w>gf");
        assert_eq!(e.take_requests(), vec![Request::Window("v".into(), None), Request::Window("+".into(), Some(2)), Request::Window("C-w".into(), None), Request::Window("gf".into(), None)]);
        e.feed("<C-d>");
        assert_eq!(e.cursor().line, 5);
        assert_eq!(e.take_requests(), vec![Request::Top(5)]);
        e.set_screen(5, 10, 40);
        e.feed("<C-e><C-e>");
        assert_eq!(e.take_requests(), vec![Request::Top(6), Request::Top(7)]);
        e.feed("30Gzz");
        assert_eq!(e.take_requests(), vec![Request::Top(24)]);
        e.feed("zt");
        assert_eq!(e.take_requests(), vec![Request::Top(29)]);
    }

    #[test]
    fn the_jump_list_goes_back_and_forth() {
        let mut e = lines(100);
        e.feed("50G");
        e.feed("80G");
        e.feed("<C-o>");
        assert_eq!(e.cursor().line, 49);
        e.feed("<C-o>");
        assert_eq!(e.cursor().line, 0);
        e.feed("<C-i>");
        assert_eq!(e.cursor().line, 49);
        e.feed("<Tab>");
        assert_eq!(e.cursor().line, 79);
    }
}

impl Editor {
    /// The window's top line moves to `t`: asked of the window, and kept here until it says otherwise.
    fn vim_top(&mut self, t: usize) {
        self.vim.screen.0 = t;
        self.vim.requests.push(Request::Top(t));
    }
}
