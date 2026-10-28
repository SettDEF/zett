//! The window: a QML front end over the same workspace the CLI uses
//! (cxx-qt bridge). QML gets the pages and the open page's blocks as JSON and
//! calls back into Rust for every edit; each edit is saved to its `.md` file.

use std::pin::Pin;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, pages)]
        #[qproperty(QString, blocks)]
        #[qproperty(QString, title)]
        #[qproperty(QString, theme)]
        #[qproperty(QString, dir)]
        #[qproperty(i32, current)]
        #[qproperty(QString, shot)]
        #[qproperty(QString, backlinks)]
        #[qproperty(QString, shot_keys, cxx_name = "shotKeys")]
        #[qproperty(QString, shot_type, cxx_name = "shotType")]
        #[qproperty(QString, shot_engine, cxx_name = "shotEngine")]
        type Zett = super::ZettRust;

        /// opens page `index` of the list
        #[qinvokable]
        fn open(self: Pin<&mut Zett>, index: i32);
        #[qinvokable]
        #[cxx_name = "setText"]
        fn set_text(self: Pin<&mut Zett>, block: i32, text: &QString);
        #[qinvokable]
        #[cxx_name = "renamePage"]
        fn rename_page(self: Pin<&mut Zett>, text: &QString);
        #[qinvokable]
        fn toggle(self: Pin<&mut Zett>, block: i32);
        /// a new block after `after` (-1: at the end); kind: paragraph, heading, todo, bullet, quote, code
        #[qinvokable]
        #[cxx_name = "addBlock"]
        fn add_block(self: Pin<&mut Zett>, after: i32, kind: &QString) -> i32;
        #[qinvokable]
        #[cxx_name = "setKind"]
        fn set_kind(self: Pin<&mut Zett>, block: i32, kind: &QString);
        #[qinvokable]
        fn remove(self: Pin<&mut Zett>, block: i32);
        #[qinvokable]
        #[cxx_name = "newPage"]
        fn new_page(self: Pin<&mut Zett>, title: &QString);
        /// a page to the trash (gio trash; restorable), then the list again
        #[qinvokable]
        #[cxx_name = "trashPage"]
        fn trash_page(self: Pin<&mut Zett>, index: i32) -> bool;
        #[qinvokable]
        fn markdown(self: &Zett) -> QString;
        /// opens the page with this title, making it if there is none ([[links]], quick open)
        #[qinvokable]
        #[cxx_name = "openTitle"]
        fn open_title(self: Pin<&mut Zett>, title: &QString);
        /// takes in pages changed on disk by other programs (Neovim, git…)
        /// keys for the Lua engine (tokens joined by U+001F): {state: ran|pending|none, menu, ui, notes}
        #[qinvokable]
        #[cxx_name = "feedKeys"]
        fn feed_keys(self: Pin<&mut Zett>, keys: &QString) -> QString;
        /// the keys stopped (timeoutlen): run an exact mapping that waited for longer ones
        #[qinvokable]
        #[cxx_name = "keyTimeout"]
        fn key_timeout(self: Pin<&mut Zett>, keys: &QString) -> QString;
        #[qinvokable]
        #[cxx_name = "runCommand"]
        fn run_command(self: Pin<&mut Zett>, name: &QString) -> QString;
        #[qinvokable]
        #[cxx_name = "evalLua"]
        fn eval_lua(self: Pin<&mut Zett>, code: &QString) -> QString;
        #[qinvokable]
        #[cxx_name = "commandList"]
        fn command_list(self: Pin<&mut Zett>) -> QString;
        #[qinvokable]
        #[cxx_name = "bindingList"]
        fn binding_list(self: Pin<&mut Zett>) -> QString;
        #[qinvokable]
        #[cxx_name = "reloadConfig"]
        fn reload_config(self: Pin<&mut Zett>) -> QString;
        /// a code block as rich text, coloured by language from the theme
        #[qinvokable]
        fn highlight(self: &Zett, lang: &QString, code: &QString) -> QString;
        #[qinvokable]
        #[cxx_name = "configPath"]
        fn config_path(self: &Zett) -> QString;

        #[qinvokable]
        #[cxx_name = "checkDisk"]
        fn check_disk(self: Pin<&mut Zett>);
        /// Vim: normal mode on a block (Esc); returns {text, cursor, mode}
        #[qinvokable]
        #[cxx_name = "vimBegin"]
        fn vim_begin(self: Pin<&mut Zett>, block: i32, text: &QString, cursor: i32) -> QString;
        /// one key in Vim notation (<Esc>, <C-r>, x…); returns {text, cursor, mode}
        #[qinvokable]
        #[cxx_name = "vimKey"]
        fn vim_key(self: Pin<&mut Zett>, key: &QString) -> QString;
    }
}

