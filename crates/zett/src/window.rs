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
        /// every open to-do in the workspace: [{page, title, block, text}]
        #[qinvokable]
        fn todos(self: &Zett) -> QString;
        /// the workspace folder as a tree with git status: [{path, rel, name, depth, dir, git}]
        #[qinvokable]
        fn files(self: &Zett) -> QString;
        /// a file's text for the file view (at most 1 MB)
        /// search the workspace's files (not the pages): opts {regex, case, word}; [{path, rel, line, col, len, text}]
        #[qinvokable]
        #[cxx_name = "grepFiles"]
        fn grep_files(self: &Zett, query: &QString, opts: &QString) -> QString;
        /// replace in them (rels: a JSON list of relative paths, all when empty); how many places changed
        #[qinvokable]
        #[cxx_name = "replaceFiles"]
        fn replace_files(self: &Zett, query: &QString, with: &QString, opts: &QString, rels: &QString) -> i32;
        /// a file's lines changed since the last commit (git diff -U0): [{line, count, kind: added|changed|removed}], 0-based
        #[qinvokable]
        #[cxx_name = "gitLines"]
        fn git_lines(self: &Zett, path: &QString) -> QString;
        /// git in the workspace (Source Control): args as a JSON array; {code, out} with stdout and stderr
        #[qinvokable]
        fn git(self: &Zett, args: &QString) -> QString;
        /// the language server for a file opened / changed in the file editor (one server per language, started once)
        #[qinvokable]
        #[cxx_name = "lspOpen"]
        fn lsp_open(self: Pin<&mut Zett>, path: &QString, text: &QString) -> bool;
        #[qinvokable]
        #[cxx_name = "lspChange"]
        fn lsp_change(self: &Zett, path: &QString, text: &QString);
        /// completion, hover, definition, references, symbols, format at a 0-based line and column
        #[qinvokable]
        #[cxx_name = "lspAsk"]
        fn lsp_ask(self: &Zett, kind: &QString, path: &QString, line: i32, col: i32);
        /// what a server said, as JSON: {type: diagnostics|completion|hover|definition|references|symbols|edits|status, …}
        #[qsignal]
        #[cxx_name = "lspEvent"]
        fn lsp_event(self: Pin<&mut Zett>, json: QString);
        #[qinvokable]
        #[cxx_name = "readFile"]
        fn read_file(self: &Zett, path: &QString) -> QString;
        /// a file of the workspace written (the file view saving); false outside the workspace or on an error
        #[qinvokable]
        #[cxx_name = "writeFile"]
        fn write_file(self: &Zett, path: &QString, text: &QString) -> bool;
        /// an environment variable (empty when unset), for views that need HOME or XDG_RUNTIME_DIR
        #[qinvokable]
        fn env(self: &Zett, name: &QString) -> QString;
        /// start a program detached: argv as a JSON array; false when it could not start
        #[qinvokable]
        fn spawn(self: &Zett, argv: &QString) -> bool;
        /// every #tag in the workspace with how many blocks carry it: [{tag, count}]
        #[qinvokable]
        fn tags(self: &Zett) -> QString;
        /// the open page's properties (its front matter besides the title): [[key, value]]
        #[qinvokable]
        #[cxx_name = "pageProps"]
        fn page_props(self: &Zett) -> QString;
        /// sets one (an empty value removes it) and saves
        #[qinvokable]
        #[cxx_name = "setProp"]
        fn set_prop(self: Pin<&mut Zett>, key: &QString, value: &QString);
        /// every block of every page whose text has all words of the query (case-insensitive), pages whose
        /// title has them first: [{page, title, block, kind, text, titleHit}], at most 300
        #[qinvokable]
        fn search(self: &Zett, query: &QString) -> QString;
        /// moves block `from` of the open page to position `to` (among the page's blocks); false if it cannot
        #[qinvokable]
        #[cxx_name = "moveBlock"]
        fn move_block(self: Pin<&mut Zett>, from: i32, to: i32) -> bool;
        /// a list item under the one above it / back out one level (Tab / Shift+Tab); false when it cannot
        #[qinvokable]
        fn indent(self: Pin<&mut Zett>, block: i32) -> bool;
        #[qinvokable]
        fn outdent(self: Pin<&mut Zett>, block: i32) -> bool;
        /// the last change taken back / done again, across blocks; false when there is nothing
        #[qinvokable]
        fn undo(self: Pin<&mut Zett>) -> bool;
        #[qinvokable]
        fn redo(self: Pin<&mut Zett>) -> bool;
        /// another page's blocks (the side view): [{kind, level, checked, lang, text}]
        #[qinvokable]
        #[cxx_name = "pageBlocks"]
        fn page_blocks(self: &Zett, index: i32) -> QString;
        /// the window's own settings (~/.config/zett/gui.json), "{}" when there are none
        #[qinvokable]
        fn settings(self: &Zett) -> QString;
        #[qinvokable]
        #[cxx_name = "saveSettings"]
        fn save_settings(self: &Zett, json: &QString) -> bool;
        /// the system share sheet (~/.scripts/apps/share/share) with these items, one per line
        #[qinvokable]
        #[cxx_name = "shareItems"]
        fn share_items(self: &Zett, items: &QString) -> bool;
        /// moves a file or folder into a folder of the workspace (drag and drop); the page list follows
        #[qinvokable]
        #[cxx_name = "movePath"]
        fn move_path(self: Pin<&mut Zett>, from: &QString, to_dir: &QString) -> bool;
        /// copies dropped files (file:// URLs, one per line) into a folder of the workspace; how many
        /// an image on the clipboard saved into the workspace (assets/paste-TIME.png); its path from the workspace, or ""
        #[qinvokable]
        #[cxx_name = "pasteImage"]
        fn paste_image(self: &Zett) -> QString;
        #[qinvokable]
        #[cxx_name = "importFiles"]
        fn import_files(self: Pin<&mut Zett>, urls: &QString, to_dir: &QString) -> i32;
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
        /// the drawer: runs a shell command in the workspace on its own thread; output comes as runOutput
        #[qinvokable]
        #[cxx_name = "runShell"]
        fn run_shell(self: Pin<&mut Zett>, id: i32, cmd: &QString) -> bool;
        /// stops the drawer's running command (and what it started)
        #[qinvokable]
        #[cxx_name = "stopShell"]
        fn stop_shell(self: Pin<&mut Zett>);
        /// a real terminal (a shell in a pty) in folder; its id, -1 on failure. palette: 18 hex colours (16, fg, bg)
        #[qinvokable]
        #[cxx_name = "termStart"]
        fn term_start(self: &Zett, cwd: &QString, cols: i32, rows: i32, palette: &QString) -> i32;
        /// a key for it: Qt key code, its text, modifiers
        #[qinvokable]
        #[cxx_name = "termKey"]
        fn term_key(self: &Zett, id: i32, key: i32, text: &QString, mods: i32);
        /// text for it as typed (paste)
        #[qinvokable]
        #[cxx_name = "termWrite"]
        fn term_write(self: &Zett, id: i32, text: &QString);
        #[qinvokable]
        #[cxx_name = "termResize"]
        fn term_resize(self: &Zett, id: i32, cols: i32, rows: i32);
        /// scroll back (positive) or forward by lines
        #[qinvokable]
        #[cxx_name = "termScroll"]
        fn term_scroll(self: &Zett, id: i32, lines: i32);
        /// changes so far: draw again when it moved (-1: gone)
        #[qinvokable]
        #[cxx_name = "termRev"]
        fn term_rev(self: &Zett, id: i32) -> i32;
        /// the screen as JSON: rows of rich text, cursor, title, dead
        #[qinvokable]
        #[cxx_name = "termScreen"]
        fn term_screen(self: &Zett, id: i32) -> QString;
        #[qinvokable]
        #[cxx_name = "termClose"]
        fn term_close(self: &Zett, id: i32);
        /// a piece of a command's output; done with its exit code at the end
        #[qsignal]
        #[cxx_name = "runOutput"]
        fn run_output(self: Pin<&mut Zett>, id: i32, chunk: QString, done: bool, code: i32);
    }
    impl cxx_qt::Threading for Zett {}
}

