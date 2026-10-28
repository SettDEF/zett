//! The window's language servers: one zett-lsp Client per server, started on the first file of its
//! language; what they say turned into JSON for QML (Zett::lspEvent).
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Mutex};
use zett_lsp::{Client, Event};

/// the running clients, by their command line
static CLIENTS: Mutex<Option<HashMap<String, Client>>> = Mutex::new(None);
/// which client a file belongs to
static FILES: Mutex<Option<HashMap<PathBuf, String>>> = Mutex::new(None);

/// The client for `path`, started (with `on_event` getting what it says) when it is the first of its language.
pub fn open(path: &Path, text: &str, root: &Path, on_event: impl Fn(String) + Send + 'static) -> bool {
    let Some((cmd, lang)) = zett_lsp::server_for(path) else { return false };
    let key = cmd.join(" ");
    let mut clients = CLIENTS.lock().unwrap();
    let all = clients.get_or_insert_with(HashMap::new);
    if !all.contains_key(&key) {
        let (tx, rx) = mpsc::channel::<Event>();
        let Ok(c) = Client::start(&cmd, root, tx) else { return false };
        std::thread::spawn(move || { for e in rx { on_event(event_json(&e).to_string()) } });
        all.insert(key.clone(), c);
    }
    let c = all.get_mut(&key).unwrap();
    if c.is_open(path) { c.did_change(path, text) } else { c.did_open(path, lang, text) }
    FILES.lock().unwrap().get_or_insert_with(HashMap::new).insert(path.to_path_buf(), key);
    true
}

/// `f` on the client of an opened file
pub fn with(path: &Path, f: impl FnOnce(&mut Client)) {
    let Some(key) = FILES.lock().unwrap().as_ref().and_then(|m| m.get(path).cloned()) else { return };
    if let Some(c) = CLIENTS.lock().unwrap().as_mut().and_then(|m| m.get_mut(&key)) { f(c) }
}

fn loc(l: &zett_lsp::Location) -> Value { json!({ "path": l.path, "line": l.line, "col": l.col }) }

/// An event as QML gets it
pub fn event_json(e: &Event) -> Value {
    match e {
        Event::Diagnostics(p, ds) => json!({ "type": "diagnostics", "path": p, "items": ds.iter().map(|d| json!({ "line": d.line, "col": d.col, "endLine": d.end_line, "endCol": d.end_col, "severity": d.severity, "message": d.message })).collect::<Vec<_>>() }),
        Event::Definition(ls) => json!({ "type": "definition", "items": ls.iter().map(loc).collect::<Vec<_>>() }),
        Event::References(ls) => json!({ "type": "references", "items": ls.iter().map(loc).collect::<Vec<_>>() }),
        Event::Hover(s) => json!({ "type": "hover", "text": s }),
        Event::Symbols(ss) => json!({ "type": "symbols", "items": ss.iter().map(|(n, k, l, c, d)| json!({ "name": n, "kind": k, "line": l, "col": c, "depth": d })).collect::<Vec<_>>() }),
        Event::Edits(fs) => json!({ "type": "edits", "files": fs.iter().map(|(p, es)| json!({ "path": p, "edits": es.iter().map(|(a, b, c, d, t)| json!([a, b, c, d, t])).collect::<Vec<_>>() })).collect::<Vec<_>>() }),
        Event::Completion(line, items, more) => json!({ "type": "completion", "line": line, "more": more, "items": items.iter().take(200).map(|i| json!({ "label": i.label, "insert": i.insert, "detail": i.detail, "kind": i.kind, "sort": i.sort, "filter": i.filter })).collect::<Vec<_>>() }),
        Event::Resolved(i) => json!({ "type": "resolved", "label": i.label, "detail": i.detail }),
        Event::Status(s) => json!({ "type": "status", "text": s }),
        Event::Progress { server, title, message, percent, done, .. } => json!({ "type": "progress", "server": server, "title": title, "message": message, "percent": percent, "done": done }),
        Event::Exited(t) => json!({ "type": "exited", "tag": t }),
    }
}