use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use zett_core::folder::Folder;
use zett_core::{markdown, BlockId, Kind};

pub struct ZettRust {
    folder: Option<Folder>,
    order: Vec<BlockId>,
    pages: QString,
    blocks: QString,
    title: QString,
    theme: QString,
    dir: QString,
    current: i32,
    /// ZETT_SHOT: render, save a picture of the window there, quit (headless checks)
    shot: QString,
    /// pages that link to the open one: [{title, text}]
    backlinks: QString,
    /// ZETT_SHOT_KEYS: Vim keys fed into the second block before the capture (tests)
    shot_keys: QString,
    /// ZETT_SHOT_TYPE: text typed into a new block at the end before the capture (tests)
    shot_type: QString,
    /// ZETT_SHOT_ENGINE: keys (space-separated tokens) fed to the Lua engine before the capture (tests)
    shot_engine: QString,
    /// the block in Vim normal mode and its editor (undo and registers live here)
    vim: Option<(i32, zett_edit::Editor)>,
    /// the Lua engine (init.lua, plugins); made on first use
    engine: Option<zett_script::Engine>,
}

pub fn workspace_dir() -> std::path::PathBuf {
    std::env::var_os("ZETT_DIR").map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::env::var_os("HOME").map(std::path::PathBuf::from).unwrap_or_default().join("zett"))
}

fn theme_json() -> String {
    let home = std::env::var_os("HOME").map(std::path::PathBuf::from).unwrap_or_default();
    std::fs::read_to_string(home.join(".local/state/quickshell/user/generated/colors.json")).unwrap_or_else(|_| "{}".into())
}

impl Default for ZettRust {
    fn default() -> Self {
        let dir = workspace_dir();
        let folder = Folder::open(&dir).ok();
        let mut z = ZettRust {
            folder,
            order: Vec::new(),
            pages: QString::default(),
            blocks: QString::default(),
            title: QString::default(),
            theme: QString::from(&theme_json()),
            dir: QString::from(&dir.display().to_string()),
            current: -1,
            shot: QString::from(&std::env::var("ZETT_SHOT").unwrap_or_default()),
            backlinks: QString::from("[]"),
            vim: None,
            shot_keys: QString::from(&std::env::var("ZETT_SHOT_KEYS").unwrap_or_default()),
            shot_type: QString::from(&std::env::var("ZETT_SHOT_TYPE").unwrap_or_default()),
            shot_engine: QString::from(&std::env::var("ZETT_SHOT_ENGINE").unwrap_or_default()),
            engine: None,
        };
        z.refresh_pages();
        // open the page worked on last: the most recently changed file
        if let Some(f) = &z.folder {
            let newest = z.order.iter().enumerate().max_by_key(|(_, p)| {
                f.path_of(**p).and_then(|x| std::fs::metadata(x).ok()).and_then(|m| m.modified().ok())
            }).map(|(i, _)| i as i32);
            if let Some(i) = newest {
                z.current = i;
                z.refresh_page();
            }
        }
        z
    }
}

