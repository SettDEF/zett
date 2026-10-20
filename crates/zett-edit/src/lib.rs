//! zett's modal engine: Vim's grammar over a text buffer.
//!
//! WASM-clean (no I/O): the window, the terminal UI and the browser extension
//! all feed it keys and draw what it holds.
//!
//! M0 covers the core grammar: counts, `h j k l w b e 0 ^ $ gg G`, operators
//! `d c y` with those motions (and doubled for lines), `x X D C r p P J`,
//! insert entries `i a I A o O`, undo and redo as a tree (`u C-r g- g+`); visual `v` `V` with operators, case,
//! replace and put; text objects (words, quotes, brackets, paragraphs); `f t F T ; ,`.
//! Every case in tests/nvim-cases.tsv behaves exactly as in Neovim. Still to come:
//! registers by name, dot-repeat, macros and search.

pub mod emacs;
pub mod keys;
mod undo;
pub use keys::{Code, Key};

use zett_core::{Buffer, Pos};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
    /// `v`: characters from the anchor to the cursor (both included)
    Visual,
    /// `V`: whole lines from the anchor's to the cursor's
    VisualLine,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Op {
    Delete,
    Change,
    Yank,
}

/// What a motion covers: characters (exclusive end) or whole lines.
#[derive(Clone, Copy, Debug)]
enum Span {
    Chars(usize, usize),
    Lines(usize, usize),
}

#[derive(Clone, Debug, Default)]
struct Register {
    text: String,
    linewise: bool,
}

use undo::{Change, Edit};

pub struct Editor {
    pub buf: Buffer,
    cursor: usize,
    mode: Mode,
    /// the column j/k try to keep
    want_col: usize,
    count: Option<usize>,
    op: Option<Op>,
    op_count: Option<usize>,
    /// a key waiting for its second key: `g`, `r`
    prefix: Option<char>,
    reg: Register,
    undo: undo::Tree,
    /// the change being recorded, closed by the next one or by `u`
    open: Option<Change>,
    /// visual mode: where the selection started
    anchor: usize,
    /// the last f F t T and its character, for ; and ,
    last_find: Option<(char, char)>,
    /// Emacs profile (emacs.rs): modeless, keys are commands
    emacs: bool,
    /// Emacs's mark: with the cursor it makes the region
    mark: Option<usize>,
    /// the region is active (shown, acted on by default): transient-mark-mode
    mark_active: bool,
    kill_ring: Vec<String>,
    /// counts kills, so a front end can tell when to copy to the system clipboard
    kills: u64,
    /// the last yank (start, end, kill-ring index), for yank-pop
    yanked: Option<(usize, usize, usize)>,
    /// the command that ran last (kills append to kills, typing groups into one undo step)
    last_cmd: String,
}

fn class(c: char) -> u8 {
    if c.is_whitespace() { 0 } else if c.is_alphanumeric() || c == '_' { 2 } else { 1 }
}

impl Editor {
    pub fn new(buf: Buffer) -> Self {
        Self {
            buf,
            cursor: 0,
            mode: Mode::Normal,
            anchor: 0,
            last_find: None,
            want_col: 0,
            count: None,
            op: None,
            op_count: None,
            prefix: None,
            reg: Register::default(),
            undo: undo::Tree::default(),
            open: None,
            emacs: false,
            mark: None,
            mark_active: false,
            kill_ring: Vec::new(),
            kills: 0,
            yanked: None,
            last_cmd: String::new(),
        }
    }

    pub fn from_str(s: &str) -> Self {
        Self::new(Buffer::from_str(s))
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }
    pub fn text(&self) -> String {
        self.buf.text()
    }
    pub fn cursor(&self) -> Pos {
        self.buf.to_pos(self.cursor)
    }
    pub fn set_cursor(&mut self, p: Pos) {
        self.cursor = self.buf.to_char(p);
        self.clamp();
        self.want_col = self.cursor().col;
    }

    /// Feeds keys in Vim notation (`"dwihello<Esc>"`).
    pub fn feed(&mut self, keys: &str) {
        for k in keys::parse(keys) {
            self.key(k);
        }
    }

    pub fn key(&mut self, k: Key) {
        match self.mode {
            Mode::Insert => self.insert_key(k),
            Mode::Normal => self.normal_key(k),
            Mode::Visual | Mode::VisualLine => self.visual_key(k),
        }
    }

    // ── positions ──

    fn line_start(&self, line: usize) -> usize {
        self.buf.to_char(Pos::new(line, 0))
    }
    fn line_end(&self, line: usize) -> usize {
        // the char index just past the line's last char (before "\n")
        self.line_start(line) + self.buf.line_len(line)
    }
    fn cur_line(&self) -> usize {
        self.cursor().line
    }
    /// Normal mode keeps the cursor ON a character, never past the last one.
    fn clamp(&mut self) {
        let line = self.cur_line().min(self.buf.len_lines() - 1);
        let (s, len) = (self.line_start(line), self.buf.line_len(line));
        // normal and visual stay on a character; visual after `$` may sit on the line break
        let on_char = self.mode == Mode::Normal || (matches!(self.mode, Mode::Visual | Mode::VisualLine) && self.want_col != usize::MAX);
        let max = if on_char && len > 0 { s + len - 1 } else { s + len };
        self.cursor = self.cursor.clamp(s, max);
    }
    fn char(&self, i: usize) -> Option<char> {
        self.buf.char_at(i)
    }

    // ── word motions (Vim's `w b e`: keyword runs, punctuation runs, blank lines stop `w`) ──

    fn word_forward(&self, mut i: usize) -> usize {
        let n = self.buf.len_chars();
        if i >= n {
            return n;
        }
        let c0 = class(self.char(i).unwrap());
        if c0 != 0 {
            while i < n && class(self.char(i).unwrap()) == c0 {
                i += 1;
            }
        }
        // skip blanks; an empty line is a word of its own
        while i < n {
            let c = self.char(i).unwrap();
            if c == '\n' && i + 1 < n && self.char(i + 1) == Some('\n') {
                return i + 1;
            }
            if !c.is_whitespace() {
                break;
            }
            i += 1;
        }
        i
    }

    fn word_end(&self, mut i: usize) -> usize {
        let n = self.buf.len_chars();
        if n == 0 {
            return 0;
        }
        i += 1;
        while i < n && self.char(i).unwrap().is_whitespace() {
            i += 1;
        }
        if i >= n {
            return n - 1;
        }
        let c0 = class(self.char(i).unwrap());
        while i + 1 < n && class(self.char(i + 1).unwrap()) == c0 {
            i += 1;
        }
        i
    }

