//! zett's Lua engine: one registry of commands, keymaps per mode (with a
//! leader and which-key groups), hooks, and the `zett` API, loaded from the
//! built-in defaults, plugins and the user's `init.lua`.
//!
//! The window, the CLI and the terminal front end all drive the same engine:
//! they feed it keys or command names and act on the effects it returns
//! (open a page, refresh, show a notice, open a piece of UI). Built-in
//! commands are Lua too (`defaults.lua`), written against the public API, so
//! a plugin can do anything the app itself does.

pub mod docs;

use mlua::{Function, Lua, RegistryKey, Table, Value};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use zett_core::blocks::{BlockId, Kind};
use zett_core::folder::Folder;

/// What the host should do after keys or a command ran.
#[derive(Clone, Debug, PartialEq)]
pub enum Effect {
    /// show this page
    Open(BlockId),
    /// the shown page or the page list changed
    Refresh,
    /// a short message for the user
    Notify(String),
    /// a piece of UI the host owns: "quickOpen", "commandLine", "whichKey", "help"…
    Ui(String, String),
}

/// One entry of the which-key popup.
#[derive(Clone, Debug, PartialEq)]
pub struct MenuItem {
    pub key: String,
    pub desc: String,
    /// more keys follow (a group)
    pub group: bool,
}

/// What fed keys did.
#[derive(Debug, PartialEq)]
pub enum Feed {
    /// a mapping ran
    Ran(Vec<Effect>),
    /// the keys start one or more mappings: wait for more (show these choices)
    Pending(Vec<MenuItem>),
    /// no mapping: the host handles the keys itself
    None,
}

enum Action {
    Command(String),
    Lua(RegistryKey),
}

struct Map {
    keys: Vec<String>,
    action: Action,
    desc: String,
}

struct Command {
    desc: String,
    func: RegistryKey,
    /// where it came from ("zett", a plugin name, "init.lua")
    source: String,
}

#[derive(Default)]
struct State {
    folder: Option<Rc<RefCell<Folder>>>,
    current: Option<BlockId>,
    effects: Vec<Effect>,
    /// pages changed by scripts, saved when the run ends
    dirty: HashSet<BlockId>,
    /// mode → keymaps
    maps: HashMap<String, Vec<Map>>,
    /// which-key group names: key sequence (tokens joined) → name
    groups: HashMap<String, String>,
    commands: BTreeMap<String, Command>,
    hooks: HashMap<String, Vec<RegistryKey>>,
    /// who is loading right now (for command sources)
    loading: String,
    leader: String,
}

pub struct Engine {
    lua: Lua,
    st: Rc<RefCell<State>>,
    pub errors: Vec<String>,
    pub plugins: Vec<String>,
}

const DEFAULTS: &str = include_str!("defaults.lua");
/// The code editor's layer (Doom's SPC tree), on top of the defaults when `frontend` is "editor".
const EDITOR: &str = include_str!("editor.lua");

/// Splits Vim key notation into keys: `<leader>pn` → `<leader>`, `p`, `n`; `<C-s>` stays one.
pub fn keys(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '<' {
            let rest: String = chars.clone().collect();
            if let Some(end) = rest.find('>') {
                let inner = &rest[..end];
                if !inner.is_empty() && !inner.contains('<') && !inner.contains(' ') {
                    out.push(normalize(&format!("<{inner}>")));
                    for _ in 0..=end {
                        chars.next();
                    }
                    continue;
                }
            }
        }
        out.push(if c == ' ' { "<Space>".into() } else { c.to_string() });
    }
    out
}

