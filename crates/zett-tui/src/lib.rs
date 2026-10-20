//! zett in the terminal: the code editor, Doom-style, in the terminal's own colours.

mod doom;
mod lsp;
mod panels;
mod hl;
use hl::hl;
mod clipboard;
mod cmdline;
mod ex;
mod minibuf;
use minibuf::{fuzzy, Cand, Minibuf, Placement};
use doom::{project_of, Pick, WTree, Win};

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEventKind};
use ratatui::crossterm::{cursor::SetCursorStyle, execute};
use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};
use ratatui::{DefaultTerminal, Frame};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use syntect::highlighting::{HighlightIterator, HighlightState, Highlighter, ScopeSelectors, StyleModifier, Theme, ThemeItem, ThemeSettings};
use syntect::parsing::{ParseState, ScopeStack, SyntaxReference, SyntaxSet};
use zett_core::{Buffer, Pos};
use zett_edit::{Code, Editor, Key, Mode};
use zett_script::{Effect, Engine, Feed, MenuItem, PanelView};

const TAB: usize = 4;

// ── syntax colours from the terminal's palette ──

/// A syntect colour that means "ANSI colour n" (alpha 0) or "the default" (alpha 1).
fn ansi(n: u8) -> syntect::highlighting::Color {
    syntect::highlighting::Color { r: n, g: 0, b: 0, a: 0 }
}