    fn word_back(&self, mut i: usize) -> usize {
        if i == 0 {
            return 0;
        }
        i -= 1;
        while i > 0 && self.char(i).unwrap().is_whitespace() {
            i -= 1;
        }
        let c0 = class(self.char(i).unwrap());
        while i > 0 && class(self.char(i - 1).unwrap()) == c0 {
            i -= 1;
        }
        i
    }

    /// The whitespace a line starts with.
    fn indent_of(&self, line: usize) -> String {
        let s = self.line_start(line);
        self.buf.slice(s, self.first_non_blank(line).max(s))
    }

    fn first_non_blank(&self, line: usize) -> usize {
        let s = self.line_start(line);
        let len = self.buf.line_len(line);
        let mut i = s;
        while i < s + len && self.char(i).is_some_and(|c| c == ' ' || c == '\t') {
            i += 1;
        }
        i
    }

    // ── normal mode ──

    fn take_count(&mut self) -> usize {
        let c = self.count.take().unwrap_or(1) * self.op_count.take().unwrap_or(1);
        c.max(1)
    }

    fn normal_key(&mut self, k: Key) {
        if k.code == Code::Esc {
            self.count = None;
            self.op = None;
            self.op_count = None;
            self.prefix = None;
            return;
        }
        if k.ctrl {
            if k.code == Code::Char('r') {
                let n = self.take_count();
                for _ in 0..n {
                    self.redo_once();
                }
            }
            return;
        }
        let Some(c) = k.char() else {
            // arrows behave like h j k l
            let mapped = match k.code {
                Code::Left => 'h',
                Code::Right => 'l',
                Code::Up => 'k',
                Code::Down => 'j',
                Code::Home => '0',
                Code::End => '$',
                _ => return,
            };
            return self.normal_key(Key::ch(mapped));
        };

        // second key of a two-key command
        if let Some(p) = self.prefix.take() {
            match (p, c) {
                // g- g+: back and forth in time, across branches
                ('g', '-') => {
                    let n = self.take_count();
                    return self.undo_goto(self.undo.cur.saturating_sub(n));
                }
                ('g', '+') => {
                    let n = self.take_count();
                    return self.undo_goto((self.undo.cur + n).min(self.undo.nodes.len() - 1));
                }
                ('g', 'g') => {
                    let line = self.count.take().map(|n| n.saturating_sub(1)).unwrap_or(0);
                    return self.motion(Span::Lines(self.cur_line(), line.min(self.buf.len_lines() - 1)), Some(self.first_non_blank(line.min(self.buf.len_lines() - 1))));
                }
                ('r', ch) if matches!(self.mode, Mode::Visual | Mode::VisualLine) => return self.visual_replace(ch),
                ('r', ch) => return self.replace_chars(ch),
                ('i' | 'a', obj) => return self.text_object_key(p == 'i', obj),
                ('f' | 'F' | 't' | 'T', ch) => {
                    self.last_find = Some((p, ch));
                    return self.find(p, ch);
                }
                _ => return,
            }
        }

        if c.is_ascii_digit() && (c != '0' || self.count.is_some()) {
            let d = c.to_digit(10).unwrap() as usize;
            self.count = Some(self.count.unwrap_or(0).saturating_mul(10).saturating_add(d));
            return;
        }

        let line = self.cur_line();
        match c {
            // motions
            'h' => {
                let n = self.take_count();
                let to = self.cursor.saturating_sub(n).max(self.line_start(line));
                self.motion(Span::Chars(to, self.cursor), Some(to));
            }
            'l' => {
                let n = self.take_count();
                let to = (self.cursor + n).min(self.line_end(line));
                self.motion(Span::Chars(self.cursor, to), Some(to));
            }
            'j' | 'k' => {
                let n = self.take_count();
                let last = self.buf.len_lines() - 1;
                let to = if c == 'j' { (line + n).min(last) } else { line.saturating_sub(n) };
                self.motion(Span::Lines(line, to), None);
                if self.op.is_none() {
                    self.vertical_to(to);
                }
            }
            'w' | 'W' => {
                let n = self.take_count();
                let mut to = self.cursor;
                for _ in 0..n {
                    to = self.word_forward(to);
                }
                if let Some(op) = self.op {
                    // `cw` on a word is `ce`; `dw` never eats the line break
                    if op == Op::Change && self.char(self.cursor).is_some_and(|ch| !ch.is_whitespace()) {
                        let mut e = self.cursor;
                        for i in 0..n {
                            e = if i == 0 && self.is_word_end(e) { e } else { self.word_end(e) };
                        }
                        return self.motion(Span::Chars(self.cursor, e + 1), None);
                    }
                    let end = to.min(self.line_end(line).max(self.cursor + 1));
                    let end = if self.buf.to_pos(to).line > line { self.line_end(line) } else { end };
                    return self.motion(Span::Chars(self.cursor, end), None);
                }
                self.motion(Span::Chars(self.cursor, to), Some(to));
            }
            'e' | 'E' => {
                let n = self.take_count();
                let mut to = self.cursor;
                for _ in 0..n {
                    to = self.word_end(to);
                }
                self.motion(Span::Chars(self.cursor, to + 1), Some(to));
            }
            'b' | 'B' => {
                let n = self.take_count();
                let mut to = self.cursor;
                for _ in 0..n {
                    to = self.word_back(to);
                }
                self.motion(Span::Chars(to, self.cursor), Some(to));
            }
            '0' => {
                let s = self.line_start(line);
                self.motion(Span::Chars(s, self.cursor), Some(s));
            }
            '^' => {
                let s = self.first_non_blank(line);
                self.motion(Span::Chars(s.min(self.cursor), s.max(self.cursor)), Some(s));
            }
            '$' => {
                let n = self.take_count();
                let target = (line + n - 1).min(self.buf.len_lines() - 1);
                let e = self.line_end(target);
                self.motion(Span::Chars(self.cursor, e), Some(e));
                self.want_col = usize::MAX;
            }
            'G' => {
                let last = self.buf.len_lines() - 1;
                let to = self.count.take().map(|n| n.saturating_sub(1).min(last)).unwrap_or(last);
                self.motion(Span::Lines(line, to), Some(self.first_non_blank(to)));
            }
            'g' | 'r' | 'f' | 'F' | 't' | 'T' => self.prefix = Some(c),
            ';' | ',' => {
                if let Some((k, ch)) = self.last_find {
                    // , goes the other way
                    let k = if c == ',' { match k { 'f' => 'F', 'F' => 'f', 't' => 'T', _ => 't' } } else { k };
                    self.find(k, ch);
                } else {
                    self.count = None;
                    self.op = None;
                }
            }

            // operators: `dd` `cc` `yy` act on lines
            'd' | 'c' | 'y' => {
                let op = match c { 'd' => Op::Delete, 'c' => Op::Change, _ => Op::Yank };
                if self.op == Some(op) {
                    let n = self.take_count();
                    let to = (line + n - 1).min(self.buf.len_lines() - 1);
                    return self.apply(op, Span::Lines(line, to));
                }
                self.op = Some(op);
                self.op_count = self.count.take();
            }

            // shortcuts
            'x' => { let n = self.take_count(); let e = (self.cursor + n).min(self.line_end(line)); if e > self.cursor { self.apply(Op::Delete, Span::Chars(self.cursor, e)); } }
            'X' => { let n = self.take_count(); let s = self.cursor.saturating_sub(n).max(self.line_start(line)); if s < self.cursor { self.apply(Op::Delete, Span::Chars(s, self.cursor)); } }
            'D' => { let e = self.line_end(line); self.apply(Op::Delete, Span::Chars(self.cursor, e)); }
            'C' => { let e = self.line_end(line); self.apply(Op::Change, Span::Chars(self.cursor, e)); }
            'p' | 'P' => { let n = self.take_count(); self.put(c == 'p', n); }
            'J' => { let n = self.take_count().max(2) - 1; self.join(n); }

            // into insert mode
            // with an operator pending, i and a start a text object (`diw`, `ca"`)
            'i' | 'a' if self.op.is_some() => self.prefix = Some(c),
            'v' => { self.anchor = self.cursor; self.mode = Mode::Visual; self.count = None; }
            'V' => { self.anchor = self.cursor; self.mode = Mode::VisualLine; self.count = None; }
            'i' => self.start_insert(self.cursor),
            'a' => { let at = (self.cursor + 1).min(self.line_end(line)); self.start_insert(at) }
            'I' => self.start_insert(self.first_non_blank(line)),
            'A' => self.start_insert(self.line_end(line)),
            // autoindent (Neovim's default): the new line starts at this one's indent
            'o' => { self.begin(); let ind = self.indent_of(line); let e = self.line_end(line); self.ins(e, &format!("\n{ind}")); self.mode = Mode::Insert; self.cursor = e + 1 + ind.chars().count(); }
            'O' => { self.begin(); let ind = self.indent_of(line); let s = self.line_start(line); self.ins(s, &format!("{ind}\n")); self.mode = Mode::Insert; self.cursor = s + ind.chars().count(); }

            'u' => { let n = self.take_count(); for _ in 0..n { self.undo_once(); } }
            _ => { self.count = None; self.op = None; }
        }
    }

