//! zett's modal engine: Vim's grammar over a text buffer.
//!
//! WASM-clean (no I/O): the window, the terminal UI and the browser extension
//! all feed it keys and draw what it holds.
//!
//! M0 covers the core grammar: counts, `h j k l w b e 0 ^ $ gg G`, operators
//! `d c y` with those motions (and doubled for lines), `x X D C r p P J`,
//! insert entries `i a I A o O`, undo and redo. Visual mode, text objects,
//! registers by name, dot-repeat, macros and search come in M1.

pub mod keys;
pub use keys::{Code, Key};

use zett_core::{Buffer, Pos};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
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

/// One edit: at `at`, `removed` was taken out and `inserted` put in.
#[derive(Clone, Debug)]
struct Edit {
    at: usize,
    removed: String,
    inserted: String,
}

/// One undo step: the edits of one change (a whole insert session is one).
/// Only what changed is kept, never a copy of the text: undo costs the size
/// of the change, not the size of the file.
#[derive(Clone, Debug, Default)]
struct Change {
    edits: Vec<Edit>,
    cursor: usize,
}

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
    undo: Vec<Change>,
    redo: Vec<Change>,
    /// the change being recorded, closed by the next one or by `u`
    open: Option<Change>,
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
            want_col: 0,
            count: None,
            op: None,
            op_count: None,
            prefix: None,
            reg: Register::default(),
            undo: Vec::new(),
            redo: Vec::new(),
            open: None,
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
        let max = if self.mode == Mode::Normal && len > 0 { s + len - 1 } else { s + len };
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
                ('g', 'g') => {
                    let line = self.count.take().map(|n| n.saturating_sub(1)).unwrap_or(0);
                    return self.motion(Span::Lines(self.cur_line(), line.min(self.buf.len_lines() - 1)), Some(self.first_non_blank(line.min(self.buf.len_lines() - 1))));
                }
                ('r', ch) => return self.replace_chars(ch),
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
            'g' | 'r' => self.prefix = Some(c),

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
            'i' => self.start_insert(self.cursor),
            'a' => { let at = (self.cursor + 1).min(self.line_end(line)); self.start_insert(at) }
            'I' => self.start_insert(self.first_non_blank(line)),
            'A' => self.start_insert(self.line_end(line)),
            'o' => { self.begin(); let e = self.line_end(line); self.ins(e, "\n"); self.mode = Mode::Insert; self.cursor = e + 1; }
            'O' => { self.begin(); let s = self.line_start(line); self.ins(s, "\n"); self.mode = Mode::Insert; self.cursor = s; }

            'u' => { let n = self.take_count(); for _ in 0..n { self.undo_once(); } }
            _ => { self.count = None; self.op = None; }
        }
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
                self.ins(self.cursor, "\n");
                self.cursor += 1;
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

    // ── undo: changes made of edits (a whole insert session is one change) ──

    /// Starts recording a new change; the previous one is finished.
    fn begin(&mut self) {
        self.close();
        self.open = Some(Change { edits: Vec::new(), cursor: self.cursor });
        self.redo.clear();
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
        if let Some(c) = self.undo.pop() {
            for e in c.edits.iter().rev() {
                self.buf.remove(e.at, e.at + e.inserted.chars().count());
                self.buf.insert(e.at, &e.removed);
            }
            self.cursor = c.cursor;
            self.mode = Mode::Normal;
            self.clamp();
            self.redo.push(c);
        }
    }
    fn redo_once(&mut self) {
        self.close();
        if let Some(c) = self.redo.pop() {
            for e in &c.edits {
                self.buf.remove(e.at, e.at + e.removed.chars().count());
                self.buf.insert(e.at, &e.inserted);
            }
            // Vim puts the cursor where the change starts
            self.cursor = c.edits.first().map_or(c.cursor, |e| e.at);
            self.clamp();
            self.undo.push(c);
        }
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
