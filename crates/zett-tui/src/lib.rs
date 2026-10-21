//! zett in the terminal: the code editor, Doom-style, in the terminal's own colours.

mod commands;
mod doom;
mod draw;
mod input;
mod lsp;
mod panels;
mod hl;
use hl::hl;
mod clipboard;
mod cmdline;
mod ex;
mod minibuf;
mod travel;
mod agent;
mod query;
mod refs;
mod notebook;
mod everything;
mod git;
mod nvim_options;
mod settings;
mod display;
mod statusline;
mod files;
mod quickfix;
mod qfcmd;
mod tagcmd;
mod excmds;
mod mapcmds;
mod dircmd;
mod defsearch;
mod diff;
mod runtime;
mod signs;
mod emacs;
mod vc;
mod tabs;
mod chars;
use minibuf::{fuzzy, Cand, Minibuf, Placement};
use doom::{project_of, Pick, WTree, Win};

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind};
use ratatui::crossterm::{cursor::SetCursorStyle, execute};
use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::{DefaultTerminal, Frame};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use syntect::highlighting::{HighlightIterator, HighlightState, Highlighter, ScopeSelectors, StyleModifier, Theme, ThemeItem, ThemeSettings};
use syntect::parsing::{ParseState, ScopeStack, SyntaxReference, SyntaxSet};
use zett_core::{Buffer, Pos};
use zett_edit::{Code, Editor, Key, Mode};
use zett_script::{Effect, Engine, Feed, MenuItem, PanelView};

thread_local! {
    /// the buffer being drawn: its tabstop and whether list shows tabs (set per buffer, read below)
    static SHOW: std::cell::Cell<(usize, bool)> = const { std::cell::Cell::new((4, false)) };
}
/// Columns a tab takes (the tabstop setting of the buffer being drawn or edited).
fn tab() -> usize {
    SHOW.with(|s| s.get().0)
}

// ── syntax colours from the terminal's palette ──

/// A syntect colour that means "ANSI colour n" (alpha 0) or "the default" (alpha 1).
fn ansi(n: u8) -> syntect::highlighting::Color {
    syntect::highlighting::Color { r: n, g: 0, b: 0, a: 0 }
}

fn to_color(c: syntect::highlighting::Color) -> Color {
    match c.a {
        0 => Color::Indexed(c.r),
        255 => hl::rgb(c.r, c.g, c.b),
        _ => Color::Reset,
    }
}

/// A highlight group's colour in syntect's terms (see `ansi`).
fn group_color(name: &str) -> Option<syntect::highlighting::Color> {
    match hl::fg(name)? {
        Color::Indexed(n) => Some(ansi(n)),
        Color::Rgb(r, g, b) => Some(syntect::highlighting::Color { r, g, b, a: 255 }),
        _ => None,
    }
}

/// Scopes mapped onto highlight groups (Comment, String, Keyword, …), whose colours are the
/// terminal's 16 unless a config says otherwise.
fn palette_theme() -> Theme {
    use syntect::highlighting::FontStyle;
    let item = |scope: &str, group: &str| {
        let st = hl(group);
        let mut fs = FontStyle::empty();
        if st.add_modifier.contains(Modifier::BOLD) {
            fs |= FontStyle::BOLD;
        }
        if st.add_modifier.contains(Modifier::ITALIC) {
            fs |= FontStyle::ITALIC;
        }
        if st.add_modifier.contains(Modifier::UNDERLINED) {
            fs |= FontStyle::UNDERLINE;
        }
        ThemeItem { scope: scope.parse::<ScopeSelectors>().unwrap_or_default(), style: StyleModifier { foreground: group_color(group), background: None, font_style: Some(fs) } }
    };
    Theme {
        name: Some("terminal".into()),
        author: None,
        settings: ThemeSettings { foreground: Some(syntect::highlighting::Color { r: 0, g: 0, b: 0, a: 1 }), ..Default::default() },
        scopes: vec![
            item("comment, punctuation.definition.comment", "Comment"),
            item("string, punctuation.definition.string", "String"),
            item("constant.numeric, constant.language, constant.character", "Number"),
            item("constant.other, variable.other.constant", "Constant"),
            item("keyword, storage.modifier, storage.type.function, keyword.control", "Keyword"),
            item("keyword.operator", "Operator"),
            item("storage.type, entity.name.type, support.type, entity.name.class, entity.name.struct, entity.name.enum, entity.name.trait", "Type"),
            item("entity.name.function, support.function, meta.function-call variable.function, variable.function", "Function"),
            item("entity.name.tag", "Tag"),
            item("entity.other.attribute-name", "Attribute"),
            item("support.macro, entity.name.macro, meta.macro", "Macro"),
            item("variable.parameter", "Parameter"),
            item("variable.language", "Special"),
            item("markup.heading, entity.name.section", "Heading"),
            item("markup.bold", "Bold"),
            item("markup.italic", "Italic"),
            item("markup.raw, markup.inline.raw", "Raw"),
            item("markup.underline.link, string.other.link", "Link"),
            item("markup.list.unnumbered, markup.list.numbered, punctuation.definition.list", "ListMarker"),
            item("markup.quote", "Quote"),
            // diffs (git, :diffpatch, C-x v =): Neovim's Added / Removed / Changed
            item("markup.inserted", "Added"),
            item("markup.deleted", "Removed"),
            item("markup.changed, meta.diff.range, meta.diff.header", "Changed"),
            item("invalid", "Invalid"),
        ],
    }
}

// ── a buffer on screen ──

struct Doc {
    ed: Editor,
    name: String,
    syntax: String,
    top: usize,
    left: usize,
    /// highlight state at the start of each line computed so far
    states: Vec<(ParseState, HighlightState)>,
    /// to see what an edit touched: (lines, chars, cursor line)
    shape: (usize, usize, usize),
    /// what its language server reported, and whether it has one
    diags: Vec<zett_lsp::Diag>,
    lsp: bool,
    lsp_rev: u64,
    /// :setlocal values (settings.rs)
    local: std::collections::HashMap<String, String>,
    /// when its file was last read or written here (autoread, :checktime compare)
    mtime: Option<std::time::SystemTime>,
    /// its modeline and file permissions were looked at (sync_profile, once)
    seen: bool,
    /// the tab's own folder (:tcd; zett's tabs are its buffers)
    tcd: Option<PathBuf>,
    /// the signs placed in it (:sign place)
    signs: Vec<signs::Placed>,
}