    // ── visual mode ──

    /// The selection as a span: characters from the anchor to the cursor (both
    /// included), or whole lines in `V`.
    fn selection(&self) -> Span {
        if self.mode == Mode::VisualLine {
            return Span::Lines(self.buf.to_pos(self.anchor).line, self.cur_line());
        }
        let (a, b) = (self.anchor.min(self.cursor), self.anchor.max(self.cursor));
        Span::Chars(a, (b + 1).min(self.buf.len_chars()))
    }

    /// The selected characters (start, end exclusive), for drawing and for commands on a
    /// selection: Vim's visual selection, or the Emacs profile's active region.
    pub fn selection_range(&self) -> Option<(usize, usize)> {
        if self.emacs {
            return self.region().filter(|(a, b)| a < b);
        }
        match self.mode {
            Mode::Visual | Mode::VisualLine => Some(match self.selection() {
                Span::Chars(a, b) => (a, b),
                Span::Lines(l1, l2) => {
                    let (l1, l2) = (l1.min(l2), l1.max(l2));
                    (self.line_start(l1), self.line_end(l2))
                }
            }),
            _ => None,
        }
    }

    fn leave_visual(&mut self) {
        self.mode = Mode::Normal;
        self.want_col = self.want_col.min(usize::MAX - 1);
        self.clamp();
    }

    /// An operator on the selection (`d`, `c`, `y`); `lines` forces whole lines (`D`, `Y`, `X`, `S`).
    fn visual_op(&mut self, op: Op, lines: bool) {
        let span = match self.selection() {
            Span::Chars(a, b) if lines => Span::Lines(self.buf.to_pos(a).line, self.buf.to_pos(b.saturating_sub(1).max(a)).line),
            s => s,
        };
        let start = match span {
            Span::Chars(a, _) => a,
            Span::Lines(l1, l2) => self.line_start(l1.min(l2)),
        };
        self.mode = Mode::Normal;
        self.want_col = 0;
        if op == Op::Yank {
            // the cursor goes to the start of what was yanked
            self.apply(op, span);
            self.cursor = match span {
                Span::Chars(a, _) => a,
                Span::Lines(l1, l2) => self.line_start(l1.min(l2)) + self.buf.to_pos(self.anchor.min(self.cursor)).col.min(0),
            };
            if let Span::Lines(..) = span {
                self.cursor = start.max(self.cursor.min(start));
            }
            self.clamp();
            self.want_col = self.cursor().col;
            return;
        }
        self.apply(op, span);
    }