/// One spelling per key: `<c-S>` → `<C-s>`, `<space>` → `<Space>`, `<CR>` → `<Enter>`.
fn normalize(k: &str) -> String {
    let inner = &k[1..k.len() - 1];
    let lower = inner.to_lowercase();
    let named = |n: &str| -> Option<&'static str> {
        Some(match n {
            "space" => "<Space>",
            "cr" | "enter" | "return" => "<Enter>",
            "esc" | "escape" => "<Esc>",
            "tab" => "<Tab>",
            "bs" | "backspace" => "<BS>",
            "leader" => "<leader>",
            "up" => "<Up>",
            "down" => "<Down>",
            "left" => "<Left>",
            "right" => "<Right>",
            _ => return None,
        })
    };
    if let Some(n) = named(&lower) {
        return n.into();
    }
    // modifiers: C- A-/M- S-, then the key (letters lowercase with Ctrl)
    let mut mods = String::new();
    let mut rest = inner;
    loop {
        let l = rest.to_lowercase();
        if l.starts_with("c-") && rest.len() > 2 {
            mods.push_str("C-");
        } else if (l.starts_with("a-") || l.starts_with("m-")) && rest.len() > 2 {
            mods.push_str("A-");
        } else if l.starts_with("s-") && rest.len() > 2 {
            mods.push_str("S-");
        } else {
            break;
        }
        rest = &rest[2..];
    }
    let key = match named(&rest.to_lowercase()) {
        Some(n) => n[1..n.len() - 1].to_string(),
        None if mods.contains("C-") => rest.to_lowercase(),
        None => rest.to_string(),
    };
    format!("<{mods}{key}>")
}

fn kind_from(s: &str) -> Option<Kind> {
    Some(match s {
        "paragraph" | "text" | "p" => Kind::Paragraph,
        "h1" | "heading" => Kind::Heading(1),
        "h2" => Kind::Heading(2),
        "h3" => Kind::Heading(3),
        "todo" | "task" => Kind::Todo { checked: false },
        "done" => Kind::Todo { checked: true },
        "bullet" | "li" => Kind::Bullet,
        "quote" => Kind::Quote,
        "code" => Kind::Code { lang: String::new() },
        _ => return None,
    })
}

fn kind_name(k: &Kind) -> &'static str {
    match k {
        Kind::Page => "page",
        Kind::Paragraph => "paragraph",
        Kind::Heading(1) => "h1",
        Kind::Heading(2) => "h2",
        Kind::Heading(_) => "h3",
        Kind::Todo { checked: false } => "todo",
        Kind::Todo { checked: true } => "done",
        Kind::Bullet => "bullet",
        Kind::Quote => "quote",
        Kind::Code { .. } => "code",
    }
}

fn lerr(e: impl std::fmt::Display) -> mlua::Error {
    mlua::Error::RuntimeError(e.to_string())
}

/// Where the user's config lives: $ZETT_CONFIG, else $XDG_CONFIG_HOME/zett, else ~/.config/zett.
pub fn config_dir() -> PathBuf {
    if let Some(d) = std::env::var_os("ZETT_CONFIG") {
        return PathBuf::from(d);
    }
    let base = std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from).unwrap_or_else(|| {
        std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default().join(".config")
    });
    base.join("zett")
}

impl Engine {
    pub fn new(folder: Option<Rc<RefCell<Folder>>>) -> Engine {
        let lua = Lua::new();
        let st = Rc::new(RefCell::new(State { folder, leader: "<Space>".into(), loading: "zett".into(), ..Default::default() }));
        let mut e = Engine { lua, st, errors: Vec::new(), plugins: Vec::new() };
        if let Err(err) = e.install_api() {
            e.errors.push(format!("zett API: {err}"));
        }
        e
    }

    /// Which front end loads the config (`zett.frontend` in Lua): "editor" (the terminal
    /// code editor) or "window" (the notes window). Set before load_config.
    pub fn set_frontend(&mut self, name: &str) {
        if let Ok(z) = self.lua.globals().get::<Table>("zett") {
            let _ = z.set("frontend", name);
        }
    }

    pub fn frontend(&self) -> String {
        self.lua.globals().get::<Table>("zett").and_then(|z| z.get::<String>("frontend")).unwrap_or_default()
    }