use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use zett_core::folder::Folder;
use zett_core::{markdown, BlockId, Kind};

pub struct ZettRust {
    folder: Option<Folder>,
    /// undo / redo across blocks and pages (Loro's, edits within 0.8 s are one step)
    undo: Option<loro::UndoManager>,
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
            undo: None,
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
        // from the start, so the first edit can be taken back
        z.undo = z.folder.as_ref().map(|f| f.ws.undo_manager(800));
        z
    }
}

/// A page's blocks in reading order with their depth: nested list items follow their parent.
fn flat(ws: &zett_core::Workspace, page: BlockId) -> Vec<(BlockId, usize)> {
    fn walk(ws: &zett_core::Workspace, b: BlockId, depth: usize, out: &mut Vec<(BlockId, usize)>) {
        for c in ws.children(b) {
            out.push((c, depth));
            walk(ws, c, depth + 1, out);
        }
    }
    let mut out = Vec::new();
    walk(ws, page, 0, &mut out);
    out
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
        // "code:lua": a code block in that language
        k if k.starts_with("code:") => Kind::Code { lang: k[5..].trim().to_string() },
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
        let list: Vec<_> = pages.iter().map(|p| serde_json::json!({ "title": f.ws.text(*p), "blocks": f.ws.children(*p).len(), "path": f.path_of(*p).map(|x| x.to_string_lossy().to_string()).unwrap_or_default(), "modified": f.path_of(*p).and_then(|x| std::fs::metadata(x).ok()?.modified().ok()?.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0) })).collect();
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
        let list: Vec<_> = flat(&f.ws, p).into_iter().map(|(b, depth)| {
            let mut v = kind_json(&f.ws.kind(b));
            v["text"] = serde_json::Value::String(f.ws.text(b));
            v["depth"] = depth.into();
            v
        }).collect();
        let title = QString::from(&f.ws.text(p));
        let blocks = QString::from(&serde_json::Value::Array(list).to_string());
        // backlinks: blocks of other pages that mention [[this page]]
        let needle = format!("[[{}]]", f.ws.text(p).to_lowercase());
        let mut back = Vec::new();
        for q in f.ws.pages() {
            if q == p { continue; }
            for (b, _) in flat(&f.ws, q) {
                let t = f.ws.text(b);
                if t.to_lowercase().contains(&needle) {
                    back.push(serde_json::json!({ "title": f.ws.text(q), "text": t }));
                }
            }
        }
        Some((blocks, title, QString::from(&serde_json::Value::Array(back).to_string())))
    }

    /// block `i` of the open page, counting nested ones in reading order (as QML lists them)
    fn block(&self, i: i32) -> Option<BlockId> {
        let (f, p) = (self.folder.as_ref()?, self.page()?);
        flat(&f.ws, p).get(usize::try_from(i).ok()?).map(|(b, _)| *b)
    }

    fn save(&self) {
        if let (Some(f), Some(p)) = (&self.folder, self.page()) {
            f.ws.commit();
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
            // after block `after`: its sibling right below it (at the same depth); else at the end of the page
            let list = flat(&f.ws, p);
            let made = match usize::try_from(after).ok().and_then(|i| list.get(i)).copied() {
                Some((b, _)) => {
                    let parent = f.ws.parent(b).unwrap_or(p);
                    let at = f.ws.children(parent).iter().position(|x| *x == b).map(|i| i + 1).unwrap_or(0);
                    f.ws.insert(parent, at, kind_from(&kind.to_string()), "")
                }
                None => f.ws.insert(p, f.ws.children(p).len(), kind_from(&kind.to_string()), ""),
            };
            r.save();
            match made { Ok(id) => flat(&f.ws, p).iter().position(|(x, _)| *x == id).map(|i| i as i32).unwrap_or(-1), Err(_) => -1 }
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

    fn todos(&self) -> QString {
        let r = self.rust();
        let Some(f) = &r.folder else { return QString::from("[]") };
        let mut out = Vec::new();
        for (pi, p) in r.order.iter().enumerate() {
            for (bi, (b, _)) in flat(&f.ws, *p).into_iter().enumerate() {
                if matches!(f.ws.kind(b), Kind::Todo { checked: false }) {
                    out.push(serde_json::json!({ "page": pi, "title": f.ws.text(*p), "block": bi, "text": f.ws.text(b) }));
                }
            }
        }
        QString::from(&serde_json::Value::Array(out).to_string())
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
/// the code colours: the terminal's languages (syntect) and a theme made from the window's colours; built once
pub(crate) fn code_syntax() -> (&'static syntect::parsing::SyntaxSet, &'static syntect::highlighting::Theme) {
    use std::sync::OnceLock;
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
    (ss, th)
}

fn highlight_html(lang: &str, code: &str) -> String {
    use syntect::easy::HighlightLines;
    use syntect::highlighting::FontStyle;
    let (ss, th) = code_syntax();
    let syn = (!lang.is_empty()).then(|| ss.find_syntax_by_token(lang)).flatten().unwrap_or_else(|| ss.find_syntax_plain_text());
    let mut h = HighlightLines::new(syn, th);
    // a div with <br>, not <pre>: <pre> spaces lines a pixel tighter than the editor laid over it, so the cursor drifted
    let mut out = String::from("<div style=\"white-space:pre\">");
    for line in syntect::util::LinesWithEndings::from(code) {
        let Ok(parts) = h.highlight_line(line, ss) else { out += &esc(line).replace('\n', "<br>"); continue };
        for (st, s) in parts {
            let f = st.foreground;
            let (b, i) = (st.font_style.contains(FontStyle::BOLD), st.font_style.contains(FontStyle::ITALIC));
            out += &format!("<span style=\"color:#{:02x}{:02x}{:02x}\">{}{}{}{}{}</span>", f.r, f.g, f.b, if b { "<b>" } else { "" }, if i { "<i>" } else { "" }, esc(s).replace('\n', "<br>"), if i { "</i>" } else { "" }, if b { "</b>" } else { "" });
        }
    }
    out + "</div>"
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

// ── the sidebar's Files and Tags ──
impl qobject::Zett {
    fn files(&self) -> QString {
        let root = std::path::PathBuf::from(self.dir.to_string());
        // git status by path, relative to the repo's top
        let mut git = std::collections::HashMap::new();
        if let Ok(o) = std::process::Command::new("git").arg("-C").arg(&root).args(["status", "--porcelain=v1", "-z", "--untracked-files=all", "."]).stderr(std::process::Stdio::null()).output() {
            let top = std::process::Command::new("git").arg("-C").arg(&root).args(["rev-parse", "--show-toplevel"]).output().ok()
                .map(|t| std::path::PathBuf::from(String::from_utf8_lossy(&t.stdout).trim())).unwrap_or_else(|| root.clone());
            for e in o.stdout.split(|b| *b == 0).filter(|e| e.len() > 3) {
                let s = String::from_utf8_lossy(e);
                let code = s[..2].trim().chars().next().unwrap_or(' ');
                git.insert(top.join(&s[3..]), if code == '?' { "U".to_string() } else { code.to_string() });
            }
        }
        let mut out = Vec::new();
        fn walk(d: &std::path::Path, root: &std::path::Path, depth: usize, git: &std::collections::HashMap<std::path::PathBuf, String>, out: &mut Vec<serde_json::Value>) {
            let Ok(rd) = std::fs::read_dir(d) else { return };
            let mut es: Vec<_> = rd.flatten().map(|e| e.path()).filter(|p| {
                let n = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                !n.starts_with('.') && !matches!(n.as_str(), "target" | "node_modules" | "build" | "__pycache__")
            }).collect();
            es.sort_by_key(|p| (!p.is_dir(), p.file_name().map(|n| n.to_string_lossy().to_lowercase())));
            for p in es {
                if out.len() >= 400 { return }
                let is_dir = p.is_dir();
                // a folder shows the status of anything changed inside it
                let g = if is_dir { git.keys().any(|k| k.starts_with(&p)).then(|| "•".to_string()).unwrap_or_default() } else { git.get(&p).cloned().unwrap_or_default() };
                out.push(serde_json::json!({
                    "path": p.to_string_lossy(), "rel": p.strip_prefix(root).unwrap_or(&p).to_string_lossy(),
                    "name": p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(),
                    "depth": depth, "dir": is_dir, "git": g,
                }));
                if is_dir && depth < 6 { walk(&p, root, depth + 1, git, out) }
            }
        }
        walk(&root, &root, 0, &git, &mut out);
        QString::from(&serde_json::Value::Array(out).to_string())
    }

    fn env(&self, name: &QString) -> QString {
        QString::from(&std::env::var(name.to_string()).unwrap_or_default())
    }

    fn spawn(&self, argv: &QString) -> bool {
        let v: Vec<String> = serde_json::from_str(&argv.to_string()).unwrap_or_default();
        let Some((cmd, args)) = v.split_first() else { return false };
        match std::process::Command::new(cmd).args(args).stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).spawn() {
            // reaped on a thread, so no zombie waits for zett to exit
            Ok(mut c) => { std::thread::spawn(move || { let _ = c.wait(); }); true }
            Err(_) => false,
        }
    }

    fn grep_files(&self, query: &QString, opts: &QString) -> QString {
        let o: serde_json::Value = serde_json::from_str(&opts.to_string()).unwrap_or_default();
        QString::from(&crate::grep::search(&workspace_dir(), &query.to_string(), &o).to_string())
    }

    fn replace_files(&self, query: &QString, with: &QString, opts: &QString, rels: &QString) -> i32 {
        let o: serde_json::Value = serde_json::from_str(&opts.to_string()).unwrap_or_default();
        let only: Vec<String> = serde_json::from_str(&rels.to_string()).unwrap_or_default();
        crate::grep::replace(&workspace_dir(), &query.to_string(), &with.to_string(), &o, &only) as i32
    }

    fn term_start(&self, cwd: &QString, cols: i32, rows: i32, palette: &QString) -> i32 { crate::term::start(&cwd.to_string(), cols.clamp(2, 1000) as u16, rows.clamp(1, 500) as u16, &palette.to_string()) }
    fn term_key(&self, id: i32, key: i32, text: &QString, mods: i32) { crate::term::key(id, key, &text.to_string(), mods) }
    fn term_write(&self, id: i32, text: &QString) { crate::term::write(id, text.to_string().as_bytes()) }
    fn term_resize(&self, id: i32, cols: i32, rows: i32) { crate::term::resize(id, cols.clamp(2, 1000) as u16, rows.clamp(1, 500) as u16) }
    fn term_scroll(&self, id: i32, lines: i32) { crate::term::scroll(id, lines) }
    fn term_rev(&self, id: i32) -> i32 { crate::term::rev(id) }
    fn term_screen(&self, id: i32) -> QString { QString::from(&crate::term::screen(id)) }
    fn term_close(&self, id: i32) { crate::term::close(id) }

    fn git(&self, args: &QString) -> QString {
        let args: Vec<String> = serde_json::from_str(&args.to_string()).unwrap_or_default();
        let out = std::process::Command::new("git").arg("-C").arg(workspace_dir()).args(&args).stdin(std::process::Stdio::null()).output();
        let v = match out {
            Ok(o) => serde_json::json!({ "code": o.status.code().unwrap_or(-1), "out": format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)) }),
            Err(e) => serde_json::json!({ "code": -1, "out": e.to_string() }),
        };
        QString::from(&v.to_string())
    }

    fn git_lines(&self, path: &QString) -> QString {
        let p = std::path::PathBuf::from(path.to_string());
        let Some(dir) = p.parent() else { return QString::from("[]") };
        let out = std::process::Command::new("git").arg("-C").arg(dir).args(["diff", "-U0", "--no-color", "HEAD", "--"]).arg(&p).output();
        let text = out.map(|o| String::from_utf8_lossy(&o.stdout).to_string()).unwrap_or_default();
        // an untracked file: every line new
        let tracked = std::process::Command::new("git").arg("-C").arg(dir).args(["ls-files", "--error-unmatch"]).arg(&p).output().is_ok_and(|o| o.status.success());
        if !tracked { let n = std::fs::read_to_string(&p).map(|s| s.lines().count()).unwrap_or(0); return QString::from(&serde_json::json!([{ "line": 0, "count": n, "kind": "added" }]).to_string()) }
        let mut v = Vec::new();
        for l in text.lines().filter(|l| l.starts_with("@@")) {
            // @@ -a,b +c,d @@
            let nums = |s: &str| { let mut it = s[1..].split(','); let a: usize = it.next().and_then(|x| x.parse().ok()).unwrap_or(0); let b: usize = it.next().and_then(|x| x.parse().ok()).unwrap_or(1); (a, b) };
            let mut parts = l.split_whitespace().skip(1);
            let (Some(old), Some(new)) = (parts.next(), parts.next()) else { continue };
            let ((_, ob), (nc, nd)) = (nums(old), nums(new));
            let kind = if nd == 0 { "removed" } else if ob == 0 { "added" } else { "changed" };
            v.push(serde_json::json!({ "line": if nd == 0 { nc } else { nc.saturating_sub(1) }, "count": nd.max(1), "kind": kind }));
        }
        QString::from(&serde_json::Value::Array(v).to_string())
    }

    fn lsp_open(self: Pin<&mut Self>, path: &QString, text: &QString) -> bool {
        use cxx_qt::Threading;
        let qt = self.qt_thread();
        let on = move |j: String| { let _ = qt.queue(move |o| o.lsp_event(QString::from(&j))); };
        crate::lsp::open(std::path::Path::new(&path.to_string()), &text.to_string(), &workspace_dir(), on)
    }

    fn lsp_change(&self, path: &QString, text: &QString) {
        let (p, t) = (std::path::PathBuf::from(path.to_string()), text.to_string());
        crate::lsp::with(&p, |c| c.did_change(&p, &t));
    }

    fn lsp_ask(&self, kind: &QString, path: &QString, line: i32, col: i32) {
        let p = std::path::PathBuf::from(path.to_string());
        let (l, c) = (line.max(0) as usize, col.max(0) as usize);
        match kind.to_string().as_str() {
            "completion" => crate::lsp::with(&p, |cl| cl.completion(&p, l, c, None)),
            "hover" => crate::lsp::with(&p, |cl| cl.hover(&p, l, c)),
            "definition" => crate::lsp::with(&p, |cl| cl.definition(&p, l, c)),
            "references" => crate::lsp::with(&p, |cl| cl.references(&p, l, c)),
            "symbols" => crate::lsp::with(&p, |cl| cl.symbols(&p)),
            "format" => crate::lsp::with(&p, |cl| cl.format(&p, 4)),
            "save" => crate::lsp::with(&p, |cl| cl.did_save(&p)),
            // "rename:NEW": the symbol at the cursor renamed everywhere (the edits come back as an event)
            "signature" => crate::lsp::with(&p, |cl| cl.signature(&p, l, c)),
            // "actions:[diagnostics as the server sent them]": quick fixes and refactors at the cursor
            k if k.starts_with("actions:") => { let d = serde_json::from_str(&k[8..]).unwrap_or(serde_json::json!([])); crate::lsp::with(&p, |cl| cl.code_actions(&p, l, c, d)) }
            k if k.starts_with("rename:") => crate::lsp::with(&p, |cl| cl.rename(&p, l, c, &k[7..])),
            _ => {}
        }
    }

    fn write_file(&self, path: &QString, text: &QString) -> bool {
        let p = std::path::PathBuf::from(path.to_string());
        let root = std::fs::canonicalize(workspace_dir()).unwrap_or_else(|_| workspace_dir());
        let inside = p.parent().and_then(|d| std::fs::canonicalize(d).ok()).is_some_and(|d| d.starts_with(&root));
        inside && std::fs::write(&p, text.to_string()).is_ok()
    }

    fn read_file(&self, path: &QString) -> QString {
        let p = std::path::PathBuf::from(path.to_string());
        match std::fs::metadata(&p) {
            Ok(m) if m.len() > 1 << 20 => QString::from("(file larger than 1 MB, not shown)"),
            Ok(_) => QString::from(&std::fs::read(&p).map(|b| String::from_utf8_lossy(&b).to_string()).unwrap_or_default()),
            Err(e) => QString::from(&format!("({e})")),
        }
    }

    fn tags(&self) -> QString {
        let r = self.rust();
        let Some(f) = &r.folder else { return QString::from("[]") };
        let mut n: std::collections::BTreeMap<String, usize> = Default::default();
        for p in f.ws.pages() {
            for (b, _) in flat(&f.ws, p) {
                if matches!(f.ws.kind(b), Kind::Code { .. }) { continue }
                for w in f.ws.text(b).split(|c: char| c.is_whitespace() || c == ',' || c == '(' || c == ')') {
                    let Some(t) = w.strip_prefix('#') else { continue };
                    let t = t.trim_end_matches(|c: char| !c.is_alphanumeric());
                    if t.chars().next().is_some_and(|c| c.is_alphabetic()) && t.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '/') {
                        *n.entry(t.to_lowercase()).or_default() += 1;
                    }
                }
            }
        }
        let mut v: Vec<_> = n.into_iter().collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        QString::from(&serde_json::Value::Array(v.into_iter().map(|(t, c)| serde_json::json!({ "tag": t, "count": c })).collect()).to_string())
    }
}

