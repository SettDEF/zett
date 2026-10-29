//! Real terminals for the panel: a shell in a pty on zash's engine (alacritty_terminal); QML gets the screen
//! as one line of rich text per row, and sends keys here.
use alacritty_terminal::event::{Event, EventListener, WindowSize};
use alacritty_terminal::event_loop::{EventLoop, EventLoopSender, Msg};
use alacritty_terminal::grid::{Dimensions, Scroll};
use alacritty_terminal::index::{Column, Line};
use alacritty_terminal::sync::FairMutex;
use alacritty_terminal::term::cell::Flags;
use alacritty_terminal::term::{test::TermSize, Config, Term, TermMode};
use alacritty_terminal::tty;
use alacritty_terminal::vte::ansi::{Color, NamedColor, Rgb};
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

/// What the terminal tells us: redraws counted, replies sent back, the end noted.
#[derive(Clone)]
struct Listener {
    rev: Arc<AtomicU32>,
    dead: Arc<AtomicBool>,
    title: Arc<Mutex<String>>,
    tx: Arc<Mutex<Option<EventLoopSender>>>,
    size: Arc<Mutex<(u16, u16)>>,
}

impl EventListener for Listener {
    fn send_event(&self, e: Event) {
        let reply = |s: String| if let Some(tx) = self.tx.lock().unwrap().as_ref() { let _ = tx.send(Msg::Input(Cow::Owned(s.into_bytes()))); };
        match e {
            Event::Wakeup | Event::MouseCursorDirty => {}
            Event::Title(t) => *self.title.lock().unwrap() = t,
            Event::PtyWrite(s) => reply(s),
            Event::TextAreaSizeRequest(f) => { let (c, r) = *self.size.lock().unwrap(); reply(f(WindowSize { num_cols: c, num_lines: r, cell_width: 8, cell_height: 16 })) }
            Event::ChildExit(_) | Event::Exit => self.dead.store(true, Ordering::Relaxed),
            _ => {}
        }
        self.rev.fetch_add(1, Ordering::Relaxed);
    }
}

struct Session {
    term: Arc<FairMutex<Term<Listener>>>,
    tx: EventLoopSender,
    l: Listener,
    /// 16 colours, then foreground and background
    palette: Vec<String>,
}

thread_local! {
    static TERMS: RefCell<HashMap<i32, Session>> = RefCell::new(HashMap::new());
    static NEXT: RefCell<i32> = const { RefCell::new(1) };
}

const DEFAULT: [&str; 18] = ["#1d1f21", "#cc6666", "#b5bd68", "#f0c674", "#81a2be", "#b294bb", "#8abeb7", "#c5c8c6", "#666666", "#d54e53", "#b9ca4a", "#e7c547", "#7aa6da", "#c397d8", "#70c0b1", "#eaeaea", "#c5c8c6", "#1d1f21"];

fn ws(cols: u16, rows: u16) -> WindowSize { WindowSize { num_cols: cols, num_lines: rows, cell_width: 8, cell_height: 16 } }

/// A shell in `cwd`: its id, or -1.
pub fn start(cwd: &str, cols: u16, rows: u16, palette: &str) -> i32 {
    let (cols, rows) = (cols.max(2), rows.max(1));
    let l = Listener { rev: Arc::default(), dead: Arc::default(), title: Arc::default(), tx: Arc::default(), size: Arc::new(Mutex::new((cols, rows))) };
    let term = Arc::new(FairMutex::new(Term::new(Config { scrolling_history: 5000, ..Default::default() }, &TermSize::new(cols as usize, rows as usize), l.clone())));
    let mut env = HashMap::new();
    env.insert("TERM".into(), "xterm-256color".into());
    env.insert("COLORTERM".into(), "truecolor".into());
    env.insert("ZETT_TERMINAL".into(), "1".into());
    // the shell is in zett, not zash or zeit (it inherits their variables when zett was started there): plugins
    // that draw with zash's own escapes (zash-popup) would draw nothing here and keys like Enter go to that
    env.insert("TERM_PROGRAM".into(), "zett".into());
    for k in ["ZASH", "ZASH_PANE", "ZEIT", "ZEIT_PANE", "ZEIT_FX", "ZEIT_SOCKET", "ZEIT_CMD", "TERM_PROGRAM_VERSION"] { env.insert(k.into(), String::new()); }
    let opts = tty::Options { shell: user_shell().map(|s| tty::Shell::new(s, Vec::new())), working_directory: Some(cwd.into()).filter(|p: &std::path::PathBuf| p.is_dir()), drain_on_exit: true, env };
    let Ok(pty) = tty::new(&opts, ws(cols, rows), 0) else { return -1 };
    let Ok(ev) = EventLoop::new(term.clone(), l.clone(), pty, true, false) else { return -1 };
    let tx = ev.channel();
    *l.tx.lock().unwrap() = Some(tx.clone());
    ev.spawn();
    let pal = palette_for(palette);
    let id = NEXT.with(|n| { let mut n = n.borrow_mut(); *n += 1; *n - 1 });
    TERMS.with(|t| t.borrow_mut().insert(id, Session { term, tx, l, palette: pal }));
    id
}