    /// Built-in defaults, then plugins, then the user's init.lua (the user has the last word).
    pub fn load_config(&mut self, dir: &Path) {
        self.exec_named(DEFAULTS, "zett", "defaults.lua");
        if self.frontend() == "editor" {
            self.exec_named(EDITOR, "zett", "editor.lua");
        }
        let plugins = dir.join("plugins");
        let mut names: Vec<PathBuf> = std::fs::read_dir(&plugins).map(|r| r.flatten().map(|e| e.path()).filter(|p| p.join("init.lua").is_file()).collect()).unwrap_or_default();
        names.sort();
        for p in names {
            let name = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
            // `require("name.module")` finds the plugin's own lua/ folder, as in Neovim
            let lua_dir = p.join("lua");
            if let Ok(package) = self.lua.globals().get::<Table>("package") {
                let path: String = package.get("path").unwrap_or_default();
                let _ = package.set("path", format!("{}/?.lua;{}/?/init.lua;{path}", lua_dir.display(), lua_dir.display()));
            }
            match std::fs::read_to_string(p.join("init.lua")) {
                Ok(code) => {
                    self.exec_named(&code, &name, &format!("plugins/{name}/init.lua"));
                    self.plugins.push(name);
                }
                Err(e) => self.errors.push(format!("plugin {name}: {e}")),
            }
        }
        let init = dir.join("init.lua");
        if let Ok(code) = std::fs::read_to_string(&init) {
            if let Ok(package) = self.lua.globals().get::<Table>("package") {
                let path: String = package.get("path").unwrap_or_default();
                let _ = package.set("path", format!("{}/lua/?.lua;{}/lua/?/init.lua;{path}", dir.display(), dir.display()));
            }
            self.exec_named(&code, "init.lua", "init.lua");
        }
        self.leader_from_opts();
        let _ = self.take_effects();
        self.emit("start", Value::Nil);
    }

    fn exec_named(&mut self, code: &str, source: &str, chunk: &str) {
        self.st.borrow_mut().loading = source.to_string();
        if let Err(e) = self.lua.load(code).set_name(format!("@{chunk}")).exec() {
            self.errors.push(format!("{chunk}: {e}"));
        }
        self.st.borrow_mut().loading = "zett".into();
    }

    fn leader_from_opts(&mut self) {
        let leader = self.opt("leader").unwrap_or_else(|| " ".into());
        let k = keys(&leader).into_iter().next().unwrap_or_else(|| "<Space>".into());
        self.st.borrow_mut().leader = k;
    }

    /// A setting from `zett.opt` as text.
    pub fn opt(&self, name: &str) -> Option<String> {
        let z: Table = self.lua.globals().get("zett").ok()?;
        let o: Table = z.get("opt").ok()?;
        match o.get::<Value>(name).ok()? {
            Value::String(s) => Some(s.to_string_lossy()),
            Value::Integer(i) => Some(i.to_string()),
            Value::Number(n) => Some(n.to_string()),
            Value::Boolean(b) => Some(b.to_string()),
            _ => None,
        }
    }

    /// Lends the workspace to the engine for one call and takes it back: the host keeps
    /// owning its folder, scripts see it only while they run.
    pub fn lend<R>(&mut self, folder: Folder, f: impl FnOnce(&mut Engine) -> R) -> (Folder, R) {
        let rc = Rc::new(RefCell::new(folder));
        self.st.borrow_mut().folder = Some(rc.clone());
        let r = f(self);
        self.st.borrow_mut().folder = None;
        let folder = match Rc::try_unwrap(rc) {
            Ok(c) => c.into_inner(),
            Err(_) => unreachable!("scripts never keep the workspace"),
        };
        (folder, r)
    }

    pub fn leader(&self) -> String {
        self.st.borrow().leader.clone()
    }

    pub fn set_current(&self, page: Option<BlockId>) {
        self.st.borrow_mut().current = page;
    }