fn kind_json(k: &Kind) -> serde_json::Value {
    match k {
        Kind::Page => serde_json::json!({ "kind": "page" }),
        Kind::Paragraph => serde_json::json!({ "kind": "paragraph" }),
        Kind::Heading(l) => serde_json::json!({ "kind": "heading", "level": l }),
        Kind::Todo { checked } => serde_json::json!({ "kind": "todo", "checked": checked }),
        Kind::Bullet => serde_json::json!({ "kind": "bullet" }),
        Kind::Quote => serde_json::json!({ "kind": "quote" }),
        Kind::Code { lang } => serde_json::json!({ "kind": "code", "lang": lang }),
    }
}

fn kind_from(s: &str) -> Kind {
    match s {
        "h1" | "heading" => Kind::Heading(1),
        "h2" => Kind::Heading(2),
        "h3" => Kind::Heading(3),
        "todo" => Kind::Todo { checked: false },
        "bullet" => Kind::Bullet,
        "quote" => Kind::Quote,
        "code" => Kind::Code { lang: String::new() },
        _ => Kind::Paragraph,
    }
}

impl ZettRust {
    /// The page order (kept here) and the page list for QML (returned, not stored: the
    /// setter stores it, so QML hears about the change)
    fn compute_pages(&mut self) -> QString {
        let Some(f) = &self.folder else { return self.pages.clone() };
        let mut pages: Vec<BlockId> = f.ws.pages();
        pages.sort_by_key(|p| f.ws.text(*p).to_lowercase());
        let list: Vec<_> = pages.iter().map(|p| serde_json::json!({ "title": f.ws.text(*p), "blocks": f.ws.children(*p).len(), "path": f.path_of(*p).map(|x| x.to_string_lossy().to_string()).unwrap_or_default() })).collect();
        self.order = pages;
        QString::from(&serde_json::Value::Array(list).to_string())
    }

    fn refresh_pages(&mut self) {
        self.pages = self.compute_pages();
    }

    fn page(&self) -> Option<BlockId> {
        self.order.get(usize::try_from(self.current).ok()?).copied()
    }

    fn refresh_page(&mut self) {
        if let Some((b, t, bk)) = self.compute_page() {
            (self.blocks, self.title, self.backlinks) = (b, t, bk);
        }
    }

    /// The open page for QML: blocks, title, backlinks (returned, not stored; see compute_pages)
    fn compute_page(&self) -> Option<(QString, QString, QString)> {
        let (Some(f), Some(p)) = (&self.folder, self.page()) else { return None };
        let list: Vec<_> = f.ws.children(p).into_iter().map(|b| {
            let mut v = kind_json(&f.ws.kind(b));
            v["text"] = serde_json::Value::String(f.ws.text(b));
            v
        }).collect();
        let title = QString::from(&f.ws.text(p));
        let blocks = QString::from(&serde_json::Value::Array(list).to_string());
        // backlinks: blocks of other pages that mention [[this page]]
        let needle = format!("[[{}]]", f.ws.text(p).to_lowercase());
        let mut back = Vec::new();
        for q in f.ws.pages() {
            if q == p { continue; }
            for b in f.ws.children(q) {
                let t = f.ws.text(b);
                if t.to_lowercase().contains(&needle) {
                    back.push(serde_json::json!({ "title": f.ws.text(q), "text": t }));
                }
            }
        }
        Some((blocks, title, QString::from(&serde_json::Value::Array(back).to_string())))
    }

    fn block(&self, i: i32) -> Option<BlockId> {
        let (f, p) = (self.folder.as_ref()?, self.page()?);
        f.ws.children(p).get(usize::try_from(i).ok()?).copied()
    }

    fn save(&self) {
        if let (Some(f), Some(p)) = (&self.folder, self.page()) {
            if let Err(e) = f.save_page(p) {
                eprintln!("zett: could not save: {e}");
            }
        }
    }
}