fn to_color(c: syntect::highlighting::Color) -> Color {
    match c.a {
        0 => Color::Indexed(c.r),
        255 => Color::Rgb(c.r, c.g, c.b),
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
}

impl Doc {
    fn open(path: &Path, ss: &SyntaxSet) -> std::io::Result<Doc> {
        let buf = Buffer::open(path)?;
        let name = path.display().to_string();
        let syntax = ss.find_syntax_for_file(path).ok().flatten().map(|s| s.name.clone()).unwrap_or_else(|| {
            let first = buf.line(0);
            ss.find_syntax_by_first_line(&first).map(|s| s.name.clone()).unwrap_or_else(|| "Plain Text".into())
        });
        Ok(Doc::new(Editor::new(buf), name, syntax))
    }

    fn scratch() -> Doc {
        Doc::new(Editor::from_str(""), "[scratch]".into(), "Plain Text".into())
    }

    fn new(ed: Editor, name: String, syntax: String) -> Doc {
        let shape = (ed.buf.len_lines(), ed.buf.len_chars(), ed.cursor().line);
        Doc { ed, name, syntax, top: 0, left: 0, states: Vec::new(), shape, diags: Vec::new(), lsp: false, lsp_rev: 0 }
    }

    fn dirty(&self) -> bool {
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
    s.replace('\t', &" ".repeat(TAB))
}

/// The screen column of a character column in a line (tabs count as TAB).
fn screen_col(line: &str, col: usize) -> usize {
    line.chars().take(col).map(|c| if c == '\t' { TAB } else { 1 }).sum()
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
    minibuf: Option<Minibuf>,
    /// where the minibuffer is drawn (zett.opt.minibuffer)
    placement: Placement,
    popup: Option<Popup>,
    quit: bool,
    /// editor area from the last draw (for the mouse and scrolling)
    area: Rect,
    gutter: u16,
    /// windows (by id; closed ones are None), their split tree, the focused one
    wins: Vec<Option<Win>>,
    tree: WTree,
    focus: usize,
    win_rects: Vec<(usize, Rect)>,
    /// the dashboard shows (no file yet) and its chosen button
    dash: bool,
    dash_sel: usize,
    last_doc: Option<usize>,
    last_compile: Option<String>,
    numbers: bool,
    relative: bool,
    tabs: bool,
    /// the key profile: Emacs (default) or Vim (`zett.opt.keys = "vim"`), read with the options
    emacs: bool,
    /// Emacs profile: Esc was pressed, the next key gets Meta (Esc f = M-f, as in Emacs and zsh)
    meta: bool,
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
}

/// Opens the editor in the terminal: files (or a folder) from the command line.
pub fn run(paths: Vec<PathBuf>) -> std::io::Result<()> {
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
            root,
            ss,
            theme: palette_theme(),
            engine,
            keyseq: Vec::new(),
            menu: Vec::new(),
            key_at: Instant::now(),
            cmdline: None,
            msg,
            minibuf: None,
            placement: Placement::Bottom,
            popup: None,
            quit: false,
            area: Rect::default(),
            gutter: 4,
            wins: vec![Some(Win::default())],
            tree: WTree::Leaf(0),
            focus: 0,
            win_rects: Vec::new(),
            dash: false,
            dash_sel: 0,
            last_doc: None,
            last_compile: None,
            numbers: true,
            relative: false,
            tabs: false,
            emacs: true,
            meta: false,
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
        if !m.is_empty() {
            self.history.push(m.clone());
            if self.history.len() > 200 {
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
            term.draw(|f| self.draw(f))?;
            // the cursor's shape follows the mode
            let bar = self.cmdline.is_some() || self.minibuf.is_some() || self.docs[self.cur].ed.mode() == Mode::Insert;
            if shape != Some(bar) {
                let _ = execute!(std::io::stdout(), if bar { SetCursorStyle::SteadyBar } else { SetCursorStyle::SteadyBlock });
                shape = Some(bar);
            }
            // a pending key sequence ends after a moment (Neovim's timeoutlen)
            let wait = if self.keyseq.is_empty() { Duration::from_millis(500) } else { Duration::from_millis(900).saturating_sub(self.key_at.elapsed()).max(Duration::from_millis(50)) };
            // servers answer on their own time: look more often while any runs
            let wait = if self.lsp.clients.is_empty() { wait } else { wait.min(Duration::from_millis(100)) };
            // a plugin's timer (zett.defer) wakes the loop when it is due
            let wait = match self.engine.next_timer() {
                Some(t) => wait.min(t.saturating_duration_since(Instant::now())),
                None => wait,
            };
            if !event::poll(wait)? {
                if !self.keyseq.is_empty() && self.key_at.elapsed() >= Duration::from_millis(900) {
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

    // ── input ──

    fn key(&mut self, k: KeyEvent) {
        if let Some(p) = self.popup.take() {
            // any key closes help (Esc and q too)
            let _ = p;
            return;
        }
        if self.minibuf.is_some() {
            return self.minibuf_key(k);
        }
        if self.cmdline.is_some() {
            return self.cmdline_key(k);
        }
        if self.panel_key(k) {
            return;
        }
        let ctrl = k.modifiers.contains(KeyModifiers::CONTROL);
        // the Emacs profile: every key through its keymap. C-s is isearch and C-p the previous
        // line there, so the Vim profile's shortcuts below must not catch them first
        if self.emacs_keys() {
            return self.emacs_key(&k);
        }
        // Vim profile: Ctrl+S saves in any mode, Ctrl+P finds a file
        if ctrl && k.code == KeyCode::Char('s') {
            return self.save(None);
        }
        if ctrl && k.code == KeyCode::Char('p') {
            return self.open_picker();
        }
        let Some(ek) = to_key(&k) else { return };
        let mode = self.docs[self.cur].ed.mode();
        // gc in visual mode comments the selection (the engine only sees normal-mode keys)
        if matches!(mode, Mode::Visual | Mode::VisualLine) && !ctrl {
            if self.vis_g {
                self.vis_g = false;
                if k.code == KeyCode::Char('c') {
                    return self.comment();
                }
                self.edit_key(Key::ch('g'));
            } else if k.code == KeyCode::Char('g') {
                self.vis_g = true;
                return;
            }
        }
        // : in visual mode: the selected lines as the range ('<,'>), as Vim does
        if matches!(mode, Mode::Visual | Mode::VisualLine) && k.code == KeyCode::Char(':') {
            let d = &self.docs[self.cur];
            if let Some((a, b)) = d.ed.selection_range() {
                let (la, lb) = (d.ed.buf.to_pos(a).line, d.ed.buf.to_pos(b.saturating_sub(1).max(a)).line);
                self.vis_range = Some((la.min(lb), la.max(lb)));
            }
            self.edit_key(Key::code(Code::Esc));
            self.cmdline = Some("'<,'>".into());
            self.cmd_sel = 0;
            return;
        }
        // the dashboard's own keys (the SPC tree works there too)
        if self.dash && mode == Mode::Normal && self.keyseq.is_empty() && !ctrl {
            match k.code {
                KeyCode::Char('j') | KeyCode::Down | KeyCode::Tab => {
                    self.dash_sel = (self.dash_sel + 1) % self.dash_len();
                    return;
                }
                KeyCode::Char('k') | KeyCode::Up | KeyCode::BackTab => {
                    self.dash_sel = (self.dash_sel + self.dash_len() - 1) % self.dash_len();
                    return;
                }
                KeyCode::Enter => return self.dash_run(),
                KeyCode::Char('q') => {
                    self.quit = true;
                    return;
                }
                _ => {}
            }
        }
        if mode == Mode::Normal {
            let tok = token(&k);
            if tok.is_empty() {
                return;
            }
            if tok == "<CR>" && self.keyseq.is_empty() && self.follow_help() {
                return;
            }
            if tok == "<Esc>" && !self.keyseq.is_empty() {
                self.keyseq.clear();
                self.menu.clear();
                return;
            }
            let mut seq = std::mem::take(&mut self.keyseq);
            seq.push(tok);
            let feed = self.with_lua(|e| e.feed("n", &seq));
            return self.after_feed(feed, seq);
        }
        // the completion popup takes its keys first
        if mode == Mode::Insert && self.comp.is_some() {
            let n = self.comp.as_ref().map(|c| c.shown.len()).unwrap_or(0);
            let c = self.comp.as_mut().unwrap();
            match k.code {
                KeyCode::Tab | KeyCode::Down => return c.sel = (c.sel + 1) % n.max(1),
                KeyCode::Char('n') if ctrl => return c.sel = (c.sel + 1) % n.max(1),
                KeyCode::BackTab | KeyCode::Up => return c.sel = (c.sel + n.max(1) - 1) % n.max(1),
                KeyCode::Char('p') if ctrl => return c.sel = (c.sel + n.max(1) - 1) % n.max(1),
                KeyCode::Enter => return self.comp_accept(),
                KeyCode::Char('y') if ctrl => return self.comp_accept(),
                KeyCode::Esc => self.comp = None,
                _ => {}
            }
        }
        let typed = match k.code {
            KeyCode::Char(c) if !ctrl => Some(c),
            KeyCode::Backspace => None,
            _ => Some('\0'),
        };
        self.edit_key(ek);
        if mode == Mode::Insert {
            self.comp_after_key(typed);
        }
    }

    /// What the engine did with keys: ran a mapping, waits for more, or left them to Vim.
    fn after_feed(&mut self, feed: Feed, seq: Vec<String>) {
        match feed {
            Feed::Ran(fx) => {
                self.menu.clear();
                self.after_lua(fx);
            }
            Feed::Pending(menu) => {
                self.keyseq = seq;
                self.menu = menu;
                self.key_at = Instant::now();
            }
            Feed::None => {
                self.menu.clear();
                for t in seq {
                    for k in zett_edit::keys::parse(&t) {
                        self.edit_key(k);
                    }
                }
            }
        }
    }

    /// Started without a file: what `zett.opt.startup` asks for (see [`Startup`]).
    fn startup(&mut self) {
        match Startup::from_opt(self.engine.opt("startup")) {
            Startup::Scratch => {
                // the scratch buffer's own text, when the config gives one; the buffer stays clean
                if let Some(text) = self.engine.opt("scratch_message").filter(|t| !t.is_empty()) {
                    if let Some(d) = self.docs.iter_mut().find(|d| d.name == "[scratch]") {
                        d.ed = Editor::from_str(&text);
                    }
                } else if self.emacs_keys() {
                    self.say("C-x C-f opens a file · M-x runs a command · C-h d reads the documentation");
                }
            }
            Startup::Dashboard => self.dash = true,
            Startup::Command(cmd) => match self.with_lua(|e| e.run(&cmd)) {
                Ok(fx) => self.after_lua(fx),
                Err(e) => self.say(format!("zett.opt.startup: {e}")),
            },
        }
    }

    /// Every buffer in step with the app: the key profile, and the one kill ring they share.
    /// Runs before each draw, so no place that opens a buffer has to.
    fn sync_profile(&mut self) {
        let emacs = self.emacs;
        for d in self.docs.iter_mut() {
            if d.ed.is_emacs() != emacs {
                d.ed.set_emacs(emacs);
            }
            d.ed.use_kill_ring(&self.kill_ring);
        }
    }

    /// The key profile: Emacs unless `zett.opt.keys = "vim"`.
    fn emacs_keys(&self) -> bool {
        self.emacs
    }
    /// The keymap mode keys go through: "e" (Emacs) or Vim's normal-mode "n".
    fn keymode(&self) -> &'static str {
        if self.emacs_keys() { "e" } else { "n" }
    }

    /// Emacs profile: every key goes through the keymap (chords like C-x C-f wait for more);
    /// an unbound printable key types itself, anything else unbound says so.
    fn emacs_key(&mut self, k: &KeyEvent) {
        // Esc, as in Emacs and zsh: Meta on the next key (Esc f = M-f). Esc Esc, or Esc in the
        // middle of a chord, cancels everything (Emacs' keyboard-escape-quit)
        let pending_meta = std::mem::take(&mut self.meta);
        if k.code == KeyCode::Esc && k.modifiers.is_empty() {
            if pending_meta || !self.keyseq.is_empty() {
                self.keyseq.clear();
                self.menu.clear();
                return self.after_lua(vec![Effect::Edit("keyboard-quit".into(), 1)]);
            }
            self.meta = true;
            return self.say("ESC-");
        }
        // the pending Meta joins the key itself, so token() is the one place keys are spelled
        let mut k = *k;
        if pending_meta {
            k.modifiers |= KeyModifiers::ALT;
            self.msg = None; // the "ESC-" prompt is answered
        }
        let tok = token(&k);
        if tok.is_empty() {
            return;
        }
        // C-g abandons a half-typed chord or argument (and, bound, deactivates the mark)
        if tok == "<C-g>" && (!self.keyseq.is_empty() || self.arg.active()) {
            self.keyseq.clear();
            self.menu.clear();
            self.arg.take();
            return self.say("Quit");
        }
        // typing a numeric argument: digits (and a leading -) are part of it, not text
        if self.arg.active() && self.keyseq.is_empty() {
            let plain = !k.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT);
            match k.code {
                KeyCode::Char(c) if plain && c.is_ascii_digit() => {
                    self.arg.digit(c.to_digit(10).unwrap_or(0));
                    return self.say(self.arg.prompt());
                }
                KeyCode::Char('-') if plain && self.arg.accepts_minus() => {
                    self.arg.minus();
                    return self.say(self.arg.prompt());
                }
                _ => {}
            }
        }
        self.last_key = tok.clone();
        self.engine.set_arg(self.arg.value());
        let mut seq = std::mem::take(&mut self.keyseq);
        seq.push(tok);
        // a search in progress has its own keymap (the isearch plugin's), as Emacs's isearch-mode-map
        let searching = self.docs[self.cur].ed.isearch().is_some();
        match self.with_lua(|e| e.feed(if searching { "isearch" } else { "e" }, &seq)) {
            Feed::None if searching => {
                self.menu.clear();
                let plain = !k.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT);
                match k.code {
                    KeyCode::Char(c) if plain => self.doc().ed.isearch_char(c),
                    // any other key ends the search where it is, then does what it always does
                    _ => {
                        self.doc().ed.exec("isearch-exit", 1);
                        self.emacs_key(&k);
                    }
                }
            }
            Feed::None => {
                self.menu.clear();
                let plain = !k.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT);
                match k.code {
                    KeyCode::Char(c) if plain && seq.len() == 1 => {
                        // C-u 5 a types aaaaa
                        let times = self.arg.take().map_or(1, |n| n.unsigned_abs().max(1));
                        let before = self.docs[self.cur].ed.cursor().line;
                        for _ in 0..times {
                            self.doc().ed.self_insert(c);
                        }
                        self.doc().touched(before);
                        self.edit_at = Instant::now();
                    }
                    _ => {
                        self.arg.take();
                        self.say(format!("{} is undefined", seq.join(" ")))
                    }
                }
            }
            f => {
                // a Lua command used the argument (zett.arg); an editing one takes it in effects()
                if let Feed::Ran(fx) = &f {
                    if !fx.iter().any(|e| matches!(e, Effect::Edit(..))) {
                        self.arg.take();
                    }
                }
                self.after_feed(f, seq)
            }
        }
    }

    fn edit_key(&mut self, k: Key) {
        let before = self.docs[self.cur].ed.cursor().line;
        let rev = self.docs[self.cur].ed.buf.rev();
        self.doc().ed.key(k);
        self.doc().touched(before);
        if self.docs[self.cur].ed.buf.rev() != rev {
            self.edit_at = Instant::now();
        }
    }

    fn effects(&mut self, fx: Vec<Effect>) {
        for e in fx {
            match e {
                Effect::Notify(m) => {
                    // plugins may show messages too (fidget); a message they send is not sent back
                    if !self.notifying && self.engine.has_hooks("notify") {
                        self.notifying = true;
                        let fx = self.with_lua(|e| e.emit_fields("notify", &[("message", m.clone())]));
                        self.after_lua(fx);
                        self.notifying = false;
                    }
                    self.say(m)
                }
                Effect::Ui(name, arg) if self.doom_ui(&name, &arg) => {}
                Effect::Ui(name, arg) => match name.as_str() {
                    "quickOpen" | "search" => self.open_picker(),
                    "commandLine" => self.cmdline = Some(arg),
                    "help" => self.help(&arg),
                    "reload" => {
                        let mut e = Engine::new(None);
                        e.set_frontend("editor");
                        e.load_config(&zett_script::config_dir());
                        let n = e.plugins.len();
                        self.engine = e;
                        hl::reset();
                        self.take_opts();
                        self.say(format!("config reloaded ({n} plugins)"));
                    }
                    "openConfig" => {
                        let p = zett_script::config_dir().join("init.lua");
                        let _ = std::fs::create_dir_all(zett_script::config_dir());
                        self.open(&p);
                    }
                    "toggleSidebar" | "focusBlock" => {}
                    "openFile" => {
                        let (how, path) = arg.split_once('\t').unwrap_or(("", arg.as_str()));
                        match how {
                            "right" | "below" => {
                                self.engine.focus_panel(None);
                                self.doom_ui("split", how);
                            }
                            _ => {}
                        }
                        self.open(Path::new(path));
                    }
                    "ex" => self.command(&arg),
                    "clipboard" => {
                        if !clipboard::copy(&arg) {
                            self.say("could not copy to the clipboard");
                        }
                    }
                    other => self.say(format!("{other}: only in the window (zett --gui)")),
                },
                // C-u, M-0…M-9, M--: build the argument for the next command
                Effect::Edit(name, _) if zett_edit::prefix::is_command(&name) => {
                    match name.as_str() {
                        "universal-argument" => self.arg.universal(),
                        "negative-argument" => self.arg.minus(),
                        _ => {
                            let d = self.last_key.chars().rev().find_map(|c| c.to_digit(10)).unwrap_or(0);
                            self.arg.digit(d);
                        }
                    }
                    self.say(self.arg.prompt());
                }
                Effect::Edit(name, n) => {
                    let before = self.docs[self.cur].ed.cursor().line;
                    let rev = self.docs[self.cur].ed.buf.rev();
                    let kills = self.docs[self.cur].ed.kill_count();
                    // the kill ring and the system clipboard are one, as in Emacs
                    if name == "yank" {
                        if let Some(t) = clipboard::paste() {
                            self.doc().ed.offer_paste(t);
                        }
                    }
                    // the typed argument, else the count the effect carries
                    let arg = self.arg.take().or((n > 1).then_some(n as i64));
                    if arg.is_some() {
                        self.msg = None; // the "C-u 4-" prompt is answered
                    }
                    self.doc().ed.exec_arg(&name, arg);
                    if self.docs[self.cur].ed.kill_count() != kills {
                        if let Some(t) = self.docs[self.cur].ed.last_kill() {
                            clipboard::copy(&t);
                        }
                    }
                    self.doc().touched(before);
                    if self.docs[self.cur].ed.buf.rev() != rev {
                        self.edit_at = Instant::now();
                    }
                }
                Effect::Open(_) | Effect::Refresh => {}
            }
        }
        for e in std::mem::take(&mut self.engine.errors) {
            self.say(e);
        }
    }

    fn help(&mut self, what: &str) {
        let rows: Vec<(String, String)> = if what == "commands" {
            self.engine.commands().into_iter().map(|(n, d, _)| (n, d)).collect()
        } else {
            let leader = self.engine.leader();
            self.engine.bindings(self.keymode()).into_iter().map(|(k, d)| (k.replace("<leader>", if leader == "<Space>" { "SPC " } else { &leader }), d)).collect()
        };
        self.popup = Some(Popup::Help(if what == "commands" { "Commands".into() } else { "Keys".into() }, rows));
    }

    fn open(&mut self, p: &Path) {
        // already open: show it
        let name = p.display().to_string();
        if let Some(i) = self.docs.iter().position(|d| d.name == name) {
            self.show_doc(i);
            return;
        }
        if p.is_file() {
            doom::remember(p);
        }
        match Doc::open(p, &self.ss) {
            Ok(d) => {
                // an untouched scratch buffer gives way
                if self.docs.len() == 1 && self.docs[0].name == "[scratch]" && !self.docs[0].dirty() {
                    self.docs.clear();
                }
                self.docs.push(d);
                let i = self.docs.len() - 1;
                self.show_doc(i);
                self.lsp_open(i);
            }
            Err(e) => self.say(format!("{}: {e}", p.display())),
        }
    }

    fn open_picker(&mut self) {
        self.pick(Pick::Files(self.root.clone()));
    }

    fn cmdline_key(&mut self, k: KeyEvent) {
        let Some(line) = self.cmdline.as_mut() else { return };
        match k.code {
            KeyCode::Esc => {
                self.cmdline = None;
                if self.prompt.take().is_some() {
                    let fx = self.with_lua(|e| e.input_done(None));
                    self.after_lua(fx);
                }
            }
            KeyCode::Enter => {
                let l = std::mem::take(line);
                self.cmdline = None;
                if self.prompt.take().is_some() {
                    let fx = self.with_lua(|e| e.input_done(Some(l)));
                    return self.after_lua(fx);
                }
                self.command(l.trim());
            }
            KeyCode::Backspace => {
                if line.pop().is_none() && self.prompt.is_none() {
                    self.cmdline = None;
                }
            }
            KeyCode::Tab if self.prompt.is_some() => {}
            KeyCode::Tab => self.complete_cmdline(),
            KeyCode::Down => self.cmd_sel += 1,
            KeyCode::Up => self.cmd_sel = self.cmd_sel.saturating_sub(1),
            KeyCode::Char('n') if k.modifiers.contains(KeyModifiers::CONTROL) => self.cmd_sel += 1,
            KeyCode::Char('p') if k.modifiers.contains(KeyModifiers::CONTROL) => self.cmd_sel = self.cmd_sel.saturating_sub(1),
            KeyCode::Char(c) => {
                line.push(c);
                self.cmd_sel = 0;
            }
            _ => {}
        }
    }

    /// A `:` line: Vim's file commands, then zett's commands and `lua …`.
    fn command(&mut self, l: &str) {
        // Neovim's commands with ranges and the rest of its everyday ones
        if self.ex(l) {
            return;
        }
        let (cmd, arg) = l.split_once(' ').map(|(a, b)| (a, b.trim())).unwrap_or((l, ""));
        match cmd {
            "" => {}
            "w" => self.save((!arg.is_empty()).then(|| PathBuf::from(arg))),
            "q" | "quit" => {
                if !self.close_win() {
                    self.close(false);
                }
            }
            "compile" => self.compile(arg),
            "h" | "help" => self.open_help(arg),
            "set" | "se" => self.set_command(arg),
            "hi" | "highlight" => self.hi_command(arg),
            "rename" => self.lsp_rename(arg),
            "wa" | "wall" => {
                let keep = self.cur;
                for i in 0..self.docs.len() {
                    if self.docs[i].dirty() && self.docs[i].ed.buf.path().is_some() {
                        self.cur = i;
                        self.save(None);
                    }
                }
                self.cur = keep;
            }
            "sp" | "split" => {
                self.doom_ui("split", "below");
            }
            "vs" | "vsplit" => {
                self.doom_ui("split", "right");
            }
            "only" => {
                self.doom_ui("onlyWin", "");
            }
            "q!" | "quit!" => self.close(true),
            "qa" | "qall" => {
                if self.docs.iter().any(|d| d.dirty()) {
                    self.say("unsaved changes: :wqa saves them, :qa! drops them");
                } else {
                    self.quit = true;
                }
            }
            "qa!" | "qall!" => self.quit = true,
            "wq" | "x" => {
                self.save(None);
                if !self.docs[self.cur].dirty() {
                    self.close(false);
                }
            }
            "wqa" | "xa" => {
                for i in 0..self.docs.len() {
                    self.cur = i;
                    if self.docs[i].dirty() {
                        self.save(None);
                    }
                }
                if !self.docs.iter().any(|d| d.dirty()) {
                    self.quit = true;
                }
            }
            "e" | "edit" if !arg.is_empty() => {
                let p = if Path::new(arg).is_absolute() { PathBuf::from(arg) } else { self.root.join(arg) };
                self.open(&p);
            }
            "e" | "edit" | "find" | "files" => self.open_picker(),
            "bn" | "bnext" => self.show_doc((self.cur + 1) % self.docs.len()),
            "bp" | "bprevious" => self.show_doc((self.cur + self.docs.len() - 1) % self.docs.len()),
            "bd" | "bdelete" => self.close(false),
            "bm" | "bmove" => match arg.trim() {
                "" => self.say("usage: :bmove +1 | -1 | N (its place in the tab line)"),
                a => match (a.strip_prefix('+'), a.strip_prefix('-'), a.parse::<i32>()) {
                    (Some(n), _, _) => self.move_doc(n.parse().unwrap_or(1)),
                    (_, Some(n), _) => self.move_doc(-n.parse().unwrap_or(1)),
                    (_, _, Ok(n)) => self.move_doc(n - 1 - self.cur as i32),
                    _ => self.say(format!("not a place: {a}")),
                },
            },
            "bo" | "bonly" | "bufonly" => self.close_others(),
            "ls" | "buffers" => {
                let list: Vec<String> = self.docs.iter().enumerate().map(|(i, d)| format!("{}{}{}", if i == self.cur { "%" } else { " " }, d.name, if d.dirty() { " [+]" } else { "" })).collect();
                self.say(list.join("   "));
            }
            "cd" if !arg.is_empty() => {
                let p = PathBuf::from(arg);
                if p.is_dir() {
                    self.root = std::fs::canonicalize(&p).unwrap_or(p);
                    self.say(format!("project: {}", self.root.display()));
                }
            }
            "lua" => match self.with_lua(|e| e.eval(arg)) {
                Ok((out, fx)) => {
                    self.after_lua(fx);
                    if out != "nil" {
                        self.say(out);
                    }
                }
                Err(e) => self.say(e),
            },
            n if n.chars().all(|c| c.is_ascii_digit()) => {
                // :42 goes to line 42
                let line: usize = n.parse().unwrap_or(1);
                let last = self.docs[self.cur].ed.buf.len_lines().saturating_sub(1);
                self.doc().ed.set_cursor(Pos { line: line.saturating_sub(1).min(last), col: 0 });
            }
            other => match self.with_lua(|e| e.run(other)) {
                Ok(fx) => self.after_lua(fx),
                Err(_) => self.say(format!("not a command: {other}")),
            },
        }
    }

    fn save(&mut self, to: Option<PathBuf>) {
        let root = self.root.clone();
        let d = self.doc();
        if let Some(p) = to {
            let p = if p.is_absolute() { p } else { root.join(p) };
            d.name = p.display().to_string();
            d.ed.buf.set_path(p);
        }
        if d.ed.buf.path().is_none() {
            return self.say("no file name: :w name");
        }
        let lines = d.ed.buf.len_lines();
        match d.ed.buf.save() {
            Ok(()) => {
                let m = format!("\"{}\" {lines} lines written", d.name);
                self.say(m);
                let i = self.cur;
                self.lsp_saved(i);
                self.emit("write");
            }
            Err(e) => self.say(format!("could not save: {e}")),
        }
    }

    fn close(&mut self, force: bool) {
        if self.docs[self.cur].dirty() && !force {
            return self.say("unsaved changes: :w saves, :q! drops them");
        }
        let gone = self.cur;
        self.docs.remove(gone);
        if self.docs.is_empty() {
            self.quit = true;
            self.docs.push(Doc::scratch());
        }
        self.cur = self.cur.min(self.docs.len() - 1);
        self.doc_removed(gone);
        self.last_doc = None;
    }

    fn mouse(&mut self, m: ratatui::crossterm::event::MouseEvent) {
        if self.panel_mouse(&m) {
            return;
        }
        if let MouseEventKind::Down(_) = m.kind {
            self.engine.focus_panel(None);
        }
        // a click in another window focuses it first
        if let MouseEventKind::Down(_) = m.kind {
            if let Some(&(id, r)) = self.win_rects.iter().find(|(_, r)| m.column >= r.x && m.column < r.right() && m.row >= r.y && m.row < r.bottom()) {
                if id != self.focus {
                    self.focus_win(id);
                    self.area = Rect { height: r.height.saturating_sub(1), ..r };
                    if let Some(Some(w)) = self.wins.get(id) {
                        self.docs[self.cur].top = w.top;
                        self.docs[self.cur].left = w.left;
                    }
                }
            }
        }
        let a = self.area;
        match m.kind {
            MouseEventKind::ScrollDown | MouseEventKind::ScrollUp => {
                let d = &mut self.docs[self.cur];
                let last = d.ed.buf.len_lines().saturating_sub(1);
                if m.kind == MouseEventKind::ScrollDown { d.top = (d.top + 3).min(last) } else { d.top = d.top.saturating_sub(3) }
                // the cursor stays on screen, as in Vim
                let c = d.ed.cursor();
                let bottom = d.top + a.height.saturating_sub(1) as usize;
                if c.line < d.top || c.line > bottom {
                    let line = c.line.clamp(d.top, bottom.min(last));
                    d.ed.set_cursor(Pos { line, col: c.col });
                }
            }
            MouseEventKind::Down(_) if m.row >= a.y && m.row < a.y + a.height && m.column >= a.x + self.gutter => {
                let d = &mut self.docs[self.cur];
                let line = (d.top + (m.row - a.y) as usize).min(d.ed.buf.len_lines().saturating_sub(1));
                let want = d.left + (m.column - a.x - self.gutter) as usize;
                // screen column back to a character column (tabs are wide)
                let text = d.ed.buf.line(line);
                let mut sc = 0;
                let mut col = 0;
                for ch in text.chars() {
                    let w = if ch == '\t' { TAB } else { 1 };
                    if sc + w > want {
                        break;
                    }
                    sc += w;
                    col += 1;
                }
                d.ed.set_cursor(Pos { line, col });
            }
            _ => {}
        }
    }

    // ── drawing ──

    fn draw(&mut self, f: &mut Frame) {
        // a highlight group changed (zett.hl, :hi): new styles, and the syntax colours again
        if hl::sync(&self.engine) {
            self.theme = palette_theme();
            for d in self.docs.iter_mut() {
                d.states.clear();
            }
        }
        let tabs = self.tabs && self.docs.len() > 1;
        // the minibuffer's candidates take rows from the windows, as in Emacs
        let mini = self.minibuf.as_ref().filter(|_| self.placement == Placement::Bottom).map_or(0, |m| m.rows() as u16);
        let [tabline, body, list, cmd] =
            Layout::vertical([Constraint::Length(if tabs { 1 } else { 0 }), Constraint::Min(2), Constraint::Length(mini), Constraint::Length(1)]).areas(f.area());
        if tabs {
            self.draw_tabs(f, tabline);
        }
        if self.branch_at.elapsed() > Duration::from_secs(3) {
            self.branch = doom::git_branch(&self.root);
            self.branch_at = Instant::now();
        }
        // the focused window shows the current buffer
        self.cur = self.cur.min(self.docs.len() - 1);
        if let Some(w) = self.wins.get_mut(self.focus).and_then(|w| w.as_mut()) {
            w.doc = self.cur;
        }
        // plugin panels take the sides
        let body = self.draw_panels(f, body);
        let mut placed = Vec::new();
        let mut seps = Vec::new();
        self.tree.place(body, &mut placed, &mut seps);
        self.win_rects = placed.clone();
        for s in seps {
            let bar: Vec<Line> = (0..s.height).map(|_| Line::from(Span::styled("│", hl("WinSeparator")))).collect();
            f.render_widget(Paragraph::new(bar), s);
        }
        for (id, r) in placed {
            let Some(w) = self.wins.get(id).and_then(|w| w.clone()) else { continue };
            let focused = id == self.focus;
            let di = w.doc.min(self.docs.len() - 1);
            let text = Rect { height: r.height.saturating_sub(1), ..r };
            let line = Rect { y: r.bottom().saturating_sub(1), height: 1, ..r };
            if focused && self.dash && di == self.cur && self.docs[di].name == "[scratch]" && !self.docs[di].dirty() {
                self.area = text;
                self.draw_dashboard(f, text);
            } else {
                if focused {
                    self.dash = false;
                }
                // a window keeps its own scroll position, even on a buffer shown twice
                self.docs[di].top = w.top;
                self.docs[di].left = w.left;
                self.draw_text(f, text, di, focused);
                if let Some(Some(w)) = self.wins.get_mut(id) {
                    w.top = self.docs[di].top;
                    w.left = self.docs[di].left;
                }
            }
            self.draw_modeline(f, line, di, focused);
        }
        self.draw_float_panels(f, body);
        self.draw_floats(f, body);
        self.draw_cmd(f, cmd);
        self.draw_suggestions(f, cmd);
        if !self.menu.is_empty() {
            self.draw_menu(f, body);
        }
        if self.comp.is_some() {
            self.draw_comp(f);
        }
        if self.minibuf.is_some() {
            self.draw_minibuf(f, list, cmd);
        }
        if let Some(Popup::Help(title, rows)) = &self.popup {
            draw_help(f, title, rows);
        }
    }

    /// The editor settings a config can set (zett.opt.tabs = true, number, relativenumber), as :set does.
    fn take_opts(&mut self) {
        let on = |v: Option<String>| v.map(|v| v == "true");
        if let Some(b) = on(self.engine.opt("tabs")) {
            self.tabs = b;
        }
        if let Some(b) = on(self.engine.opt("number")) {
            self.numbers = b;
        }
        if let Some(b) = on(self.engine.opt("relativenumber")) {
            self.relative = b;
        }
        self.emacs = self.engine.opt("keys").is_none_or(|k| k != "vim");
        self.placement = Placement::from_opt(self.engine.opt("minibuffer"), self.emacs);
    }

    fn draw_tabs(&self, f: &mut Frame, a: Rect) {
        let mut spans = Vec::new();
        for (i, d) in self.docs.iter().enumerate() {
            let name = Path::new(&d.name).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or(d.name.clone());
            // numbered, as Alt+1…9 and :b N count them
            let label = format!(" {} {name}{} ", i + 1, if d.dirty() { " ●" } else { "" });
            let st = if i == self.cur { hl("TabLineSel") } else { hl("TabLine") };
            spans.push(Span::styled(label, st));
            spans.push(Span::raw(" "));
        }
        f.render_widget(Paragraph::new(Line::from(spans)), a);
    }

    fn draw_text(&mut self, f: &mut Frame, a: Rect, di: usize, focused: bool) {
        if focused {
            self.area = a;
        }
        let (ss, theme) = (&self.ss, &self.theme);
        let hl = Highlighter::new(theme);
        let (numbers, relative) = (self.numbers, self.relative);
        let d = &mut self.docs[di];
        let lines = d.ed.buf.len_lines();
        let gutter = if numbers { (lines.to_string().len() + 2).max(4) as u16 } else { 1 };
        if focused {
            self.gutter = gutter;
        }
        let width = a.width.saturating_sub(gutter) as usize;
        let height = a.height as usize;
        // keep the cursor in view (a few lines of margin, like scrolloff)
        let c = d.ed.cursor();
        let margin = 3.min(height / 4);
        if c.line < d.top + margin {
            d.top = c.line.saturating_sub(margin);
        } else if c.line + margin >= d.top + height {
            d.top = (c.line + margin + 1).saturating_sub(height);
        }
        d.top = d.top.min(lines.saturating_sub(1));
        let cur_line_text = d.ed.buf.line(c.line);
        let ccol = screen_col(&cur_line_text, c.col);
        if ccol < d.left {
            d.left = ccol;
        } else if ccol >= d.left + width.max(1) {
            d.left = ccol + 1 - width.max(1);
        }
        let sel = d.ed.selection_range();
        let mut out: Vec<Line> = Vec::with_capacity(height);
        // a search in progress: its visible matches, found once per frame (Search; IncSearch for the
        // one the cursor is on)
        let (found, current) = match d.ed.isearch() {
            Some(s) if !s.query().is_empty() => {
                let current = s.current();
                let from = d.ed.buf.to_char(Pos { line: d.top.min(lines), col: 0 });
                let end = (d.top + height).min(lines);
                let to = if end >= lines { d.ed.buf.len_chars() } else { d.ed.buf.to_char(Pos { line: end, col: 0 }) };
                (d.ed.isearch_matches(from, to), current)
            }
            _ => (Vec::new(), None),
        };
        for row in 0..height {
            let ln = d.top + row;
            if ln >= lines {
                out.push(Line::from(Span::styled(format!("{:>w$} ", "~", w = gutter as usize - 1), hl::hl("NonText"))));
                continue;
            }
            let num_style = if ln == c.line { hl::hl("CursorLineNr") } else { hl::hl("LineNr") };
            let shown = if relative && ln != c.line { ln.abs_diff(c.line) } else { ln + 1 };
            let line_text = d.ed.buf.line(ln);
            // diagnostics: a sign in the gutter, the ranges underlined (in screen columns)
            let sign = d.diags.iter().filter(|x| x.line == ln).map(|x| x.severity).min();
            let marks: Vec<(usize, usize, u8)> = d
                .diags
                .iter()
                .filter(|x| x.line <= ln && x.end_line >= ln)
                .map(|x| {
                    let s = if x.line == ln { screen_col(&line_text, x.col) } else { 0 };
                    let e = if x.end_line == ln { screen_col(&line_text, x.end_col) } else { usize::MAX };
                    (s, e.max(s + 1), x.severity)
                })
                .collect();
            let mut spans = vec![if numbers {
                let num = format!("{:>w$} ", shown, w = gutter as usize - 1);
                match sign {
                    Some(s) => Span::styled(format!("●{}", &num[num.char_indices().nth(1).map(|(i, _)| i).unwrap_or(num.len())..]), num_style.fg(lsp::severity_color(s))),
                    None => Span::styled(num, num_style),
                }
            } else if let Some(s) = sign {
                Span::styled("●", Style::default().fg(lsp::severity_color(s)))
            } else {
                Span::raw(" ")
            }];
            // the selection on this line, in screen columns
            let sel_cols = sel.and_then(|(s, e)| {
                let ls = d.ed.buf.to_char(Pos { line: ln, col: 0 });
                let le = ls + line_text.chars().count();
                let (a, b) = (s.max(ls), e.min(le + 1));
                (a < b).then(|| (screen_col(&line_text, a - ls), screen_col(&line_text, b.min(le) - ls) + usize::from(b > le)))
            });
            // search matches on this line, in screen columns (true: the current one)
            let hits: Vec<(usize, usize, bool)> = if found.is_empty() {
                Vec::new()
            } else {
                let ls = d.ed.buf.to_char(Pos { line: ln, col: 0 });
                let le = ls + line_text.chars().count();
                found
                    .iter()
                    .filter(|(a, b)| *a < le && *b > ls)
                    .map(|&(a, b)| (screen_col(&line_text, a.max(ls) - ls), screen_col(&line_text, b.min(le) - ls), current == Some((a, b))))
                    .collect()
            };
            let mut x = 0usize;
            for (st, text) in d.spans(ln, ss, &hl) {
                for ch in text.chars() {
                    if x >= d.left && x < d.left + width {
                        let mut s = st;
                        if sel_cols.is_some_and(|(a, b)| x >= a && x < b) {
                            s = s.add_modifier(Modifier::REVERSED);
                        }
                        if let Some(h) = hits.iter().find(|h| x >= h.0 && x < h.1) {
                            s = s.patch(hl::hl(if h.2 { "IncSearch" } else { "Search" }));
                        }
                        if let Some(&(_, _, sev)) = marks.iter().filter(|m| x >= m.0 && x < m.1).min_by_key(|m| m.2) {
                            s = s.add_modifier(Modifier::UNDERLINED).underline_color(lsp::severity_color(sev));
                        }
                        spans.push(Span::styled(ch.to_string(), s));
                    }
                    x += 1;
                }
            }
            // a selection over the line end shows one cell for the line break
            if sel_cols.is_some_and(|(a, b)| x >= a && x < b) && x >= d.left && x < d.left + width {
                spans.push(Span::styled(" ", Style::default().add_modifier(Modifier::REVERSED)));
            }
            // the current line stands out, and fades over the lines around it (CursorLine, CursorLineGlow…)
            let band = match ln.abs_diff(c.line) {
                _ if !focused => None,
                0 => Some("CursorLine"),
                1 => Some("CursorLineGlow"),
                2 => Some("CursorLineGlow2"),
                _ => None,
            };
            match band.map(hl::hl).filter(|s| s.bg.is_some()) {
                Some(s) => {
                    let used: usize = spans.iter().map(|s| s.width()).sum();
                    spans.push(Span::raw(" ".repeat((a.width as usize).saturating_sub(used))));
                    out.push(Line::from(spans).style(s));
                }
                None => out.push(Line::from(spans)),
            }
        }
        f.render_widget(Paragraph::new(out), a);
        if focused && self.cmdline.is_none() && self.minibuf.is_none() && self.popup.is_none() {
            let x = a.x + gutter + (ccol - d.left) as u16;
            let y = a.y + (c.line - d.top) as u16;
            self.cursor_xy = (x, y);
            f.set_cursor_position(Position { x: x.min(a.right().saturating_sub(1)), y });
        }
    }

    fn draw_cmd(&mut self, f: &mut Frame, a: Rect) {
        if let Some(l) = &self.cmdline {
            let p = self.prompt.clone().unwrap_or_else(|| ":".into());
            f.render_widget(Paragraph::new(Line::from(vec![Span::styled(p.clone(), hl("Prompt")), Span::raw(l.as_str())])), a);
            f.set_cursor_position(Position { x: a.x + (p.chars().count() + l.chars().count()) as u16, y: a.y });
            return;
        }
        // a search in progress: "I-search: query", Emacs's echo area
        if let Some(s) = self.docs.get(self.cur).and_then(|d| d.ed.isearch()) {
            let p = s.prompt();
            let (label, query) = p.split_once(": ").unwrap_or((p.as_str(), ""));
            let style = if label.starts_with("Failing") { hl("Invalid") } else { hl("Prompt") };
            f.render_widget(Paragraph::new(Line::from(vec![Span::styled(format!("{label}: "), style), Span::raw(query.to_string())])), a);
            return;
        }
        if let Some((m, at)) = &self.msg {
            if at.elapsed() < Duration::from_secs(6) {
                f.render_widget(Paragraph::new(m.as_str()).style(hl("MsgArea")), a);
                return;
            }
            self.msg = None;
        }
        if !self.keyseq.is_empty() {
            let shown: Vec<String> = self.keyseq.iter().map(|k| if k == "<Space>" { "SPC".into() } else { k.clone() }).collect();
            f.render_widget(Paragraph::new(shown.join(" ")).style(hl("KeySeq")), a);
            return;
        }
        // the error on this line (as Doom shows it), else what a language server is doing
        if let Some((sev, m)) = self.line_diag() {
            f.render_widget(Paragraph::new(m).style(Style::default().fg(lsp::severity_color(sev))), a);
        } else if !self.lsp_status.is_empty() {
            f.render_widget(Paragraph::new(self.lsp_status.as_str()).style(hl("StatusLineDim")), a);
        }
    }

    fn draw_menu(&self, f: &mut Frame, body: Rect) {
        let n = self.menu.len();
        let cols = (body.width as usize / 28).clamp(1, 4);
        let rows = n.div_ceil(cols);
        let h = (rows as u16 + 2).min(body.height);
        let a = Rect { x: body.x, y: body.bottom().saturating_sub(h), width: body.width, height: h };
        f.render_widget(Clear, a);
        let title: Vec<String> = self.keyseq.iter().map(|k| if k == "<Space>" { "SPC".into() } else { k.clone() }).collect();
        let block = Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(hl("FloatBorderDim")).title(format!(" {} … ", title.join(" ")));
        let inner = block.inner(a);
        f.render_widget(block, a);
        let colw = inner.width as usize / cols;
        let mut lines: Vec<Line> = Vec::new();
        for r in 0..rows {
            let mut spans = Vec::new();
            for c in 0..cols {
                let Some(m) = self.menu.get(c * rows + r) else { continue };
                let key = if m.key == "<Space>" { "SPC".to_string() } else { m.key.clone() };
                let desc = if m.group { format!("+{}", m.desc) } else { m.desc.clone() };
                let cell = format!(" {key:>3}  {desc}");
                let cell: String = cell.chars().take(colw.saturating_sub(1)).collect();
                let pad = colw.saturating_sub(cell.chars().count());
                spans.push(Span::styled(format!(" {key:>3} "), hl("WhichKeyKey")));
                let d: String = format!(" {desc}").chars().take(colw.saturating_sub(6)).collect();
                let dpad = colw.saturating_sub(5 + d.chars().count());
                spans.push(Span::styled(d, if m.group { hl("WhichKeyGroup") } else { Style::default() }));
                spans.push(Span::raw(" ".repeat(dpad.min(pad + 5))));
            }
            lines.push(Line::from(spans));
        }
        f.render_widget(Paragraph::new(lines), inner);
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
        .border_type(BorderType::Rounded)
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
        _ => return None,
    };
    Some(Key { code, ctrl, alt })
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