    /// Runs Lua code (the `:lua` command, `zett lua`), returning what it printed or returned.
    pub fn eval(&mut self, code: &str) -> Result<(String, Vec<Effect>), String> {
        // an expression first (`1 + 1`, `zett.pages()`), then statements
        let r = match self.lua.load(format!("return {code}")).set_name("@:lua").eval::<Value>() {
            Ok(v) => Ok(v),
            Err(_) => self.lua.load(code).set_name("@:lua").eval::<Value>(),
        };
        let out = match r {
            Ok(v) => show(&v),
            Err(e) => {
                self.finish();
                return Err(e.to_string());
            }
        };
        Ok((out, self.finish()))
    }

    /// Runs a command by name.
    pub fn run(&mut self, name: &str) -> Result<Vec<Effect>, String> {
        let f: Function = {
            let st = self.st.borrow();
            let c = st.commands.get(name).ok_or_else(|| format!("no command {name:?}"))?;
            self.lua.registry_value(&c.func).map_err(|e| e.to_string())?
        };
        let r = f.call::<()>(());
        let fx = self.finish();
        r.map_err(|e| e.to_string())?;
        Ok(fx)
    }

    /// Commands for the palette and `:`: (name, description, source).
    pub fn commands(&self) -> Vec<(String, String, String)> {
        self.st.borrow().commands.iter().map(|(n, c)| (n.clone(), c.desc.clone(), c.source.clone())).collect()
    }

    /// The keys of each command in a mode, shown as typed (`SPC f f`, `gd`).
    pub fn keys_of(&self, mode: &str) -> std::collections::HashMap<String, Vec<String>> {
        let st = self.st.borrow();
        let mut out: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
        for m in st.maps.get(mode).map(|v| v.as_slice()).unwrap_or_default() {
            if let Action::Command(c) = &m.action {
                let shown: Vec<String> = m.keys.iter().map(|k| if k == "<leader>" || k == "<Space>" || k == " " { "SPC".to_string() } else { k.clone() }).collect();
                // Doom's way: `SPC c c` spaced, Vim's sequences as typed (`gcc`, `]e`, `<C-w>v`)
                let text = if shown.first().map(String::as_str) == Some("SPC") { shown.join(" ") } else { shown.concat() };
                out.entry(c.clone()).or_default().push(text);
            }
        }
        for v in out.values_mut() {
            v.sort_by_key(|k| (k.len(), k.clone()));
        }
        out
    }

    /// Every mapping of a mode as (keys as typed, description), for `SPC h b`.
    pub fn bindings(&self, mode: &str) -> Vec<(String, String)> {
        let st = self.st.borrow();
        let mut v: Vec<(String, String)> = st.maps.get(mode).map(|m| m.iter().map(|m| (m.keys.concat(), m.desc.clone())).collect()).unwrap_or_default();
        v.sort();
        v
    }

    /// Keys typed in a mode (`n`, `i`, …). A mapping that matches exactly runs; a prefix of
    /// longer mappings waits and lists what can follow (which-key).
    pub fn feed(&mut self, mode: &str, typed: &[String]) -> Feed {
        let (exact, pending) = {
            let st = self.st.borrow();
            let leader = st.leader.clone();
            let Some(maps) = st.maps.get(mode) else { return Feed::None };
            let expand = |k: &Vec<String>| k.iter().map(|t| if t == "<leader>" { leader.clone() } else { t.clone() }).collect::<Vec<_>>();
            let mut exact = None;
            let mut longer = false;
            for (i, m) in maps.iter().enumerate() {
                let k = expand(&m.keys);
                if k == typed {
                    exact = Some(i);
                } else if k.len() > typed.len() && k[..typed.len()] == *typed {
                    longer = true;
                }
            }
            (exact, longer)
        };
        if let Some(i) = exact.filter(|_| !pending) {
            return Feed::Ran(self.run_map(mode, i));
        }
        if pending {
            return Feed::Pending(self.menu(mode, typed));
        }
        if let Some(i) = exact {
            return Feed::Ran(self.run_map(mode, i));
        }
        Feed::None
    }

