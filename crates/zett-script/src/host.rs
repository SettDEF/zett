//! What plugins build on: side panels, the current buffer (with its undo tree), files, prompts.

use super::{Effect, Engine};
use mlua::{Function, Lua, RegistryKey, Table, Value};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use zett_edit::Editor;

/// A panel a plugin keeps at the side of the editor: styled lines and a cursor.
pub(crate) struct Panel {
    id: String,
    side: String,
    width: u16,
    height: u16,
    title: String,
    /// each line: (text, highlight group) pieces
    lines: Vec<Vec<(String, String)>>,
    cursor: usize,
    /// a floating panel moved from the middle (cells)
    dx: i16,
    dy: i16,
    /// a float placed as Neovim places one: (relative to "editor" | "cursor" | "win", row, col);
    /// width and height are then in cells
    at: Option<(String, i32, i32)>,
    on_key: Option<RegistryKey>,
    on_move: Option<RegistryKey>,
    on_close: Option<RegistryKey>,
}

/// A panel as the host draws it.
#[derive(Clone, Debug)]
pub struct PanelView {
    pub id: String,
    /// "left" or "right"
    pub side: String,
    pub width: u16,
    /// 0: share what is left of the side
    pub height: u16,
    pub title: String,
    pub lines: Vec<Vec<(String, String)>>,
    pub cursor: usize,
    pub focused: bool,
    /// side "float": moved from the middle by this many cells
    pub dx: i16,
    pub dy: i16,
    /// side "float" placed at (relative to, row, col), its size in cells (nvim_open_win)
    pub at: Option<(String, i32, i32)>,
}

#[derive(Default)]
pub(crate) struct Host {
    pub panels: Vec<Panel>,
    pub focus: Option<String>,
    /// the buffer lent for one call, its path and the project folder
    pub buf: Option<Rc<RefCell<Editor>>>,
    pub path: String,
    pub root: PathBuf,
    /// a question for the user: (prompt, default, what gets the answer)
    pub input: Option<(String, String, RegistryKey)>,
    pub asking: Option<RegistryKey>,
    /// the open buffers' paths (zett.buffers; "" for one without a file), in their order
    pub buffers: Vec<String>,
    /// the current buffer's number in that list (1-based; 0 not known)
    pub cur_buffer: usize,
    /// the windows: (number, buffer number, width, height, row, col), and the focused one's number
    pub wins: Vec<(usize, usize, u16, u16, u16, u16)>,
    pub cur_win: usize,
    /// the quickfix list in use: (title, the item last gone to, items as (file, line, col, kind, nr, text, valid))
    pub qf: (String, usize, Vec<(String, usize, usize, char, usize, String, bool)>),
    /// the : line being typed (its text, the cursor in it) and the command history
    pub cmdline: Option<(String, usize)>,
    /// each window's first line shown (0-based), in the order of wins
    pub tops: Vec<usize>,
    /// the current buffer's own settings (filetype, commentstring …): vim.bo / vim.o read them first
    pub locals: std::collections::HashMap<String, String>,
    /// the other open buffers (by place; None: the current one, lent as zett.buf), read by vim.api for a buffer not current
    pub others: Vec<Option<zett_core::Buffer>>,
    /// buffers (numbers) Lua wrote lines to while they were not current: the editor applies them after
    pub other_writes: Vec<usize>,
    /// a plugin watches keys (vim.on_key): the editor tells Lua each one
    pub on_key: bool,
    /// keys waiting to be read by getchar (a test, or keys typed ahead), then the source that waits for one
    pub pending_keys: std::collections::VecDeque<String>,
    pub key_source: Option<Box<dyn FnMut() -> Option<String>>>,
    pub hist: Vec<String>,
    /// a g waiting for its second g
    pub g: bool,
    /// floating boxes in a corner (zett.float): id, corner, lines
    pub floats: Vec<(String, String, Vec<Vec<(String, String)>>)>,
    /// zett.defer: when, and what runs then
    pub timers: Vec<(std::time::Instant, RegistryKey)>,
    /// processes started by zett.job.spawn (vim.system): each one's result arrives on its channel
    pub jobs: Vec<Job>,
    /// processes started by zett.job.stream (jobstart): output arrives in pieces while they run
    pub streams: Vec<Stream>,
    /// every option the editor has: name, short names, kind (bool, int, text), default (zett.option_info)
    pub options: Vec<(String, Vec<String>, String, String)>,
}

/// A Nerd Font icon for a file, by its extension.
pub fn file_icon(name: &str) -> &'static str {
    let ext = Path::new(name).extension().and_then(|e| e.to_str()).unwrap_or("");
    match ext {
        "rs" => "\u{e7a8}",
        "lua" => "\u{e620}",
        "py" => "\u{e606}",
        "md" => "\u{f48a}",
        "toml" => "\u{e6b2}",
        "js" | "mjs" => "\u{e74e}",
        "ts" | "tsx" => "\u{e628}",
        "json" => "\u{e60b}",
        "sh" | "zsh" | "bash" | "fish" => "\u{f489}",
        "c" | "h" => "\u{e61e}",
        "cpp" | "hpp" | "cc" => "\u{e61d}",
        "html" => "\u{e736}",
        "css" | "scss" => "\u{e749}",
        "nix" => "\u{f313}",
        "go" => "\u{e627}",
        "qml" => "\u{f0ac}",
        "conf" | "ini" | "cfg" => "\u{e615}",
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" => "\u{f1c5}",
        "lock" => "\u{f023}",
        _ => "\u{f15b}",
    }
}

/// Lines of `a` and `b` as (op, text): ' ' kept, '-' only in a, '+' only in b.
pub fn diff_lines(a: &str, b: &str) -> Vec<(char, String)> {
    let a: Vec<&str> = a.lines().collect();
    let b: Vec<&str> = b.lines().collect();
    let pre = a.iter().zip(&b).take_while(|(x, y)| x == y).count();
    let suf = a[pre..].iter().rev().zip(b[pre..].iter().rev()).take_while(|(x, y)| x == y).count();
    let (am, bm) = (&a[pre..a.len() - suf], &b[pre..b.len() - suf]);
    let mut out: Vec<(char, String)> = a[..pre].iter().map(|l| (' ', l.to_string())).collect();
    if am.len() * bm.len() > 4_000_000 {
        // too big to align: all out, all in
        out.extend(am.iter().map(|l| ('-', l.to_string())));
        out.extend(bm.iter().map(|l| ('+', l.to_string())));
    } else {
        // longest common subsequence, walked from the start
        let (n, m) = (am.len(), bm.len());
        let mut t = vec![0u32; (n + 1) * (m + 1)];
        for i in (0..n).rev() {
            for j in (0..m).rev() {
                t[i * (m + 1) + j] = if am[i] == bm[j] { t[(i + 1) * (m + 1) + j + 1] + 1 } else { t[(i + 1) * (m + 1) + j].max(t[i * (m + 1) + j + 1]) };
            }
        }
        let (mut i, mut j) = (0, 0);
        while i < n || j < m {
            if i < n && j < m && am[i] == bm[j] {
                out.push((' ', am[i].to_string()));
                i += 1;
                j += 1;
            } else if i < n && (j == m || t[(i + 1) * (m + 1) + j] >= t[i * (m + 1) + j + 1]) {
                // what went comes before what came, as diff shows it
                out.push(('-', am[i].to_string()));
                i += 1;
            } else {
                out.push(('+', bm[j].to_string()));
                j += 1;
            }
        }
    }
    out.extend(a[a.len() - suf..].iter().map(|l| (' ', l.to_string())));
    out
}

fn copy_all(a: &Path, b: &Path) -> std::io::Result<()> {
    if a.is_dir() {
        std::fs::create_dir_all(b)?;
        for e in std::fs::read_dir(a)?.flatten() {
            copy_all(&e.path(), &b.join(e.file_name()))?;
        }
        Ok(())
    } else {
        if let Some(p) = b.parent() {
            std::fs::create_dir_all(p)?;
        }
        std::fs::copy(a, b).map(|_| ())
    }
}

fn lines_of(v: Value) -> Vec<Vec<(String, String)>> {
    let Value::Table(t) = v else { return Vec::new() };
    t.sequence_values::<Value>()
        .flatten()
        .map(|l| match l {
            Value::String(s) => vec![(s.to_string_lossy(), String::new())],
            Value::Table(parts) => parts
                .sequence_values::<Value>()
                .flatten()
                .filter_map(|p| match p {
                    Value::String(s) => Some((s.to_string_lossy(), String::new())),
                    Value::Table(p) => Some((p.get::<String>(1).unwrap_or_default(), p.get::<String>(2).unwrap_or_default())),
                    _ => None,
                })
                .collect(),
            _ => Vec::new(),
        })
        .collect()
}

/// Calls a stored function; an error becomes a message.
fn call(lua: &Lua, k: &Option<RegistryKey>, args: impl mlua::IntoLuaMulti) -> Result<Value, String> {
    let Some(k) = k else { return Ok(Value::Nil) };
    let f: Function = lua.registry_value(k).map_err(|e| e.to_string())?;
    f.call::<Value>(args).map_err(|e| e.to_string())
}