    fn visual_key(&mut self, k: Key) {
        if k.code == Code::Esc {
            self.count = None;
            self.prefix = None;
            return self.leave_visual();
        }
        if k.ctrl || self.prefix.is_some() {
            return self.normal_key(k);
        }
        let Some(c) = k.char() else { return self.normal_key(k) };
        if c.is_ascii_digit() && (c != '0' || self.count.is_some()) {
            return self.normal_key(k);
        }
        let line = self.cur_line();
        match c {
            'v' | 'V' => {
                let want = if c == 'v' { Mode::Visual } else { Mode::VisualLine };
                if self.mode == want { self.leave_visual() } else { self.mode = want }
            }
            'o' | 'O' => std::mem::swap(&mut self.anchor, &mut self.cursor),
            'i' | 'a' => self.prefix = Some(c),
            'd' | 'x' => self.visual_op(Op::Delete, false),
            'X' | 'D' => self.visual_op(Op::Delete, true),
            'c' | 's' => self.visual_op(Op::Change, false),
            'C' | 'S' | 'R' => self.visual_op(Op::Change, true),
            'y' => self.visual_op(Op::Yank, false),
            'Y' => self.visual_op(Op::Yank, true),
            'J' => {
                let (l1, l2) = match self.selection() {
                    Span::Lines(a, b) => (a.min(b), a.max(b)),
                    Span::Chars(a, b) => (self.buf.to_pos(a).line, self.buf.to_pos(b.saturating_sub(1).max(a)).line),
                };
                self.mode = Mode::Normal;
                self.cursor = self.line_start(l1);
                self.join((l2 - l1).max(1));
            }
            '~' | 'u' | 'U' => self.visual_case(c),
            'p' | 'P' => self.visual_put(),
            'I' | 'A' => {
                let (a, b) = self.selection_range().unwrap_or((self.cursor, self.cursor));
                self.mode = Mode::Normal;
                self.start_insert(if c == 'I' { a } else { b });
            }
            '$' => {
                // past the last character: the line break is selected too
                let n = self.take_count();
                let target = (line + n - 1).min(self.buf.len_lines() - 1);
                self.cursor = self.line_end(target);
                self.want_col = usize::MAX;
            }
            _ => self.normal_key(k),
        }
    }

    fn visual_case(&mut self, how: char) {
        let (a, b) = self.selection_range().unwrap_or((self.cursor, self.cursor));
        let text = self.buf.slice(a, b);
        let changed: String = match how {
            'u' => text.to_lowercase(),
            'U' => text.to_uppercase(),
            _ => text.chars().map(|ch| if ch.is_uppercase() { ch.to_lowercase().next().unwrap_or(ch) } else { ch.to_uppercase().next().unwrap_or(ch) }).collect(),
        };
        self.mode = Mode::Normal;
        if changed != text {
            self.begin();
            self.rem(a, b);
            self.ins(a, &changed);
        }
        self.cursor = a;
        self.clamp();
        self.want_col = self.cursor().col;
    }

    fn visual_replace(&mut self, ch: char) {
        let (a, b) = self.selection_range().unwrap_or((self.cursor, self.cursor));
        let text = self.buf.slice(a, b);
        let changed: String = text.chars().map(|c| if c == '\n' { c } else { ch }).collect();
        self.mode = Mode::Normal;
        self.begin();
        self.rem(a, b);
        self.ins(a, &changed);
        self.cursor = a;
        self.clamp();
    }

    /// `p` on a selection: the register replaces it, and what was selected goes to the register.
    fn visual_put(&mut self) {
        let saved = self.reg.clone();
        if self.mode == Mode::VisualLine {
            let Span::Lines(l1, l2) = self.selection() else { return };
            let (l1, l2) = (l1.min(l2), l1.max(l2));
            let at_end = l2 + 1 >= self.buf.len_lines();
            self.visual_op(Op::Delete, true);
            let removed = std::mem::replace(&mut self.reg, Register { text: saved.text.clone(), linewise: true });
            if !self.reg.text.ends_with('\n') {
                self.reg.text.push('\n');
            }
            // the lines go where the selection was: after the line above when it was the end
            self.close();
            if at_end && l1 > 0 {
                self.cursor = self.line_start(self.buf.len_lines() - 1);
                self.put(true, 1);
            } else {
                self.put(false, 1);
            }
            self.reg = removed;
            return;
        }
        let Span::Chars(a, b) = self.selection() else { return };
        self.mode = Mode::Normal;
        self.begin();
        let removed = self.rem(a, b);
        if saved.linewise {
            // whole lines into the middle of a line: they go on lines of their own
            let text = format!("\n{}\n", saved.text.trim_end_matches('\n'));
            self.ins(a, &text);
            self.cursor = a + 1;
        } else {
            self.ins(a, &saved.text);
            self.cursor = a + saved.text.chars().count().saturating_sub(1);
        }
        self.reg = Register { text: removed, linewise: false };
        self.clamp();
        self.want_col = self.cursor().col;
    }

    /// f F t T: to (or up to) the next or previous `ch` on the line, count times.
    fn find(&mut self, kind: char, ch: char) {
        let n = self.take_count();
        let line = self.cur_line();
        let (ls, le) = (self.line_start(line), self.line_end(line));
        let forward = kind == 'f' || kind == 't';
        let till = kind == 't' || kind == 'T';
        let mut i = self.cursor;
        let mut found = None;
        let mut left = n;
        loop {
            // t/T start one further, so repeating them moves on
            let next = if forward { i + 1 } else { i.wrapping_sub(1) };
            if (forward && next >= le) || (!forward && (i == 0 || next < ls)) {
                break;
            }
            i = next;
            if self.char(i) == Some(ch) {
                left -= 1;
                if left == 0 {
                    found = Some(i);
                    break;
                }
            }
        }
        let Some(p) = found else {
            self.op = None;
            self.op_count = None;
            return;
        };
        let to = match (forward, till) {
            (true, true) => p - 1,
            (false, true) => p + 1,
            _ => p,
        };
        if forward {
            // f and t include the character they land on
            self.motion(Span::Chars(self.cursor, to + 1), Some(to));
        } else {
            self.motion(Span::Chars(to, self.cursor), Some(to));
        }
    }

    // ── text objects ──

    fn text_object_key(&mut self, inner: bool, obj: char) {
        let Some((s, e, lines)) = self.text_object(inner, obj) else {
            self.op = None;
            self.count = None;
            return;
        };
        match self.mode {
            Mode::Visual | Mode::VisualLine => {
                if lines && self.mode == Mode::Visual {
                    self.mode = Mode::VisualLine;
                }
                self.anchor = s;
                self.cursor = e.saturating_sub(1).max(s);
            }
            _ if self.op.is_some() => {
                let span = if lines { Span::Lines(self.buf.to_pos(s).line, self.buf.to_pos(e.saturating_sub(1).max(s)).line) } else { Span::Chars(s, e) };
                self.motion(span, None);
            }
            _ => {}
        }
    }

    /// The text object at the cursor: (start, end exclusive, whole lines?).
    fn text_object(&self, inner: bool, obj: char) -> Option<(usize, usize, bool)> {
        match obj {
            'w' | 'W' => self.word_object(inner, obj == 'W'),
            '"' | '\'' | '`' => self.quote_object(inner, obj),
            '(' | ')' | 'b' => self.bracket_object(inner, '(', ')'),
            '[' | ']' => self.bracket_object(inner, '[', ']'),
            '{' | '}' | 'B' => self.bracket_object(inner, '{', '}'),
            '<' | '>' => self.bracket_object(inner, '<', '>'),
            'p' => self.paragraph_object(inner),
            _ => None,
        }
    }