    /// The keys stopped (the host's timeout, like Neovim's `timeoutlen`): a mapping that is
    /// exactly these keys runs even though longer ones start the same way.
    pub fn timeout(&mut self, mode: &str, typed: &[String]) -> Feed {
        let exact = {
            let st = self.st.borrow();
            let leader = st.leader.clone();
            st.maps.get(mode).and_then(|maps| maps.iter().position(|m| m.keys.iter().map(|t| if t == "<leader>" { leader.clone() } else { t.clone() }).collect::<Vec<_>>() == typed))
        };
        match exact {
            Some(i) => Feed::Ran(self.run_map(mode, i)),
            None => Feed::None,
        }
    }

    fn run_map(&mut self, mode: &str, i: usize) -> Vec<Effect> {
        enum Do {
            Cmd(String),
            Fun(Function),
        }
        let what = {
            let st = self.st.borrow();
            match &st.maps[mode][i].action {
                Action::Command(c) => Do::Cmd(c.clone()),
                Action::Lua(k) => match self.lua.registry_value::<Function>(k) {
                    Ok(f) => Do::Fun(f),
                    Err(e) => return vec![Effect::Notify(e.to_string())],
                },
            }
        };
        match what {
            Do::Cmd(c) => self.run(&c).unwrap_or_else(|e| vec![Effect::Notify(e)]),
            Do::Fun(f) => {
                let r = f.call::<()>(());
                let mut fx = self.finish();
                if let Err(e) = r {
                    fx.push(Effect::Notify(e.to_string()));
                }
                fx
            }
        }
    }

    /// What can follow these keys: one entry per next key, groups named by `zett.group`.
    pub fn menu(&self, mode: &str, typed: &[String]) -> Vec<MenuItem> {
        let st = self.st.borrow();
        let leader = st.leader.clone();
        let mut next: BTreeMap<String, MenuItem> = BTreeMap::new();
        for m in st.maps.get(mode).into_iter().flatten() {
            let k: Vec<String> = m.keys.iter().map(|t| if t == "<leader>" { leader.clone() } else { t.clone() }).collect();
            if k.len() <= typed.len() || k[..typed.len()] != *typed {
                continue;
            }
            let key = k[typed.len()].clone();
            let group = k.len() > typed.len() + 1;
            let desc = if group {
                // the group's name, keyed in the spelling it was given (with <leader>)
                let raw: Vec<String> = m.keys[..typed.len() + 1].to_vec();
                st.groups.get(&raw.concat()).or_else(|| st.groups.get(&k[..typed.len() + 1].concat())).cloned().unwrap_or_else(|| "+more".into())
            } else {
                m.desc.clone()
            };
            next.entry(key.clone()).and_modify(|e| e.group |= group).or_insert(MenuItem { key, desc, group });
        }
        next.into_values().collect()
    }

    /// Runs the hooks of an event (`page.open`, `page.save`, `start`, …).
    pub fn emit(&mut self, event: &str, data: Value) -> Vec<Effect> {
        let fs: Vec<Function> = {
            let st = self.st.borrow();
            st.hooks.get(event).map(|v| v.iter().filter_map(|k| self.lua.registry_value(k).ok()).collect()).unwrap_or_default()
        };
        for f in fs {
            if let Err(e) = f.call::<()>(data.clone()) {
                self.errors.push(format!("hook {event}: {e}"));
            }
        }
        self.finish()
    }

    /// Hooks with a page title as their data.
    pub fn emit_page(&mut self, event: &str, page: BlockId) -> Vec<Effect> {
        let title = self.st.borrow().folder.as_ref().map(|f| f.borrow().ws.text(page)).unwrap_or_default();
        let data = self.lua.create_table().ok().map(|t| {
            let _ = t.set("title", title);
            Value::Table(t)
        });
        self.emit(event, data.unwrap_or(Value::Nil))
    }

