//! A language-server client: one server process per project and language.

use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};

/// The server for a file, by its extension: the command line and the language id.
pub fn server_for(path: &Path) -> Option<(Vec<String>, &'static str)> {
    let ext = path.extension()?.to_str()?;
    let (cmd, lang): (&[&str], &str) = match ext {
        "rs" => (&["rust-analyzer"], "rust"),
        "c" | "h" => (&["clangd"], "c"),
        "cpp" | "cc" | "hpp" | "cxx" => (&["clangd"], "cpp"),
        "ts" | "tsx" => (&["typescript-language-server", "--stdio"], "typescript"),
        "js" | "jsx" | "mjs" => (&["typescript-language-server", "--stdio"], "javascript"),
        "py" => (&["pyright-langserver", "--stdio"], "python"),
        "lua" => (&["lua-language-server"], "lua"),
        "sh" | "bash" => (&["bash-language-server", "start"], "shellscript"),
        "go" => (&["gopls"], "go"),
        "toml" => (&["taplo", "lsp", "stdio"], "toml"),
        "md" => (&["marksman", "server"], "markdown"),
        "qml" => (&["qmlls"], "qml"),
        _ => return None,
    };
    let found = std::env::var_os("PATH").is_some_and(|p| std::env::split_paths(&p).any(|d| d.join(cmd[0]).is_file()));
    found.then(|| (cmd.iter().map(|s| s.to_string()).collect(), lang))
}

pub fn uri(p: &Path) -> String {
    format!("file://{}", p.display())
}

pub fn path_of(uri: &str) -> PathBuf {
    let raw = uri.strip_prefix("file://").unwrap_or(uri);
    // %XX escapes (spaces and the like)
    let mut out = Vec::new();
    let b = raw.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let Ok(v) = u8::from_str_radix(&raw[i + 1..i + 3], 16) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    PathBuf::from(String::from_utf8_lossy(&out).to_string())
}

