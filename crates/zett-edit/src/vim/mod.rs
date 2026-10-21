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
    /// the next (+1) or previous (-1) diagnostic, the first (-2) or last (+2): [d ]d [D ]D
    Diagnostic(i32),
    /// the next (+1) or previous (-1) changed part (a git hunk): ]c [c
    Hunk(i32),
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
    /// keys no command took (the parity probe counts a key only when none was left over)
    pub unknown: u32,
    /// where the mouse is (the window says, before a mouse key)
    pub mouse: usize,
    /// U: the last changed line as it was before its changes; the line the command under way started on
    pub uline: Option<(usize, String)>,
    typing_line: (usize, String),
    /// the register the last recording went into (Q replays it), g' jumping without the jump list
    pub last_recorded: Option<char>,
    pub no_jump: bool,
    /// long lines wrap on screen (gj gk g0 g$ go by screen lines then)
    pub wrap: bool,
    /// manual folds (zf): from the start of their first line to the end of their last, closed or open;
    /// and folding on or off (zn zN zi)
    pub folds: Vec<(usize, usize, bool)>,
    pub folds_off: bool,
    /// z{height}<CR>: the height typed so far
    pub z_height: usize,
    /// a block reaching every line's end ($), and a block insert waiting for Esc: (first line, last line,
    /// column, where typing began, pad short lines)
    pub block_eol: bool,
    pub block_insert: Option<(usize, usize, usize, usize, bool)>,
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
        for f in self.folds.iter_mut() {
            mv(&mut f.0);
            mv(&mut f.1);
        }
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
        for f in self.folds.iter_mut() {
            mv(&mut f.0);
            mv(&mut f.1);
        }
        if let Some(p) = self.last_insert.as_mut() {
            mv(p);
        }
    }
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

mod block;
mod brackets;
mod folds;
mod g;
mod keys;
mod motions;
mod ops;
mod search;
mod special;
mod track;
mod window;
#[cfg(test)]
mod tests;

/// Prefixes beyond the visible keys: after CTRL-W, after CTRL-W g, after CTRL-\.
pub(crate) const CW: char = '\u{17}';
pub(crate) const CWG: char = '\u{1e}';
pub(crate) const CBS: char = '\u{1c}';

impl Editor {
    /// What the window should do, since the last time it asked.
    pub fn take_requests(&mut self) -> Vec<Request> {
        std::mem::take(&mut self.vim.requests)
    }
    /// Lines shown (top, height) and the window's width: for H M L and gm.
    pub fn set_screen(&mut self, top: usize, height: usize, width: usize) {
        self.vim.screen = (top, height, width);
    }
    /// The wrap setting: gj gk g0 g^ g$ gm move by screen lines when on.
    pub fn set_wrap(&mut self, on: bool) {
        self.vim.wrap = on;
    }
    /// A count typed before `:`, taken (the window opens the line with it as a range).
    pub fn take_pending_count(&mut self) -> Option<usize> {
        self.count.take()
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

    /// The window's top line moves to `t`: asked of the window, and kept here until it says otherwise.
    pub(crate) fn vim_top(&mut self, t: usize) {
        self.vim.screen.0 = t;
        self.vim.requests.push(Request::Top(t));
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