    fn word_object(&self, inner: bool, big: bool) -> Option<(usize, usize, bool)> {
        let line = self.cur_line();
        let (ls, le) = (self.line_start(line), self.line_end(line));
        if ls == le {
            return None;
        }
        let i = self.cursor.min(le - 1);
        let kind = |c: char| if c.is_whitespace() { 0 } else if big { 1 } else { class(c) };
        let k = kind(self.char(i)?);
        let (mut s, mut e) = (i, i + 1);
        while s > ls && self.char(s - 1).is_some_and(|c| kind(c) == k) {
            s -= 1;
        }
        while e < le && self.char(e).is_some_and(|c| kind(c) == k) {
            e += 1;
        }
        if inner {
            return Some((s, e, false));
        }
        if k == 0 {
            // on spaces: the spaces and the word after them
            let k2 = self.char(e).map(kind);
            while e < le && self.char(e).is_some_and(|c| Some(kind(c)) == k2) {
                e += 1;
            }
            return Some((s, e, false));
        }
        // a word: with the spaces after it, or (at the line end) the ones before it
        let mut e2 = e;
        while e2 < le && self.char(e2).is_some_and(|c| c == ' ' || c == '\t') {
            e2 += 1;
        }
        if e2 > e {
            return Some((s, e2, false));
        }
        let mut s2 = s;
        while s2 > ls && self.char(s2 - 1).is_some_and(|c| c == ' ' || c == '\t') {
            s2 -= 1;
        }
        Some((s2, e, false))
    }

    fn quote_object(&self, inner: bool, q: char) -> Option<(usize, usize, bool)> {
        let line = self.cur_line();
        let (ls, le) = (self.line_start(line), self.line_end(line));
        let quotes: Vec<usize> = (ls..le).filter(|i| self.char(*i) == Some(q) && (*i == ls || self.char(i - 1) != Some('\\'))).collect();
        let cur = self.cursor;
        // the pair around the cursor, else the first pair after it (as Vim does)
        let mut pair = None;
        for w in quotes.chunks(2) {
            if let [a, b] = w {
                if cur >= *a && cur <= *b {
                    pair = Some((*a, *b));
                    break;
                }
                if pair.is_none() && *a > cur {
                    pair = Some((*a, *b));
                    break;
                }
            }
        }
        let (a, b) = pair?;
        if inner {
            return Some((a + 1, b, false));
        }
        // around: with the spaces after, or else the ones before
        let mut e = b + 1;
        while e < le && self.char(e).is_some_and(|c| c == ' ' || c == '\t') {
            e += 1;
        }
        let mut s = a;
        if e == b + 1 {
            while s > ls && self.char(s - 1).is_some_and(|c| c == ' ' || c == '\t') {
                s -= 1;
            }
        }
        Some((s, e, false))
    }

    fn bracket_object(&self, inner: bool, open: char, close: char) -> Option<(usize, usize, bool)> {
        let len = self.buf.len_chars();
        // the open bracket: the one under the cursor, or the nearest unmatched one before it
        let mut depth = 0i32;
        let mut i = self.cursor.min(len.saturating_sub(1));
        let start = loop {
            let c = self.char(i)?;
            if c == close && i != self.cursor {
                depth += 1;
            } else if c == open {
                if depth == 0 {
                    break i;
                }
                depth -= 1;
            }
            if i == 0 {
                return None;
            }
            i -= 1;
        };
        let mut depth = 0i32;
        let mut j = start + 1;
        let end = loop {
            let c = self.char(j)?;
            if c == open {
                depth += 1;
            } else if c == close {
                if depth == 0 {
                    break j;
                }
                depth -= 1;
            }
            j += 1;
            if j >= len {
                return None;
            }
        };
        if !inner {
            return Some((start, end + 1, false));
        }
        let (mut s, mut e) = (start + 1, end);
        // a block over several lines: the line break after the open bracket and the
        // indent before the close one are not inside
        if self.char(s) == Some('\n') {
            s += 1;
            let close_line = self.buf.to_pos(end).line;
            let cs = self.line_start(close_line);
            if (cs..end).all(|k| self.char(k).is_some_and(|c| c == ' ' || c == '\t')) && cs > s {
                e = cs;
                return Some((s, e, true));
            }
        }
        Some((s, e.max(s), false))
    }

    fn paragraph_object(&self, inner: bool) -> Option<(usize, usize, bool)> {
        let last = self.buf.len_lines() - 1;
        let blank = |l: usize| self.buf.line(l).trim().is_empty();
        let l = self.cur_line();
        let kind = blank(l);
        let (mut a, mut b) = (l, l);
        while a > 0 && blank(a - 1) == kind {
            a -= 1;
        }
        while b < last && blank(b + 1) == kind {
            b += 1;
        }
        if !inner {
            // with the blank lines after it (or before, at the end)
            let b0 = b;
            while b < last && blank(b + 1) != kind {
                b += 1;
                if !blank(b) {
                    b -= 1;
                    break;
                }
            }
            if b == b0 {
                while a > 0 && blank(a - 1) != kind && blank(a - 1) {
                    a -= 1;
                }
            }
        }
        let e = if b + 1 <= last { self.line_start(b + 1) } else { self.buf.len_chars() };
        Some((self.line_start(a), e, true))
    }

    fn is_word_end(&self, i: usize) -> bool {
        match (self.char(i), self.char(i + 1)) {
            (Some(a), Some(b)) => class(a) != class(b) || b.is_whitespace(),
            (Some(_), None) => true,
            _ => false,
        }
    }

    /// A motion: moves the cursor, or, with an operator pending, applies it.
    fn motion(&mut self, span: Span, to: Option<usize>) {
        if let Some(op) = self.op.take() {
            self.op_count = None;
            return self.apply(op, span);
        }
        if let Some(t) = to {
            self.cursor = t;
            self.clamp();
            self.want_col = self.cursor().col;
        }
    }

    fn vertical_to(&mut self, line: usize) {
        let len = self.buf.line_len(line);
        let max = if len > 0 { len - 1 } else { 0 };
        self.cursor = self.line_start(line) + self.want_col.min(max);
    }