    fn take_effects(&mut self) -> Vec<Effect> {
        std::mem::take(&mut self.st.borrow_mut().effects)
    }

    /// The end of a run: save what scripts changed, hand over the effects.
    fn finish(&mut self) -> Vec<Effect> {
        let (dirty, folder) = {
            let mut st = self.st.borrow_mut();
            (std::mem::take(&mut st.dirty), st.folder.clone())
        };
        if let Some(f) = folder {
            let f = f.borrow();
            for p in &dirty {
                if let Err(e) = f.save_page(*p) {
                    self.errors.push(format!("save: {e}"));
                }
            }
        }
        let mut fx = self.take_effects();
        if !dirty.is_empty() && !fx.contains(&Effect::Refresh) {
            fx.push(Effect::Refresh);
        }
        fx
    }

    fn install_api(&mut self) -> mlua::Result<()> {
        let lua = &self.lua;
        let z = lua.create_table()?;
        z.set("version", env!("CARGO_PKG_VERSION"))?;
        z.set("opt", lua.create_table()?)?;
        z.set("config_dir", config_dir().display().to_string())?;

        // zett.map(mode, keys, command-name | function, { desc = "" })
        let st = self.st.clone();
        z.set("map", lua.create_function(move |lua, (modes, lhs, rhs, opts): (Value, String, Value, Option<Table>)| {
            let desc: String = opts.as_ref().and_then(|o| o.get("desc").ok()).unwrap_or_default();
            let modes: Vec<String> = match modes {
                Value::String(s) => vec![s.to_string_lossy()],
                Value::Table(t) => t.sequence_values::<String>().flatten().collect(),
                _ => return Err(lerr("zett.map: mode is a string or a list")),
            };
            for mode in modes {
                let action = match &rhs {
                    Value::String(s) => Action::Command(s.to_string_lossy()),
                    Value::Function(f) => Action::Lua(lua.create_registry_value(f.clone())?),
                    _ => return Err(lerr("zett.map: the action is a command name or a function")),
                };
                let k = keys(&lhs);
                let desc = if desc.is_empty() { if let Value::String(s) = &rhs { s.to_string_lossy() } else { String::new() } } else { desc.clone() };
                let mut s = st.borrow_mut();
                let list = s.maps.entry(mode).or_default();
                list.retain(|m| m.keys != k);
                list.push(Map { keys: k, action, desc });
            }
            Ok(())
        })?)?;

        let st = self.st.clone();
        z.set("unmap", lua.create_function(move |_, (mode, lhs): (String, String)| {
            let k = keys(&lhs);
            if let Some(l) = st.borrow_mut().maps.get_mut(&mode) {
                l.retain(|m| m.keys != k);
            }
            Ok(())
        })?)?;

        // zett.group("<leader>p", "pages"): the name which-key shows for a prefix
        let st = self.st.clone();
        z.set("group", lua.create_function(move |_, (lhs, name): (String, String)| {
            st.borrow_mut().groups.insert(keys(&lhs).concat(), name);
            Ok(())
        })?)?;

        // zett.command(name, function, { desc = "" })
        let st = self.st.clone();
        z.set("command", lua.create_function(move |lua, (name, f, opts): (String, Function, Option<Table>)| {
            let desc: String = opts.and_then(|o| o.get("desc").ok()).unwrap_or_default();
            let func = lua.create_registry_value(f)?;
            let mut s = st.borrow_mut();
            let source = s.loading.clone();
            s.commands.insert(name, Command { desc, func, source });
            Ok(())
        })?)?;

        // zett.run(name): another command, from Lua
        let st = self.st.clone();
        z.set("run", lua.create_function(move |lua, name: String| {
            let f: Function = {
                let s = st.borrow();
                let c = s.commands.get(&name).ok_or_else(|| lerr(format!("no command {name:?}")))?;
                lua.registry_value(&c.func)?
            };
            f.call::<()>(())
        })?)?;

        // zett.on(event, function)
        let st = self.st.clone();
        z.set("on", lua.create_function(move |lua, (event, f): (String, Function)| {
            let k = lua.create_registry_value(f)?;
            st.borrow_mut().hooks.entry(event).or_default().push(k);
            Ok(())
        })?)?;

        let st = self.st.clone();
        z.set("notify", lua.create_function(move |_, msg: String| {
            st.borrow_mut().effects.push(Effect::Notify(msg));
            Ok(())
        })?)?;

        // zett.ui(name, arg): open a piece of the host's UI (quick open, the command line, help…)
        let st = self.st.clone();
        z.set("ui", lua.create_function(move |_, (name, arg): (String, Option<String>)| {
            st.borrow_mut().effects.push(Effect::Ui(name, arg.unwrap_or_default()));
            Ok(())
        })?)?;

        // zett.commands(): { {name=, desc=, source=}, … }
        let st = self.st.clone();
        z.set("commands", lua.create_function(move |lua, ()| {
            let t = lua.create_table()?;
            for (i, (n, c)) in st.borrow().commands.iter().enumerate() {
                let e = lua.create_table()?;
                e.set("name", n.clone())?;
                e.set("desc", c.desc.clone())?;
                e.set("source", c.source.clone())?;
                t.set(i + 1, e)?;
            }
            Ok(t)
        })?)?;

        // ── the workspace ──
        // zett.pages(): { {title=, file=}, … }
        let st = self.st.clone();
        z.set("pages", lua.create_function(move |lua, ()| {
            let t = lua.create_table()?;
            let s = st.borrow();
            let Some(f) = s.folder.as_ref() else { return Ok(t) };
            let f = f.borrow();
            for (i, p) in f.ws.pages().into_iter().enumerate() {
                let e = lua.create_table()?;
                e.set("title", f.ws.text(p))?;
                e.set("file", f.path_of(p).map(|p| p.display().to_string()).unwrap_or_default())?;
                t.set(i + 1, e)?;
            }
            Ok(t)
        })?)?;

        let page = lua.create_table()?;
        // zett.page.current(): {title=, blocks={ {kind=, text=}, … }} or nil
        let st = self.st.clone();
        page.set("current", lua.create_function(move |lua, ()| {
            let s = st.borrow();
            let (Some(f), Some(p)) = (s.folder.as_ref(), s.current) else { return Ok(Value::Nil) };
            let f = f.borrow();
            if !f.ws.exists(p) {
                return Ok(Value::Nil);
            }
            let t = lua.create_table()?;
            t.set("title", f.ws.text(p))?;
            let blocks = lua.create_table()?;
            for (i, b) in f.ws.children(p).into_iter().enumerate() {
                let e = lua.create_table()?;
                e.set("kind", kind_name(&f.ws.kind(b)))?;
                e.set("text", f.ws.text(b))?;
                blocks.set(i + 1, e)?;
            }
            t.set("blocks", blocks)?;
            Ok(Value::Table(t))
        })?)?;

        // zett.page.open(title): opens it, making it when there is none
        let st = self.st.clone();
        page.set("open", lua.create_function(move |_, title: String| {
            let mut s = st.borrow_mut();
            let folder = s.folder.clone().ok_or_else(|| lerr("no workspace"))?;
            let mut f = folder.borrow_mut();
            let found = f.ws.pages().into_iter().find(|p| f.ws.text(*p).eq_ignore_ascii_case(&title));
            let p = match found {
                Some(p) => p,
                None => {
                    let p = f.new_page(&title).map_err(lerr)?;
                    f.ws.append(p, Kind::Paragraph, "").map_err(lerr)?;
                    s.dirty.insert(p);
                    p
                }
            };
            s.current = Some(p);
            s.effects.push(Effect::Open(p));
            Ok(())
        })?)?;

        fn with_current<R>(st: &Rc<RefCell<State>>, f: impl FnOnce(&mut State, &mut Folder, BlockId) -> mlua::Result<R>) -> mlua::Result<R> {
            let mut s = st.borrow_mut();
            let folder = s.folder.clone().ok_or_else(|| lerr("no workspace"))?;
            let p = s.current.ok_or_else(|| lerr("no page is open"))?;
            let mut fo = folder.borrow_mut();
            let r = f(&mut s, &mut fo, p)?;
            s.dirty.insert(p);
            Ok(r)
        }

        // zett.page.add(kind, text [, at]): a block (at: 1-based position; default the end) → its index
        let st = self.st.clone();
        page.set("add", lua.create_function(move |_, (kind, text, at): (String, String, Option<usize>)| {
            let k = kind_from(&kind).ok_or_else(|| lerr(format!("no block kind {kind:?}")))?;
            with_current(&st, |s, f, p| {
                let n = f.ws.children(p).len();
                let i = at.map(|a| a.saturating_sub(1).min(n)).unwrap_or(n);
                f.ws.insert(p, i, k, &text).map_err(lerr)?;
                s.effects.push(Effect::Refresh);
                Ok(i + 1)
            })
        })?)?;

        // zett.page.set(i, text)
        let st = self.st.clone();
        page.set("set", lua.create_function(move |_, (i, text): (usize, String)| {
            with_current(&st, |s, f, p| {
                let b = *f.ws.children(p).get(i.wrapping_sub(1)).ok_or_else(|| lerr(format!("no block {i}")))?;
                f.ws.set_text(b, &text).map_err(lerr)?;
                s.effects.push(Effect::Refresh);
                Ok(())
            })
        })?)?;

        // zett.page.kind(i, kind)
        let st = self.st.clone();
        page.set("kind", lua.create_function(move |_, (i, kind): (usize, String)| {
            let k = kind_from(&kind).ok_or_else(|| lerr(format!("no block kind {kind:?}")))?;
            with_current(&st, |s, f, p| {
                let b = *f.ws.children(p).get(i.wrapping_sub(1)).ok_or_else(|| lerr(format!("no block {i}")))?;
                f.ws.set_kind(b, &k).map_err(lerr)?;
                s.effects.push(Effect::Refresh);
                Ok(())
            })
        })?)?;

        // zett.page.remove(i)
        let st = self.st.clone();
        page.set("remove", lua.create_function(move |_, i: usize| {
            with_current(&st, |s, f, p| {
                let b = *f.ws.children(p).get(i.wrapping_sub(1)).ok_or_else(|| lerr(format!("no block {i}")))?;
                f.ws.remove(b).map_err(lerr)?;
                s.effects.push(Effect::Refresh);
                Ok(())
            })
        })?)?;
        z.set("page", page)?;

        lua.globals().set("zett", z)?;
        // print() goes to the host's notices, not to a terminal nobody sees
        let st = self.st.clone();
        lua.globals().set("print", lua.create_function(move |_, args: mlua::Variadic<Value>| {
            let line = args.iter().map(show).collect::<Vec<_>>().join("\t");
            st.borrow_mut().effects.push(Effect::Notify(line));
            Ok(())
        })?)?;
        Ok(())
    }
}

/// A Lua value as text (for `:lua` results and print).
fn show(v: &Value) -> String {
    match v {
        Value::Nil => "nil".into(),
        Value::Boolean(b) => b.to_string(),
        Value::Integer(i) => i.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.to_string_lossy(),
        Value::Table(t) => {
            let mut parts = Vec::new();
            for pair in t.clone().pairs::<Value, Value>().flatten().take(50) {
                let (k, v) = pair;
                parts.push(match k {
                    Value::Integer(_) => show(&v),
                    k => format!("{} = {}", show(&k), show(&v)),
                });
            }
            format!("{{ {} }}", parts.join(", "))
        }
        Value::Function(_) => "<function>".into(),
        other => format!("<{}>", other.type_name()),
    }
}

#[cfg(test)]
mod tests;
