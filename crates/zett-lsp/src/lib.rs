//! A language-server client: one server process per project and language.

use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};

/// From the config (zett.opt.lsp): servers by extension (command line, language id) that come
/// before the built-in ones, and more folders to find server programs in (Mason's bin).
static CONFIG: Mutex<(Vec<(String, Vec<String>, &'static str)>, Vec<PathBuf>)> = Mutex::new((Vec::new(), Vec::new()));

/// Sets those (again on a config reload).
pub fn configure(servers: Vec<(String, Vec<String>, String)>, dirs: Vec<PathBuf>) {
    let mut c = CONFIG.lock().unwrap_or_else(|e| e.into_inner());
    c.0 = servers.into_iter().map(|(e, cmd, l)| (e, cmd, &*Box::leak(l.into_boxed_str()))).collect();
    c.1 = dirs;
}

/// A server's name: its program without the folder (Mason's full path shown short).
fn prog_name(cmd: &str) -> String {
    Path::new(cmd).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| cmd.to_string())
}

/// A program by name: in PATH, else in the config's folders (then by its full path).
fn find_program(name: &str) -> Option<String> {
    if name.contains('/') {
        return Path::new(name).is_file().then(|| name.to_string());
    }
    if std::env::var_os("PATH").is_some_and(|p| std::env::split_paths(&p).any(|d| d.join(name).is_file())) {
        return Some(name.to_string());
    }
    let c = CONFIG.lock().unwrap_or_else(|e| e.into_inner());
    c.1.iter().map(|d| d.join(name)).find(|p| p.is_file()).map(|p| p.display().to_string())
}

/// The server for a file, by its extension: the command line and the language id.
pub fn server_for(path: &Path) -> Option<(Vec<String>, &'static str)> {
    // files known by name (no extension, or a name that says more than it)
    let name = path.file_name()?.to_str()?;
    let ext = match name {
        "Dockerfile" | "Containerfile" => "dockerfile",
        "CMakeLists.txt" => "cmake",
        "docker-compose.yml" | "docker-compose.yaml" | "compose.yml" | "compose.yaml" => "compose",
        "justfile" | "Justfile" | ".justfile" => "just",
        _ => path.extension()?.to_str()?,
    };
    let own = CONFIG.lock().unwrap_or_else(|e| e.into_inner()).0.iter().find(|(e, _, _)| e == ext).map(|(_, c, l)| (c.clone(), *l));
    if let Some((mut cmd, lang)) = own {
        let prog = find_program(cmd.first()?)?;
        cmd[0] = prog;
        return Some((cmd, lang));
    }
    // the servers for each language, the first one installed is used
    let (cands, lang): (&[&[&str]], &str) = match ext {
        "rs" => (&[&["rust-analyzer"]], "rust"),
        "c" | "h" => (&[&["clangd"], &["ccls"]], "c"),
        "cpp" | "cc" | "hpp" | "cxx" | "hh" | "hxx" | "ino" => (&[&["clangd"], &["ccls"]], "cpp"),
        "m" | "mm" => (&[&["clangd"]], "objective-c"),
        "cu" | "cuh" => (&[&["clangd"]], "cuda"),
        "ts" | "mts" | "cts" => (&[&["typescript-language-server", "--stdio"], &["vtsls", "--stdio"]], "typescript"),
        "tsx" => (&[&["typescript-language-server", "--stdio"], &["vtsls", "--stdio"]], "typescriptreact"),
        "js" | "mjs" | "cjs" => (&[&["typescript-language-server", "--stdio"], &["vtsls", "--stdio"]], "javascript"),
        "jsx" => (&[&["typescript-language-server", "--stdio"], &["vtsls", "--stdio"]], "javascriptreact"),
        "vue" => (&[&["vue-language-server", "--stdio"]], "vue"),
        "svelte" => (&[&["svelteserver", "--stdio"]], "svelte"),
        "astro" => (&[&["astro-ls", "--stdio"]], "astro"),
        "html" | "htm" | "xhtml" => (&[&["vscode-html-language-server", "--stdio"], &["superhtml", "lsp"]], "html"),
        "css" => (&[&["vscode-css-language-server", "--stdio"]], "css"),
        "scss" => (&[&["vscode-css-language-server", "--stdio"], &["some-sass-language-server", "--stdio"]], "scss"),
        "sass" => (&[&["some-sass-language-server", "--stdio"]], "sass"),
        "less" => (&[&["vscode-css-language-server", "--stdio"]], "less"),
        "json" => (&[&["vscode-json-language-server", "--stdio"], &["biome", "lsp-proxy"]], "json"),
        "jsonc" | "json5" => (&[&["vscode-json-language-server", "--stdio"]], "jsonc"),
        "yaml" | "yml" => (&[&["yaml-language-server", "--stdio"]], "yaml"),
        "compose" => (&[&["docker-compose-langserver", "--stdio"], &["yaml-language-server", "--stdio"]], "dockercompose"),
        "toml" => (&[&["taplo", "lsp", "stdio"], &["tombi", "lsp"]], "toml"),
        "xml" | "xsd" | "xsl" | "svg" => (&[&["lemminx"]], "xml"),
        "md" | "markdown" => (&[&["marksman", "server"], &["markdown-oxide"]], "markdown"),
        "tex" | "bib" | "sty" | "cls" => (&[&["texlab"]], "latex"),
        "typ" => (&[&["tinymist"]], "typst"),
        "py" | "pyi" => (&[&["pyright-langserver", "--stdio"], &["basedpyright-langserver", "--stdio"], &["pylsp"], &["jedi-language-server"]], "python"),
        "lua" => (&[&["lua-language-server"]], "lua"),
        "sh" | "bash" | "zsh" => (&[&["bash-language-server", "start"]], "shellscript"),
        "fish" => (&[&["fish-lsp", "start"]], "fish"),
        "ps1" | "psm1" => (&[&["pwsh-language-server"]], "powershell"),
        "go" => (&[&["gopls"]], "go"),
        "zig" | "zon" => (&[&["zls"]], "zig"),
        "nix" => (&[&["nil"], &["nixd"]], "nix"),
        "java" => (&[&["jdtls"]], "java"),
        "kt" | "kts" => (&[&["kotlin-language-server"], &["kotlin-lsp"]], "kotlin"),
        "scala" | "sc" | "sbt" => (&[&["metals"]], "scala"),
        "groovy" | "gradle" => (&[&["groovy-language-server"]], "groovy"),
        "cs" => (&[&["csharp-ls"], &["OmniSharp", "-lsp"]], "csharp"),
        "fs" | "fsx" | "fsi" => (&[&["fsautocomplete"]], "fsharp"),
        "swift" => (&[&["sourcekit-lsp"]], "swift"),
        "dart" => (&[&["dart", "language-server", "--protocol=lsp"]], "dart"),
        "rb" | "rake" | "gemspec" => (&[&["ruby-lsp"], &["solargraph", "stdio"]], "ruby"),
        "php" => (&[&["intelephense", "--stdio"], &["phpactor", "language-server"]], "php"),
        "ex" | "exs" | "heex" => (&[&["lexical"], &["elixir-ls"], &["nextls", "--stdio"]], "elixir"),
        "erl" | "hrl" => (&[&["erlang_ls"], &["elp", "server"]], "erlang"),
        "gleam" => (&[&["gleam", "lsp"]], "gleam"),
        "hs" | "lhs" => (&[&["haskell-language-server-wrapper", "--lsp"], &["haskell-language-server", "--lsp"]], "haskell"),
        "ml" | "mli" => (&[&["ocamllsp"]], "ocaml"),
        "clj" | "cljs" | "cljc" | "edn" => (&[&["clojure-lsp"]], "clojure"),
        "el" => (&[&["ellsp"]], "elisp"),
        "rkt" => (&[&["racket", "-l", "racket-langserver"]], "racket"),
        "jl" => (&[&["julia-lsp"]], "julia"),
        "r" | "R" => (&[&["R", "--slave", "-e", "languageserver::run()"]], "r"),
        "nim" | "nims" => (&[&["nimlangserver"]], "nim"),
        "odin" => (&[&["ols"]], "odin"),
        "v" => (&[&["v-analyzer"]], "v"),
        "d" => (&[&["serve-d"]], "d"),
        "cr" => (&[&["crystalline"]], "crystal"),
        "sol" => (&[&["vscode-solidity-server", "--stdio"], &["nomicfoundation-solidity-language-server", "--stdio"]], "solidity"),
        // the system's first: the same Qt as the program (Mason's qmlls is built for another one, and
        // lacked a library here)
        "qml" => (&[&["qmlls6"], &["/usr/lib/qt6/bin/qmlls"], &["qmlls"]], "qml"),
        "slint" => (&[&["slint-lsp"]], "slint"),
        "gd" | "gdscript" => (&[&["godot-lsp-stdio-bridge"]], "gdscript"),
        "glsl" | "vert" | "frag" | "geom" | "comp" | "tesc" | "tese" => (&[&["glsl_analyzer"], &["glslls", "--stdin"]], "glsl"),
        "wgsl" => (&[&["wgsl-analyzer"]], "wgsl"),
        "hlsl" => (&[&["shader-language-server", "--stdio"]], "hlsl"),
        "cmake" => (&[&["neocmakelsp", "stdio"], &["neocmakelsp", "--stdio"], &["cmake-language-server"]], "cmake"),   // 0.11 takes "stdio"
        "dockerfile" => (&[&["docker-langserver", "--stdio"], &["docker-language-server", "start", "--stdio"]], "dockerfile"),
        "tf" | "tfvars" => (&[&["terraform-ls", "serve"], &["tofu-ls", "serve"]], "terraform"),
        "hcl" => (&[&["terraform-ls", "serve"]], "hcl"),
        "sql" => (&[&["sqls"], &["sql-language-server", "up", "--method", "stdio"]], "sql"),
        "graphql" | "gql" => (&[&["graphql-lsp", "server", "-m", "stream"]], "graphql"),
        "proto" => (&[&["buf", "lsp", "serve"], &["protols"]], "proto"),
        "prisma" => (&[&["prisma-language-server", "--stdio"]], "prisma"),
        "vim" => (&[&["vim-language-server", "--stdio"]], "vim"),
        "just" => (&[&["just-lsp"]], "just"),
        "kdl" => (&[&["kdl-lsp"]], "kdl"),
        "asm" | "s" | "S" => (&[&["asm-lsp"]], "asm"),
        "vhd" | "vhdl" => (&[&["vhdl_ls"]], "vhdl"),
        "sv" | "svh" => (&[&["verible-verilog-ls"], &["svls"]], "systemverilog"),
        "tla" => (&[&["tlaplus-ls"]], "tla"),
        "lean" => (&[&["lake", "serve"]], "lean"),
        "elm" => (&[&["elm-language-server"]], "elm"),
        "purs" => (&[&["purescript-language-server", "--stdio"]], "purescript"),
        "fnl" => (&[&["fennel-ls"]], "fennel"),
        "teal" | "tl" => (&[&["teal-language-server"]], "teal"),
        "ron" => (&[&["ron-lsp"]], "ron"),
        "hypr" => (&[&["hyprls"]], "hyprlang"),
        _ => return None,
    };
    cands.iter().find_map(|cmd| {
        let prog = find_program(cmd[0])?;
        let mut cmd: Vec<String> = cmd.iter().map(|s| s.to_string()).collect();
        cmd[0] = prog;
        Some((cmd, lang))
    })
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
    References(Vec<Location>),
    /// a file's symbols: (name, kind, line, col, depth)
    Symbols(Vec<(String, u8, usize, usize, usize)>),
    /// edits to make (a rename, a format): per file, (start line, col, end line, col, text)
    Edits(Vec<(PathBuf, Vec<(usize, usize, usize, usize, String)>)>),
    /// completions at a position: the line asked about, the items, and whether the list is only
    /// a start (isIncomplete: ask again as more is typed)
    Completion(usize, Vec<Item>, bool),
    /// an item with the rest filled in (completionItem/resolve)
    Resolved(Item),
    /// progress and log lines worth a glance
    Status(String),
    /// a piece of work a server reports ($/progress): server, token, title, message, percent, done
    Progress { server: String, token: String, title: String, message: String, percent: Option<u64>, done: bool },
    /// the signature at the cursor: its label and the active parameter's range in it (start, end; 0, 0: none)
    Signature(String, usize, usize),
    /// code actions at a position: titles with their edits (actions that are only commands are left out)
    CodeActions(Vec<(String, Vec<(PathBuf, Vec<(usize, usize, usize, usize, String)>)>)>),
    /// a client's tag (Client::tag): its server is gone
    Exited(String),
}

enum Pending {
    Definition,
    Hover,
    References,
    Symbols,
    Edits(Option<PathBuf>),
    Completion(usize),
    Signature,
    CodeActions,
    Resolve,
    Init,
}

pub struct Client {
    pub name: String,
    /// this client alone (the name and a number): an old one exiting after a restart must not take
    /// the new one with it
    pub tag: String,
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
    /// from initialize: characters that ask for completions, and completionItem/resolve
    triggers: Vec<String>,
    resolve: bool,
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
        let mut child = Command::new(&cmd[0]).args(&cmd[1..]).current_dir(root).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
        // what it prints on stderr: the last line kept (a server that dies at start says why), all of it in
        // ZETT_LSP_LOG=FILE when set
        let last_err: Arc<Mutex<String>> = Arc::default();
        if let Some(err) = child.stderr.take() {
            let keep = last_err.clone();
            let mut log = std::env::var_os("ZETT_LSP_LOG").and_then(|f| std::fs::OpenOptions::new().create(true).append(true).open(f).ok());
            std::thread::spawn(move || {
                use std::io::{BufRead, Write};
                for line in std::io::BufReader::new(err).lines().map_while(Result::ok) {
                    if let Some(f) = log.as_mut() { let _ = writeln!(f, "{line}"); }
                    if !line.trim().is_empty() { *keep.lock().unwrap_or_else(|e| e.into_inner()) = line.trim().chars().take(160).collect(); }
                }
            });
        }
        let stdin = Arc::new(Mutex::new(child.stdin.take().ok_or_else(|| std::io::Error::other("no stdin"))?));
        let stdout = child.stdout.take().ok_or_else(|| std::io::Error::other("no stdout"))?;
        let pending: Arc<Mutex<HashMap<i64, Pending>>> = Arc::default();
        let encoding = Arc::new(Mutex::new("utf-16".to_string()));
        let gate: Arc<Mutex<Gate>> = Arc::default();
        let name = prog_name(&cmd[0]);
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let tag = format!("{name}#{}", NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed));
        {
            let (pending, stdin, encoding, name, gate, tag) = (pending.clone(), stdin.clone(), encoding.clone(), name.clone(), gate.clone(), tag.clone());
            std::thread::Builder::new().name(format!("lsp {name}")).spawn(move || read_loop(stdout, pending, stdin, encoding, events, name, gate, tag, last_err))?;
        }
        let c = Client { name, tag, root: root.to_path_buf(), stdin, child, next: 2, pending, gate, encoding, versions: HashMap::new() };
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
                    "signatureHelp": { "signatureInformation": { "parameterInformation": { "labelOffsetSupport": true } } },
                    "codeAction": { "codeActionLiteralSupport": { "codeActionKind": { "valueSet": ["quickfix", "refactor", "refactor.extract", "refactor.inline", "refactor.rewrite", "source"] } } },
                    "completion": { "completionItem": { "snippetSupport": true, "documentationFormat": ["markdown", "plaintext"], "labelDetailsSupport": true, "insertReplaceSupport": false, "resolveSupport": { "properties": ["documentation", "detail", "additionalTextEdits"] } }, "contextSupport": true },
                    "references": {},
                    "documentSymbol": { "hierarchicalDocumentSymbolSupport": true },
                    "rename": { "prepareSupport": false },
                    "formatting": {},
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

    pub fn references(&mut self, path: &Path, line: usize, col: usize) {
        let id = self.request("textDocument/references", json!({ "textDocument": { "uri": uri(path) }, "position": { "line": line, "character": col }, "context": { "includeDeclaration": true } }));
        self.pending.lock().unwrap().insert(id, Pending::References);
    }

    pub fn symbols(&mut self, path: &Path) {
        let id = self.request("textDocument/documentSymbol", json!({ "textDocument": { "uri": uri(path) } }));
        self.pending.lock().unwrap().insert(id, Pending::Symbols);
    }

    pub fn rename(&mut self, path: &Path, line: usize, col: usize, name: &str) {
        let id = self.request("textDocument/rename", json!({ "textDocument": { "uri": uri(path) }, "position": { "line": line, "character": col }, "newName": name }));
        self.pending.lock().unwrap().insert(id, Pending::Edits(None));
    }

    pub fn signature(&mut self, path: &Path, line: usize, col: usize) {
        let id = self.request("textDocument/signatureHelp", json!({ "textDocument": { "uri": uri(path) }, "position": { "line": line, "character": col } }));
        self.pending.lock().unwrap().insert(id, Pending::Signature);
    }

    /// `diags`: the diagnostics on the line, as the server sent them (quick fixes are offered for them)
    pub fn code_actions(&mut self, path: &Path, line: usize, col: usize, diags: Value) {
        let pos = json!({ "line": line, "character": col });
        let id = self.request("textDocument/codeAction", json!({ "textDocument": { "uri": uri(path) }, "range": { "start": pos, "end": pos }, "context": { "diagnostics": diags } }));
        self.pending.lock().unwrap().insert(id, Pending::CodeActions);
    }

    pub fn format(&mut self, path: &Path, tab: usize) {
        let id = self.request("textDocument/formatting", json!({ "textDocument": { "uri": uri(path) }, "options": { "tabSize": tab, "insertSpaces": true } }));
        self.pending.lock().unwrap().insert(id, Pending::Edits(Some(path.to_path_buf())));
    }

    /// `line` is echoed back with the answer, so a late one can be told apart; `trigger` is the
    /// character that asked (one of the server's trigger characters), if one did.
    pub fn completion(&mut self, path: &Path, line: usize, col: usize, trigger: Option<char>) {
        let context = match trigger {
            Some(c) => json!({ "triggerKind": 2, "triggerCharacter": c.to_string() }),
            None => json!({ "triggerKind": 1 }),
        };
        let id = self.request("textDocument/completion", json!({ "textDocument": { "uri": uri(path) }, "position": { "line": line, "character": col }, "context": context }));
        self.pending.lock().unwrap().insert(id, Pending::Completion(line));
    }

    /// The rest of an item (its documentation, the imports it adds), when the server offers that.
    pub fn resolve(&mut self, it: &Item) -> bool {
        if !self.can_resolve() || it.resolved || it.data.is_null() {
            return false;
        }
        let id = self.request("completionItem/resolve", it.data.clone());
        self.pending.lock().unwrap().insert(id, Pending::Resolve);
        true
    }

    /// The characters that ask for completions by themselves (`.`, `::`'s `:`, `<`, `"` …).
    pub fn triggers(&self) -> Vec<String> {
        self.gate.lock().map(|g| g.triggers.clone()).unwrap_or_default()
    }

    pub fn can_resolve(&self) -> bool {
        self.gate.lock().map(|g| g.resolve).unwrap_or(false)
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

#[allow(clippy::too_many_arguments)]
fn read_loop(out: std::process::ChildStdout, pending: Arc<Mutex<HashMap<i64, Pending>>>, stdin: Arc<Mutex<ChildStdin>>, encoding: Arc<Mutex<String>>, events: Sender<Event>, name: String, gate: Arc<Mutex<Gate>>, tag: String, last_err: Arc<Mutex<String>>) {
    // it went away: its last stderr line says why (a broken install, a missing module), once
    let gone = |events: &Sender<Event>, tag: String| {
        std::thread::sleep(std::time::Duration::from_millis(150));
        let why = last_err.lock().map(|s| s.clone()).unwrap_or_default();
        if !why.is_empty() { let _ = events.send(Event::Status(format!("{name} stopped: {why}"))); }
        let _ = events.send(Event::Exited(tag));
    };
    let mut r = BufReader::new(out);
    loop {
        // headers, then the body
        let mut len = 0usize;
        loop {
            let mut h = String::new();
            match r.read_line(&mut h) {
                Ok(0) | Err(_) => {
                    gone(&events, tag);
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
            gone(&events, tag);
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
                    let token = match &msg["params"]["token"] {
                        serde_json::Value::String(s) => s.clone(),
                        t => t.to_string(),
                    };
                    let _ = events.send(Event::Progress {
                        server: name.to_string(),
                        token,
                        title: v["title"].as_str().unwrap_or("").to_string(),
                        message: v["message"].as_str().unwrap_or("").to_string(),
                        percent: v["percentage"].as_u64(),
                        done: v["kind"] == "end",
                    });
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
                        let cp = &res["capabilities"]["completionProvider"];
                        g.triggers = cp["triggerCharacters"].as_array().map(|a| a.iter().filter_map(|t| t.as_str().map(String::from)).collect()).unwrap_or_default();
                        g.resolve = cp["resolveProvider"].as_bool().unwrap_or(false);
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
                    Some(Pending::Completion(line)) => {
                        let list = if res.is_array() { res.clone() } else { res["items"].clone() };
                        let items = list.as_array().map(|a| a.iter().map(item).collect()).unwrap_or_default();
                        let _ = events.send(Event::Completion(line, items, res["isIncomplete"].as_bool().unwrap_or(false)));
                    }
                    Some(Pending::Resolve) => {
                        if res.is_object() {
                            let mut it = item(res);
                            it.resolved = true;
                            let _ = events.send(Event::Resolved(it));
                        }
                    }
                    Some(Pending::References) => {
                        let locs = res.as_array().map(|a| a.iter().filter_map(location).collect()).unwrap_or_default();
                        let _ = events.send(Event::References(locs));
                    }
                    Some(Pending::Symbols) => {
                        let mut out = Vec::new();
                        if let Some(a) = res.as_array() {
                            for s in a {
                                symbols(s, 0, &mut out);
                            }
                        }
                        let _ = events.send(Event::Symbols(out));
                    }
                    Some(Pending::Edits(file)) => {
                        if let Some(e) = msg.get("error") {
                            let _ = events.send(Event::Status(format!("{name}: {}", e["message"].as_str().unwrap_or("refused"))));
                        } else {
                            let _ = events.send(Event::Edits(edits(res, file)));
                        }
                    }
                    Some(Pending::Signature) => {
                        let i = res["activeSignature"].as_u64().unwrap_or(0) as usize;
                        let s = &res["signatures"][i];
                        let label = s["label"].as_str().unwrap_or("").to_string();
                        let ap = s["activeParameter"].as_u64().or(res["activeParameter"].as_u64()).unwrap_or(0) as usize;
                        let (a, b) = match &s["parameters"][ap]["label"] {
                            Value::Array(r) if r.len() == 2 => (r[0].as_u64().unwrap_or(0) as usize, r[1].as_u64().unwrap_or(0) as usize),
                            Value::String(p) => label.find(p.as_str()).map(|a| (a, a + p.len())).unwrap_or((0, 0)),
                            _ => (0, 0),
                        };
                        let _ = events.send(Event::Signature(label, a, b));
                    }
                    Some(Pending::CodeActions) => {
                        let acts = res.as_array().map(|a| a.iter().filter(|x| x["edit"].is_object()).map(|x| (x["title"].as_str().unwrap_or("").to_string(), edits(&x["edit"], None))).collect()).unwrap_or_default();
                        let _ = events.send(Event::CodeActions(acts));
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
    /// servers not to start (:lsp disable), by command name
    pub disabled: std::collections::HashSet<String>,
    pub rx: Option<Receiver<Event>>,
    tx: Option<Sender<Event>>,
}

impl Servers {
    pub fn new() -> Servers {
        let (tx, rx) = channel();
        Servers { clients: HashMap::new(), disabled: Default::default(), rx: Some(rx), tx: Some(tx) }
    }

    /// The client for a file (started when needed); None: no server for it.
    pub fn for_file(&mut self, path: &Path, root: &Path) -> Option<(&mut Client, &'static str)> {
        let (cmd, lang) = server_for(path)?;
        let name = prog_name(&cmd[0]);
        if self.disabled.contains(&name) {
            return None;
        }
        let key = (root.to_path_buf(), name);
        if !self.clients.contains_key(&key) {
            let c = Client::start(&cmd, root, self.tx.clone()?).ok()?;
            self.clients.insert(key.clone(), c);
        }
        self.clients.get_mut(&key).map(|c| (c, lang))
    }

    /// The server a file would use (its command name), started or not.
    pub fn name_for(path: &Path) -> Option<String> {
        server_for(path).map(|(c, _)| prog_name(&c[0]))
    }

    /// Stops the servers by that command name (all with None): dropped, each says shutdown and exit.
    pub fn stop(&mut self, name: Option<&str>) -> usize {
        let n = self.clients.len();
        self.clients.retain(|(_, cmd), _| name.is_some_and(|w| w != cmd));
        n - self.clients.len()
    }

    /// The running servers: (project folder, command).
    pub fn running(&self) -> Vec<(PathBuf, String)> {
        let mut v: Vec<(PathBuf, String)> = self.clients.keys().cloned().collect();
        v.sort();
        v
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

/// A completion: what the list shows, what goes in, a hint beside it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Item {
    pub label: String,
    /// the text that goes in, snippet stops left out (plain text when `snippet` is false)
    pub insert: String,
    /// insertTextFormat 2: `body` is a snippet ($1, ${2:x}) for vim.snippet
    pub snippet: bool,
    pub body: String,
    pub detail: String,
    /// LSP's CompletionItemKind (2 method, 3 function, 5 field, 6 variable, 9 module, 14 keyword …)
    pub kind: u8,
    pub sort: String,
    /// what typing is matched against (filterText, else the label)
    pub filter: String,
    /// the server's own range to replace (textEdit): start line, col, end line, col
    pub range: Option<(usize, usize, usize, usize)>,
    /// more edits to make with it (an import at the top)
    pub extra: Vec<(usize, usize, usize, usize, String)>,
    pub doc: String,
    /// the item as the server sent it, for completionItem/resolve
    pub data: Value,
    pub resolved: bool,
}

fn item(v: &Value) -> Item {
    let label = v["label"].as_str().unwrap_or("").to_string();
    let te = &v["textEdit"];
    let body = te["newText"].as_str().or(v["insertText"].as_str()).unwrap_or(&label).to_string();
    let snippet = v["insertTextFormat"].as_u64() == Some(2);
    let n = |x: &Value| x.as_u64().unwrap_or(0) as usize;
    // a TextEdit's range, or an InsertReplaceEdit's insert part
    let r = if te["range"].is_object() { &te["range"] } else { &te["insert"] };
    let range = r.is_object().then(|| (n(&r["start"]["line"]), n(&r["start"]["character"]), n(&r["end"]["line"]), n(&r["end"]["character"])));
    let extra = v["additionalTextEdits"]
        .as_array()
        .map(|a| a.iter().map(|e| (n(&e["range"]["start"]["line"]), n(&e["range"]["start"]["character"]), n(&e["range"]["end"]["line"]), n(&e["range"]["end"]["character"]), e["newText"].as_str().unwrap_or("").to_string())).collect())
        .unwrap_or_default();
    let doc = match &v["documentation"] {
        Value::String(s) => s.clone(),
        d => d["value"].as_str().unwrap_or("").to_string(),
    };
    let detail = v["detail"].as_str().map(String::from).unwrap_or_else(|| {
        let ld = &v["labelDetails"];
        format!("{}{}", ld["detail"].as_str().unwrap_or(""), ld["description"].as_str().map(|d| format!(" {d}")).unwrap_or_default()).trim().to_string()
    });
    Item {
        detail,
        kind: v["kind"].as_u64().unwrap_or(0) as u8,
        sort: v["sortText"].as_str().unwrap_or(&label).to_string(),
        filter: v["filterText"].as_str().unwrap_or(&label).to_string(),
        insert: if snippet { strip_snippet(&body) } else { body.clone() },
        snippet,
        body,
        range,
        extra,
        doc,
        data: v.clone(),
        resolved: false,
        label,
    }
}

/// `foo(${1:x}, $2)$0` → `foo(x, )`: what a snippet means without its jumps.
pub fn strip_snippet(s: &str) -> String {
    let mut out = String::new();
    let mut it = s.chars().peekable();
    while let Some(c) = it.next() {
        if c == '\\' {
            if let Some(n) = it.next() {
                out.push(n);
            }
        } else if c == '$' {
            if it.peek() == Some(&'{') {
                it.next();
                let mut depth = 1;
                let mut inner = String::new();
                for d in it.by_ref() {
                    match d {
                        '{' => depth += 1,
                        '}' => {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        _ => {}
                    }
                    inner.push(d);
                }
                // ${1:default} keeps the default; ${1} nothing
                if let Some((_, def)) = inner.split_once(':') {
                    out.push_str(&strip_snippet(def));
                }
            } else {
                while it.peek().is_some_and(|d| d.is_ascii_digit()) {
                    it.next();
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod snippet_tests {
    #[test]
    fn snippets_become_text() {
        assert_eq!(super::strip_snippet("add(${1:a}, ${2:b})$0"), "add(a, b)");
        assert_eq!(super::strip_snippet("vec![$0]"), "vec![]");
        assert_eq!(super::strip_snippet("a\\$b"), "a$b");
    }
}

fn location(l: &Value) -> Option<Location> {
    let uri = l["uri"].as_str().or(l["targetUri"].as_str())?;
    let range = if l["range"].is_object() { &l["range"] } else { &l["targetSelectionRange"] };
    Some(Location { path: path_of(uri), line: range["start"]["line"].as_u64()? as usize, col: range["start"]["character"].as_u64()? as usize })
}

/// DocumentSymbol trees (children nested) or flat SymbolInformation.
fn symbols(s: &Value, depth: usize, out: &mut Vec<(String, u8, usize, usize, usize)>) {
    let name = s["name"].as_str().unwrap_or("").to_string();
    let kind = s["kind"].as_u64().unwrap_or(0) as u8;
    let r = if s["selectionRange"].is_object() { &s["selectionRange"] } else { &s["location"]["range"] };
    let (line, col) = (r["start"]["line"].as_u64().unwrap_or(0) as usize, r["start"]["character"].as_u64().unwrap_or(0) as usize);
    out.push((name, kind, line, col, depth));
    if let Some(ch) = s["children"].as_array() {
        for c in ch {
            symbols(c, depth + 1, out);
        }
    }
}

/// A WorkspaceEdit (changes or documentChanges), or a format's plain edit list for `file`.
fn edits(res: &Value, file: Option<PathBuf>) -> Vec<(PathBuf, Vec<(usize, usize, usize, usize, String)>)> {
    let conv = |list: &Value| -> Vec<(usize, usize, usize, usize, String)> {
        let n = |v: &Value| v.as_u64().unwrap_or(0) as usize;
        list.as_array()
            .map(|a| a.iter().map(|e| (n(&e["range"]["start"]["line"]), n(&e["range"]["start"]["character"]), n(&e["range"]["end"]["line"]), n(&e["range"]["end"]["character"]), e["newText"].as_str().unwrap_or("").to_string())).collect())
            .unwrap_or_default()
    };
    if let Some(f) = file {
        return vec![(f, conv(res))];
    }
    let mut out = Vec::new();
    if let Some(m) = res["changes"].as_object() {
        for (u, list) in m {
            out.push((path_of(u), conv(list)));
        }
    }
    if let Some(a) = res["documentChanges"].as_array() {
        for d in a {
            if let Some(u) = d["textDocument"]["uri"].as_str() {
                out.push((path_of(u), conv(&d["edits"])));
            }
        }
    }
    out
}
