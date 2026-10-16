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
        #[qinvokable]
        fn markdown(self: &Zett) -> QString;
        /// opens the page with this title, making it if there is none ([[links]], quick open)
        #[qinvokable]
        #[cxx_name = "openTitle"]
        fn open_title(self: Pin<&mut Zett>, title: &QString);
        /// takes in pages changed on disk by other programs (Neovim, git…)
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
    /// the block in Vim normal mode and its editor (undo and registers live here)
    vim: Option<(i32, zett_edit::Editor)>,
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
    fn refresh_pages(&mut self) {
        let Some(f) = &self.folder else { return };
        let mut pages: Vec<BlockId> = f.ws.pages();
        pages.sort_by_key(|p| f.ws.text(*p).to_lowercase());
        let list: Vec<_> = pages.iter().map(|p| serde_json::json!({ "title": f.ws.text(*p), "blocks": f.ws.children(*p).len() })).collect();
        self.order = pages;
        self.pages = QString::from(&serde_json::Value::Array(list).to_string());
    }

    fn page(&self) -> Option<BlockId> {
        self.order.get(usize::try_from(self.current).ok()?).copied()
    }

    fn refresh_page(&mut self) {
        let (Some(f), Some(p)) = (&self.folder, self.page()) else { return };
        let list: Vec<_> = f.ws.children(p).into_iter().map(|b| {
            let mut v = kind_json(&f.ws.kind(b));
            v["text"] = serde_json::Value::String(f.ws.text(b));
            v
        }).collect();
        self.title = QString::from(&f.ws.text(p));
        self.blocks = QString::from(&serde_json::Value::Array(list).to_string());
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
        self.backlinks = QString::from(&serde_json::Value::Array(back).to_string());
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
        let (blocks, title, pages, back) = {
            let r = self.as_mut().rust_mut();
            let r = r.get_mut();
            r.refresh_pages();
            r.refresh_page();
            (r.blocks.clone(), r.title.clone(), r.pages.clone(), r.backlinks.clone())
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
            r.refresh_pages();
            r.pages.clone()
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
            r.refresh_pages();
            r.order.iter().position(|x| *x == p).map(|i| i as i32).unwrap_or(0)
        };
        self.as_mut().set_current(idx);
        self.publish();
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
    let mode = match e.mode() { zett_edit::Mode::Normal => "normal", zett_edit::Mode::Insert => "insert" };
    QString::from(&serde_json::json!({ "text": e.text(), "cursor": cursor, "mode": mode }).to_string())
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
            r.refresh_pages();
            r.order.iter().position(|x| *x == page).map(|i| i as i32).unwrap_or(0)
        };
        self.as_mut().set_current(idx);
        self.publish();
    }
}