    fn apply(&mut self, op: Op, span: Span) {
        self.op = None;
        self.count = None;
        match span {
            Span::Chars(a, b) => {
                let (a, b) = (a.min(b), a.max(b));
                self.reg = Register { text: self.buf.slice(a, b), linewise: false };
                if op == Op::Yank {
                    self.cursor = a;
                } else {
                    self.begin();
                    self.rem(a, b);
                    self.cursor = a;
                    if op == Op::Change {
                        self.mode = Mode::Insert;
                        return;
                    }
                }
            }
            Span::Lines(l1, l2) => {
                let (l1, l2) = (l1.min(l2), l1.max(l2));
                let s = self.line_start(l1);
                let e = if l2 + 1 < self.buf.len_lines() { self.line_start(l2 + 1) } else { self.buf.len_chars() };
                let mut text = self.buf.slice(s, e);
                if !text.ends_with('\n') {
                    text.push('\n');
                }
                self.reg = Register { text, linewise: true };
                match op {
                    Op::Yank => self.cursor = s.min(self.cursor),
                    Op::Delete => {
                        self.begin();
                        // the last lines take the line break BEFORE them
                        let (s2, e2) = if e == self.buf.len_chars() && s > 0 { (s - 1, e) } else { (s, e) };
                        self.rem(s2, e2);
                        let line = l1.min(self.buf.len_lines() - 1);
                        self.cursor = self.first_non_blank(line);
                    }
                    Op::Change => {
                        self.begin();
                        let indent = self.first_non_blank(l1) - s;
                        let keep = self.buf.slice(s, s + indent);
                        let end = self.line_end(l2);
                        self.rem(s, end);
                        self.ins(s, &keep);
                        self.cursor = s + indent;
                        self.mode = Mode::Insert;
                        return;
                    }
                }
            }
        }
        self.clamp();
        self.want_col = self.cursor().col;
    }

    fn put(&mut self, after: bool, n: usize) {
        if self.reg.text.is_empty() {
            return;
        }
        self.begin();
        let text = self.reg.text.repeat(n);
        let line = self.cur_line();
        if self.reg.linewise {
            let last = line + 1 >= self.buf.len_lines();
            let no_final_break = self.buf.len_chars() > 0 && self.buf.char_at(self.buf.len_chars() - 1) != Some('\n');
            let at = if after && last && no_final_break {
                // after the last line of a file without a final line break: the
                // break goes BEFORE the new lines, so the file still has none
                let e = self.buf.len_chars();
                self.ins(e, &format!("\n{}", text.strip_suffix('\n').unwrap_or(&text)));
                e + 1
            } else {
                let at = if after {
                    if last { self.buf.len_chars() } else { self.line_start(line + 1) }
                } else {
                    self.line_start(line)
                };
                self.ins(at, &text);
                at
            };
            self.cursor = at;
            let l = self.cur_line();
            self.cursor = self.first_non_blank(l);
        } else {
            let at = if after && self.buf.line_len(line) > 0 { self.cursor + 1 } else { self.cursor };
            self.ins(at, &text);
            self.cursor = at + text.chars().count() - 1;
        }
        self.clamp();
    }

    fn join(&mut self, times: usize) {
        self.begin();
        for _ in 0..times {
            let line = self.cur_line();
            if line + 1 >= self.buf.len_lines() {
                break;
            }
            let e = self.line_end(line);
            let next = self.first_non_blank(line + 1);
            let joined_at = e;
            self.rem(e, next);
            let needs_space = joined_at > self.line_start(line) && self.char(joined_at).is_some_and(|c| c != '\n' && c != ')');
            if needs_space {
                self.ins(joined_at, " ");
            }
            self.cursor = joined_at;
        }
        self.clamp();
    }

    fn replace_chars(&mut self, ch: char) {
        let n = self.take_count();
        let line = self.cur_line();
        if self.cursor + n > self.line_end(line) {
            return;
        }
        self.begin();
        self.rem(self.cursor, self.cursor + n);
        self.ins(self.cursor, &ch.to_string().repeat(n));
        self.cursor += n - 1;
    }

    // ── insert mode ──

    fn start_insert(&mut self, at: usize) {
        self.begin();
        self.count = None;
        self.mode = Mode::Insert;
        self.cursor = at;
    }

    fn insert_key(&mut self, k: Key) {
        match k.code {
            Code::Esc => {
                self.mode = Mode::Normal;
                // Vim steps back onto the last typed char
                if self.cursor > self.line_start(self.cur_line()) {
                    self.cursor -= 1;
                }
                self.clamp();
                self.want_col = self.cursor().col;
            }
            Code::Enter => {
                // autoindent: the indent before the cursor carries over
                let l = self.cur_line();
                let ind: String = self.indent_of(l).chars().take(self.cursor - self.line_start(l)).collect();
                self.ins(self.cursor, &format!("\n{ind}"));
                self.cursor += 1 + ind.chars().count();
            }
            Code::Tab => {
                self.ins(self.cursor, "\t");
                self.cursor += 1;
            }
            Code::Backspace => {
                if self.cursor > 0 {
                    self.rem(self.cursor - 1, self.cursor);
                    self.cursor -= 1;
                }
            }
            Code::Delete => {
                self.rem(self.cursor, self.cursor + 1);
            }
            Code::Left => self.cursor = self.cursor.saturating_sub(1).max(self.line_start(self.cur_line())),
            Code::Right => self.cursor = (self.cursor + 1).min(self.line_end(self.cur_line())),
            Code::Char(c) if !k.ctrl && !k.alt => {
                self.ins(self.cursor, &c.to_string());
                self.cursor += 1;
            }
            _ => {}
        }
    }

    /// Comments lines `from..=to` out with `prefix` (`//`, `#`), or back in when every
    /// non-empty one already is (evil-commentary's gc). One undo step; leaves visual mode.
    pub fn toggle_comment(&mut self, from: usize, to: usize, prefix: &str) {
        let last = self.buf.len_lines().saturating_sub(1);
        let (from, to) = (from.min(last), to.min(last));
        let lines: Vec<(usize, String)> = (from..=to).map(|l| (l, self.buf.line(l))).collect();
        let body = |s: &str| s.trim_end_matches(['\n', '\r']).to_string();
        let filled: Vec<&(usize, String)> = lines.iter().filter(|(_, t)| !body(t).trim().is_empty()).collect();
        if filled.is_empty() {
            return;
        }
        let commented = filled.iter().all(|(_, t)| body(t).trim_start().starts_with(prefix));
        let indent = filled.iter().map(|(_, t)| body(t).len() - body(t).trim_start().len()).min().unwrap_or(0);
        self.begin();
        // bottom up, so earlier char positions stay right
        for (l, t) in filled.iter().rev() {
            let t = body(t);
            let start = self.buf.to_char(Pos { line: *l, col: 0 });
            if commented {
                let lead = t.chars().count() - t.trim_start().chars().count();
                let mut n = prefix.chars().count();
                if t.trim_start()[prefix.len()..].starts_with(' ') {
                    n += 1;
                }
                self.rem(start + lead, start + lead + n);
            } else {
                self.ins(start + t.chars().take(indent).count(), &format!("{prefix} "));
            }
        }
        self.close();
        self.mode = Mode::Normal;
        let p = self.cursor();
        self.set_cursor(p);
    }