// ── drag and drop ──
impl qobject::Zett {
    /// inside the workspace only, never onto itself or into its own subfolder
    fn inside(&self, p: &std::path::Path) -> bool {
        let root = std::path::PathBuf::from(self.dir.to_string());
        std::fs::canonicalize(p).ok().is_some_and(|c| std::fs::canonicalize(&root).ok().is_some_and(|r| c.starts_with(r)))
    }

    fn move_path(mut self: Pin<&mut Self>, from: &QString, to_dir: &QString) -> bool {
        let (from, dir) = (std::path::PathBuf::from(from.to_string()), std::path::PathBuf::from(to_dir.to_string()));
        let Some(name) = from.file_name() else { return false };
        let to = dir.join(name);
        if !self.inside(&from) || !self.inside(&dir) || !dir.is_dir() || to.exists() || dir.starts_with(&from) { return false }
        if std::fs::rename(&from, &to).is_err() { return false }
        self.as_mut().disk_changed();
        true
    }

    fn paste_image(&self) -> QString {
        let types = std::process::Command::new("wl-paste").arg("--list-types").output().map(|o| String::from_utf8_lossy(&o.stdout).to_string()).unwrap_or_default();
        let Some(ty) = ["image/png", "image/jpeg", "image/webp", "image/gif"].into_iter().find(|t| types.lines().any(|l| l.trim() == *t)) else { return QString::default() };
        let Ok(out) = std::process::Command::new("wl-paste").args(["--no-newline", "-t", ty]).output() else { return QString::default() };
        if out.stdout.is_empty() { return QString::default() }
        let ext = ty.trim_start_matches("image/").replace("jpeg", "jpg");
        let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let rel = format!("assets/paste-{secs}.{ext}");
        let path = workspace_dir().join(&rel);
        if let Some(d) = path.parent() { let _ = std::fs::create_dir_all(d); }
        if std::fs::write(&path, &out.stdout).is_err() { return QString::default() }
        QString::from(&rel)
    }