impl Engine {
    /// Lends the current buffer to scripts for one call (`zett.buf`), with its path and project.
    pub fn lend_buffer<R>(&mut self, ed: &mut Editor, path: &str, root: &Path, f: impl FnOnce(&mut Engine) -> R) -> R {
        let rc = Rc::new(RefCell::new(std::mem::replace(ed, Editor::from_str(""))));
        {
            let mut st = self.st.borrow_mut();
            st.host.buf = Some(rc.clone());
            st.host.path = path.to_string();
            st.host.root = root.to_path_buf();
        }
        let r = f(self);
        self.st.borrow_mut().host.buf = None;
        *ed = match Rc::try_unwrap(rc) {
            Ok(c) => c.into_inner(),
            Err(_) => unreachable!("scripts never keep the buffer"),
        };
        r
    }

    /// The open buffers' paths, for `zett.buffers()`.
    /// The editor's options, for vim.o and zett.option_info: (name, short names, "bool" | "int" | "text", default).
    pub fn set_options(&mut self, options: Vec<(String, Vec<String>, String, String)>) {
        self.st.borrow_mut().host.options = options;
    }

    pub fn set_buffers(&mut self, paths: Vec<String>) {
        self.st.borrow_mut().host.buffers = paths;
    }

    /// The : line being typed (None: none) and the command history (getcmdline, histget).
    pub fn set_cmdline(&mut self, line: Option<(String, usize)>, hist: Option<Vec<String>>) {
        let mut st = self.st.borrow_mut();
        st.host.cmdline = line;
        if let Some(h) = hist {
            st.host.hist = h;
        }
    }

    /// The first line each window shows (line("w0"), winline, screenpos).
    pub fn set_tops(&mut self, tops: Vec<usize>) {
        self.st.borrow_mut().host.tops = tops;
    }

    /// The current buffer's own settings, for vim.bo (as `:setlocal` and the file type set them).
    pub fn set_locals(&mut self, locals: std::collections::HashMap<String, String>) {
        self.st.borrow_mut().host.locals = locals;
    }

    /// The quickfix list in use (getqflist), given when it changed.
    pub fn set_qf(&mut self, title: String, idx: usize, items: Vec<(String, usize, usize, char, usize, String, bool)>) {
        self.st.borrow_mut().host.qf = (title, idx, items);
    }

    /// The current buffer's number (in set_buffers' list) and the windows (see Host::wins).
    pub fn set_layout(&mut self, cur_buffer: usize, wins: Vec<(usize, usize, u16, u16, u16, u16)>, cur_win: usize) {
        let mut st = self.st.borrow_mut();
        st.host.cur_buffer = cur_buffer;
        st.host.wins = wins;
        st.host.cur_win = cur_win;
    }

    pub fn panels(&self) -> Vec<PanelView> {
        let st = self.st.borrow();
        st.host
            .panels
            .iter()
            .map(|p| PanelView { id: p.id.clone(), side: p.side.clone(), width: p.width, height: p.height, title: p.title.clone(), lines: p.lines.clone(), cursor: p.cursor, focused: st.host.focus.as_deref() == Some(&p.id), dx: p.dx, dy: p.dy, at: p.at.clone() })
            .collect()
    }

    /// The floating boxes: (id, corner "SE" | "NE" | "SW" | "NW", lines).
    pub fn floats(&self) -> Vec<(String, String, Vec<Vec<(String, String)>>)> {
        self.st.borrow().host.floats.clone()
    }

    /// When the next zett.defer is due.
    pub fn next_timer(&self) -> Option<std::time::Instant> {
        let st = self.st.borrow();
        let t = st.host.timers.iter().map(|t| t.0).min();
        // a running job (vim.system) is checked every 5 ms for its end
        if st.host.jobs.is_empty() && st.host.streams.is_empty() { t } else { Some(t.unwrap_or_else(std::time::Instant::now).min(std::time::Instant::now() + std::time::Duration::from_millis(5))) }
    }

    /// Runs the zett.defer functions that are due.
    pub fn run_timers(&mut self) -> Vec<Effect> {
        let mut errs = Vec::new();
        poll_jobs(&self.lua, &self.st, &mut errs);
        self.errors.extend(errs);
        let now = std::time::Instant::now();
        let due: Vec<RegistryKey> = {
            let mut st = self.st.borrow_mut();
            let (due, keep): (Vec<_>, Vec<_>) = std::mem::take(&mut st.host.timers).into_iter().partition(|t| t.0 <= now);
            st.host.timers = keep;
            due.into_iter().map(|t| t.1).collect()
        };
        for k in due {
            if let Err(e) = call(&self.lua, &Some(k), ()) {
                self.errors.push(format!("timer: {e}"));
            }
        }
        self.finish()
    }

    pub fn focused_panel(&self) -> Option<String> {
        self.st.borrow().host.focus.clone()
    }

    pub fn focus_panel(&mut self, id: Option<&str>) {
        let mut st = self.st.borrow_mut();
        st.host.focus = id.filter(|i| st.host.panels.iter().any(|p| p.id == *i)).map(String::from);
    }

    fn set_panel_cursor(&mut self, id: &str, row: usize) -> Vec<Effect> {
        let moved = {
            let mut st = self.st.borrow_mut();
            let Some(p) = st.host.panels.iter_mut().find(|p| p.id == id) else { return Vec::new() };
            let row = row.min(p.lines.len().saturating_sub(1));
            let moved = p.cursor != row;
            p.cursor = row;
            moved.then(|| p.on_move.as_ref().map(|k| self.lua.registry_value::<Function>(k)))
        };
        if let Some(Some(Ok(f))) = moved {
            if let Err(e) = f.call::<()>(row + 1) {
                self.errors.push(format!("{id}: {e}"));
            }
        }
        self.finish()
    }

    /// A key in the focused panel: its plugin first, then moving (j k gg G C-d C-u), Esc leaves,
    /// q closes. None: no panel has the keyboard, or the key means nothing there.
    pub fn panel_key(&mut self, key: &str, page: usize) -> Option<Vec<Effect>> {
        let id = self.focused_panel()?;
        let key = if key == "<lt>" { "<".to_string() } else { super::keys(key).concat() };
        let key = key.as_str();
        let (handler, row, len) = {
            let st = self.st.borrow();
            let p = st.host.panels.iter().find(|p| p.id == id)?;
            (p.on_key.as_ref().and_then(|k| self.lua.registry_value::<Function>(k).ok()), p.cursor, p.lines.len())
        };
        if let Some(f) = handler {
            match f.call::<Value>((key, row + 1)) {
                Ok(Value::Boolean(true)) => return Some(self.finish()),
                Ok(_) => {}
                Err(e) => {
                    let mut fx = self.finish();
                    fx.push(Effect::Notify(format!("{id}: {e}")));
                    return Some(fx);
                }
            }
        }
        // a floating panel moves with Alt+h j k l
        let float = self.st.borrow().host.panels.iter().any(|p| p.id == id && p.side == "float");
        if float {
            let d = match key {
                "<A-h>" => Some((-4, 0)),
                "<A-l>" => Some((4, 0)),
                "<A-k>" => Some((0, -2)),
                "<A-j>" => Some((0, 2)),
                _ => None,
            };
            if let Some((dx, dy)) = d {
                if let Some(p) = self.st.borrow_mut().host.panels.iter_mut().find(|p| p.id == id) {
                    p.dx = (p.dx + dx).clamp(-200, 200);
                    p.dy = (p.dy + dy).clamp(-100, 100);
                }
                return Some(Vec::new());
            }
        }
        let last = len.saturating_sub(1);
        let g = std::mem::take(&mut self.st.borrow_mut().host.g);
        let to = match key {
            "g" if g => 0,
            "g" => {
                self.st.borrow_mut().host.g = true;
                return Some(Vec::new());
            }
            "j" | "<Down>" => row + 1,
            "k" | "<Up>" => row.saturating_sub(1),
            "G" => last,
            "<C-d>" => row + page / 2,
            "<C-u>" => row.saturating_sub(page / 2),
            "<Esc>" | "<Tab>" | "<C-l>" => {
                self.focus_panel(None);
                return Some(self.finish());
            }
            "q" => {
                self.close_panel(&id);
                return Some(self.finish());
            }
            _ => return None,
        };
        Some(self.set_panel_cursor(&id, to.min(last)))
    }

    /// A click on a panel's line: it gets the keyboard and the cursor.
    pub fn panel_click(&mut self, id: &str, row: usize) -> Vec<Effect> {
        self.focus_panel(Some(id));
        self.set_panel_cursor(id, row)
    }

    pub fn close_panel(&mut self, id: &str) {
        let p = {
            let mut st = self.st.borrow_mut();
            if st.host.focus.as_deref() == Some(id) {
                st.host.focus = None;
            }
            let i = st.host.panels.iter().position(|p| p.id == id);
            i.map(|i| st.host.panels.remove(i))
        };
        if let Some(p) = p {
            if let Err(e) = call(&self.lua, &p.on_close, ()) {
                self.errors.push(format!("{id}: {e}"));
            }
        }
    }