impl Doc {
    fn open(path: &Path, ss: &SyntaxSet) -> std::io::Result<Doc> {
        let buf = Buffer::open(path)?;
        let name = path.display().to_string();
        let syntax = ss.find_syntax_for_file(path).ok().flatten().map(|s| s.name.clone()).unwrap_or_else(|| {
            let first = buf.line(0);
            ss.find_syntax_by_first_line(&first).map(|s| s.name.clone()).unwrap_or_else(|| "Plain Text".into())
        });
        let mut d = Doc::new(Editor::new(buf), name, syntax);
        // the history it was saved with, so time travel and undo reach past this session
        d.load_history();
        Ok(d)
    }

    fn scratch() -> Doc {
        Doc::new(Editor::from_str(""), "[scratch]".into(), "Plain Text".into())
    }

    fn new(ed: Editor, name: String, syntax: String) -> Doc {
        let shape = (ed.buf.len_lines(), ed.buf.len_chars(), ed.cursor().line);
        Doc { ed, name, syntax, top: 0, left: 0, states: Vec::new(), shape, diags: Vec::new(), lsp: false, lsp_rev: 0, local: Default::default(), mtime: None, seen: false, tcd: None, signs: Vec::new() }
    }

    pub(crate) fn dirty(&self) -> bool {
        self.ed.buf.is_dirty()
    }

    /// After a key: drop highlight states from where the text may have changed.
    fn touched(&mut self, before_line: usize) {
        let now = (self.ed.buf.len_lines(), self.ed.buf.len_chars(), self.ed.cursor().line);
        if (now.0, now.1) != (self.shape.0, self.shape.1) {
            let from = before_line.min(now.2).saturating_sub(1);
            self.states.truncate(from + 1);
        }
        self.shape = now;
    }

    /// Highlighted spans of one line (tabs expanded), computing states up to it.
    fn spans(&mut self, line: usize, ss: &SyntaxSet, hl: &Highlighter) -> Vec<(Style, String)> {
        let syntax: &SyntaxReference = ss.find_syntax_by_name(&self.syntax).unwrap_or_else(|| ss.find_syntax_plain_text());
        if self.states.is_empty() {
            self.states.push((ParseState::new(syntax), HighlightState::new(hl, ScopeStack::new())));
        }
        while self.states.len() <= line {
            let i = self.states.len() - 1;
            let (mut ps, mut hs) = self.states[i].clone();
            let text = format!("{}\n", self.ed.buf.line(i));
            if let Ok(ops) = ps.parse_line(&text, ss) {
                for _ in HighlightIterator::new(&mut hs, &ops, &text, hl) {}
            }
            self.states.push((ps, hs));
        }
        let (mut ps, mut hs) = self.states[line].clone();
        let text = format!("{}\n", self.ed.buf.line(line));
        let Ok(ops) = ps.parse_line(&text, ss) else { return vec![(Style::default(), expand(&text))] };
        HighlightIterator::new(&mut hs, &ops, &text, hl)
            .map(|(st, s)| {
                let mut style = Style::default().fg(to_color(st.foreground));
                use syntect::highlighting::FontStyle as F;
                if st.font_style.contains(F::BOLD) {
                    style = style.add_modifier(Modifier::BOLD);
                }
                if st.font_style.contains(F::ITALIC) {
                    style = style.add_modifier(Modifier::ITALIC);
                }
                if st.font_style.contains(F::UNDERLINE) {
                    style = style.add_modifier(Modifier::UNDERLINED);
                }
                (style, expand(s.trim_end_matches('\n')))
            })
            .collect()
    }
}

fn expand(s: &str) -> String {
    let (t, list) = SHOW.with(|s| s.get());
    // list: placeholder cells the window draws as listchars' tab (display.rs)
    let shown = if list {
        let mut s = display::TAB_FIRST.to_string();
        if t > 1 {
            s.extend(std::iter::repeat_n(display::TAB_FILL, t - 2));
            s.push(display::TAB_LAST);
        }
        s
    } else {
        " ".repeat(t)
    };
    s.replace('\t', &shown)
}

/// Where a wrapped line's rows start, in screen columns (wrap): every `width` columns, or with
/// `words` (linebreak) after the last space that fits, so a word moves to the next row whole.
fn wrap_starts(line: &str, width: usize, words: Option<&[char]>, cont: usize) -> Vec<usize> {
    let cells: Vec<char> = expand(line.trim_end_matches('\n')).chars().collect();
    let width = width.max(1);
    let mut starts = vec![0];
    let mut s = 0;
    // the rows after the first are `cont` narrower (showbreak, breakindent)
    let row_w = |s: usize| if s == 0 { width } else { (width - cont.min(width - 1)).max(1) };
    while cells.len() - s > row_w(s) {
        let mut e = s + row_w(s);
        // linebreak: after the last character of breakat that fits
        if let Some(brk) = words {
            if let Some(sp) = (s + 1..e).rev().find(|&i| brk.contains(&cells[i - 1]) && cells[i] != ' ') {
                e = sp;
            }
        }
        starts.push(e);
        s = e;
    }
    starts
}

/// The screen column of a character column in a line (a tab counts as tabstop columns).
fn screen_col(line: &str, col: usize) -> usize {
    line.chars().take(col).map(|c| if c == '\t' { tab() } else { 1 }).sum()
}

// ── the file finder ──

fn list_files(root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&dir) else { continue };
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with('.') || matches!(name.as_str(), "target" | "node_modules" | "__pycache__" | "build" | "dist") {
                continue;
            }
            let p = e.path();
            match e.file_type() {
                Ok(t) if t.is_dir() => stack.push(p),
                Ok(t) if t.is_file() => out.push(p.strip_prefix(root).unwrap_or(&p).display().to_string()),
                _ => {}
            }
            if out.len() > 50_000 {
                return out;
            }
        }
    }
    out.sort();
    out
}

