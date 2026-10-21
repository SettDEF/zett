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
pub mod killring;
pub mod outline;
pub mod prefix;
pub mod probe;
pub mod search;
pub mod spell;
pub mod suggest;
pub mod textops;
pub mod vim;
mod change;
mod history;
mod insert;
mod motions;
mod normal;
mod objects;
mod text;
mod visual;
#[cfg(test)]
mod tests;
mod undo;
pub use keys::{Code, Key, MouseButton};
pub use killring::{KillRing, SharedKillRing};
pub use search::Isearch;

use zett_core::{Buffer, Pos};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
    /// `v`: characters from the anchor to the cursor (both included)
    Visual,
    /// `V`: whole lines from the anchor's to the cursor's
    VisualLine,
    /// CTRL-V: a block, the anchor and the cursor its corners
    VisualBlock,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Op {
    Delete,
    Change,
    Yank,
    // vim.rs: g~ gu gU g? > < = gq ! g@
    Toggle,
    Lower,
    Upper,
    Rot13,
    ShiftRight,
    ShiftLeft,
    Reindent,
    Format,
    Filter,
    Func,
    // zf: a manual fold over what the motion covers
    Fold,
    // zy: a yank without the blanks at line ends
    YankTrim,
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
    /// a block (CTRL-V): its lines go in one under another
    block: bool,
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
    /// searching: (ignore case, unless the query has capitals) — the ignorecase and smartcase settings
    case: (bool, bool),
    /// Emacs's mark: with the cursor it makes the region
    mark: Option<usize>,
    /// the region is active (shown, acted on by default): transient-mark-mode
    mark_active: bool,
    /// the kill ring: this editor's own until a front end shares one (`use_kill_ring`)
    kill_ring: SharedKillRing,
    /// an incremental search in progress (C-s / C-r)
    isearch: Option<Isearch>,
    /// the last finished search, for C-s C-s
    last_search: String,
    /// the last yank (start, end, kill-ring index), for yank-pop
    yanked: Option<(usize, usize, usize)>,
    /// the command that ran last (kills append to kills, typing groups into one undo step)
    last_cmd: String,
    /// folds and narrowing (outline.rs)
    view: outline::View,
    /// Vim's keys beyond the core (vim.rs): marks, registers, macros, the dot, requests
    vim: crate::vim::VimState,
    /// an agent's suggestions, not yet taken (suggest.rs)
    pending: suggest::Pending,
    /// textops.rs: where M-q wraps, M-@ in a row, M-/ in a row
    fill_column: usize,
    last_cmd_was_mark_word: bool,
    mark_word_end: usize,
    dabbrev: Option<textops::Dabbrev>,
    /// expandtab: Tab inserts spaces to the next multiple of this (None: a tab character)
    soft_tab: Option<usize>,
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
            case: (true, true),
            mark: None,
            mark_active: false,
            fill_column: 80,
            last_cmd_was_mark_word: false,
            mark_word_end: 0,
            dabbrev: None,
            soft_tab: None,
            kill_ring: KillRing::shared(),
            isearch: None,
            last_search: String::new(),
            yanked: None,
            last_cmd: String::new(),
            view: outline::View::default(),
            vim: crate::vim::VimState::default(),
            pending: suggest::Pending::default(),
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
        // langmap: outside insert mode another layout's characters are the commands they stand for
        let k = match k.char().and_then(|c| self.vim.langmap.get(&c).copied()) {
            Some(to) if !self.emacs && self.mode != Mode::Insert && self.isearch.is_none() && self.prefix.is_none() && !k.ctrl && !k.alt => Key { code: Code::Char(to), ..k },
            _ => k,
        };
        // the Vim search line, macros, the dot (vim.rs)
        if self.vim_before(k) {
            return self.vim_after();
        }
        match self.mode {
            Mode::Insert => self.insert_key(k),
            Mode::Normal => self.normal_key(k),
            Mode::Visual | Mode::VisualLine => self.visual_key(k),
            Mode::VisualBlock => self.block_key(k),
        }
        self.vim_after();
    }
}