    /// A question a script asked (`zett.input`): (prompt, default), for the host's input line.
    pub fn take_input(&mut self) -> Option<(String, String)> {
        let mut st = self.st.borrow_mut();
        let (prompt, default, k) = st.host.input.take()?;
        st.host.asking = Some(k);
        Some((prompt, default))
    }

    /// The answer (None: cancelled, the script hears nothing).
    pub fn input_done(&mut self, answer: Option<String>) -> Vec<Effect> {
        let k = self.st.borrow_mut().host.asking.take();
        if let Some(a) = answer {
            if let Err(e) = call(&self.lua, &k, a) {
                self.errors.push(e);
            }
        }
        self.finish()
    }

    pub fn has_hooks(&self, event: &str) -> bool {
        self.st.borrow().hooks.get(event).is_some_and(|v| !v.is_empty())
    }

    /// Hooks with `{ path = … }` as their data (`buf_enter`, `write`, `text_changed`).
    /// Hooks with a table of text fields as their data (`lsp_progress`, `notify`).
    pub fn emit_fields(&mut self, event: &str, fields: &[(&str, String)]) -> Vec<Effect> {
        let data = self.lua.create_table().ok().map(|t| {
            for (k, v) in fields {
                let _ = t.set(*k, v.as_str());
            }
            Value::Table(t)
        });
        self.emit(event, data.unwrap_or(Value::Nil))
    }

    pub fn emit_path(&mut self, event: &str, path: &str) -> Vec<Effect> {
        let data = self.lua.create_table().ok().map(|t| {
            let _ = t.set("path", path);
            Value::Table(t)
        });
        self.emit(event, data.unwrap_or(Value::Nil))
    }

    /// Every setting `:set` knows: (name, current value, default, a line about it).
    pub fn options(&self) -> Vec<(String, String, String, String)> {
        let mut out = vec![];
        let Ok(z) = self.lua.globals().get::<Table>("zett") else { return out };
        let opt: Table = z.get("opt").unwrap_or_else(|_| self.lua.create_table().unwrap());
        out.push(("leader".into(), opt.get::<Value>("leader").map(|v| super::show(&v)).unwrap_or_default(), "\" \"".into(), "the leader key".into()));
        let Ok(reg) = z.get::<Table>("_options") else { return out };
        for (name, spec) in reg.pairs::<String, Table>().flatten() {
            let Ok(defaults) = spec.get::<Table>("defaults") else { continue };
            let about: Option<Table> = spec.get("about").ok();
            let user: Option<Table> = opt.get(name.as_str()).ok();
            for (k, d) in defaults.pairs::<String, Value>().flatten() {
                if k == "keys" {
                    continue;
                }
                let cur = user.as_ref().and_then(|u| u.get::<Value>(k.as_str()).ok()).filter(|v| !v.is_nil()).unwrap_or(d.clone());
                let line = about.as_ref().and_then(|a| a.get::<String>(k.as_str()).ok()).unwrap_or_default();
                out.push((format!("{name}.{k}"), super::show(&cur), super::show(&d), line));
            }
        }
        out.sort();
        out
    }

    /// `:set explorer.width 40`: the value takes its default's type. Returns the line to show.
    pub fn set_option(&mut self, name: &str, value: &str) -> Result<String, String> {
        let known = self.options().into_iter().find(|o| o.0 == name).ok_or_else(|| format!("no setting {name} (:set lists them)"))?;
        let v = value.trim();
        let parsed = if known.2 == "true" || known.2 == "false" {
            match v {
                "true" | "on" | "yes" | "1" | "" => Value::Boolean(true),
                "false" | "off" | "no" | "0" => Value::Boolean(false),
                _ => return Err(format!("{name} is on or off (true / false)")),
            }
        } else if known.2.parse::<f64>().is_ok() {
            match v.parse::<i64>() {
                Ok(i) => Value::Integer(i),
                Err(_) => Value::Number(v.parse::<f64>().map_err(|_| format!("{name} is a number"))?),
            }
        } else if known.2.starts_with('{') {
            return Err(format!("{name} is a list: set it in init.lua or with :lua"));
        } else {
            Value::String(self.lua.create_string(v.trim_matches('"')).map_err(|e| e.to_string())?)
        };
        let shown = super::show(&parsed);
        let opt: Table = self.lua.globals().get::<Table>("zett").and_then(|z| z.get("opt")).map_err(|e| e.to_string())?;
        match name.split_once('.') {
            Some((plugin, key)) => {
                let t = match opt.get::<Value>(plugin) {
                    Ok(Value::Table(t)) => t,
                    _ => {
                        let t = self.lua.create_table().map_err(|e| e.to_string())?;
                        opt.set(plugin, t.clone()).map_err(|e| e.to_string())?;
                        t
                    }
                };
                t.set(key, parsed).map_err(|e| e.to_string())?;
                Ok(format!("{name} = {shown} (for good: zett.opt.{plugin} = {{ {key} = {} }} in init.lua)", if matches!(t.get::<Value>(key), Ok(Value::String(_))) { format!("\"{shown}\"") } else { shown.clone() }))
            }
            None => {
                opt.set(name, parsed).map_err(|e| e.to_string())?;
                if name == "leader" {
                    self.leader_from_opts();
                }
                Ok(format!("{name} = {shown}"))
            }
        }
    }