// ── the app ──

enum Popup {
    Help(String, Vec<(String, String)>),
}

struct App {
    docs: Vec<Doc>,
    cur: usize,
    root: PathBuf,
    ss: SyntaxSet,
    theme: Theme,
    engine: Engine,
    keyseq: Vec<String>,
    menu: Vec<MenuItem>,
    key_at: Instant,
    cmdline: Option<String>,
    msg: Option<(String, Instant)>,
    /// lines that pulse for a moment (a help section just found): (doc, from, to, since)
    flash: Option<(usize, usize, usize, Instant)>,
    minibuf: Option<Minibuf>,
    /// time travel through a buffer's history, while it is open
    travel: Option<travel::Travel>,
    /// a question to the agent on its way, and the echo area asking one
    agent: Option<agent::Job>,
    asking: bool,
    /// the live query blocks' results (query.rs)
    queries: query::Cache,
    /// what the block references show (refs.rs)
    embeds: refs::Embeds,
    /// code blocks running and what they printed (notebook.rs)
    notebook: notebook::Notebook,
    /// the git buffer (git.rs), and the echo area asking for a commit message
    git: Option<git::GitView>,
    committing: bool,
    /// where the minibuffer is drawn (zett.opt.minibuffer)
    placement: Placement,
    popup: Option<Popup>,
    quit: bool,
    /// editor area from the last draw (for the mouse and scrolling)
    area: Rect,
    gutter: u16,
    /// the terminal title (title, titlestring) as drawn, and as last set
    title_want: Option<String>,
    title_set: Option<String>,
    /// the command being run had a ! (:w!)
    bang: bool,
    /// the last mouse click (when, column, row, how many in a row: double and triple clicks), and
    /// where a drag began
    click: Option<(Instant, u16, u16, u8)>,
    /// the mouse event a mapping ran for (<LeftMouse> in its keys is done there)
    last_mouse: Option<ratatui::crossterm::event::MouseEvent>,
    drag_from: Option<Pos>,
    /// when files were last checked for changes made outside (autoread)
    checked: Instant,
    /// windows (by id; closed ones are None), their split tree, the focused one
    wins: Vec<Option<Win>>,
    tree: WTree,
    focus: usize,
    win_rects: Vec<(usize, Rect)>,
    /// the dashboard shows (no file yet) and its chosen button
    dash: bool,
    dash_sel: usize,
    last_doc: Option<usize>,
    /// the window focused before (CTRL-W p), the : lines run (q:), a full redraw asked for (CTRL-L)
    last_win: Option<usize>,
    /// the quickfix lists, each window's location lists, and a :make or :grep under way (qfcmd.rs)
    qf: quickfix::QfStack,
    loclists: std::collections::HashMap<usize, quickfix::QfStack>,
    qf_job: Option<qfcmd::QfJob>,
    /// the tag stack: each jump a tag made, newest last (tagcmd.rs)
    tag_stack: Vec<tagcmd::TagEntry>,
    /// the global folder (:cd; root follows the scope that applies, dircmd.rs), and the one before
    /// each scope's last change (global, tab, window) for :cd -
    global_dir: PathBuf,
    prev_dirs: [Option<PathBuf>; 3],
    /// zash's drawn marks (draw.rs FxMark): this frame's, where each was sent last, when each was
    /// first shown (it draws once, then shows drawn), the view they were sent for (buffer, top,
    /// cursor line, since when), and the one under the mouse (sent, to take away when it leaves)
    fx_marks: Vec<crate::draw::FxMark>,
    fx_sent: std::collections::HashMap<String, (u16, u16)>,
    fx_first: std::collections::HashMap<String, Instant>,
    fx_view: Option<(usize, usize, usize, Instant)>,
    fx_hover: Option<crate::draw::FxMark>,
    fx_hover_sent: Option<(u16, u16, u16, String)>,
    /// the argument list and the one in use; the last :! command; :match per window; :redir under way
    args: Vec<String>,
    arg_idx: usize,
    last_bang: Option<String>,
    /// the register the last :@ ran (:@@)
    last_at: Option<char>,
    /// :match :2match :3match: each window's patterns (by window and number) and their groups
    win_match: std::collections::HashMap<(usize, u8), (String, String)>,
    redir: Option<(String, String)>,
    /// the window being drawn (draw_text: its :match)
    drawing_win: usize,
    /// mapped keys being typed in again, and how many of those skip the mappings (:noremap)
    replaying: usize,
    noremap: usize,
    /// :abbreviate, :command, :autocmd (mapcmds.rs): what they hold; the :augroup in use and those
    /// defined; how deep autocommands run each other; the mode last frame (InsertEnter/Leave)
    abbrevs: Vec<(String, String)>,
    /// Emacs's mode abbrevs (C-x a +): by file type, before the global ones
    mode_abbrevs: std::collections::HashMap<String, Vec<(String, String)>>,
    /// the expansion C-x a g asks a name for; abbrev-mode (expand as you type, Emacs profile); M-'
    pending_abbrev: Option<(String, bool)>,
    abbrev_mode: bool,
    abbrev_prefix: Option<usize>,
    /// the : line's abbreviations (:cabbrev; :abbreviate sets both)
    cabbrevs: Vec<(String, String)>,
    /// CTRL-K on the : line: waiting for the digraph's first character, or its second
    cmd_digraph: Option<Option<char>>,
    /// :sign define: each sign's look
    sign_defs: std::collections::HashMap<String, signs::SignDef>,
    /// :append :insert :change: where the lines go, the lines they replace, and the lines so far
    ex_text: Option<(usize, Option<(usize, usize)>, Vec<String>)>,
    /// the files :source, :runtime and the rest ran (:scriptnames), the packages :packadd added, whether
    /// :packloadall ran, and the colour scheme's name (:colorscheme)
    sourced: Vec<PathBuf>,
    packs: Vec<PathBuf>,
    packs_loaded: bool,
    colors_name: String,
    /// diff mode: the changes between two buffers, with the revisions and 'diffopt' they are for
    diff_cache: std::collections::HashMap<(usize, usize), ((u64, u64), String, Vec<diff::Block>)>,
    user_cmds: std::collections::HashMap<String, mapcmds::UserCmd>,
    autocmds: Vec<mapcmds::AutoCmd>,
    augroup: String,
    augroups: std::collections::HashSet<String>,
    au_depth: usize,
    last_mode: Option<Mode>,
    /// :menu entries (Neovim's PopUp first) and :menutranslate names
    menus: Vec<mapcmds::MenuEntry>,
    menu_tr: std::collections::HashMap<String, String>,
    cmd_hist: Vec<String>,
    clear: bool,
    suspend: bool,
    /// z= suggestions waiting for the picker
    spell_offer: Vec<String>,
    last_compile: Option<String>,
    numbers: bool,
    relative: bool,
    tabs: bool,
    /// the key profile: Emacs (default) or Vim (`zett.opt.keys = "vim"`), read with the options
    emacs: bool,
    /// Emacs profile: Esc was pressed, the next key gets Meta (Esc f = M-f, as in Emacs and zsh)
    meta: bool,
    /// a command waiting for its key (a register, zap-to-char's character, C-q): (name, argument)
    char_for: Option<(String, Option<i64>)>,
    /// M-%: the match shown and what replaces it (emacs.rs)
    qr: Option<emacs::QueryReplace>,
    /// C-h k (false) or C-h c (true) waiting for keys, and the keys so far
    describing: Option<(bool, Vec<String>)>,
    /// the last 300 keys, for C-h l
    lossage: std::collections::VecDeque<String>,
    /// how many keys the command running now took (C-x ) leaves them out of the macro)
    ran_keys: usize,
    /// a keyboard macro being recorded (C-x (), the last one, and C-x e waiting for e to repeat
    kmacro: Option<Vec<String>>,
    last_kmacro: Vec<String>,
    kmacro_again: bool,
    /// what the last keys did, for C-x z (and z again)
    last_fx: Vec<Effect>,
    repeat_again: bool,
    /// the :match slot M-s h used last (1-3)
    hl_slot: u8,
    /// a status line (false) or a line between windows (true) being dragged: the window, where it was
    border_drag: Option<(usize, bool, u16)>,
    /// tab pages (tabs.rs): every tab (empty while there is one), the shown one, closed ones
    tab_pages: Vec<tabs::TabPage>,
    tab: usize,
    closed_tabs: Vec<tabs::TabPage>,
    /// the mouse command that ran last (mouse-3 twice kills)
    last_key_cmd: Option<String>,
    /// Emacs profile: a numeric argument being typed (C-u 4, M-5), for the next command
    arg: zett_edit::prefix::PrefixArg,
    /// the key that ran the last command (digit-argument reads its digit)
    last_key: String,
    /// one kill ring for every buffer (Emacs): handed to each in sync_profile
    kill_ring: zett_edit::SharedKillRing,
    branch: Option<String>,
    branch_at: Instant,
    started_ms: u128,
    /// visual mode: a g waiting for its c
    vis_g: bool,
    /// a script's question on the input line (zett.input), instead of `:`
    prompt: Option<String>,
    /// the chosen suggestion under `:`
    cmd_sel: usize,
    /// plugin panels: where each is scrolled to and was drawn, the last click (for double)
    panel_tops: std::collections::HashMap<String, usize>,
    panel_rects: Vec<(String, Rect)>,
    last_click: Option<(Instant, String, usize)>,
    /// for buf_enter and text_changed: the buffer and text the hooks last heard about
    seen_doc: Option<(usize, String)>,
    seen_rev: u64,
    /// a notify hook is running (its own messages are not sent back to it)
    notifying: bool,
    /// the lines of the last visual selection (:'<,'>), the last :s, the messages so far
    vis_range: Option<(usize, usize)>,
    last_sub: Option<(String, String, String)>,
    history: Vec<String>,
    /// the config files' times, looked at every two seconds (a change reloads the config)
    config_stamp: Vec<Option<std::time::SystemTime>>,
    config_at: Instant,
    lsp: zett_lsp::Servers,
    lsp_status: String,
    comp: Option<lsp::Comp>,
    comp_due: Option<Instant>,
    /// where the cursor was drawn (the completion popup sits there)
    cursor_xy: (u16, u16),
    /// the latest edit (servers hear about it once typing pauses)
    edit_at: Instant,
    /// the last key (showmatch lasts matchtime after it)
    typed_at: Instant,
    /// the focused window and its scroll position (top, left) last frame, for scrollbind
    bind_prev: Option<(usize, usize, usize)>,
}