impl qobject::Zett {
    fn publish(mut self: Pin<&mut Self>) {
        // computed, not stored: the setters store and tell QML (one given the value already
        // stored stays silent, which once kept the screen from updating at all)
        let (blocks, title, pages, back) = {
            let r = self.as_mut().rust_mut();
            let r = r.get_mut();
            let pages = r.compute_pages();
            let (b, t, bk) = r.compute_page().unwrap_or_else(|| (r.blocks.clone(), r.title.clone(), r.backlinks.clone()));
            (b, t, pages, bk)
        };
        self.as_mut().set_backlinks(back);
        self.as_mut().set_pages(pages);
        self.as_mut().set_title(title);
        self.as_mut().set_blocks(blocks);
    }

    fn open(mut self: Pin<&mut Self>, index: i32) {
        self.as_mut().set_current(index);
        self.publish();
    }

    fn set_text(mut self: Pin<&mut Self>, block: i32, text: &QString) {
        {
            let r = self.as_mut().rust_mut();
            if let (Some(b), Some(f)) = (r.block(block), r.folder.as_ref()) {
                if f.ws.text(b) == text.to_string() {
                    return;
                }
                let _ = f.ws.set_text(b, &text.to_string());
            }
            r.save();
        }
        // the text is already on screen: only the page list (block counts) changes
        let pages = {
            let r = self.as_mut().rust_mut();
            let r = r.get_mut();
            r.compute_pages()
        };
        self.as_mut().set_pages(pages);
    }

    fn rename_page(mut self: Pin<&mut Self>, text: &QString) {
        {
            let r = self.as_mut().rust_mut();
            if let (Some(f), Some(p)) = (r.folder.as_ref(), r.page()) {
                let _ = f.ws.set_text(p, &text.to_string());
            }
            r.save();
        }
        self.publish();
    }

    fn toggle(mut self: Pin<&mut Self>, block: i32) {
        {
            let r = self.as_mut().rust_mut();
            if let (Some(b), Some(f)) = (r.block(block), r.folder.as_ref()) {
                if let Kind::Todo { checked } = f.ws.kind(b) {
                    let _ = f.ws.set_kind(b, &Kind::Todo { checked: !checked });
                }
            }
            r.save();
        }
        self.publish();
    }

    fn add_block(mut self: Pin<&mut Self>, after: i32, kind: &QString) -> i32 {
        let at = {
            let r = self.as_mut().rust_mut();
            let (Some(f), Some(p)) = (r.folder.as_ref(), r.page()) else { return -1 };
            let n = f.ws.children(p).len();
            let at = if after < 0 { n } else { (after as usize + 1).min(n) };
            let _ = f.ws.insert(p, at, kind_from(&kind.to_string()), "");
            r.save();
            at as i32
        };
        self.publish();
        at
    }

    fn set_kind(mut self: Pin<&mut Self>, block: i32, kind: &QString) {
        {
            let r = self.as_mut().rust_mut();
            if let (Some(b), Some(f)) = (r.block(block), r.folder.as_ref()) {
                let _ = f.ws.set_kind(b, &kind_from(&kind.to_string()));
            }
            r.save();
        }
        self.publish();
    }

    fn remove(mut self: Pin<&mut Self>, block: i32) {
        {
            let r = self.as_mut().rust_mut();
            if let (Some(b), Some(f)) = (r.block(block), r.folder.as_ref()) {
                let _ = f.ws.remove(b);
            }
            r.save();
        }
        self.publish();
    }

    fn new_page(mut self: Pin<&mut Self>, title: &QString) {
        let idx = {
            let r = self.as_mut().rust_mut();
            let r = r.get_mut();
            let Some(f) = r.folder.as_mut() else { return };
            let Ok(p) = f.new_page(&title.to_string()) else { return };
            let _ = f.ws.append(p, Kind::Paragraph, "");
            let _ = f.save_page(p);
            let _ = r.compute_pages(); // the order only; publish() stores and announces
            r.order.iter().position(|x| *x == p).map(|i| i as i32).unwrap_or(0)
        };
        self.as_mut().set_current(idx);
        self.publish();
    }