    fn import_files(mut self: Pin<&mut Self>, urls: &QString, to_dir: &QString) -> i32 {
        let dir = std::path::PathBuf::from(to_dir.to_string());
        if !self.inside(&dir) || !dir.is_dir() { return 0 }
        let mut n = 0;
        for u in urls.to_string().lines() {
            let Some(p) = u.trim().strip_prefix("file://") else { continue };
            let p = std::path::PathBuf::from(percent_decode(p));
            let Some(name) = p.file_name() else { continue };
            let mut to = dir.join(name);
            // never overwrite: name (2).ext
            let mut k = 2;
            while to.exists() {
                let stem = p.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
                let ext = p.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
                to = dir.join(format!("{stem} ({k}){ext}"));
                k += 1;
            }
            if p.is_file() && std::fs::copy(&p, &to).is_ok() { n += 1 }
        }
        if n > 0 { self.as_mut().disk_changed() }
        n
    }

    /// the folder changed under zett: pages in again, then QML
    fn disk_changed(mut self: Pin<&mut Self>) {
        if let Some(f) = self.as_mut().rust_mut().get_mut().folder.as_mut() { f.sync_from_disk(); }
        self.publish();
    }
}

fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() + 0 && i + 2 <= b.len() - 1 {
            if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) { out.push(v); i += 3; continue }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