    pub(crate) fn install_host(&self, z: &Table) -> mlua::Result<()> {
        let lua = &self.lua;

        // ── zett.panel ──
        let panel = lua.create_table()?;
        // zett.panel.open{ id=, side="left", width=30, height=0, title="", focus=true,
        //   on_key=function(key, row) return true end, on_move=function(row) end, on_close=function() end }
        let st = self.st.clone();
        panel.set("open", lua.create_function(move |lua, o: Table| {
            let id: String = o.get("id")?;
            let key = |name: &str| -> mlua::Result<Option<RegistryKey>> { o.get::<Option<Function>>(name)?.map(|f| lua.create_registry_value(f)).transpose() };
            let (on_key, on_move, on_close) = (key("on_key")?, key("on_move")?, key("on_close")?);
            let mut s = st.borrow_mut();
            let h = &mut s.host;
            let i = match h.panels.iter().position(|p| p.id == id) {
                Some(i) => i,
                None => {
                    h.panels.push(Panel { id: id.clone(), side: "left".into(), width: 30, height: 0, title: String::new(), lines: Vec::new(), cursor: 0, dx: 0, dy: 0, at: None, on_key: None, on_move: None, on_close: None });
                    h.panels.len() - 1
                }
            };
            let p = &mut h.panels[i];
            if let Some(v) = o.get::<Option<String>>("side")? {
                p.side = if v == "right" || v == "float" { v } else { "left".into() };
            }
            p.width = o.get::<Option<u16>>("width")?.unwrap_or(p.width).clamp(1, 300);
            if p.side == "float" && o.get::<Option<u16>>("height")?.is_none() && p.height == 0 {
                p.height = 70;
            }
            p.height = o.get::<Option<u16>>("height")?.unwrap_or(p.height);
            p.title = o.get::<Option<String>>("title")?.unwrap_or(p.title.clone());
            // relative = "editor" | "cursor" | "win", row =, col =: a float where Neovim's would be
            if let Some(rel) = o.get::<Option<String>>("relative")?.filter(|r| !r.is_empty()) {
                p.at = Some((rel, o.get::<Option<i32>>("row")?.unwrap_or(0), o.get::<Option<i32>>("col")?.unwrap_or(0)));
            }
            p.on_key = on_key.or(p.on_key.take());
            p.on_move = on_move.or(p.on_move.take());
            p.on_close = on_close.or(p.on_close.take());
            if o.get::<Option<bool>>("focus")?.unwrap_or(true) {
                h.focus = Some(id);
            }
            Ok(())
        })?)?;
        // zett.panel.set_lines(id, { "text", { {"text", "Group"}, … }, … })
        let st = self.st.clone();
        panel.set("set_lines", lua.create_function(move |_, (id, lines): (String, Value)| {
            if let Some(p) = st.borrow_mut().host.panels.iter_mut().find(|p| p.id == id) {
                p.lines = lines_of(lines);
                p.cursor = p.cursor.min(p.lines.len().saturating_sub(1));
            }
            Ok(())
        })?)?;
        let st = self.st.clone();
        panel.set("cursor", lua.create_function(move |_, id: String| Ok(st.borrow().host.panels.iter().find(|p| p.id == id).map(|p| p.cursor + 1)))?)?;
        let st = self.st.clone();
        panel.set("set_cursor", lua.create_function(move |_, (id, row): (String, usize)| {
            if let Some(p) = st.borrow_mut().host.panels.iter_mut().find(|p| p.id == id) {
                p.cursor = row.saturating_sub(1).min(p.lines.len().saturating_sub(1));
            }
            Ok(())
        })?)?;
        let st = self.st.clone();
        panel.set("is_open", lua.create_function(move |_, id: String| Ok(st.borrow().host.panels.iter().any(|p| p.id == id)))?)?;
        let st = self.st.clone();
        panel.set("focused", lua.create_function(move |_, ()| Ok(st.borrow().host.focus.clone()))?)?;
        let st = self.st.clone();
        panel.set("focus", lua.create_function(move |_, id: Option<String>| {
            let mut s = st.borrow_mut();
            s.host.focus = id.filter(|i| s.host.panels.iter().any(|p| p.id == *i));
            Ok(())
        })?)?;
        // zett.panel.move(id, dx, dy): a floating panel, moved by cells
        let st = self.st.clone();
        panel.set("move", lua.create_function(move |_, (id, dx, dy): (String, i16, i16)| {
            if let Some(p) = st.borrow_mut().host.panels.iter_mut().find(|p| p.id == id) {
                p.dx = p.dx.saturating_add(dx).clamp(-200, 200);
                p.dy = p.dy.saturating_add(dy).clamp(-100, 100);
            }
            Ok(())
        })?)?;
        // zett.panel.close(id): its on_close runs
        let st = self.st.clone();
        panel.set("close", lua.create_function(move |lua, id: String| {
            let p = {
                let mut s = st.borrow_mut();
                if s.host.focus.as_deref() == Some(&id) {
                    s.host.focus = None;
                }
                let i = s.host.panels.iter().position(|p| p.id == id);
                i.map(|i| s.host.panels.remove(i))
            };
            if let Some(p) = p {
                call(lua, &p.on_close, ()).map_err(mlua::Error::runtime)?;
            }
            Ok(())
        })?)?;
        z.set("panel", panel)?;

        // ── zett.float: a box in a corner, over the windows (no keyboard) ──
        let float = lua.create_table()?;
        // zett.float.open{ id=, anchor="SE", lines={…} }: made, or its lines and corner replaced
        let st = self.st.clone();
        float.set("open", lua.create_function(move |_, o: Table| {
            let id: String = o.get("id")?;
            let anchor = o.get::<Option<String>>("anchor")?.unwrap_or_else(|| "SE".into()).to_uppercase();
            let lines = lines_of(o.get::<Value>("lines")?);
            let mut s = st.borrow_mut();
            match s.host.floats.iter_mut().find(|f| f.0 == id) {
                Some(f) => {
                    f.1 = anchor;
                    f.2 = lines;
                }
                None => s.host.floats.push((id, anchor, lines)),
            }
            Ok(())
        })?)?;
        let st = self.st.clone();
        float.set("close", lua.create_function(move |_, id: String| {
            st.borrow_mut().host.floats.retain(|f| f.0 != id);
            Ok(())
        })?)?;
        let st = self.st.clone();
        float.set("is_open", lua.create_function(move |_, id: String| Ok(st.borrow().host.floats.iter().any(|f| f.0 == id)))?)?;
        z.set("float", float)?;
        // zett.defer(ms, function): runs once, after ms milliseconds
        let st = self.st.clone();
        z.set("defer", lua.create_function(move |lua, (ms, f): (u64, Function)| {
            let k = lua.create_registry_value(f)?;
            st.borrow_mut().host.timers.push((std::time::Instant::now() + std::time::Duration::from_millis(ms), k));
            Ok(())
        })?)?;

        // ── zett.job: processes that run beside the editor (vim.system is built on it) ──
        let job = lua.create_table()?;
        // spawn(argv, { cwd, env = {K = V}, clear_env, stdin = "text" }, on_exit(result)) → id, pid
        let st = self.st.clone();
        let next = Rc::new(std::cell::Cell::new(0u64));
        let next2 = next.clone();
        job.set("spawn", lua.create_function(move |lua, (argv, o, cb): (Vec<String>, Option<mlua::Table>, Option<Function>)| {
            let get = |k: &str| o.as_ref().and_then(|t| t.get::<Option<String>>(k).ok().flatten());
            let env: Option<Vec<(String, String)>> = o.as_ref().and_then(|t| t.get::<Option<mlua::Table>>("env").ok().flatten()).map(|t| t.pairs::<String, String>().flatten().collect());
            let clear = o.as_ref().and_then(|t| t.get::<Option<bool>>("clear_env").ok().flatten()).unwrap_or(false);
            let (pid, rx) = start_job(&argv, get("cwd"), env, clear, get("stdin")).map_err(|e| mlua::Error::runtime(format!("{}: {e}", argv.first().cloned().unwrap_or_default())))?;
            let id = next.get() + 1;
            next.set(id);
            let cb = cb.map(|f| lua.create_registry_value(f)).transpose()?;
            st.borrow_mut().host.jobs.push(Job { id, pid, rx, cb });
            Ok((id, pid))
        })?)?;
        // wait(id, ms): blocks until the job ends (its on_exit runs first), its result; nil on time out
        let st = self.st.clone();
        job.set("wait", lua.create_function(move |lua, (id, ms): (u64, Option<u64>)| {
            let deadline = std::time::Instant::now() + std::time::Duration::from_millis(ms.unwrap_or(u64::MAX / 4).min(24 * 3600 * 1000));
            loop {
                let got = {
                    let mut s = st.borrow_mut();
                    let Some(i) = s.host.jobs.iter().position(|j| j.id == id) else { return Ok(None) };
                    match s.host.jobs[i].rx.recv_timeout(std::time::Duration::from_millis(5)) {
                        Ok(out) => Some((s.host.jobs.remove(i).cb, out)),
                        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => None,
                        Err(_) => {
                            s.host.jobs.remove(i);
                            return Ok(None);
                        }
                    }
                };
                if let Some((cb, out)) = got {
                    let t = out.table(lua)?;
                    if let Some(k) = cb {
                        call(lua, &Some(k), t.clone()).map_err(mlua::Error::runtime)?;
                    }
                    return Ok(Some(t));
                }
                if std::time::Instant::now() >= deadline {
                    return Ok(None);
                }
            }
        })?)?;
        // stream(argv, { cwd, env, clear_env }, cb(kind, data)): a job whose output comes in pieces (jobstart);
        // cb("stdout"|"stderr", text) while it runs, cb("exit", code) at the end → id, pid
        let st = self.st.clone();
        job.set("stream", lua.create_function(move |lua, (argv, o, cb): (Vec<String>, Option<mlua::Table>, Function)| {
            let get = |k: &str| o.as_ref().and_then(|t| t.get::<Option<String>>(k).ok().flatten());
            let env: Option<Vec<(String, String)>> = o.as_ref().and_then(|t| t.get::<Option<mlua::Table>>("env").ok().flatten()).map(|t| t.pairs::<String, String>().flatten().collect());
            let clear = o.as_ref().and_then(|t| t.get::<Option<bool>>("clear_env").ok().flatten()).unwrap_or(false);
            let (pid, stdin, rx) = start_stream(&argv, get("cwd"), env, clear).map_err(|e| mlua::Error::runtime(format!("{}: {e}", argv.first().cloned().unwrap_or_default())))?;
            let id = next2.get() + 1;
            next2.set(id);
            let cb = lua.create_registry_value(cb)?;
            st.borrow_mut().host.streams.push(Stream { id, pid, stdin: Some(stdin), rx, cb });
            Ok((id, pid))
        })?)?;
        // send(id, text): to a streamed job's stdin; false when it is closed or gone
        let st = self.st.clone();
        job.set("send", lua.create_function(move |_, (id, data): (u64, mlua::String)| {
            use std::io::Write;
            let mut s = st.borrow_mut();
            let Some(j) = s.host.streams.iter_mut().find(|j| j.id == id) else { return Ok(false) };
            Ok(j.stdin.as_mut().is_some_and(|w| w.write_all(&data.as_bytes()).and_then(|_| w.flush()).is_ok()))
        })?)?;
        // close_stdin(id): the program sees the end of its input
        let st = self.st.clone();
        job.set("close_stdin", lua.create_function(move |_, id: u64| {
            if let Some(j) = st.borrow_mut().host.streams.iter_mut().find(|j| j.id == id) { j.stdin = None }
            Ok(())
        })?)?;
        // kill(id, signal = 15)
        let st = self.st.clone();
        job.set("kill", lua.create_function(move |_, (id, sig): (u64, Option<i32>)| {
            let s = st.borrow();
            let pid = s.host.jobs.iter().find(|j| j.id == id).map(|j| j.pid).or_else(|| s.host.streams.iter().find(|j| j.id == id).map(|j| j.pid));
            if let Some(pid) = pid {
                // the signal by kill(1): no libc in this crate
                let _ = std::process::Command::new("kill").arg(format!("-{}", sig.unwrap_or(15))).arg(pid.to_string()).status();
            }
            Ok(())
        })?)?;
        // pump(ms): for vim.wait — ended jobs' callbacks and due timers run, for up to ms
        let st = self.st.clone();
        job.set("pump", lua.create_function(move |lua, ms: u64| {
            let end = std::time::Instant::now() + std::time::Duration::from_millis(ms);
            let mut errors = Vec::new();
            loop {
                poll_jobs(lua, &st, &mut errors);
                let due: Vec<RegistryKey> = {
                    let mut s = st.borrow_mut();
                    let now = std::time::Instant::now();
                    let (due, keep): (Vec<_>, Vec<_>) = std::mem::take(&mut s.host.timers).into_iter().partition(|t| t.0 <= now);
                    s.host.timers = keep;
                    due.into_iter().map(|t| t.1).collect()
                };
                for k in due {
                    if let Err(e) = call(lua, &Some(k), ()) {
                        errors.push(format!("timer: {e}"));
                    }
                }
                if std::time::Instant::now() >= end {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
            Ok(errors.join("\n"))
        })?)?;
        job.set("running", {
            let st = self.st.clone();
            lua.create_function(move |_, ()| Ok(st.borrow().host.jobs.len()))?
        })?;
        z.set("job", job)?;
        crate::wasm::register(lua, &z)?;
        // getkey(): the next key (as zett names them: "a", "<Esc>", "<C-x>"), from the keys waiting, else waited
        // for (getchar); nil when there is none to wait for
        let st = self.st.clone();
        z.set("getkey", lua.create_function(move |_, ()| {
            let queued = st.borrow_mut().host.pending_keys.pop_front();
            if queued.is_some() { return Ok(queued) }
            let src = st.borrow_mut().host.key_source.take();
            let Some(mut src) = src else { return Ok(None) };
            let k = src();
            st.borrow_mut().host.key_source = Some(src);
            Ok(k)
        })?)?;
        // on_key_watch(on): someone uses vim.on_key, so the editor sends each key to vim._zett_on_key
        let st = self.st.clone();
        z.set("on_key_watch", lua.create_function(move |_, on: bool| { st.borrow_mut().host.on_key = on; Ok(()) })?)?;
        // buf_lines_of(nr, a, b) / buf_line_count_of(nr): another open buffer's lines (nil: not one zett has)
        let st = self.st.clone();
        z.set("buf_lines_of", lua.create_function(move |_, (nr, a, b): (usize, i64, i64)| {
            let st = st.borrow();
            let Some(Some(buf)) = st.host.others.get(nr.wrapping_sub(1)) else { return Ok(None) };
            let n = buf.len_lines();
            let (a, b) = (line_index(a, n), line_index(b, n));
            Ok(Some((a..b.max(a)).map(|i| buf.line(i)).collect::<Vec<_>>()))
        })?)?;
        let st = self.st.clone();
        z.set("buf_line_count_of", lua.create_function(move |_, nr: usize| Ok(st.borrow().host.others.get(nr.wrapping_sub(1)).and_then(|b| b.as_ref()).map(|b| b.len_lines())))?)?;
        let st = self.st.clone();
        z.set("note_other_write", lua.create_function(move |_, nr: usize| { let w = &mut st.borrow_mut().host.other_writes; if !w.contains(&nr) { w.push(nr) } Ok(()) })?)?;
        // local_opt(name): the current buffer's own value of a setting, nil when it has none
        let st = self.st.clone();
        z.set("local_opt", lua.create_function(move |_, name: String| Ok(st.borrow().host.locals.get(&name).cloned()))?)?;
        // feed_keys(keys, remap): keys typed in after this Lua returns, in the mode the editor is in (an expr
        // mapping's result, nvim_feedkeys); remap false: they skip the mappings
        let st = self.st.clone();
        z.set("feed_keys", lua.create_function(move |_, (keys, remap): (String, Option<bool>)| {
            st.borrow_mut().effects.push(Effect::Keys(keys, remap.unwrap_or(false)));
            Ok(())
        })?)?;
        // vimregex(pattern, icase): a Vim pattern compiled (zett_edit's engine) → userdata with :find(text, from) → s, e (bytes, e
        // past the end); vim.regex builds match_str / match_line on it
        z.set("vimregex", lua.create_function(|_, (p, icase): (String, Option<bool>)| {
            zett_edit::vimre::compile(&p, icase.unwrap_or(false), false).map(VimRe).map_err(mlua::Error::runtime)
        })?)?;
        // zett.hrtime(): nanoseconds on a monotonic clock (vim.uv.hrtime; os.clock is CPU time)
        let t0 = std::time::Instant::now();
        z.set("hrtime", lua.create_function(move |_, ()| Ok(t0.elapsed().as_nanos() as i64))?)?;

        // ── zett.buf: the buffer being edited ──
        let buf = lua.create_table()?;
        let with = |st: Rc<RefCell<super::State>>| move || st.borrow().host.buf.clone().ok_or_else(|| mlua::Error::runtime("no buffer"));
        let st = self.st.clone();
        buf.set("path", lua.create_function(move |_, ()| Ok(st.borrow().host.path.clone()))?)?;
        let get = with(self.st.clone());
        buf.set("text", lua.create_function(move |_, ()| Ok(get()?.borrow().text()))?)?;
        // zett.buf.marks(): { a = {line, col}, … } (1-based line, 0-based col); set_mark(name, line, col); del_marks(spec)
        let get = with(self.st.clone());
        buf.set("marks", lua.create_function(move |lua, ()| {
            let t = lua.create_table()?;
            for (c, l, col) in get()?.borrow().marks() {
                t.set(c.to_string(), lua.create_sequence_from([l + 1, col])?)?;
            }
            Ok(t)
        })?)?;
        let get = with(self.st.clone());
        buf.set("set_mark", lua.create_function(move |_, (name, line, col): (String, usize, usize)| {
            let e = get()?;
            let mut e = e.borrow_mut();
            let at = e.buf.to_char(zett_core::Pos { line: line.saturating_sub(1), col });
            if let Some(c) = name.chars().next() {
                e.set_vim_mark(c, at);
            }
            Ok(())
        })?)?;
        let get = with(self.st.clone());
        buf.set("del_marks", lua.create_function(move |_, spec: String| {
            get()?.borrow_mut().del_marks(&spec);
            Ok(())
        })?)?;
        // zett.buf.register(r): text, "V" (lines) or "v"; set_register(r, text); recording(): the register or ""
        let get = with(self.st.clone());
        buf.set("register", lua.create_function(move |_, r: String| {
            let c = r.chars().next().unwrap_or('"');
            Ok(get()?.borrow().register(c).map(|(t, l)| vec![t, if l { "V" } else { "v" }.to_string()]))
        })?)?;
        let get = with(self.st.clone());
        buf.set("set_register", lua.create_function(move |_, (r, text): (String, String)| {
            get()?.borrow_mut().set_register(r.chars().next().unwrap_or('"'), &text);
            Ok(())
        })?)?;
        let get = with(self.st.clone());
        buf.set("recording", lua.create_function(move |_, ()| Ok(get()?.borrow().recording().map(|c| c.to_string()).unwrap_or_default()))?)?;
        // zett.buf.tick(): changes when the text does (b:changedtick)
        let get = with(self.st.clone());
        buf.set("tick", lua.create_function(move |_, ()| Ok(get()?.borrow().buf.rev()))?)?;
        let get = with(self.st.clone());
        buf.set("cursor", lua.create_function(move |_, ()| {
            let c = get()?.borrow().cursor();
            Ok((c.line + 1, c.col))
        })?)?;
        // zett.buf.jumps(): { {line, col}, … } (1-based line, 0-based col) and where CTRL-O stands;
        // zett.buf.changes(): the change list the same way
        let get = with(self.st.clone());
        buf.set("jumps", lua.create_function(move |_, ()| {
            let e = get()?;
            let e = e.borrow();
            let (j, at) = e.jump_list();
            let n = e.buf.len_chars();
            Ok((j.into_iter().map(|c| { let p = e.buf.to_pos(c.min(n)); vec![p.line + 1, p.col] }).collect::<Vec<_>>(), at))
        })?)?;
        let get = with(self.st.clone());
        buf.set("changes", lua.create_function(move |_, ()| {
            let e = get()?;
            let e = e.borrow();
            let n = e.buf.len_chars();
            Ok(e.change_list().into_iter().map(|c| { let p = e.buf.to_pos(c.min(n)); vec![p.line + 1, p.col] }).collect::<Vec<_>>())
        })?)?;
        // zett.buf.undo_states(): { {seq=, parent=, time=}, … } (seq 0 is the text as opened)
        let get = with(self.st.clone());
        buf.set("undo_states", lua.create_function(move |lua, ()| {
            let t = lua.create_table()?;
            for (i, (parent, time)) in get()?.borrow_mut().undo_states().into_iter().enumerate() {
                let e = lua.create_table()?;
                e.set("seq", i)?;
                e.set("parent", parent)?;
                e.set("time", time)?;
                t.set(i + 1, e)?;
            }
            Ok(t)
        })?)?;
        let get = with(self.st.clone());
        buf.set("undo_current", lua.create_function(move |_, ()| Ok(get()?.borrow_mut().undo_current()))?)?;
        let get = with(self.st.clone());
        buf.set("undo_goto", lua.create_function(move |_, n: usize| {
            get()?.borrow_mut().undo_goto(n);
            Ok(())
        })?)?;
        let get = with(self.st.clone());
        buf.set("undo", lua.create_function(move |_, ()| {
            get()?.borrow_mut().undo();
            Ok(())
        })?)?;
        let get = with(self.st.clone());
        buf.set("redo", lua.create_function(move |_, ()| {
            get()?.borrow_mut().redo();
            Ok(())
        })?)?;
        // set_opt(name, value): a buffer setting changed at once (vim.bo.modifiable = true, then writing in the same call)
        let get = with(self.st.clone());
        let st = self.st.clone();
        buf.set("set_opt", lua.create_function(move |_, (name, value): (String, String)| {
            get()?.borrow_mut().set_opt(&name, &value);
            st.borrow_mut().host.locals.insert(name, value);
            Ok(())
        })?)?;
        let get = with(self.st.clone());
        buf.set("text_at", lua.create_function(move |_, n: usize| Ok(get()?.borrow_mut().text_at(n)))?)?;
        // zett.buf.lines(a, b) / set_lines(a, b, lines): Neovim's indexing (from 0, b not included,
        // -1 the end), what vim.api.nvim_buf_get_lines / set_lines are built on
        let get = with(self.st.clone());
        buf.set("lines", lua.create_function(move |_, (a, b): (i64, i64)| {
            let e = get()?;
            let e = e.borrow();
            let n = e.buf.len_lines();
            let (a, b) = (line_index(a, n), line_index(b, n));
            Ok((a..b.max(a)).map(|i| e.buf.line(i)).collect::<Vec<_>>())
        })?)?;
        let get = with(self.st.clone());
        buf.set("set_lines", lua.create_function(move |_, (a, b, lines): (i64, i64, Vec<String>)| {
            let e = get()?;
            let mut e = e.borrow_mut();
            let n = e.buf.len_lines();
            let (a, b) = (line_index(a, n), line_index(b, n));
            let b = b.max(a);
            let len = e.buf.len_chars();
            let nl = len > 0 && e.buf.text().ends_with('\n');
            let at = |e: &Editor, l: usize| e.buf.to_char(zett_core::Pos { line: l, col: 0 });
            let (mut start, end, rep) = if b < n {
                (at(&e, a), at(&e, b), lines.iter().map(|l| format!("{l}\n")).collect::<String>())
            } else if a < n {
                let rep = format!("{}{}", lines.join("\n"), if nl && !lines.is_empty() { "\n" } else { "" });
                (at(&e, a), len, rep)
            } else {
                let rep = format!("{}{}{}", if !nl && len > 0 { "\n" } else { "" }, lines.join("\n"), if nl { "\n" } else { "" });
                (len, len, rep)
            };
            // the last lines gone from a file without a last line break: the break before them goes too
            if lines.is_empty() && b >= n && a > 0 && !nl {
                start -= 1;
            }
            let cur = e.cursor();
            e.replace_text(start, end, &rep);
            e.set_cursor(cur);
            Ok(())
        })?)?;
        let get = with(self.st.clone());
        buf.set("line_count", lua.create_function(move |_, ()| Ok(get()?.borrow().buf.len_lines()))?)?;
        // zett.buf.set_cursor(row, col): row from 1, col from 0 (as nvim_win_set_cursor)
        let get = with(self.st.clone());
        buf.set("set_cursor", lua.create_function(move |_, (row, col): (usize, usize)| {
            get()?.borrow_mut().set_cursor(zett_core::Pos { line: row.saturating_sub(1), col });
            Ok(())
        })?)?;
        z.set("buf", buf)?;
        // zett.option_names(): every option zett knows (nvim_get_all_options_info)
        let st = self.st.clone();
        z.set("option_names", lua.create_function(move |_, ()| Ok(st.borrow().host.options.iter().map(|o| o.0.clone()).collect::<Vec<String>>()))?)?;
        // zett.option_info("ts") → { name = "tabstop", kind = "int", default = "8" } (nil: no such option)
        let st = self.st.clone();
        z.set("option_info", lua.create_function(move |lua, name: String| {
            let s = st.borrow();
            let Some((n, _, kind, default)) = s.host.options.iter().find(|o| o.0 == name || o.1.contains(&name)) else { return Ok(Value::Nil) };
            let t = lua.create_table()?;
            t.set("name", n.as_str())?;
            t.set("kind", kind.as_str())?;
            t.set("default", default.as_str())?;
            Ok(Value::Table(t))
        })?)?;

        let st = self.st.clone();
        z.set("root", lua.create_function(move |_, ()| Ok(st.borrow().host.root.display().to_string()))?)?;
        // zett.open(path, "right" | "below" | nil)
        let st = self.st.clone();
        z.set("open", lua.create_function(move |_, (path, how): (String, Option<String>)| {
            st.borrow_mut().effects.push(Effect::Ui("openFile".into(), format!("{}\t{path}", how.unwrap_or_default())));
            Ok(())
        })?)?;
        let st = self.st.clone();
        z.set("clipboard", lua.create_function(move |_, text: String| {
            st.borrow_mut().effects.push(Effect::Ui("clipboard".into(), text));
            Ok(())
        })?)?;
        // zett.input({ prompt=, default= }, function(answer) end)
        let st = self.st.clone();
        z.set("input", lua.create_function(move |lua, (o, f): (Table, Function)| {
            let prompt: String = o.get::<Option<String>>("prompt")?.unwrap_or_default();
            let default: String = o.get::<Option<String>>("default")?.unwrap_or_default();
            st.borrow_mut().host.input = Some((prompt, default, lua.create_registry_value(f)?));
            Ok(())
        })?)?;
        // zett.key_name("<cr>") == "<Enter>": one spelling per key
        z.set("key_name", lua.create_function(|_, k: String| Ok(super::keys(&k).concat()))?)?;
        // zett.maps(mode): its mappings, { {lhs=, rhs=, desc=, noremap=, lua=}, … } (Neovim's keymap queries)
        let st = self.st.clone();
        z.set("maps", lua.create_function(move |lua, mode: String| {
            let t = lua.create_table()?;
            let st = st.borrow();
            for (i, m) in st.maps.get(&mode).map(|l| l.as_slice()).unwrap_or(&[]).iter().enumerate() {
                let e = lua.create_table()?;
                e.set("lhs", m.keys.concat())?;
                let (rhs, noremap, is_lua) = match &m.action {
                    super::Action::Keys(k, remap) => (k.clone(), !remap, false),
                    super::Action::Command(c) => (format!("<Cmd>{c}<CR>"), true, false),
                    super::Action::Lua(_) => (String::new(), true, true),
                };
                e.set("rhs", rhs)?;
                e.set("noremap", noremap)?;
                e.set("lua", is_lua)?;
                e.set("desc", m.desc.clone())?;
                t.set(i + 1, e)?;
            }
            Ok(t)
        })?)?;
        let st = self.st.clone();
        z.set("buffers", lua.create_function(move |_, ()| Ok(st.borrow().host.buffers.clone()))?)?;
        // zett.hl_get(name, links): a group as Neovim reports it ({ fg = 0xRRGGBB, bg =, ctermfg =, bold = true, link = … }),
        // links followed unless links is false; nil for a group nobody defined. zett.hl_names(): every group
        let st = self.st.clone();
        z.set("hl_get", lua.create_function(move |lua, (name, links): (String, Option<bool>)| {
            let st = st.borrow();
            let mut n = name.clone();
            let mut h = None;
            for _ in 0..10 {
                let Some(x) = st.hl.get(&n) else { break };
                match (&x.link, links.unwrap_or(true)) {
                    (Some(l), true) => n = l.clone(),
                    _ => {
                        h = Some(x.clone());
                        break;
                    }
                }
            }
            let Some(h) = h else { return Ok(None) };
            let t = lua.create_table()?;
            let put = |key: &str, cterm: &str, c: &Option<super::hl::HlColor>| -> mlua::Result<()> {
                match c {
                    Some(super::hl::HlColor::Rgb(r, g, b)) => t.set(key, (*r as u32) << 16 | (*g as u32) << 8 | *b as u32),
                    Some(super::hl::HlColor::Index(i)) => t.set(cterm, *i),
                    _ => Ok(()),
                }
            };
            put("fg", "ctermfg", &h.fg)?;
            put("bg", "ctermbg", &h.bg)?;
            for (k, v) in [("bold", h.bold), ("italic", h.italic), ("underline", h.underline), ("reverse", h.reverse), ("strikethrough", h.strikethrough)] {
                if v {
                    t.set(k, true)?;
                }
            }
            if let Some(l) = &h.link {
                t.set("link", l.clone())?;
            }
            Ok(Some(t))
        })?)?;
        let st = self.st.clone();
        z.set("hl_names", lua.create_function(move |_, ()| {
            let mut v: Vec<String> = st.borrow().hl.keys().cloned().collect();
            v.sort();
            Ok(v)
        })?)?;
        // zett.qflist(): { title =, idx =, items = { {file=, lnum=, col=, type=, nr=, text=, valid=}, … } }
        let st = self.st.clone();
        z.set("qflist", lua.create_function(move |lua, ()| {
            let st = st.borrow();
            let (title, idx, items) = &st.host.qf;
            let t = lua.create_table()?;
            t.set("title", title.clone())?;
            t.set("idx", *idx + 1)?;
            let l = lua.create_table()?;
            for (i, (file, line, col, kind, nr, text, valid)) in items.iter().enumerate() {
                let e = lua.create_table()?;
                e.set("file", file.clone())?;
                e.set("lnum", *line)?;
                e.set("col", *col)?;
                e.set("type", if *kind == ' ' { String::new() } else { kind.to_string() })?;
                e.set("nr", *nr)?;
                e.set("text", text.clone())?;
                e.set("valid", *valid)?;
                l.set(i + 1, e)?;
            }
            t.set("items", l)?;
            Ok(t)
        })?)?;
        // zett.cmdline(): text, pos (nil when no : line is open); zett.history(): the : lines typed
        let st = self.st.clone();
        z.set("cmdline", lua.create_function(move |_, ()| Ok(st.borrow().host.cmdline.clone().map(|(t, p)| vec![t, p.to_string()])))?)?;
        let st = self.st.clone();
        z.set("history", lua.create_function(move |_, ()| Ok(st.borrow().host.hist.clone()))?)?;
        // zett.regex(pattern, text, start, icase): the first match of a Vim pattern from byte start:
        // { s, e, cap1 … cap9 } (bytes, 0-based, end past the match; \zs \ze applied), nil when none
        z.set("regex", lua.create_function(|lua, (pat, text, start, icase): (String, String, Option<usize>, Option<bool>)| {
            thread_local! {
                static CACHE: std::cell::RefCell<Vec<((String, bool), std::rc::Rc<zett_edit::vimre::VimRegex>)>> = const { std::cell::RefCell::new(Vec::new()) };
            }
            let ic = icase.unwrap_or(false);
            let re = CACHE.with(|c| -> Result<std::rc::Rc<zett_edit::vimre::VimRegex>, String> {
                let mut c = c.borrow_mut();
                if let Some((_, r)) = c.iter().find(|(k, _)| k.0 == pat && k.1 == ic) {
                    return Ok(r.clone());
                }
                let r = std::rc::Rc::new(zett_edit::vimre::compile(&pat, ic, false)?);
                c.push(((pat.clone(), ic), r.clone()));
                if c.len() > 64 {
                    c.remove(0);
                }
                Ok(r)
            }).map_err(mlua::Error::runtime)?;
            let from = start.unwrap_or(0).min(text.len());
            let Some((s, e)) = re.find_at(&text, from) else { return Ok(None) };
            let t = lua.create_table()?;
            t.set(1, s)?;
            t.set(2, e)?;
            // the groups (Vim's numbers), from the match found again at its start
            if let Ok(Some(c)) = re.re.captures_from_pos(&text, text[..s].rfind('\n').map_or(0, |n| n + 1)) {
                for g in 1..=9 {
                    t.set(g + 2, c.get(re.group(g)).map(|m| m.as_str().to_string()).unwrap_or_default())?;
                }
            }
            Ok(Some(t))
        })?)?;
        let st = self.st.clone();
        z.set("cur_buffer", lua.create_function(move |_, ()| Ok(st.borrow().host.cur_buffer))?)?;
        // zett.windows(): { {nr=, buf=, width=, height=, row=, col=}, … } and zett.cur_window()
        let st = self.st.clone();
        z.set("windows", lua.create_function(move |lua, ()| {
            let t = lua.create_table()?;
            for (i, (nr, buf, w, h, row, col)) in st.borrow().host.wins.iter().enumerate() {
                let e = lua.create_table()?;
                e.set("top", st.borrow().host.tops.get(i).copied().unwrap_or(0) + 1)?;
                e.set("nr", *nr)?;
                e.set("buf", *buf)?;
                e.set("width", *w)?;
                e.set("height", *h)?;
                e.set("row", *row)?;
                e.set("col", *col)?;
                t.set(i + 1, e)?;
            }
            Ok(t)
        })?)?;
        let st = self.st.clone();
        z.set("cur_window", lua.create_function(move |_, ()| Ok(st.borrow().host.cur_win))?)?;
        z.set("icon", lua.create_function(|_, name: String| Ok(file_icon(&name)))?)?;
        // zett.diff(a, b): { {"+", line}, {"-", line}, {" ", line}, … }
        z.set("diff", lua.create_function(|lua, (a, b): (String, String)| {
            let t = lua.create_table()?;
            for (i, (op, l)) in diff_lines(&a, &b).into_iter().enumerate() {
                t.set(i + 1, lua.create_sequence_from([op.to_string(), l])?)?;
            }
            Ok(t)
        })?)?;

        // ── zett.fs ──
        let fs = lua.create_table()?;
        fs.set("list", lua.create_function(|lua, dir: String| {
            let t = lua.create_table()?;
            let Ok(rd) = std::fs::read_dir(&dir) else { return Ok(t) };
            for (i, e) in rd.flatten().enumerate() {
                let x = lua.create_table()?;
                let p = e.path();
                x.set("name", e.file_name().to_string_lossy().to_string())?;
                x.set("path", p.display().to_string())?;
                x.set("dir", p.is_dir())?;
                x.set("link", e.file_type().is_ok_and(|t| t.is_symlink()))?;
                t.set(i + 1, x)?;
            }
            Ok(t)
        })?)?;
        fs.set("exists", lua.create_function(|_, p: String| Ok(Path::new(&p).exists()))?)?;
        // zett.fs.stat(path): { size=, mtime=, dir= } or nil
        fs.set("stat", lua.create_function(|lua, p: String| {
            let Ok(m) = std::fs::metadata(&p) else { return Ok(None) };
            let t = lua.create_table()?;
            t.set("size", m.len())?;
            t.set("mtime", m.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map_or(0, |d| d.as_secs()))?;
            // finer, for watching (two writes in one second)
            t.set("mtime_ns", m.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map_or(0, |d| d.as_nanos() as i64))?;
            t.set("dir", m.is_dir())?;
            Ok(Some(t))
        })?)?;
        fs.set("is_dir", lua.create_function(|_, p: String| Ok(Path::new(&p).is_dir()))?)?;
        let res = |r: std::io::Result<()>| Ok(match r {
            Ok(()) => (true, None),
            Err(e) => (false, Some(e.to_string())),
        });
        fs.set("mkdir", lua.create_function(move |_, p: String| res(std::fs::create_dir_all(p)))?)?;
        fs.set("touch", lua.create_function(move |_, p: String| {
            let p = PathBuf::from(p);
            res((|| {
                if let Some(d) = p.parent() {
                    std::fs::create_dir_all(d)?;
                }
                std::fs::OpenOptions::new().create(true).append(true).open(&p).map(|_| ())
            })())
        })?)?;
        fs.set("rename", lua.create_function(move |_, (a, b): (String, String)| {
            res((|| {
                if Path::new(&b).exists() {
                    return Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, format!("{b} exists")));
                }
                if let Some(d) = Path::new(&b).parent() {
                    std::fs::create_dir_all(d)?;
                }
                std::fs::rename(&a, &b)
            })())
        })?)?;
        fs.set("copy", lua.create_function(move |_, (a, b): (String, String)| res(copy_all(Path::new(&a), Path::new(&b))))?)?;
        // zett.fs.remove(path, trash): to the trash (gio) when asked and possible, else gone
        fs.set("remove", lua.create_function(move |_, (p, trash): (String, Option<bool>)| {
            if trash.unwrap_or(true) && std::process::Command::new("gio").args(["trash", &p]).stderr(std::process::Stdio::null()).status().is_ok_and(|s| s.success()) {
                return Ok((true, None));
            }
            let path = Path::new(&p);
            res(if path.is_dir() && !path.is_symlink() { std::fs::remove_dir_all(path) } else { std::fs::remove_file(path) })
        })?)?;
        // zett.fs.exec({ "git", "status" }, cwd): stdout, exit code
        fs.set("exec", lua.create_function(|_, (argv, cwd): (Vec<String>, Option<String>)| {
            let Some((prog, args)) = argv.split_first() else { return Ok((String::new(), -1)) };
            let mut c = std::process::Command::new(prog);
            c.args(args).stdin(std::process::Stdio::null()).stderr(std::process::Stdio::null());
            if let Some(d) = cwd {
                c.current_dir(d);
            }
            Ok(match c.output() {
                Ok(o) => (String::from_utf8_lossy(&o.stdout).to_string(), o.status.code().unwrap_or(-1)),
                Err(_) => (String::new(), -1),
            })
        })?)?;
        z.set("fs", fs)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn diff_aligns_lines() {
        let d = super::diff_lines("a\nb\nc\n", "a\nx\nc\nd\n");
        let ops: String = d.iter().map(|(o, _)| *o).collect();
        assert_eq!(ops, " -+ +");
    }
}

/// A Neovim line index (from 0; negative from the end, -1 just past the last line) in 0..=n.
fn line_index(i: i64, n: usize) -> usize {
    if i < 0 { (n as i64 + 1 + i).max(0) as usize } else { (i as usize).min(n) }
}

/// A process started from Lua (zett.job.spawn, vim.system): its result arrives on `rx` from the
/// thread that waits for it; `cb` runs on the main loop with it, once.
pub(crate) struct Job {
    pub id: u64,
    pub pid: u32,
    pub rx: std::sync::mpsc::Receiver<JobOut>,
    pub cb: Option<RegistryKey>,
}

pub(crate) struct JobOut {
    pub code: i32,
    pub signal: i32,
    pub stdout: String,
    pub stderr: String,
}

impl JobOut {
    fn table(&self, lua: &mlua::Lua) -> mlua::Result<mlua::Table> {
        let t = lua.create_table()?;
        t.set("code", self.code)?;
        t.set("signal", self.signal)?;
        t.set("stdout", self.stdout.as_str())?;
        t.set("stderr", self.stderr.as_str())?;
        Ok(t)
    }
}

/// Starts `argv` in its own thread: stdin written, stdout and stderr collected, the exit sent back.
fn start_job(argv: &[String], cwd: Option<String>, env: Option<Vec<(String, String)>>, clear_env: bool, stdin: Option<String>) -> std::io::Result<(u32, std::sync::mpsc::Receiver<JobOut>)> {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let (prog, args) = argv.split_first().ok_or_else(|| std::io::Error::other("no command"))?;
    let mut c = Command::new(prog);
    c.args(args).stdout(Stdio::piped()).stderr(Stdio::piped()).stdin(if stdin.is_some() { Stdio::piped() } else { Stdio::null() });
    if let Some(d) = cwd {
        c.current_dir(d);
    }
    if clear_env {
        c.env_clear();
    }
    for (k, v) in env.unwrap_or_default() {
        c.env(k, v);
    }
    let mut child = c.spawn()?;
    let pid = child.id();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        if let (Some(text), Some(mut w)) = (stdin, child.stdin.take()) {
            // a big stdin in its own thread: the child may fill stdout before it has read it all
            std::thread::spawn(move || {
                let _ = w.write_all(text.as_bytes());
            });
        }
        let out = match child.wait_with_output() {
            Ok(o) => {
                use std::os::unix::process::ExitStatusExt;
                JobOut { code: o.status.code().unwrap_or(128 + o.status.signal().unwrap_or(0)), signal: o.status.signal().unwrap_or(0), stdout: String::from_utf8_lossy(&o.stdout).into_owned(), stderr: String::from_utf8_lossy(&o.stderr).into_owned() }
            }
            Err(e) => JobOut { code: -1, signal: 0, stdout: String::new(), stderr: e.to_string() },
        };
        let _ = tx.send(out);
    });
    Ok((pid, rx))
}