/// Opens the editor in the terminal: files (or a folder) from the command line.
/// The settings and Neovim options with what zett does with each: (name, short names, true when
/// it acts on it). For `zett options` and the parity count.
pub fn options() -> Vec<(&'static str, &'static [&'static str], bool)> {
    settings::all().into_iter().map(|s| (s.name, s.short, settings::works(s))).collect()
}

/// The exit status zett leaves with (:cquit sets it).
pub static EXIT_CODE: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);

pub fn run(paths: Vec<PathBuf>) -> std::io::Result<()> {
    // "+ and "* reach the system clipboard
    fn clip_get() -> Option<String> {
        clipboard::paste()
    }
    fn clip_set(t: &str) {
        clipboard::copy(t);
    }
    let _ = zett_edit::vim::CLIPBOARD.set((clip_get, clip_set));
    let mut app = App::new(paths);
    let mut term = ratatui::init();
    let _ = execute!(std::io::stdout(), ratatui::crossterm::event::EnableMouseCapture);
    let r = app.main_loop(&mut term);
    let _ = execute!(std::io::stdout(), ratatui::crossterm::event::DisableMouseCapture, SetCursorStyle::DefaultUserShape);
    ratatui::restore();
    r
}

impl App {
    fn new(paths: Vec<PathBuf>) -> App {
        let started = Instant::now();
        let ss = SyntaxSet::load_defaults_newlines();
        let mut root = std::env::current_dir().unwrap_or_default();
        let mut docs = Vec::new();
        let mut msg = None;
        for p in &paths {
            if p.is_dir() {
                root = p.clone();
            } else {
                if let Some(parent) = p.parent().filter(|d| !d.as_os_str().is_empty() && paths.len() == 1) {
                    if !root.join(p).exists() || p.is_absolute() {
                        root = std::fs::canonicalize(parent).unwrap_or(parent.to_path_buf());
                    }
                }
                match Doc::open(p, &ss) {
                    Ok(d) => docs.push(d),
                    Err(e) => msg = Some((format!("{}: {e}", p.display()), Instant::now())),
                }
            }
        }
        let opened_files = !docs.is_empty();
        if docs.is_empty() {
            docs.push(Doc::scratch());
        }
        let mut engine = Engine::new(None);
        engine.set_frontend("editor");
        engine.load_config(&zett_script::config_dir());
        if let Some(e) = engine.errors.first() {
            msg = Some((e.clone(), Instant::now()));
        }
        let mut app = App {
            docs,
            cur: 0,
            root: root.clone(),
            ss,
            theme: palette_theme(),
            engine,
            keyseq: Vec::new(),
            menu: Vec::new(),
            key_at: Instant::now(),
            cmdline: None,
            msg,
            flash: None,
            minibuf: None,
            travel: None,
            agent: None,
            queries: Default::default(),
            embeds: Default::default(),
            notebook: Default::default(),
            git: None,
            committing: false,
            asking: false,
            placement: Placement::Bottom,
            popup: None,
            quit: false,
            area: Rect::default(),
            gutter: 4,
            title_want: None,
            title_set: None,
            bang: false,
            click: None,
            last_mouse: None,
            drag_from: None,
            checked: Instant::now(),
            wins: vec![Some(Win::default())],
            tree: WTree::Leaf(0),
            focus: 0,
            win_rects: Vec::new(),
            dash: false,
            dash_sel: 0,
            last_doc: None,
            last_win: None,
            qf: Default::default(),
            loclists: Default::default(),
            qf_job: None,
            tag_stack: Vec::new(),
            global_dir: root.clone(),
            prev_dirs: [None, None, None],
            fx_marks: Vec::new(),
            fx_sent: Default::default(),
            fx_first: Default::default(),
            fx_view: None,
            fx_hover: None,
            fx_hover_sent: None,
            args: paths.iter().filter(|p| !p.is_dir()).map(|p| p.display().to_string()).collect(),
            arg_idx: 0,
            last_bang: None,
            last_at: None,
            win_match: Default::default(),
            redir: None,
            drawing_win: 0,
            replaying: 0,
            noremap: 0,
            abbrevs: Vec::new(),
            mode_abbrevs: Default::default(),
            pending_abbrev: None,
            abbrev_mode: false,
            abbrev_prefix: None,
            cabbrevs: Vec::new(),
            cmd_digraph: None,
            sign_defs: Default::default(),
            ex_text: None,
            sourced: Vec::new(),
            packs: Vec::new(),
            packs_loaded: false,
            colors_name: "default".into(),
            diff_cache: Default::default(),
            user_cmds: Default::default(),
            autocmds: Vec::new(),
            augroup: String::new(),
            augroups: Default::default(),
            au_depth: 0,
            last_mode: None,
            menus: mapcmds::default_menus(),
            menu_tr: Default::default(),
            cmd_hist: Vec::new(),
            clear: false,
            suspend: false,
            spell_offer: Vec::new(),
            last_compile: None,
            numbers: true,
            relative: false,
            tabs: false,
            emacs: true,
            meta: false,
            char_for: None,
            qr: None,
            describing: None,
            lossage: Default::default(),
            ran_keys: 0,
            kmacro: None,
            last_kmacro: Vec::new(),
            kmacro_again: false,
            last_fx: Vec::new(),
            repeat_again: false,
            hl_slot: 0,
            border_drag: None,
            tab_pages: Vec::new(),
            tab: 0,
            closed_tabs: Vec::new(),
            last_key_cmd: None,
            arg: Default::default(),
            last_key: String::new(),
            kill_ring: zett_edit::KillRing::shared(),
            branch: None,
            branch_at: Instant::now() - Duration::from_secs(60),
            started_ms: 0,
            vis_g: false,
            prompt: None,
            cmd_sel: 0,
            panel_tops: Default::default(),
            panel_rects: Vec::new(),
            last_click: None,
            seen_doc: None,
            seen_rev: 0,
            notifying: false,
            vis_range: None,
            last_sub: None,
            history: Vec::new(),
            config_stamp: Vec::new(),
            config_at: Instant::now(),
            lsp: zett_lsp::Servers::new(),
            lsp_status: String::new(),
            comp: None,
            comp_due: None,
            cursor_xy: (0, 0),
            edit_at: Instant::now(),
            typed_at: Instant::now(),
            bind_prev: None,
        };
        // the options first: the startup depends on them (the key profile)
        app.take_opts();
        // a folder: the file finder; nothing: what zett.opt.startup says (scratch by default)
        if !opened_files {
            if paths.iter().any(|p| p.is_dir()) {
                app.open_picker();
            } else {
                app.startup();
            }
        }
        // and what the config's start hooks asked for (zett.on("start", …))
        let fx = app.engine.take_start();
        app.after_lua(fx);
        app.started_ms = started.elapsed().as_millis();
        for i in 0..app.docs.len() {
            app.lsp_open(i);
        }
        app
    }