    /// Replaces ranges (from, to, new text) as one undo step: a rename or a format
    /// from a language server. Ranges must not overlap; any order.
    pub fn apply_edits(&mut self, mut edits: Vec<(Pos, Pos, String)>) {
        if edits.is_empty() {
            return;
        }
        let keep = self.cursor();
        // last first, so earlier positions stay right
        edits.sort_by(|a, b| (b.0.line, b.0.col).cmp(&(a.0.line, a.0.col)));
        self.begin();
        for (a, b, text) in edits {
            let (s, e) = (self.buf.to_char(a), self.buf.to_char(b));
            if e > s {
                self.rem(s, e);
            }
            self.ins(s, &text);
        }
        self.close();
        self.mode = Mode::Normal;
        self.set_cursor(keep);
    }

    // ── undo: changes made of edits (a whole insert session is one change) ──

    /// Starts recording a new change; the previous one is finished.
    fn begin(&mut self) {
        self.close();
        self.open = Some(Change { edits: Vec::new(), cursor: self.cursor });
    }
    fn close(&mut self) {
        if let Some(c) = self.open.take() {
            if !c.edits.is_empty() {
                self.undo.push(c);
            }
        }
    }
    /// Inserts, recording the edit in the open change.
    fn ins(&mut self, at: usize, s: &str) {
        self.buf.insert(at, s);
        if let Some(c) = self.open.as_mut() {
            c.edits.push(Edit { at, removed: String::new(), inserted: s.to_string() });
        }
    }
    /// Removes `a..b`, recording the edit in the open change.
    fn rem(&mut self, a: usize, b: usize) -> String {
        let gone = self.buf.remove(a, b);
        if let Some(c) = self.open.as_mut() {
            c.edits.push(Edit { at: a.min(b), removed: gone.clone(), inserted: String::new() });
        }
        gone
    }
    fn undo_once(&mut self) {
        self.close();
        let n = self.undo.cur;
        if n == 0 {
            return;
        }
        self.undo.nodes[n].change.revert(&mut self.buf);
        self.cursor = self.undo.nodes[n].change.cursor;
        let p = self.undo.nodes[n].parent;
        self.undo.nodes[p].redo = Some(n);
        self.undo.cur = p;
        self.mode = Mode::Normal;
        self.clamp();
    }
    fn redo_once(&mut self) {
        self.close();
        if let Some(n) = self.undo.nodes[self.undo.cur].redo {
            self.forward(n);
            // Vim puts the cursor where the change starts
            let c = &self.undo.nodes[n].change;
            self.cursor = c.edits.first().map_or(c.cursor, |e| e.at);
            self.clamp();
        }
    }
    /// Applies child `n` of the current node.
    fn forward(&mut self, n: usize) {
        self.undo.nodes[n].change.apply(&mut self.buf);
        let p = self.undo.nodes[n].parent;
        self.undo.nodes[p].redo = Some(n);
        self.undo.cur = n;
    }

    // ── the undo tree, for undotree-like views ──