/// A streamed job (jobstart): stdin kept for writing, its output and its end arrive on the channel.
pub(crate) struct Stream {
    pub id: u64,
    pub pid: u32,
    pub stdin: Option<std::process::ChildStdin>,
    pub rx: std::sync::mpsc::Receiver<StreamMsg>,
    pub cb: RegistryKey,
}

pub(crate) enum StreamMsg {
    /// "stdout" or "stderr", and what came
    Data(&'static str, String),
    Exit(i32),
}

/// Starts `argv` with stdin, stdout and stderr piped; a thread per output sends what it reads.
pub(crate) fn start_stream(argv: &[String], cwd: Option<String>, env: Option<Vec<(String, String)>>, clear_env: bool) -> std::io::Result<(u32, std::process::ChildStdin, std::sync::mpsc::Receiver<StreamMsg>)> {
    use std::io::Read;
    use std::process::{Command, Stdio};
    let (prog, args) = argv.split_first().ok_or_else(|| std::io::Error::other("no command"))?;
    let mut c = Command::new(prog);
    c.args(args).stdout(Stdio::piped()).stderr(Stdio::piped()).stdin(Stdio::piped());
    if let Some(d) = cwd { c.current_dir(d); }
    if clear_env { c.env_clear(); }
    for (k, v) in env.unwrap_or_default() { c.env(k, v); }
    let mut child = c.spawn()?;
    let pid = child.id();
    let stdin = child.stdin.take().ok_or_else(|| std::io::Error::other("no stdin"))?;
    let (tx, rx) = std::sync::mpsc::channel();
    let mut readers = Vec::new();
    for (name, pipe) in [("stdout", child.stdout.take().map(|p| Box::new(p) as Box<dyn Read + Send>)), ("stderr", child.stderr.take().map(|p| Box::new(p) as Box<dyn Read + Send>))] {
        let (Some(mut pipe), tx) = (pipe, tx.clone()) else { continue };
        readers.push(std::thread::spawn(move || {
            let mut buf = [0u8; 8192];
            while let Ok(n) = pipe.read(&mut buf) {
                if n == 0 || tx.send(StreamMsg::Data(name, String::from_utf8_lossy(&buf[..n]).into_owned())).is_err() { break }
            }
        }));
    }
    std::thread::spawn(move || {
        // the end after all output, so on_exit comes last as in Neovim
        for r in readers { let _ = r.join(); }
        use std::os::unix::process::ExitStatusExt;
        let code = child.wait().map(|s| s.code().unwrap_or(128 + s.signal().unwrap_or(0))).unwrap_or(-1);
        let _ = tx.send(StreamMsg::Exit(code));
    });
    Ok((pid, stdin, rx))
}

/// Runs the callbacks of the jobs that have ended (each once); true when one ran.
pub(crate) fn poll_jobs(lua: &mlua::Lua, st: &Rc<RefCell<super::State>>, errors: &mut Vec<String>) -> bool {
    // streamed jobs first: each piece of output to its callback as (kind, text), the end as ("exit", code)
    let msgs: Vec<(mlua::Function, Vec<StreamMsg>)> = {
        let mut s = st.borrow_mut();
        let mut out = Vec::new();
        let mut ended = Vec::new();
        for j in &s.host.streams {
            let got: Vec<StreamMsg> = j.rx.try_iter().collect();
            if got.iter().any(|m| matches!(m, StreamMsg::Exit(_))) { ended.push(j.id) }
            if !got.is_empty() { if let Ok(f) = lua.registry_value::<mlua::Function>(&j.cb) { out.push((f, got)) } }
        }
        s.host.streams.retain(|j| !ended.contains(&j.id));
        out
    };
    let streamed = !msgs.is_empty();
    for (f, list) in msgs {
        for m in list {
            let r = match m { StreamMsg::Data(k, d) => f.call::<()>((k, d)), StreamMsg::Exit(c) => f.call::<()>(("exit", c)) };
            if let Err(e) = r { errors.push(format!("jobstart: {e}")) }
        }
    }
    let done: Vec<(Option<RegistryKey>, JobOut)> = {
        let mut s = st.borrow_mut();
        let mut done = Vec::new();
        s.host.jobs.retain_mut(|j| match j.rx.try_recv() {
            Ok(out) => {
                done.push((j.cb.take(), out));
                false
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => true,
            Err(_) => false,
        });
        done
    };
    let any = !done.is_empty() || streamed;
    for (cb, out) in done {
        if let Some(k) = cb {
            let r = match out.table(lua) {
                Ok(t) => call(lua, &Some(k), t),
                Err(e) => Err(e.to_string()),
            };
            if let Err(e) = r {
                errors.push(format!("vim.system on_exit: {e}"));
            }
        }
    }
    any
}

/// A compiled Vim pattern for Lua (vim.regex).
pub(crate) struct VimRe(zett_edit::vimre::VimRegex);
impl mlua::UserData for VimRe {
    fn add_methods<M: mlua::UserDataMethods<Self>>(m: &mut M) {
        m.add_method("find", |_, r, (s, from): (mlua::String, Option<usize>)| {
            let s = String::from_utf8_lossy(&s.as_bytes()).into_owned();
            Ok(match r.0.find_at(&s, from.unwrap_or(0).min(s.len())) { Some((a, b)) => (Some(a), Some(b)), None => (None, None) })
        });
    }
}

impl crate::Engine {
    /// A plugin listens to keys (vim.on_key).
    pub fn wants_keys(&self) -> bool {
        self.st.borrow().host.on_key
    }
}

impl crate::Engine {
    /// Where getchar waits for a key (the editor: the terminal), when none are queued.
    pub fn set_key_source(&mut self, f: Box<dyn FnMut() -> Option<String>>) {
        self.st.borrow_mut().host.key_source = Some(f);
    }
    /// Keys getchar reads first (tests; keys typed ahead).
    pub fn queue_keys(&mut self, keys: Vec<String>) {
        self.st.borrow_mut().host.pending_keys = keys.into();
    }
    /// How many queued keys are still unread.
    pub fn queued_keys(&self) -> usize {
        self.st.borrow().host.pending_keys.len()
    }
}

impl crate::Engine {
    /// The other open buffers, by place (None for the current one), for vim.api on a buffer not current.
    pub fn set_others(&mut self, others: Vec<Option<zett_core::Buffer>>) {
        self.st.borrow_mut().host.others = others;
    }
    /// The buffers Lua wrote to while they were not current (apply each: lend it, vim._zett_apply_writes(nr)).
    pub fn take_other_writes(&mut self) -> Vec<usize> {
        std::mem::take(&mut self.st.borrow_mut().host.other_writes)
    }
}