    fn trash_page(mut self: Pin<&mut Self>, index: i32) -> bool {
        let ok = {
            let r = self.as_mut().rust_mut();
            let r = r.get_mut();
            let Some(p) = usize::try_from(index).ok().and_then(|i| r.order.get(i).copied()) else { return false };
            let Some(f) = r.folder.as_mut() else { return false };
            let Some(path) = f.path_of(p).map(|x| x.to_path_buf()) else { return false };
            let ok = std::process::Command::new("gio").arg("trash").arg(&path).stderr(std::process::Stdio::null()).status().is_ok_and(|s| s.success()) || {
                // no desktop trash there (tmpfs, no gio): the workspace's own .trash folder
                let bin = path.parent().unwrap_or(std::path::Path::new(".")).join(".trash");
                std::fs::create_dir_all(&bin).is_ok() && std::fs::rename(&path, bin.join(path.file_name().unwrap_or_default())).is_ok()
            };
            if ok { f.sync_from_disk(); }
            ok
        };
        if ok {
            let cur = self.current;
            self.as_mut().set_current(if cur >= index { (cur - 1).max(0) } else { cur });
            self.publish();
        }
        ok
    }

    fn markdown(&self) -> QString {
        let r = self.rust();
        match (&r.folder, r.page()) {
            (Some(f), Some(p)) => QString::from(&markdown::to_markdown(&f.ws, p)),
            _ => QString::default(),
        }
    }
}

pub fn run() {
    use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QUrl};
    let mut app = QGuiApplication::new();
    let mut engine = QQmlApplicationEngine::new();
    if let Some(engine) = engine.as_mut() {
        engine.load(&QUrl::from("qrc:/qt/qml/dev/zett/app/qml/Main.qml"));
    }
    if let Some(app) = app.as_mut() {
        app.exec();
    }
}

fn vim_state(e: &zett_edit::Editor) -> QString {
    let p = e.cursor();
    let cursor = e.buf.to_char(p);
    let mode = match e.mode() {
        zett_edit::Mode::Normal => "normal",
        zett_edit::Mode::Insert => "insert",
        zett_edit::Mode::Visual => "visual",
        zett_edit::Mode::VisualLine => "visual-line",
        zett_edit::Mode::VisualBlock => "visual-block",
    };
    let sel = e.selection_range().map(|(a, b)| serde_json::json!([a, b])).unwrap_or(serde_json::Value::Null);
    QString::from(&serde_json::json!({ "text": e.text(), "cursor": cursor, "mode": mode, "sel": sel }).to_string())
}

impl qobject::Zett {
    fn vim_begin(mut self: Pin<&mut Self>, block: i32, text: &QString, cursor: i32) -> QString {
        let mut e = zett_edit::Editor::from_str(&text.to_string());
        // Esc steps back onto the last typed character, as in Vim
        let at = e.buf.to_pos((cursor.max(0) as usize).saturating_sub(1));
        e.set_cursor(at);
        let state = vim_state(&e);
        self.as_mut().rust_mut().get_mut().vim = Some((block, e));
        state
    }

    fn vim_key(mut self: Pin<&mut Self>, key: &QString) -> QString {
        let r = self.as_mut().rust_mut();
        let r = r.get_mut();
        let Some((block, e)) = r.vim.as_mut() else { return QString::from("{}") };
        let before = e.text();
        e.feed(&key.to_string());
        let state = vim_state(e);
        let (block, after, insert) = (*block, e.text(), e.mode() == zett_edit::Mode::Insert);
        if after != before {
            if let (Some(b), Some(f)) = (r.block(block), r.folder.as_ref()) {
                let _ = f.ws.set_text(b, &after);
            }
            r.save();
        }
        if insert {
            // insert mode belongs to the text field again
            r.vim = None;
        }
        state
    }
}