// ── the system share sheet ──
impl qobject::Zett {
    fn share_items(&self, items: &QString) -> bool {
        let home = std::env::var_os("HOME").map(std::path::PathBuf::from).unwrap_or_default();
        let bin = home.join(".scripts/apps/share/share");
        let items: Vec<String> = items.to_string().lines().filter(|l| !l.is_empty()).map(String::from).collect();
        !items.is_empty() && bin.is_file() && std::process::Command::new(bin).args(items).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).spawn().is_ok()
    }
}

// ── the window's settings: ~/.config/zett/gui.json (the editor's own options stay in init.lua) ──
impl qobject::Zett {
    fn settings(&self) -> QString {
        let p = zett_script::config_dir().join("gui.json");
        QString::from(&std::fs::read_to_string(p).ok().filter(|s| serde_json::from_str::<serde_json::Value>(s).is_ok()).unwrap_or_else(|| "{}".into()))
    }

    fn save_settings(&self, json: &QString) -> bool {
        let s = json.to_string();
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&s) else { return false };
        let dir = zett_script::config_dir();
        let tmp = dir.join(".gui.json.tmp");
        std::fs::create_dir_all(&dir).is_ok()
            && std::fs::write(&tmp, serde_json::to_string_pretty(&v).unwrap_or(s)).is_ok()
            && std::fs::rename(&tmp, dir.join("gui.json")).is_ok()
    }
}