    /// Every state: (its parent, when it was made in seconds since the epoch). State 0 is
    /// the text as opened; a state's number is its place in time.
    pub fn undo_states(&mut self) -> Vec<(usize, u64)> {
        // a finished change counts at once; one still being typed does not
        if self.mode != Mode::Insert {
            self.close();
        }
        self.undo.nodes.iter().map(|n| (n.parent, n.time)).collect()
    }
    /// `u` and `C-r` once, for scripts.
    pub fn undo(&mut self) {
        self.undo_once();
    }
    pub fn redo(&mut self) {
        self.redo_once();
    }
    /// The state the text is in.
    pub fn undo_current(&mut self) -> usize {
        if self.mode != Mode::Insert {
            self.close();
        }
        self.undo.cur
    }
    /// The text as it is at state `n`, without going there.
    pub fn text_at(&mut self, n: usize) -> String {
        if self.mode != Mode::Insert {
            self.close();
        }
        if n >= self.undo.nodes.len() { self.text() } else { self.undo.text_at(&self.text(), n) }
    }
    /// Goes to state `n`, along the tree (undoing up to the common state, then redoing down).
    pub fn undo_goto(&mut self, n: usize) {
        self.close();
        if n >= self.undo.nodes.len() || n == self.undo.cur {
            return;
        }
        let (up, down) = self.undo.path(self.undo.cur, n);
        for _ in up {
            self.undo_once();
        }
        for d in down {
            self.forward(d);
            self.cursor = self.undo.nodes[d].change.edits.first().map_or(self.cursor, |e| e.at);
        }
        self.mode = Mode::Normal;
        self.clamp();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Runs keys on a text and returns (text, cursor as (line, col)) — what Vim would show.
    fn run(text: &str, keys: &str) -> (String, (usize, usize)) {
        let mut e = Editor::from_str(text);
        e.feed(keys);
        let c = e.cursor();
        (e.text(), (c.line, c.col))
    }

    #[test]
    fn insert_and_escape() {
        assert_eq!(run("", "ihello<Esc>"), ("hello".into(), (0, 4)));
        assert_eq!(run("ac", "ab<Esc>"), ("abc".into(), (0, 1)));
        assert_eq!(run("one", "A two<Esc>"), ("one two".into(), (0, 6)));
        assert_eq!(run("  x", "Iy<Esc>"), ("  yx".into(), (0, 2)));
        assert_eq!(run("a\nb", "onew<Esc>"), ("a\nnew\nb".into(), (1, 2)));
        assert_eq!(run("a\nb", "jOnew<Esc>"), ("a\nnew\nb".into(), (1, 2)));
    }

    #[test]
    fn motions() {
        assert_eq!(run("hello world foo", "w").1, (0, 6));
        assert_eq!(run("hello world foo", "2w").1, (0, 12));
        assert_eq!(run("hello world foo", "$b").1, (0, 12));
        assert_eq!(run("hello world", "e").1, (0, 4));
        assert_eq!(run("foo.bar baz", "w").1, (0, 3), "punctuation is its own word");
        assert_eq!(run("abc\n  def", "j^").1, (1, 2));
        assert_eq!(run("a\nb\nc", "G").1, (2, 0));
        assert_eq!(run("a\nb\nc", "Ggg").1, (0, 0));
        assert_eq!(run("a\nb\nc", "2G").1, (1, 0));
        assert_eq!(run("abc", "$l").1, (0, 2), "normal mode stays on the last char");
    }

    #[test]
    fn column_is_kept_across_short_lines() {
        assert_eq!(run("abcdef\nx\nabcdef", "4ljj").1, (2, 4));
    }

    #[test]
    fn delete_and_change() {
        assert_eq!(run("hello world", "dw"), ("world".into(), (0, 0)));
        assert_eq!(run("hello world", "cwbye<Esc>"), ("bye world".into(), (0, 2)), "cw acts like ce");
        assert_eq!(run("one\ntwo", "$dw"), ("on\ntwo".into(), (0, 1)), "dw does not eat the line break");
        assert_eq!(run("a b c d", "d2w"), ("c d".into(), (0, 0)));
        assert_eq!(run("a b c d", "2dw"), ("c d".into(), (0, 0)));
        assert_eq!(run("hello", "x"), ("ello".into(), (0, 0)));
        assert_eq!(run("hello", "$X"), ("helo".into(), (0, 3)));
        assert_eq!(run("hello world", "wD"), ("hello ".into(), (0, 5)));
        assert_eq!(run("abc", "rx"), ("xbc".into(), (0, 0)));
    }

    #[test]
    fn linewise() {
        assert_eq!(run("a\nb\nc", "dd"), ("b\nc".into(), (0, 0)));
        assert_eq!(run("a\nb\nc", "Gdd"), ("a\nb".into(), (1, 0)), "the last line takes the break before it");
        assert_eq!(run("a\nb\nc", "2dd"), ("c".into(), (0, 0)));
        assert_eq!(run("a\nb\nc", "dj"), ("c".into(), (0, 0)));
        assert_eq!(run("  a\nb", "ccx<Esc>"), ("  x\nb".into(), (0, 2)), "cc keeps the indent");
        assert_eq!(run("a\nb", "yyp"), ("a\na\nb".into(), (1, 0)));
        assert_eq!(run("a\nb", "jyyP"), ("a\nb\nb".into(), (1, 0)));
        assert_eq!(run("a\nb", "Gyyp"), ("a\nb\nb".into(), (2, 0)));
    }

    #[test]
    fn yank_put_chars() {
        assert_eq!(run("ab", "ylp"), ("aab".into(), (0, 1)));
        assert_eq!(run("hello world", "yw$p"), ("hello worldhello ".into(), (0, 16)));
    }

    #[test]
    fn join() {
        assert_eq!(run("a\n   b", "J"), ("a b".into(), (0, 1)));
        assert_eq!(run("a\nb\nc", "3J"), ("a b c".into(), (0, 3)));
    }

    #[test]
    fn undo_redo() {
        assert_eq!(run("abc", "xxu").0, "bc");
        assert_eq!(run("abc", "xxuu").0, "abc");
        assert_eq!(run("abc", "xxuu<C-r>").0, "bc");
        assert_eq!(run("", "ihello world<Esc>u").0, "", "an insert session is one change");
        assert_eq!(run("abc", "xu2<C-r>").0, "bc");
    }

    #[test]
    fn escape_cancels_pending() {
        assert_eq!(run("abc", "d<Esc>x"), ("bc".into(), (0, 0)));
        assert_eq!(run("abc", "3<Esc>x"), ("bc".into(), (0, 0)));
    }
}

#[cfg(test)]
mod comment_tests {
    use super::*;

    #[test]
    fn gc_comments_and_uncomments() {
        let mut e = Editor::from_str("fn a() {\n    one();\n\n    two();\n}\n");
        e.toggle_comment(1, 3, "//");
        assert_eq!(e.buf.line(1).trim_end(), "    // one();");
        assert_eq!(e.buf.line(2).trim_end(), "");
        assert_eq!(e.buf.line(3).trim_end(), "    // two();");
        e.toggle_comment(1, 3, "//");
        assert_eq!(e.buf.line(1).trim_end(), "    one();");
        assert_eq!(e.buf.line(3).trim_end(), "    two();");
        // one undo step brings the comments back
        e.feed("u");
        assert_eq!(e.buf.line(1).trim_end(), "    // one();");
    }
}

#[cfg(test)]
mod apply_tests {
    use super::*;

    #[test]
    fn edits_apply_as_one_change() {
        let mut e = Editor::from_str("let foo = 1;\nfoo + foo\n");
        e.apply_edits(vec![
            (Pos { line: 0, col: 4 }, Pos { line: 0, col: 7 }, "bar".into()),
            (Pos { line: 1, col: 0 }, Pos { line: 1, col: 3 }, "bar".into()),
            (Pos { line: 1, col: 6 }, Pos { line: 1, col: 9 }, "bar".into()),
        ]);
        assert_eq!(e.buf.line(0).trim_end(), "let bar = 1;");
        assert_eq!(e.buf.line(1).trim_end(), "bar + bar");
        e.feed("u");
        assert_eq!(e.buf.line(1).trim_end(), "foo + foo");
    }
}

#[cfg(test)]
mod undo_tree_tests {
    use super::*;

    #[test]
    fn branches_are_kept() {
        let mut e = Editor::from_str("a\n");
        e.feed("Ab<Esc>"); // 1: ab
        e.feed("Ac<Esc>"); // 2: abc
        e.feed("u"); // back to 1
        e.feed("Ad<Esc>"); // 3: abd, a branch beside 2
        assert_eq!(e.text(), "abd\n");
        assert_eq!(e.undo_states().iter().map(|s| s.0).collect::<Vec<_>>(), vec![0, 0, 1, 1]);
        assert_eq!(e.text_at(2), "abc\n");
        e.undo_goto(2);
        assert_eq!(e.text(), "abc\n");
        e.undo_goto(0);
        assert_eq!(e.text(), "a\n");
        e.feed("<C-r><C-r>"); // redo follows the branch last visited
        assert_eq!(e.text(), "abc\n");
        e.feed("g-"); // in time: 2 → 1
        assert_eq!(e.text(), "ab\n");
        e.feed("g+g+"); // 1 → 2 → 3
        assert_eq!(e.text(), "abd\n");
    }
}