#[derive(Clone, Debug, PartialEq)]
pub struct Diag {
    pub line: usize,
    pub col: usize,
    pub end_line: usize,
    pub end_col: usize,
    /// 1 error, 2 warning, 3 information, 4 hint
    pub severity: u8,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Location {
    pub path: PathBuf,
    pub line: usize,
    pub col: usize,
}

/// What a server said, for the editor to act on.
#[derive(Debug)]
pub enum Event {
    Diagnostics(PathBuf, Vec<Diag>),
    /// an answer to `definition` (empty: none found)
    Definition(Vec<Location>),
    Hover(String),
    /// progress and log lines worth a glance
    Status(String),
    Exited(String),
}

enum Pending {
    Definition,
    Hover,
    Init,
}

pub struct Client {
    pub name: String,
    pub root: PathBuf,
    stdin: Arc<Mutex<ChildStdin>>,
    child: Child,
    next: i64,
    gate: Arc<Mutex<Gate>>,
    pending: Arc<Mutex<HashMap<i64, Pending>>>,
    /// column units the server counts in ("utf-32" means chars, as zett does)
    pub encoding: Arc<Mutex<String>>,
    versions: HashMap<PathBuf, i64>,
}

/// Messages wait here until the server answered `initialize` (the protocol wants that).
#[derive(Default)]
struct Gate {
    ready: bool,
    queue: Vec<Value>,
}

fn send(w: &Arc<Mutex<ChildStdin>>, v: &Value) {
    let body = v.to_string();
    if let Ok(mut w) = w.lock() {
        let _ = write!(w, "Content-Length: {}\r\n\r\n{body}", body.len());
        let _ = w.flush();
    }
}

impl Client {
    /// Starts a server for a project; what it says arrives on `events`.
    pub fn start(cmd: &[String], root: &Path, events: Sender<Event>) -> std::io::Result<Client> {
        // ZETT_LSP_LOG=FILE keeps what servers print on stderr
        let err = match std::env::var_os("ZETT_LSP_LOG").and_then(|f| std::fs::OpenOptions::new().create(true).append(true).open(f).ok()) {
            Some(f) => Stdio::from(f),
            None => Stdio::null(),
        };
        let mut child = Command::new(&cmd[0]).args(&cmd[1..]).current_dir(root).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(err).spawn()?;
        let stdin = Arc::new(Mutex::new(child.stdin.take().ok_or_else(|| std::io::Error::other("no stdin"))?));
        let stdout = child.stdout.take().ok_or_else(|| std::io::Error::other("no stdout"))?;
        let pending: Arc<Mutex<HashMap<i64, Pending>>> = Arc::default();
        let encoding = Arc::new(Mutex::new("utf-16".to_string()));
        let gate: Arc<Mutex<Gate>> = Arc::default();
        let name = cmd[0].clone();
        {
            let (pending, stdin, encoding, name, gate) = (pending.clone(), stdin.clone(), encoding.clone(), name.clone(), gate.clone());
            std::thread::Builder::new().name(format!("lsp {name}")).spawn(move || read_loop(stdout, pending, stdin, encoding, events, name, gate))?;
        }
        let c = Client { name, root: root.to_path_buf(), stdin, child, next: 2, pending, gate, encoding, versions: HashMap::new() };
        // initialize goes out at once; everything else waits for its answer (the gate)
        c.pending.lock().unwrap().insert(1, Pending::Init);
        send(&c.stdin, &json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
            "processId": std::process::id(),
            "rootUri": uri(root),
            "workspaceFolders": [{ "uri": uri(root), "name": root.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default() }],
            "capabilities": {
                "general": { "positionEncodings": ["utf-32", "utf-16"] },
                "textDocument": {
                    "synchronization": { "didSave": true },
                    "publishDiagnostics": { "relatedInformation": false },
                    "hover": { "contentFormat": ["plaintext", "markdown"] },
                    "definition": { "linkSupport": true }
                },
                "window": { "workDoneProgress": true }
            }
        }}));
        Ok(c)
    }

    fn request(&mut self, method: &str, params: Value) -> i64 {
        let id = self.next;
        self.next += 1;
        self.out(json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }));
        id
    }

    fn notify(&self, method: &str, params: Value) {
        self.out(json!({ "jsonrpc": "2.0", "method": method, "params": params }));
    }

    /// Sends now, or once the server is ready.
    fn out(&self, v: Value) {
        let mut g = self.gate.lock().unwrap();
        if g.ready {
            drop(g);
            send(&self.stdin, &v);
        } else {
            g.queue.push(v);
        }
    }

    pub fn did_open(&mut self, path: &Path, lang: &str, text: &str) {
        self.versions.insert(path.to_path_buf(), 1);
        self.notify("textDocument/didOpen", json!({ "textDocument": { "uri": uri(path), "languageId": lang, "version": 1, "text": text } }));
    }

    /// The whole text again (full sync: simple, and fast enough for source files).
    pub fn did_change(&mut self, path: &Path, text: &str) {
        let v = self.versions.entry(path.to_path_buf()).or_insert(1);
        *v += 1;
        let v = *v;
        self.notify("textDocument/didChange", json!({ "textDocument": { "uri": uri(path), "version": v }, "contentChanges": [{ "text": text }] }));
    }

    pub fn did_save(&self, path: &Path) {
        self.notify("textDocument/didSave", json!({ "textDocument": { "uri": uri(path) } }));
    }

    pub fn did_close(&mut self, path: &Path) {
        self.versions.remove(path);
        self.notify("textDocument/didClose", json!({ "textDocument": { "uri": uri(path) } }));
    }

    pub fn is_open(&self, path: &Path) -> bool {
        self.versions.contains_key(path)
    }

    /// `col` in the server's units (see `encoding`).
    pub fn definition(&mut self, path: &Path, line: usize, col: usize) {
        let id = self.request("textDocument/definition", json!({ "textDocument": { "uri": uri(path) }, "position": { "line": line, "character": col } }));
        self.pending.lock().unwrap().insert(id, Pending::Definition);
    }

    pub fn hover(&mut self, path: &Path, line: usize, col: usize) {
        let id = self.request("textDocument/hover", json!({ "textDocument": { "uri": uri(path) }, "position": { "line": line, "character": col } }));
        self.pending.lock().unwrap().insert(id, Pending::Hover);
    }

    pub fn utf16(&self) -> bool {
        self.encoding.lock().map(|e| e.as_str() != "utf-32").unwrap_or(true)
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        let id = self.request("shutdown", Value::Null);
        let _ = id;
        self.notify("exit", Value::Null);
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// A column in chars ↔ in UTF-16 units, on one line of text.
pub fn to_utf16(line: &str, col: usize) -> usize {
    line.chars().take(col).map(char::len_utf16).sum()
}

pub fn from_utf16(line: &str, units: usize) -> usize {
    let mut n = 0;
    for (i, ch) in line.chars().enumerate() {
        if n >= units {
            return i;
        }
        n += ch.len_utf16();
    }
    line.chars().count()
}

fn read_loop(out: std::process::ChildStdout, pending: Arc<Mutex<HashMap<i64, Pending>>>, stdin: Arc<Mutex<ChildStdin>>, encoding: Arc<Mutex<String>>, events: Sender<Event>, name: String, gate: Arc<Mutex<Gate>>) {
    let mut r = BufReader::new(out);
    loop {
        // headers, then the body
        let mut len = 0usize;
        loop {
            let mut h = String::new();
            match r.read_line(&mut h) {
                Ok(0) | Err(_) => {
                    let _ = events.send(Event::Exited(name));
                    return;
                }
                Ok(_) => {}
            }
            let h = h.trim();
            if h.is_empty() {
                break;
            }
            if let Some(v) = h.strip_prefix("Content-Length:") {
                len = v.trim().parse().unwrap_or(0);
            }
        }
        let mut body = vec![0u8; len];
        if r.read_exact(&mut body).is_err() {
            let _ = events.send(Event::Exited(name));
            return;
        }
        let Ok(msg) = serde_json::from_slice::<Value>(&body) else { continue };
        let method = msg.get("method").and_then(Value::as_str);
        match (msg.get("id"), method) {
            // a request from the server: answer what a small client can
            (Some(id), Some(m)) => {
                let result = match m {
                    "workspace/configuration" => Value::Array(vec![Value::Null; msg["params"]["items"].as_array().map(|a| a.len()).unwrap_or(1)]),
                    _ => Value::Null,
                };
                send(&stdin, &json!({ "jsonrpc": "2.0", "id": id, "result": result }));
            }
            // a notification
            (None, Some(m)) => match m {
                "textDocument/publishDiagnostics" => {
                    let p = &msg["params"];
                    let path = path_of(p["uri"].as_str().unwrap_or(""));
                    let diags = p["diagnostics"].as_array().map(|a| a.iter().map(diag).collect()).unwrap_or_default();
                    let _ = events.send(Event::Diagnostics(path, diags));
                }
                "$/progress" => {
                    let v = &msg["params"]["value"];
                    let text = [v["title"].as_str(), v["message"].as_str()].into_iter().flatten().collect::<Vec<_>>().join(" ");
                    if v["kind"] == "end" {
                        let _ = events.send(Event::Status(String::new()));
                    } else if !text.is_empty() {
                        let pct = v["percentage"].as_u64().map(|p| format!(" {p}%")).unwrap_or_default();
                        let _ = events.send(Event::Status(format!("{name}: {text}{pct}")));
                    }
                }
                "window/showMessage" => {
                    if let Some(t) = msg["params"]["message"].as_str() {
                        let _ = events.send(Event::Status(format!("{name}: {t}")));
                    }
                }
                _ => {}
            },
            // an answer to us
            (Some(id), None) => {
                let Some(id) = id.as_i64() else { continue };
                let what = pending.lock().unwrap().remove(&id);
                let res = &msg["result"];
                match what {
                    Some(Pending::Init) => {
                        if let Some(e) = res["capabilities"]["positionEncoding"].as_str() {
                            *encoding.lock().unwrap() = e.to_string();
                        }
                        send(&stdin, &json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} }));
                        // now what waited (didOpen and friends), in order
                        let mut g = gate.lock().unwrap();
                        for v in std::mem::take(&mut g.queue) {
                            send(&stdin, &v);
                        }
                        g.ready = true;
                    }
                    Some(Pending::Definition) => {
                        let items: Vec<Value> = match res {
                            Value::Array(a) => a.clone(),
                            Value::Null => Vec::new(),
                            v => vec![v.clone()],
                        };
                        let locs = items
                            .iter()
                            .filter_map(|l| {
                                let uri = l["uri"].as_str().or(l["targetUri"].as_str())?;
                                let range = if l["range"].is_object() { &l["range"] } else { &l["targetSelectionRange"] };
                                Some(Location { path: path_of(uri), line: range["start"]["line"].as_u64()? as usize, col: range["start"]["character"].as_u64()? as usize })
                            })
                            .collect();
                        let _ = events.send(Event::Definition(locs));
                    }
                    Some(Pending::Hover) => {
                        let c = &res["contents"];
                        let text = match c {
                            Value::String(s) => s.clone(),
                            Value::Object(o) => o.get("value").and_then(Value::as_str).unwrap_or("").to_string(),
                            Value::Array(a) => a.iter().map(|x| x.as_str().map(String::from).unwrap_or_else(|| x["value"].as_str().unwrap_or("").to_string())).collect::<Vec<_>>().join("\n\n"),
                            _ => String::new(),
                        };
                        let _ = events.send(Event::Hover(text));
                    }
                    None => {}
                }
            }
            _ => {}
        }
    }
}