    fn doc(&mut self) -> &mut Doc {
        &mut self.docs[self.cur]
    }

    fn say(&mut self, m: impl Into<String>) {
        let m = m.into();
        // :redir: what is said is kept for the file or register
        if let Some((_, text)) = self.redir.as_mut() {
            text.push_str(&m);
            text.push('\n');
        }
        // an error message: errorbells rings (visualbell flashes the window instead), unless belloff
        // has all or error (Neovim's default: all off)
        if m.starts_with('E') && m[1..].starts_with(|c: char| c.is_ascii_digit()) && self.flag(self.cur, "errorbells") {
            let off = self.setting(self.cur, "belloff");
            if !off.split(',').any(|o| o == "all" || o == "error") {
                if self.flag(self.cur, "visualbell") {
                    let lines = self.docs[self.cur].ed.buf.len_lines();
                    self.flash = Some((self.cur, 0, lines, Instant::now()));
                } else {
                    use std::io::Write;
                    let _ = std::io::stdout().write_all(b"\x07");
                }
            }
        }
        if !m.is_empty() {
            self.history.push(m.clone());
            // messagesopt history:N: how many messages :messages keeps
            let keep = self.setting(self.cur, "messagesopt").split(',').find_map(|p| p.strip_prefix("history:")?.parse::<usize>().ok()).unwrap_or(500);
            if self.history.len() > keep {
                self.history.remove(0);
            }
        }
        self.msg = Some((m, Instant::now()));
    }