// ── the side view: any page's blocks, read-only ──
impl qobject::Zett {
    fn page_blocks(&self, index: i32) -> QString {
        let r = self.rust();
        let (Some(f), Some(p)) = (&r.folder, usize::try_from(index).ok().and_then(|i| r.order.get(i).copied())) else { return QString::from("[]") };
        let list: Vec<_> = flat(&f.ws, p).into_iter().map(|(b, depth)| {
            let mut v = kind_json(&f.ws.kind(b));
            v["text"] = serde_json::Value::String(f.ws.text(b));
            v["depth"] = depth.into();
            v
        }).collect();
        QString::from(&serde_json::Value::Array(list).to_string())
    }
}

// ── the drawer: shell commands in the workspace, off the UI thread ──
static RUNNING: std::sync::Mutex<Option<u32>> = std::sync::Mutex::new(None);

impl qobject::Zett {
    fn run_shell(self: Pin<&mut Self>, id: i32, cmd: &QString) -> bool {
        use cxx_qt::Threading;
        use std::io::Read;
        use std::os::unix::process::CommandExt;
        let (cmd, dir) = (cmd.to_string(), self.dir.to_string());
        let Ok(mut child) = std::process::Command::new("bash").arg("-c").arg(format!("{cmd} 2>&1"))
            .current_dir(&dir)
            .env_remove("ZEIT").env_remove("ZEIT_PANE")   // inside zeit, a child that sees these talks to it
            .env("TERM", "dumb").env("NO_COLOR", "1")
            .stdin(std::process::Stdio::null()).stdout(std::process::Stdio::piped())
            .process_group(0)                             // stop takes the whole group
            .spawn() else { return false };
        *RUNNING.lock().unwrap() = Some(child.id());
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            let mut out = child.stdout.take().expect("piped");
            let mut buf = [0u8; 8192];
            let mut carry = Vec::new();
            loop {
                let n = out.read(&mut buf).unwrap_or(0);
                if n == 0 { break }
                carry.extend_from_slice(&buf[..n]);
                // whole UTF-8 only; the rest waits for the next read
                let cut = match std::str::from_utf8(&carry) { Ok(_) => carry.len(), Err(e) => e.valid_up_to() };
                let chunk = String::from_utf8_lossy(&carry[..cut]).to_string();
                carry.drain(..cut);
                let _ = qt.queue(move |o| o.run_output(id, QString::from(&chunk), false, 0));
            }
            let code = child.wait().ok().and_then(|s| s.code()).unwrap_or(-1);
            *RUNNING.lock().unwrap() = None;
            let _ = qt.queue(move |o| o.run_output(id, QString::default(), true, code));
        });
        true
    }

    fn stop_shell(self: Pin<&mut Self>) {
        if let Some(pid) = *RUNNING.lock().unwrap() {
            let _ = std::process::Command::new("kill").arg("--").arg(format!("-{pid}")).status();
        }
    }
}