impl qobject::Zett {
    fn check_disk(mut self: Pin<&mut Self>) {
        let changed = {
            let r = self.as_mut().rust_mut();
            let r = r.get_mut();
            // the block in Vim mode keeps its own text until it leaves; reload after
            if r.vim.is_some() {
                return;
            }
            r.folder.as_mut().is_some_and(|f| f.sync_from_disk())
        };
        if changed {
            self.publish();
        }
    }
}

impl qobject::Zett {
    fn open_title(mut self: Pin<&mut Self>, title: &QString) {
        let title = title.to_string().trim().to_string();
        if title.is_empty() {
            return;
        }
        let idx = {
            let r = self.as_mut().rust_mut();
            let r = r.get_mut();
            let Some(f) = r.folder.as_mut() else { return };
            // an existing page (title, any case), else a new one: links create pages
            let page = f.ws.pages().into_iter().find(|p| f.ws.text(*p).eq_ignore_ascii_case(&title));
            let page = match page {
                Some(p) => p,
                None => {
                    let Ok(p) = f.new_page(&title) else { return };
                    let _ = f.ws.append(p, Kind::Paragraph, "");
                    let _ = f.save_page(p);
                    p
                }
            };
            r.vim = None;
            let _ = r.compute_pages(); // the order only; publish() stores and announces
            r.order.iter().position(|x| *x == page).map(|i| i as i32).unwrap_or(0)
        };
        self.as_mut().set_current(idx);
        self.publish();
    }
}

// ── the Lua engine: keys, commands, :lua ──

use zett_script::{Effect, Engine, Feed};

impl qobject::Zett {
    /// Runs something on the engine with the workspace lent to it, then applies what it did:
    /// pages opened or changed here, UI requests and notices go back to QML as JSON.
    fn script(mut self: Pin<&mut Self>, call: impl FnOnce(&mut Engine) -> (serde_json::Value, Vec<Effect>)) -> QString {
        let (mut out, effects, errors) = {
            let r = self.as_mut().rust_mut();
            let r = r.get_mut();
            let fresh = r.engine.is_none();
            let mut eng = r.engine.take().unwrap_or_else(|| Engine::new(None));
            eng.set_current(r.page());
            let folder = r.folder.take();
            let run = |e: &mut Engine| {
                if fresh {
                    e.load_config(&zett_script::config_dir());
                }
                call(e)
            };
            let (folder, res) = match folder {
                Some(f) => {
                    let (f, res) = eng.lend(f, run);
                    (Some(f), res)
                }
                None => (None, run(&mut eng)),
            };
            r.folder = folder;
            let errors = std::mem::take(&mut eng.errors);
            r.engine = Some(eng);
            (res.0, res.1, errors)
        };
        let mut ui = Vec::new();
        let mut notes: Vec<String> = errors;
        let mut changed = false;
        for fx in effects {
            match fx {
                Effect::Open(p) => {
                    let idx = {
                        let r = self.as_mut().rust_mut();
                        let r = r.get_mut();
                        r.vim = None;
                        let _ = r.compute_pages(); // the order only; publish() stores and announces
                        r.order.iter().position(|x| *x == p).map(|i| i as i32)
                    };
                    if let Some(i) = idx {
                        self.as_mut().set_current(i);
                    }
                    changed = true;
                }
                Effect::Refresh => changed = true,
                Effect::Notify(m) => notes.push(m),
                Effect::Ui(name, arg) => ui.push(serde_json::json!({ "name": name, "arg": arg })),
                // the page view edits blocks with Vim; Emacs editing commands are the code editor's for now
                Effect::Edit(name, _) => notes.push(format!("{name}: only in the code editor (zett -t) for now")),
                Effect::Keys(k, _) => notes.push(format!("{k}: keys for the code editor (zett -t)")),
            }
        }
        if changed {
            self.as_mut().publish();
        }
        out["ui"] = serde_json::Value::Array(ui);
        out["notes"] = serde_json::Value::Array(notes.into_iter().map(serde_json::Value::String).collect());
        QString::from(&out.to_string())
    }