fn diag(d: &Value) -> Diag {
    let n = |v: &Value| v.as_u64().unwrap_or(0) as usize;
    let r = &d["range"];
    Diag {
        line: n(&r["start"]["line"]),
        col: n(&r["start"]["character"]),
        end_line: n(&r["end"]["line"]),
        end_col: n(&r["end"]["character"]),
        severity: d["severity"].as_u64().unwrap_or(1) as u8,
        message: d["message"].as_str().unwrap_or("").lines().next().unwrap_or("").to_string(),
    }
}

/// The servers of an editor: started on demand, one per (project, command).
#[derive(Default)]
pub struct Servers {
    pub clients: HashMap<(PathBuf, String), Client>,
    pub rx: Option<Receiver<Event>>,
    tx: Option<Sender<Event>>,
}

impl Servers {
    pub fn new() -> Servers {
        let (tx, rx) = channel();
        Servers { clients: HashMap::new(), rx: Some(rx), tx: Some(tx) }
    }

    /// The client for a file (started when needed); None: no server for it.
    pub fn for_file(&mut self, path: &Path, root: &Path) -> Option<(&mut Client, &'static str)> {
        let (cmd, lang) = server_for(path)?;
        let key = (root.to_path_buf(), cmd[0].clone());
        if !self.clients.contains_key(&key) {
            let c = Client::start(&cmd, root, self.tx.clone()?).ok()?;
            self.clients.insert(key.clone(), c);
        }
        self.clients.get_mut(&key).map(|c| (c, lang))
    }

    pub fn events(&self) -> Vec<Event> {
        self.rx.as_ref().map(|r| r.try_iter().collect()).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn columns_and_uris() {
        assert_eq!(to_utf16("a😀b", 2), 3);
        assert_eq!(from_utf16("a😀b", 3), 2);
        assert_eq!(path_of("file:///tmp/a%20b.rs"), PathBuf::from("/tmp/a b.rs"));
    }
}