// ── undo / redo across blocks and pages ──
impl qobject::Zett {
    fn undo(self: Pin<&mut Self>) -> bool {
        self.undo_step(false)
    }

    fn redo(self: Pin<&mut Self>) -> bool {
        self.undo_step(true)
    }

    fn undo_step(mut self: Pin<&mut Self>, redo: bool) -> bool {
        let done = {
            let r = self.as_mut().rust_mut();
            let r = r.get_mut();
            let (Some(f), Some(u)) = (r.folder.as_ref(), r.undo.as_mut()) else { return false };
            f.ws.commit();   // what was typed last is a step of its own
            let ok = if redo { u.redo() } else { u.undo() };
            let ok = ok.unwrap_or(false);
            if ok {
                let _ = f.save_all();
            }
            ok
        };
        if done {
            self.publish();
        }
        done
    }
}

// ── moving blocks (Alt+Up/Down, the drag grip) ──
impl qobject::Zett {
    fn move_block(mut self: Pin<&mut Self>, from: i32, to: i32) -> bool {
        let ok = {
            let r = self.as_mut().rust_mut();
            let r = r.get_mut();
            let (Some(b), Some(p)) = (r.block(from), r.page()) else { return false };
            let Some(f) = r.folder.as_ref() else { return false };
            // `to` counts in reading order: the block takes that block's place among its siblings (with
            // what is nested in it); never into itself
            let Some(target) = r.block(to) else { return false };
            if to == from || target == b { return false }
            let mut up = f.ws.parent(target);
            while let Some(a) = up { if a == b { return false } up = f.ws.parent(a); }
            let parent = f.ws.parent(target).unwrap_or(p);
            let sibs = f.ws.children(parent);
            let at = sibs.iter().position(|x| *x == target).unwrap_or(0);
            let ok = f.ws.move_to(b, parent, at).is_ok();
            if ok { r.save() }
            ok
        };
        if ok { self.publish() }
        ok
    }
}