    fn feed_result(feed: Feed) -> (serde_json::Value, Vec<Effect>) {
        match feed {
            Feed::Ran(fx) => (serde_json::json!({ "state": "ran" }), fx),
            Feed::Pending(menu) => {
                let m: Vec<_> = menu.iter().map(|i| serde_json::json!({ "key": i.key, "desc": i.desc, "group": i.group })).collect();
                (serde_json::json!({ "state": "pending", "menu": m }), Vec::new())
            }
            Feed::None => (serde_json::json!({ "state": "none" }), Vec::new()),
        }
    }

    fn feed_keys(self: Pin<&mut Self>, keys: &QString) -> QString {
        // one spelling per key, as the maps use it (<CR> is <Enter>, a space is <Space>)
        let typed: Vec<String> = keys.to_string().split('\u{1f}').filter(|s| !s.is_empty()).flat_map(zett_script::keys).collect();
        self.script(|e| Self::feed_result(e.feed("n", &typed)))
    }

    fn key_timeout(self: Pin<&mut Self>, keys: &QString) -> QString {
        // one spelling per key, as the maps use it (<CR> is <Enter>, a space is <Space>)
        let typed: Vec<String> = keys.to_string().split('\u{1f}').filter(|s| !s.is_empty()).flat_map(zett_script::keys).collect();
        self.script(|e| Self::feed_result(e.timeout("n", &typed)))
    }

    fn run_command(self: Pin<&mut Self>, name: &QString) -> QString {
        let line = name.to_string();
        self.script(|e| {
            // `:name args` — a few built-in lines take the rest as text
            let (cmd, rest) = line.trim().split_once(' ').map(|(a, b)| (a.to_string(), b.trim().to_string())).unwrap_or((line.trim().to_string(), String::new()));
            let res = match cmd.as_str() {
                "lua" => e.eval(&rest).map(|(out, fx)| {
                    let mut fx = fx;
                    if out != "nil" {
                        fx.push(Effect::Notify(out));
                    }
                    fx
                }),
                "page" | "open" | "e" if !rest.is_empty() => e.eval(&format!("zett.page.open({rest:?})")).map(|(_, fx)| fx),
                "capture" if !rest.is_empty() => e.eval(&format!("zett.page.open(\"Inbox\"); zett.page.add(\"todo\", {rest:?})")).map(|(_, fx)| fx),
                _ => e.run(&cmd),
            };
            match res {
                Ok(fx) => (serde_json::json!({ "state": "ran" }), fx),
                Err(err) => (serde_json::json!({ "state": "error" }), vec![Effect::Notify(err)]),
            }
        })
    }

    fn eval_lua(self: Pin<&mut Self>, code: &QString) -> QString {
        let code = code.to_string();
        self.script(|e| match e.eval(&code) {
            Ok((out, fx)) => (serde_json::json!({ "state": "ran", "result": out }), fx),
            Err(err) => (serde_json::json!({ "state": "error" }), vec![Effect::Notify(err)]),
        })
    }

    fn command_list(self: Pin<&mut Self>) -> QString {
        self.script(|e| {
            let v: Vec<_> = e.commands().into_iter().map(|(n, d, s)| serde_json::json!({ "name": n, "desc": d, "source": s })).collect();
            (serde_json::json!({ "state": "ran", "commands": v }), Vec::new())
        })
    }

    fn binding_list(self: Pin<&mut Self>) -> QString {
        self.script(|e| {
            let leader = e.leader();
            let v: Vec<_> = e.bindings("n").into_iter().map(|(k, d)| {
                let shown = k.replace("<leader>", if leader == "<Space>" { "SPC " } else { &leader });
                serde_json::json!({ "keys": shown, "desc": d })
            }).collect();
            (serde_json::json!({ "state": "ran", "bindings": v }), Vec::new())
        })
    }