pub fn write(id: i32, bytes: &[u8]) {
    TERMS.with(|t| if let Some(s) = t.borrow().get(&id) {
        // typing goes back to the bottom
        s.term.lock().scroll_display(Scroll::Bottom);
        let _ = s.tx.send(Msg::Input(Cow::Owned(bytes.to_vec())));
    });
}

/// A key from QML: Qt's key code, its text and modifiers (Qt's bits) as the bytes a terminal sends.
pub fn key(id: i32, key: i32, text: &str, mods: i32) {
    let app = TERMS.with(|t| t.borrow().get(&id).map(|s| s.term.lock().mode().contains(TermMode::APP_CURSOR))).unwrap_or(false);
    let ctrl = mods & 0x0400_0000 != 0;
    let alt = mods & 0x0800_0000 != 0;
    let shift = mods & 0x0200_0000 != 0;
    let csi = |c: char| if app { format!("\x1bO{c}") } else { format!("\x1b[{c}") };
    let tilde = |n: u8| format!("\x1b[{n}~");
    let s: String = match key {
        0x0100_0000 => "\x1b".into(),                   // Escape
        0x0100_0001 => if shift { "\x1b[Z".into() } else { "\t".into() },
        0x0100_0002 => "\x1b[Z".into(),                 // Backtab
        0x0100_0003 => if alt { "\x1b\x7f".into() } else { "\x7f".into() },
        0x0100_0004 | 0x0100_0005 => "\r".into(),       // Return, Enter
        0x0100_0006 => tilde(2),                        // Insert
        0x0100_0007 => tilde(3),                        // Delete
        0x0100_0010 => csi('H'),
        0x0100_0011 => csi('F'),
        0x0100_0012 => if ctrl { "\x1b[1;5D".into() } else { csi('D') },
        0x0100_0013 => csi('A'),
        0x0100_0014 => if ctrl { "\x1b[1;5C".into() } else { csi('C') },
        0x0100_0015 => csi('B'),
        0x0100_0016 => tilde(5),
        0x0100_0017 => tilde(6),
        k @ 0x0100_0030..=0x0100_003b => {               // F1–F12
            let n = (k - 0x0100_0030) as usize;
            ["\x1bOP", "\x1bOQ", "\x1bOR", "\x1bOS", "\x1b[15~", "\x1b[17~", "\x1b[18~", "\x1b[19~", "\x1b[20~", "\x1b[21~", "\x1b[23~", "\x1b[24~"][n].into()
        }
        _ if ctrl && (0x41..=0x5a).contains(&key) => ((key - 0x40) as u8 as char).to_string(),
        _ if ctrl && key == 0x20 => "\0".into(),
        _ if ctrl && key == 0x5b => "\x1b".into(),
        _ => if alt && !text.is_empty() { format!("\x1b{text}") } else { text.into() },
    };
    if !s.is_empty() { write(id, s.as_bytes()) }
}

