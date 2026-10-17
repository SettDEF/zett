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
    /// a g waiting for its second g
    pub g: bool,
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

    pub fn panels(&self) -> Vec<PanelView> {
        let st = self.st.borrow();
        st.host
            .panels
            .iter()
            .map(|p| PanelView { id: p.id.clone(), side: p.side.clone(), width: p.width, height: p.height, title: p.title.clone(), lines: p.lines.clone(), cursor: p.cursor, focused: st.host.focus.as_deref() == Some(&p.id) })
            .collect()
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
        let key = super::keys(key).into_iter().next().unwrap_or_default();
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
                    h.panels.push(Panel { id: id.clone(), side: "left".into(), width: 30, height: 0, title: String::new(), lines: Vec::new(), cursor: 0, on_key: None, on_move: None, on_close: None });
                    h.panels.len() - 1
                }
            };
            let p = &mut h.panels[i];
            if let Some(v) = o.get::<Option<String>>("side")? {
                p.side = if v == "right" { v } else { "left".into() };
            }
            p.width = o.get::<Option<u16>>("width")?.unwrap_or(p.width).clamp(10, 200);
            p.height = o.get::<Option<u16>>("height")?.unwrap_or(p.height);
            p.title = o.get::<Option<String>>("title")?.unwrap_or(p.title.clone());
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

        // ── zett.buf: the buffer being edited ──
        let buf = lua.create_table()?;
        let with = |st: Rc<RefCell<super::State>>| move || st.borrow().host.buf.clone().ok_or_else(|| mlua::Error::runtime("no buffer"));
        let st = self.st.clone();
        buf.set("path", lua.create_function(move |_, ()| Ok(st.borrow().host.path.clone()))?)?;
        let get = with(self.st.clone());
        buf.set("text", lua.create_function(move |_, ()| Ok(get()?.borrow().text()))?)?;
        let get = with(self.st.clone());
        buf.set("cursor", lua.create_function(move |_, ()| {
            let c = get()?.borrow().cursor();
            Ok((c.line + 1, c.col))
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
        let get = with(self.st.clone());
        buf.set("text_at", lua.create_function(move |_, n: usize| Ok(get()?.borrow_mut().text_at(n)))?)?;
        z.set("buf", buf)?;

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
        z.set("key_name", lua.create_function(|_, k: String| Ok(super::keys(&k).into_iter().next().unwrap_or_default()))?)?;
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