    fn reload_config(mut self: Pin<&mut Self>) -> QString {
        // a fresh engine: everything from defaults, plugins and init.lua again
        self.as_mut().rust_mut().get_mut().engine = None;
        self.script(|e| (serde_json::json!({ "state": "ran", "plugins": e.plugins.clone() }), vec![Effect::Notify(format!("config reloaded ({} plugins)", e.plugins.len()))]))
    }

    fn highlight(&self, lang: &QString, code: &QString) -> QString {
        QString::from(&highlight_html(&lang.to_string(), &code.to_string()))
    }

    fn config_path(&self) -> QString {
        QString::from(&zett_script::config_dir().join("init.lua").display().to_string())
    }
}

// ── code blocks: syntect (the terminal's languages) with colours from the theme ──
fn highlight_html(lang: &str, code: &str) -> String {
    use std::sync::OnceLock;
    use syntect::easy::HighlightLines;
    use syntect::highlighting::{Color, FontStyle, ScopeSelectors, StyleModifier, Theme, ThemeItem, ThemeSettings};
    use syntect::parsing::SyntaxSet;
    static SS: OnceLock<SyntaxSet> = OnceLock::new();
    static TH: OnceLock<Theme> = OnceLock::new();
    let ss = SS.get_or_init(zett_tui::syntax_set);
    let th = TH.get_or_init(|| {
        let c: serde_json::Value = serde_json::from_str(&theme_json()).unwrap_or_default();
        let col = |k: &str, d: &str| {
            let h = c.get(k).and_then(|v| v.as_str()).unwrap_or(d).trim_start_matches('#').to_string();
            let p = |i: usize| u8::from_str_radix(h.get(i..i + 2).unwrap_or("cc"), 16).unwrap_or(204);
            Color { r: p(0), g: p(2), b: p(4), a: 255 }
        };
        let item = |sel: &str, fg: Color, fs: Option<FontStyle>| ThemeItem {
            scope: sel.parse::<ScopeSelectors>().unwrap_or_default(),
            style: StyleModifier { foreground: Some(fg), background: None, font_style: fs },
        };
        Theme {
            settings: ThemeSettings { foreground: Some(col("on_surface", "#e3e2e5")), ..Default::default() },
            scopes: vec![
                item("comment, punctuation.definition.comment", col("outline", "#8c9199"), Some(FontStyle::ITALIC)),
                item("keyword, storage.modifier, storage.type, keyword.control, keyword.operator.word", col("primary", "#a8caeb"), None),
                item("entity.name.type, support.type, entity.name.class, support.class", col("secondary", "#bac8da"), None),
                item("string, string.quoted, punctuation.definition.string", col("tertiary", "#d4bee6"), None),
                item("constant, constant.numeric, constant.language, constant.character", col("error", "#ffb4ab"), None),
                item("entity.name.function, support.function, meta.function-call variable.function", col("on_surface", "#e3e2e5"), Some(FontStyle::BOLD)),
                item("entity.name.tag, meta.attribute, entity.other.attribute-name", col("primary", "#a8caeb"), None),
            ],
            ..Default::default()
        }
    });
    let syn = (!lang.is_empty()).then(|| ss.find_syntax_by_token(lang)).flatten().unwrap_or_else(|| ss.find_syntax_plain_text());
    let mut h = HighlightLines::new(syn, th);
    let mut out = String::from("<pre style=\"margin:0\">");
    for line in syntect::util::LinesWithEndings::from(code) {
        let Ok(parts) = h.highlight_line(line, ss) else { out += &esc(line); continue };
        for (st, s) in parts {
            let f = st.foreground;
            let (b, i) = (st.font_style.contains(FontStyle::BOLD), st.font_style.contains(FontStyle::ITALIC));
            out += &format!("<span style=\"color:#{:02x}{:02x}{:02x}\">{}{}{}{}{}</span>", f.r, f.g, f.b, if b { "<b>" } else { "" }, if i { "<i>" } else { "" }, esc(s), if i { "</i>" } else { "" }, if b { "</b>" } else { "" });
        }
    }
    out + "</pre>"
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}