pub fn resize(id: i32, cols: u16, rows: u16) {
    let (cols, rows) = (cols.max(2), rows.max(1));
    TERMS.with(|t| if let Some(s) = t.borrow().get(&id) {
        if *s.l.size.lock().unwrap() == (cols, rows) { return }
        *s.l.size.lock().unwrap() = (cols, rows);
        s.term.lock().resize(TermSize::new(cols as usize, rows as usize));
        let _ = s.tx.send(Msg::Resize(ws(cols, rows)));
    });
}

pub fn scroll(id: i32, lines: i32) {
    TERMS.with(|t| if let Some(s) = t.borrow().get(&id) { s.term.lock().scroll_display(Scroll::Delta(lines)); s.l.rev.fetch_add(1, Ordering::Relaxed); });
}

/// Changes so far (QML draws again when it moved); -1 when gone.
pub fn rev(id: i32) -> i32 {
    TERMS.with(|t| t.borrow().get(&id).map_or(-1, |s| s.l.rev.load(Ordering::Relaxed) as i32))
}

pub fn close(id: i32) {
    TERMS.with(|t| if let Some(s) = t.borrow_mut().remove(&id) { let _ = s.tx.send(Msg::Shutdown); });
}

fn hex(c: Rgb) -> String { format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b) }

fn indexed(i: u8, pal: &[String]) -> String {
    match i {
        0..=15 => pal[i as usize].clone(),
        16..=231 => { let i = i - 16; let v = |x: u8| if x == 0 { 0 } else { 55 + x * 40 }; hex(Rgb { r: v(i / 36), g: v(i / 6 % 6), b: v(i % 6) }) }
        _ => { let g = 8 + (i - 232) * 10; hex(Rgb { r: g, g, b: g }) }
    }
}

fn color(c: Color, pal: &[String], fg: bool, bold: bool) -> String {
    match c {
        Color::Spec(rgb) => hex(rgb),
        Color::Indexed(i) => indexed(if bold && i < 8 { i + 8 } else { i }, pal),
        Color::Named(n) => match n as usize {
            i @ 0..=15 => pal[if bold && i < 8 { i + 8 } else { i }].clone(),
            _ if n == NamedColor::Background => pal[17].clone(),
            _ if n == NamedColor::Foreground || n == NamedColor::BrightForeground => pal[16].clone(),
            _ => { let i = (n as usize).saturating_sub(NamedColor::DimBlack as usize).min(7); if fg { pal[i].clone() } else { pal[17].clone() } }
        },
    }
}

fn esc(c: char, out: &mut String) {
    match c { '<' => out.push_str("&lt;"), '>' => out.push_str("&gt;"), '&' => out.push_str("&amp;"), ' ' => out.push_str("&nbsp;"), _ => out.push(c) }
}