// ── nesting list items (Tab / Shift+Tab): in the file as indented Markdown lists ──
impl qobject::Zett {
    fn indent(mut self: Pin<&mut Self>, block: i32) -> bool {
        let ok = {
            let r = self.as_mut().rust_mut();
            let r = r.get_mut();
            let (Some(b), Some(p)) = (r.block(block), r.page()) else { return false };
            let Some(f) = r.folder.as_ref() else { return false };
            let list = |k: &Kind| matches!(k, Kind::Bullet | Kind::Todo { .. });
            let parent = f.ws.parent(b).unwrap_or(p);
            let sibs = f.ws.children(parent);
            let i = sibs.iter().position(|x| *x == b).unwrap_or(0);
            // under the list item just above it, at its end; only list items nest (Markdown lists)
            let Some(prev) = i.checked_sub(1).map(|j| sibs[j]) else { return false };
            if !list(&f.ws.kind(b)) || !list(&f.ws.kind(prev)) { return false }
            let ok = f.ws.move_to(b, prev, f.ws.children(prev).len()).is_ok();
            if ok { r.save() }
            ok
        };
        if ok { self.publish() }
        ok
    }

    fn outdent(mut self: Pin<&mut Self>, block: i32) -> bool {
        let ok = {
            let r = self.as_mut().rust_mut();
            let r = r.get_mut();
            let (Some(b), Some(p)) = (r.block(block), r.page()) else { return false };
            let Some(f) = r.folder.as_ref() else { return false };
            let Some(parent) = f.ws.parent(b).filter(|x| *x != p) else { return false };
            let grand = f.ws.parent(parent).unwrap_or(p);
            // right after its old parent; the items below it in that list come along, nested under it
            let at = f.ws.children(grand).iter().position(|x| *x == parent).map(|i| i + 1).unwrap_or(0);
            let below: Vec<_> = f.ws.children(parent).into_iter().skip_while(|x| *x != b).skip(1).collect();
            let mut ok = f.ws.move_to(b, grand, at).is_ok();
            for x in below { ok &= f.ws.move_to(x, b, f.ws.children(b).len()).is_ok(); }
            if ok { r.save() }
            ok
        };
        if ok { self.publish() }
        ok
    }
}

// ── search everything (Ctrl+Shift+F) ──
impl qobject::Zett {
    fn search(&self, query: &QString) -> QString {
        let r = self.rust();
        let Some(f) = &r.folder else { return QString::from("[]") };
        let words: Vec<String> = query.to_string().to_lowercase().split_whitespace().map(String::from).collect();
        if words.is_empty() { return QString::from("[]") }
        let has = |s: &str| { let l = s.to_lowercase(); words.iter().all(|w| l.contains(w.as_str())) };
        let mut out = Vec::new();
        for (pi, p) in r.order.iter().enumerate() {
            let title = f.ws.text(*p);
            if has(&title) {
                out.push(serde_json::json!({ "page": pi, "title": title, "block": -1, "kind": "page", "text": title, "titleHit": true }));
            }
            for (bi, (b, _)) in flat(&f.ws, *p).into_iter().enumerate() {
                let t = f.ws.text(b);
                if has(&t) {
                    out.push(serde_json::json!({ "page": pi, "title": title, "block": bi, "kind": kind_json(&f.ws.kind(b))["kind"], "text": t, "titleHit": false }));
                    if out.len() >= 300 { break }
                }
            }
            if out.len() >= 300 { break }
        }
        QString::from(&serde_json::Value::Array(out).to_string())
    }
}

// ── the page's properties (icon, tags, status… in its front matter) ──
impl qobject::Zett {
    fn page_props(&self) -> QString {
        let r = self.rust();
        let (Some(f), Some(p)) = (&r.folder, r.page()) else { return QString::from("[]") };
        QString::from(&serde_json::to_string(&f.ws.props(p)).unwrap_or_else(|_| "[]".into()))
    }

    fn set_prop(mut self: Pin<&mut Self>, key: &QString, value: &QString) {
        {
            let r = self.as_mut().rust_mut();
            if let (Some(f), Some(p)) = (r.folder.as_ref(), r.page()) {
                let _ = f.ws.set_prop(p, key.to_string().trim(), value.to_string().trim());
            }
            r.save();
        }
        self.publish();
    }
}