    fn main_loop(&mut self, term: &mut DefaultTerminal) -> std::io::Result<()> {
        let mut shape = None;
        while !self.quit {
            self.lsp_tick();
            self.watch_buffer();
            self.watch_config();
            if self.engine.next_timer().is_some_and(|t| t <= Instant::now()) {
                let fx = self.with_lua(|e| e.run_timers());
                self.after_lua(fx);
            }
            self.sync_profile();
            self.sync_dir();
            self.sync_diff();
            self.agent_poll();
            self.notebook_poll();
            self.qf_poll();
            // CTRL-L: everything drawn again; CTRL-Z: back to the shell until `fg`
            if std::mem::take(&mut self.clear) {
                term.clear()?;
            }
            if std::mem::take(&mut self.suspend) {
                let _ = ratatui::crossterm::terminal::disable_raw_mode();
                let _ = execute!(std::io::stdout(), ratatui::crossterm::terminal::LeaveAlternateScreen, ratatui::crossterm::event::DisableMouseCapture);
                let _ = std::process::Command::new("kill").args(["-TSTP", &std::process::id().to_string()]).status();
                let _ = ratatui::crossterm::terminal::enable_raw_mode();
                let _ = execute!(std::io::stdout(), ratatui::crossterm::terminal::EnterAlternateScreen, ratatui::crossterm::event::EnableMouseCapture);
                term.clear()?;
            }
            // termsync: the frame reaches the terminal as one update (no half-drawn screens)
            let sync = self.flag(self.cur, "termsync");
            if sync {
                let _ = execute!(std::io::stdout(), ratatui::crossterm::terminal::BeginSynchronizedUpdate);
            }
            let done = term.draw(|f| self.draw(f))?;
            // zash's hand-drawn marks (draw.rs FxMark): their cells written again, colours and all,
            // inside the effect. Once the view has stayed for 150 ms (a jump scrolls it more than once);
            // a mark keeps its first time, so one already drawn shows drawn (since=) when sent again
            let marks = std::mem::take(&mut self.fx_marks);
            let view = (self.cur, self.docs[self.cur].top, self.docs[self.cur].ed.cursor().line);
            if self.fx_view.is_none_or(|v| (v.0, v.1, v.2) != view) {
                self.fx_view = Some((view.0, view.1, view.2, Instant::now()));
                // the cells were written again: what was on them is gone
                self.fx_sent.clear();
            }
            let settled = self.fx_view.is_some_and(|v| v.3.elapsed() >= Duration::from_millis(150));
            let cells_of = |x: u16, y: u16, w: u16| {
                let mut cells = String::new();
                for cx in x..x.saturating_add(w) {
                    if let Some(c) = done.buffer.cell((cx, y)) {
                        cells.push_str(&sgr(c));
                        cells.push_str(c.symbol());
                    }
                }
                cells
            };
            let mut out_fx = String::new();
            if settled {
                for m in &marks {
                    if self.fx_sent.get(&m.key) == Some(&(m.x, m.y)) {
                        continue;
                    }
                    let first = *self.fx_first.entry(m.key.clone()).or_insert(m.since.min(Instant::now()));
                    let since = first.elapsed().as_millis();
                    let w = ratatui::text::Line::from(m.text.as_str()).width() as u16;
                    out_fx.push_str(&format!("\x1b[{};{}H\x1b]7733;fx;{};since={since}\x1b\\{}\x1b[0m\x1b]7733;fx;end\x1b\\", m.y + 1, m.x + 1, m.spec, cells_of(m.x, m.y, w)));
                    self.fx_sent.insert(m.key.clone(), (m.x, m.y));
                }
            }
            // under the mouse: drawn while it is there, the plain cells back when it leaves
            let hover = self.fx_hover.as_ref().map(|h| (h.x, h.y, ratatui::text::Line::from(h.text.as_str()).width() as u16, h.key.clone()));
            if hover != self.fx_hover_sent {
                if let Some((x, y, w, _)) = self.fx_hover_sent.take() {
                    out_fx.push_str(&format!("\x1b[{};{}H{}\x1b[0m", y + 1, x + 1, cells_of(x, y, w)));
                }
                if let (Some((x, y, w, _)), Some(h)) = (&hover, &self.fx_hover) {
                    out_fx.push_str(&format!("\x1b[{};{}H\x1b]7733;fx;{}\x1b\\{}\x1b[0m\x1b]7733;fx;end\x1b\\", y + 1, x + 1, h.spec, cells_of(*x, *y, *w)));
                }
                self.fx_hover_sent = hover;
            }
            if !out_fx.is_empty() {
                use std::io::Write;
                let mut out = std::io::stdout();
                let _ = write!(out, "\x1b7{out_fx}\x1b8");
                let _ = out.flush();
            }
            if sync {
                let _ = execute!(std::io::stdout(), ratatui::crossterm::terminal::EndSynchronizedUpdate);
            }
            // InsertEnter / InsertLeave, and VimEnter on the first frame (autocommands)
            let m = self.docs[self.cur].ed.mode();
            if self.last_mode != Some(m) {
                match (self.last_mode, m) {
                    (None, _) => {
                        self.fire_autocmds("vimenter", None);
                        self.fire_autocmds("uienter", None);
                    }
                    (_, Mode::Insert) => self.emit("insert_enter"),
                    (Some(Mode::Insert), _) => self.emit("insert_leave"),
                    _ => {}
                }
                self.last_mode = Some(m);
            }
            // files changed outside: every second (autoread, W11)
            if self.checked.elapsed() >= Duration::from_secs(1) {
                self.checked = Instant::now();
                self.checktime();
            }
            // title: the terminal's title, when it changed
            if self.title_want != self.title_set {
                if let Some(t) = &self.title_want {
                    let _ = execute!(std::io::stdout(), ratatui::crossterm::terminal::SetTitle(t));
                }
                self.title_set = self.title_want.clone();
            }
            // ZETT_EXIT_AFTER_DRAW=1: quit after the first frame (start-up timing)
            if std::env::var_os("ZETT_EXIT_AFTER_DRAW").is_some() {
                self.quit = true;
            }
            // the cursor's shape follows the mode, as guicursor says (empty: left as the terminal has it)
            let want = self.cursor_shape();
            if shape != Some(want) {
                if let Some(w) = want {
                    let _ = execute!(std::io::stdout(), crate::display::cursor_style(w));
                }
                shape = Some(want);
            }
            // a pending key sequence ends after timeoutlen (never with notimeout)
            let timeoutlen = if self.flag(self.cur, "timeout") { Duration::from_millis(self.num(self.cur, "timeoutlen") as u64) } else { Duration::from_secs(86_400) };
            let wait = if self.keyseq.is_empty() { Duration::from_millis(500) } else { timeoutlen.saturating_sub(self.key_at.elapsed()).max(Duration::from_millis(50)) };
            // servers answer on their own time: look more often while any runs
            let wait = if self.lsp.clients.is_empty() && self.agent.is_none() && !self.notebook.busy() { wait } else { wait.min(Duration::from_millis(100)) };
            // a plugin's timer (zett.defer) wakes the loop when it is due
            // a pulse running: frames enough to see it fade
            if self.flash.is_some_and(|f| f.3.elapsed() > Duration::from_millis(1600)) {
                self.flash = None;
            }
            let wait = if self.flash.is_some() { wait.min(Duration::from_millis(60)) } else { wait };
            let wait = match self.engine.next_timer() {
                Some(t) => wait.min(t.saturating_duration_since(Instant::now())),
                None => wait,
            };
            if !event::poll(wait)? {
                if !self.keyseq.is_empty() && self.key_at.elapsed() >= timeoutlen {
                    // keys that are exactly a mapping run now; a prefix alone keeps its menu open
                    // until the next key (as which-key does)
                    let seq = self.keyseq.clone();
                    let mode = self.keymode();
                    match self.with_lua(|e| e.timeout(mode, &seq)) {
                        Feed::Ran(fx) => {
                            self.keyseq.clear();
                            self.menu.clear();
                            self.after_lua(fx);
                        }
                        // insert and visual mode: the keys that started no mapping are typed after all
                        _ if mode != "n" && mode != "e" => {
                            let seq = std::mem::take(&mut self.keyseq);
                            self.after_feed(Feed::None, seq);
                        }
                        _ => self.key_at = Instant::now() + Duration::from_secs(3600),
                    }
                }
                continue;
            }
            match event::read()? {
                Event::Key(k) if k.kind != KeyEventKind::Release => self.key(k),
                Event::Mouse(m) => self.mouse(m),
                Event::Paste(s) => {
                    for c in s.chars() {
                        self.doc().ed.key(if c == '\n' { Key::code(Code::Enter) } else { Key::ch(c) });
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

}

fn draw_help(f: &mut Frame, title: &str, rows: &[(String, String)]) {
    let area = f.area();
    let w = (area.width * 6 / 10).max(40).min(area.width);
    let h = ((rows.len() as u16) + 4).min(area.height.saturating_sub(2)).max(5);
    let a = Rect { x: area.x + (area.width - w) / 2, y: area.y + (area.height.saturating_sub(h)) / 3, width: w, height: h };
    f.render_widget(Clear, a);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(crate::display::float_set())
        .border_style(hl("FloatBorder"))
        .title(format!(" {title} "))
        .title_bottom(" any key closes · change them in ~/.config/zett/init.lua ");
    let inner = block.inner(a);
    f.render_widget(block, a);
    let lines: Vec<Line> = rows
        .iter()
        .map(|(k, d)| Line::from(vec![Span::styled(format!(" {k:<18}"), hl("HelpKey")), Span::raw(d.as_str())]))
        .collect();
    f.render_widget(Paragraph::new(lines), inner);
}

/// A terminal key as zett-edit's key.
fn to_key(k: &KeyEvent) -> Option<Key> {
    let ctrl = k.modifiers.contains(KeyModifiers::CONTROL);
    let alt = k.modifiers.contains(KeyModifiers::ALT);
    let shift = k.modifiers.contains(KeyModifiers::SHIFT) && !matches!(k.code, KeyCode::Char(_));
    let code = match k.code {
        KeyCode::Char(c) => Code::Char(c),
        KeyCode::Esc => Code::Esc,
        KeyCode::Enter => Code::Enter,
        KeyCode::Backspace => Code::Backspace,
        KeyCode::Delete => Code::Delete,
        KeyCode::Tab => Code::Tab,
        KeyCode::Left => Code::Left,
        KeyCode::Right => Code::Right,
        KeyCode::Up => Code::Up,
        KeyCode::Down => Code::Down,
        KeyCode::Home => Code::Home,
        KeyCode::End => Code::End,
        KeyCode::PageUp => Code::PageUp,
        KeyCode::PageDown => Code::PageDown,
        KeyCode::Insert => Code::Insert,
        KeyCode::F(n) if (1..=12).contains(&n) => Code::F(n),
        KeyCode::BackTab => return Some(Key { code: Code::Tab, ctrl, alt, shift: true }),
        _ => return None,
    };
    Some(Key { code, ctrl, alt, shift })
}

/// What zett shows when it starts without a file: `zett.opt.startup`.
#[derive(Clone, Debug, PartialEq)]
enum Startup {
    /// the scratch buffer (the default, as in Emacs); its text is `zett.opt.scratch_message`
    Scratch,
    /// the start screen (made in the config with zett.dashboard{…}, or zett's own)
    Dashboard,
    /// any command: "recent-files", "find-file", one of the config's own
    Command(String),
}

impl Startup {
    fn from_opt(v: Option<String>) -> Self {
        match v.as_deref().map(str::trim) {
            None | Some("") | Some("scratch") => Startup::Scratch,
            Some("dashboard") => Startup::Dashboard,
            Some(cmd) => Startup::Command(cmd.to_string()),
        }
    }
}

/// A terminal key in the engine's notation (`a`, `<Space>`, `<C-w>`, `<CR>`).
fn token(k: &KeyEvent) -> String {
    let ctrl = k.modifiers.contains(KeyModifiers::CONTROL);
    let alt = k.modifiers.contains(KeyModifiers::ALT);
    let named = |n: &str| {
        let mut m = String::new();
        if ctrl {
            m.push_str("C-");
        }
        if alt {
            m.push_str("A-");
        }
        format!("<{m}{n}>")
    };
    match k.code {
        KeyCode::Char(' ') => named("Space"),
        KeyCode::Char('<') if !ctrl && !alt => "<lt>".into(),
        // inside <…> a bare < or > would end the notation: M-< is <A-lt>, M-> <A-gt>
        KeyCode::Char('<') => named("lt"),
        KeyCode::Char('>') if ctrl || alt => named("gt"),
        KeyCode::Char(c) if ctrl || alt => named(&c.to_lowercase().to_string()),
        KeyCode::Char(c) => c.to_string(),
        KeyCode::Esc => named("Esc"),
        KeyCode::Enter => named("CR"),
        KeyCode::Backspace => named("BS"),
        KeyCode::Tab => named("Tab"),
        KeyCode::BackTab => named("S-Tab"),
        KeyCode::Up => named("Up"),
        KeyCode::Down => named("Down"),
        KeyCode::Left => named("Left"),
        KeyCode::Right => named("Right"),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_option() {
        assert_eq!(Startup::from_opt(None), Startup::Scratch);
        assert_eq!(Startup::from_opt(Some(" scratch ".into())), Startup::Scratch);
        assert_eq!(Startup::from_opt(Some("dashboard".into())), Startup::Dashboard);
        assert_eq!(Startup::from_opt(Some("recent-files".into())), Startup::Command("recent-files".into()));
    }

    #[test]
    fn fuzzy_prefers_name_starts_and_runs() {
        let q: Vec<char> = "main".chars().collect();
        let a = fuzzy(&q, "src/main.rs").unwrap();
        let b = fuzzy(&q, "src/my_animation.rs").unwrap();
        assert!(a > b, "{a} vs {b}");
        assert!(fuzzy(&q, "src/lib.rs").is_none());
    }

    #[test]
    fn highlights_rust_with_terminal_colours() {
        let ss = SyntaxSet::load_defaults_newlines();
        let theme = palette_theme();
        let hl = Highlighter::new(&theme);
        let mut d = Doc::new(Editor::from_str("// hi\nfn main() { let s = \"x\"; }\n"), "t.rs".into(), "Rust".into());
        let l0 = d.spans(0, &ss, &hl);
        assert_eq!(l0[0].0.fg, Some(Color::Indexed(8)), "comments are colour 8: {l0:?}");
        let l1 = d.spans(1, &ss, &hl);
        assert!(l1.iter().any(|(s, t)| t.contains("fn") && s.fg == Some(Color::Indexed(5))), "{l1:?}");
        assert!(l1.iter().any(|(s, t)| t.contains("\"x\"") || (t.contains('x') && s.fg == Some(Color::Indexed(2)))), "{l1:?}");
    }

    #[test]
    fn keys_in_engine_notation() {
        let k = |code, m| KeyEvent::new(code, m);
        assert_eq!(token(&k(KeyCode::Char(' '), KeyModifiers::NONE)), "<Space>");
        assert_eq!(token(&k(KeyCode::Char('w'), KeyModifiers::CONTROL)), "<C-w>");
        assert_eq!(token(&k(KeyCode::Char('<'), KeyModifiers::NONE)), "<lt>");
        assert_eq!(token(&k(KeyCode::Enter, KeyModifiers::NONE)), "<CR>");
    }
}

/// A cell's colours and attributes as SGR (zash gets the cell exactly as ratatui drew it).
fn sgr(c: &ratatui::buffer::Cell) -> String {
    use ratatui::style::{Color, Modifier};
    let col = |c: Color, fg: bool| -> Option<String> {
        let base = if fg { 30 } else { 40 };
        Some(match c {
            Color::Reset => return None,
            Color::Rgb(r, g, b) => format!("{};2;{r};{g};{b}", base + 8),
            Color::Indexed(n) => format!("{};5;{n}", base + 8),
            Color::Black => format!("{base}"),
            Color::Red => format!("{}", base + 1),
            Color::Green => format!("{}", base + 2),
            Color::Yellow => format!("{}", base + 3),
            Color::Blue => format!("{}", base + 4),
            Color::Magenta => format!("{}", base + 5),
            Color::Cyan => format!("{}", base + 6),
            Color::Gray => format!("{}", base + 7),
            Color::DarkGray => format!("{}", base + 60),
            Color::LightRed => format!("{}", base + 61),
            Color::LightGreen => format!("{}", base + 62),
            Color::LightYellow => format!("{}", base + 63),
            Color::LightBlue => format!("{}", base + 64),
            Color::LightMagenta => format!("{}", base + 65),
            Color::LightCyan => format!("{}", base + 66),
            Color::White => format!("{}", base + 67),
        })
    };
    let mut parts = vec!["0".to_string()];
    let m = c.modifier;
    for (bit, code) in [(Modifier::BOLD, "1"), (Modifier::DIM, "2"), (Modifier::ITALIC, "3"), (Modifier::UNDERLINED, "4"), (Modifier::REVERSED, "7"), (Modifier::CROSSED_OUT, "9")] {
        if m.contains(bit) {
            parts.push(code.into());
        }
    }
    parts.extend(col(c.fg, true));
    parts.extend(col(c.bg, false));
    format!("\x1b[{}m", parts.join(";"))
}