/// The screen: {rows: [html…], cursor: [row, col], show, title, dead, cols, rows, scrolled}
pub fn screen(id: i32) -> String {
    TERMS.with(|t| {
        let t = t.borrow();
        let Some(s) = t.get(&id) else { return "{}".into() };
        let term = s.term.lock();
        let g = term.grid();
        let (rows, cols) = (g.screen_lines(), g.columns());
        let off = g.display_offset() as i32;
        let pal = &s.palette;
        let mut out = Vec::with_capacity(rows);
        for r in 0..rows {
            let row = &g[Line(r as i32 - off)];
            let mut html = String::new();
            let mut cur: Option<(String, String, Flags)> = None;
            let mut run = String::new();
            let flush = |html: &mut String, cur: &Option<(String, String, Flags)>, run: &mut String| {
                if run.is_empty() { return }
                let (fg, bg, f) = cur.as_ref().unwrap();
                let mut st = format!("color:{fg};");
                if bg != &pal[17] { st += &format!("background-color:{bg};") }
                if f.contains(Flags::BOLD) { st += "font-weight:600;" }
                if f.contains(Flags::ITALIC) { st += "font-style:italic;" }
                if f.intersects(Flags::ALL_UNDERLINES) { st += "text-decoration:underline;" }
                if f.contains(Flags::STRIKEOUT) { st += "text-decoration:line-through;" }
                html.push_str(&format!("<span style=\"{st}\">{run}</span>"));
                run.clear();
            };
            // trailing blanks with the default background are left off
            let mut last = 0;
            for c in 0..cols { let cell = &row[Column(c)]; if (cell.c != ' ' && cell.c != '\0') || cell.bg != Color::Named(NamedColor::Background) { last = c + 1 } }
            for c in 0..last {
                let cell = &row[Column(c)];
                if cell.flags.contains(Flags::WIDE_CHAR_SPACER) { continue }
                let bold = cell.flags.contains(Flags::BOLD);
                let (mut fg, mut bg) = (color(cell.fg, pal, true, bold), color(cell.bg, pal, false, false));
                if cell.flags.contains(Flags::INVERSE) { std::mem::swap(&mut fg, &mut bg) }
                if cell.flags.contains(Flags::HIDDEN) { fg = bg.clone() }
                let key = (fg, bg, cell.flags & (Flags::BOLD | Flags::ITALIC | Flags::ALL_UNDERLINES | Flags::STRIKEOUT));
                if cur.as_ref() != Some(&key) { flush(&mut html, &cur, &mut run); cur = Some(key) }
                esc(if cell.c == '\0' { ' ' } else { cell.c }, &mut run);
            }
            flush(&mut html, &cur, &mut run);
            out.push(html);
        }
        let cur = term.grid().cursor.point;
        serde_json::json!({
            "rows": out, "cursor": [cur.line.0 + off, cur.column.0], "show": term.mode().contains(TermMode::SHOW_CURSOR) && off == 0,
            "title": s.l.title.lock().unwrap().clone(), "dead": s.l.dead.load(Ordering::Relaxed), "cols": cols, "lines": rows, "scrolled": off,
            "bg": pal[17], "fg": pal[16],
        }).to_string()
    })
}

/// zash's colours (tinct's ~/.config/wezterm/tinct_colors.lua: ansi, brights, foreground, background), then
/// what zett asks for on top: {"fg": "#…", "bg": "#…"} (the panel's own background).
fn palette_for(over: &str) -> Vec<String> {
    let mut p: Vec<String> = DEFAULT.iter().map(|s| s.to_string()).collect();
    let home = std::env::var_os("HOME").map(std::path::PathBuf::from).unwrap_or_default();
    if let Ok(src) = std::fs::read_to_string(home.join(".config/wezterm/tinct_colors.lua")) {
        let mut list: Option<(usize, usize)> = None;
        for raw in src.lines() {
            let line = raw.split("--").next().unwrap_or("").trim();
            let hex = line.split('"').nth(1).filter(|h| h.starts_with('#') && h.len() == 7).map(str::to_string);
            if let Some((base, i)) = list.as_mut() {
                if line.starts_with('}') { list = None; continue }
                if let Some(h) = hex { if *i < 8 { p[*base + *i] = h } *i += 1 }
                continue;
            }
            let Some((k, v)) = line.split_once('=') else { continue };
            match k.trim() {
                "ansi" if v.contains('{') => list = Some((0, 0)),
                "brights" if v.contains('{') => list = Some((8, 0)),
                "foreground" => if let Some(h) = hex { p[16] = h },
                "background" => if let Some(h) = hex { p[17] = h },
                _ => {}
            }
        }
    }
    let o: serde_json::Value = serde_json::from_str(over).unwrap_or_default();
    if let Some(s) = o["fg"].as_str() { p[16] = s.into() }
    if let Some(s) = o["bg"].as_str() { p[17] = s.into() }
    p
}

/// The shell zash starts (zash.toml's `shell`), else $SHELL: not the login shell from passwd (bash here).
fn user_shell() -> Option<String> {
    let home = std::env::var("HOME").unwrap_or_default();
    let toml = std::fs::read_to_string(format!("{home}/.config/zash/zash.toml")).unwrap_or_default();
    let from_toml = toml.lines().map(str::trim).find(|l| l.starts_with("shell") && l.contains('=')).and_then(|l| l.split('"').nth(1)).map(str::to_string);
    from_toml.or_else(|| std::env::var("SHELL").ok()).filter(|s| std::path::Path::new(s).exists())
}
