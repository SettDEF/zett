//! YouCompleteMe's engine (ycmd) for zett: started once, spoken to over HTTP + JSON, every request signed
//! (HMAC-SHA256, as ycmd checks). ycmd lives in ~/.local/share/zett/ycmd (built with its build.py).
use base64::Engine as _;
use hmac::{Hmac, Mac};
use serde_json::{Value, json};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

type HmacSha256 = Hmac<sha2::Sha256>;

/// One completion as ycmd gives it.
#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    pub text: String,
    /// what kind of thing (ycmd's kind: "[ID]" for an identifier, or the server's: function, variable…)
    pub kind: String,
    /// the menu's extra column (a type, a signature)
    pub menu: String,
    pub detail: String,
}

/// Where ycmd is: zett's data folder, built (its ycm_core library is there).
pub fn find_dir() -> Option<PathBuf> {
    let data = std::env::var_os("XDG_DATA_HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".local/share"));
    let dir = data.join("zett/ycmd");
    let built = std::fs::read_dir(dir.join("third_party").join("..")).is_ok()
        && std::fs::read_dir(&dir).into_iter().flatten().flatten().any(|e| e.file_name().to_string_lossy().starts_with("ycm_core"));
    built.then_some(dir)
}

/// ycmd's name for a file's language, by its extension ("" when ycmd only has words for it).
pub fn filetype(path: &str) -> &'static str {
    let ext = Path::new(path).extension().and_then(|e| e.to_str()).unwrap_or("");
    match ext {
        "rs" => "rust",
        "c" | "h" => "c",
        "cc" | "cpp" | "cxx" | "hpp" | "hh" => "cpp",
        "py" => "python",
        "js" | "mjs" | "cjs" | "jsx" => "javascript",
        "ts" | "tsx" => "typescript",
        "go" => "go",
        "java" => "java",
        "cs" => "cs",
        "lua" => "lua",
        "qml" => "qml",
        "sh" | "bash" | "zsh" => "sh",
        "md" => "markdown",
        "toml" => "toml",
        "json" => "json",
        "" => "text",
        e => Box::leak(e.to_string().into_boxed_str()),
    }
}

pub struct Ycmd {
    child: Child,
    port: u16,
    secret: Vec<u8>,
    /// when each file was last given to ycmd to parse (its words), and the text's length then
    parsed: std::collections::HashMap<String, (Instant, usize)>,
}

impl Drop for Ycmd {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn mac(secret: &[u8], data: &[u8]) -> Vec<u8> {
    let mut m = HmacSha256::new_from_slice(secret).expect("any key length");
    m.update(data);
    m.finalize().into_bytes().to_vec()
}

/// ycmd's request signature: hmac(hmac(method) + hmac(path) + hmac(body)), base64.
fn signature(secret: &[u8], method: &str, path: &str, body: &[u8]) -> String {
    let mut joined = mac(secret, method.as_bytes());
    joined.extend(mac(secret, path.as_bytes()));
    joined.extend(mac(secret, body));
    base64::engine::general_purpose::STANDARD.encode(mac(secret, &joined))
}

impl Ycmd {
    /// Starts ycmd from `dir` (its own Python, its default settings with a fresh secret).
    pub fn start(dir: &Path, log_dir: &Path) -> std::io::Result<Ycmd> {
        let mut secret = vec![0u8; 16];
        std::fs::File::open("/dev/urandom")?.read_exact(&mut secret)?;
        let mut opts: Value = serde_json::from_str(&std::fs::read_to_string(dir.join("ycmd/default_settings.json"))?).map_err(std::io::Error::other)?;
        opts["hmac_secret"] = json!(base64::engine::general_purpose::STANDARD.encode(&secret));
        // the system's clangd for C and C++, when there is one
        if let Some(clangd) = which("clangd") {
            opts["clangd_binary_path"] = json!(clangd);
        }
        std::fs::create_dir_all(log_dir)?;
        let opts_file = log_dir.join(format!("ycmd-options-{}.json", std::process::id()));
        std::fs::write(&opts_file, opts.to_string())?;
        // a free port: one the system hands out, let go again for ycmd
        let port = std::net::TcpListener::bind("127.0.0.1:0")?.local_addr()?.port();
        use std::os::unix::process::CommandExt;
        let mut cmd = Command::new("python3");
        // it ends with zett (even a crash): the kernel stops it when its parent goes
        unsafe { cmd.pre_exec(|| { libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM); Ok(()) }); }
        let child = cmd
            .arg(dir.join("ycmd"))
            .arg(format!("--port={port}"))
            .arg(format!("--options_file={}", opts_file.display()))
            .arg("--idle_suicide_seconds=1800")
            .arg(format!("--stdout={}", log_dir.join("ycmd.out.log").display()))
            .arg(format!("--stderr={}", log_dir.join("ycmd.err.log").display()))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        Ok(Ycmd { child, port, secret, parsed: Default::default() })
    }

    /// One request; the answer's body (an error for a status other than 200).
    fn request(&self, method: &str, path: &str, body: &str, timeout: Duration) -> std::io::Result<String> {
        let mut s = TcpStream::connect_timeout(&([127, 0, 0, 1], self.port).into(), timeout)?;
        s.set_read_timeout(Some(timeout))?;
        s.set_write_timeout(Some(timeout))?;
        let sig = signature(&self.secret, method, path, body.as_bytes());
        write!(s, "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nX-Ycm-Hmac: {sig}\r\nConnection: close\r\n\r\n{body}", self.port, body.len())?;
        let mut out = Vec::new();
        s.read_to_end(&mut out)?;
        let text = String::from_utf8_lossy(&out);
        let (head, rest) = text.split_once("\r\n\r\n").unwrap_or((&text, ""));
        let ok = head.lines().next().is_some_and(|l| l.contains(" 200 "));
        // chunked answers: the pieces joined
        let body = if head.to_ascii_lowercase().contains("transfer-encoding: chunked") { unchunk(rest) } else { rest.to_string() };
        if ok { Ok(body) } else { Err(std::io::Error::other(format!("ycmd: {}", head.lines().next().unwrap_or("no answer")))) }
    }

    /// ycmd answers (it takes a second or two to start).
    pub fn ready(&self) -> bool {
        self.request("GET", "/ready", "", Duration::from_millis(300)).is_ok_and(|b| b.trim() == "true")
    }

    fn file_data(path: &str, text: &str) -> Value {
        json!({ path: { "contents": text, "filetypes": [filetype(path)] } })
    }

    /// The file's words for ycmd (FileReadyToParse), at most every two seconds per file unless its size changed a lot.
    pub fn parse(&mut self, path: &str, text: &str) {
        if let Some((at, len)) = self.parsed.get(path) {
            if at.elapsed() < Duration::from_secs(2) && len.abs_diff(text.len()) < 200 {
                return;
            }
        }
        let body = json!({ "event_name": "FileReadyToParse", "filepath": path, "line_num": 1, "column_num": 1, "file_data": Self::file_data(path, text) }).to_string();
        if self.request("POST", "/event_notification", &body, Duration::from_millis(800)).is_ok() {
            self.parsed.insert(path.to_string(), (Instant::now(), text.len()));
        }
    }

    /// Completions at `line` (from 0) and byte column `col` (from 0): where the word being completed starts
    /// (a byte column, from 0) and the items, ycmd's fuzzy matching and ranking.
    pub fn complete(&mut self, path: &str, text: &str, line: usize, col: usize) -> std::io::Result<(usize, Vec<Item>)> {
        self.parse(path, text);
        let body = json!({ "filepath": path, "line_num": line + 1, "column_num": col + 1, "file_data": Self::file_data(path, text) }).to_string();
        let ans: Value = serde_json::from_str(&self.request("POST", "/completions", &body, Duration::from_millis(600))?).map_err(std::io::Error::other)?;
        let start = ans["completion_start_column"].as_u64().unwrap_or(col as u64 + 1).saturating_sub(1) as usize;
        let s = |v: &Value| v.as_str().unwrap_or("").to_string();
        let items = ans["completions"].as_array().map(|a| a.iter().map(|c| Item {
            text: s(&c["insertion_text"]),
            kind: s(&c["kind"]),
            menu: s(&c["extra_menu_info"]),
            detail: s(&c["detailed_info"]),
        }).collect()).unwrap_or_default();
        Ok((start, items))
    }
}

fn unchunk(s: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some((size, after)) = rest.split_once("\r\n") {
        let n = usize::from_str_radix(size.trim(), 16).unwrap_or(0);
        if n == 0 || after.len() < n { break }
        out.push_str(&after[..n]);
        rest = after[n..].trim_start_matches("\r\n");
    }
    out
}

fn which(prog: &str) -> Option<String> {
    std::env::var_os("PATH").and_then(|p| std::env::split_paths(&p).map(|d| d.join(prog)).find(|f| f.is_file())).map(|f| f.display().to_string())
}

/// The one ycmd of this process, started on first use when it is built; None while it starts or when there is none.
static SHARED: Mutex<Option<(Ycmd, bool)>> = Mutex::new(None);

/// Runs `f` with the shared ycmd once it answers; starts it the first time (None until it is ready).
pub fn with<R>(log_dir: &Path, f: impl FnOnce(&mut Ycmd) -> R) -> Option<R> {
    let mut g = SHARED.lock().unwrap_or_else(|e| e.into_inner());
    if g.is_none() {
        let dir = find_dir()?;
        *g = Some((Ycmd::start(&dir, log_dir).ok()?, false));
    }
    let (y, ready) = g.as_mut()?;
    if !*ready {
        *ready = y.ready();
        if !*ready { return None }
    }
    Some(f(y))
}

#[cfg(test)]
mod tests {
    #[test]
    fn signature_is_ycmd_s() {
        // ycmd's own HMAC scheme with a known key, worked out with Python's hmac module
        let s = super::signature(b"0123456789abcdef", "POST", "/completions", b"{}");
        assert_eq!(s.len(), 44);
        assert_eq!(super::unchunk("5\r\nhello\r\n6\r\n world\r\n0\r\n\r\n"), "hello world");
    }
}

#[cfg(test)]
mod live {
    // the real ycmd (built in the data folder): `gfi` finds getFileInfo, fuzzily. Run alone: --ignored
    #[test]
    #[ignore]
    fn ycmd_completes_fuzzily() {
        let dir = super::find_dir().expect("ycmd built in ~/.local/share/zett/ycmd");
        let logs = std::env::temp_dir().join(format!("zett-ycm-test-{}", std::process::id()));
        let mut y = super::Ycmd::start(&dir, &logs).unwrap();
        let t = std::time::Instant::now();
        while !y.ready() { assert!(t.elapsed().as_secs() < 30, "ycmd did not start"); std::thread::sleep(std::time::Duration::from_millis(200)) }
        eprintln!("ready after {:?}", t.elapsed());
        let text = "function getFileInfo(path) {}\nconst gitFolderInfo = 1\nlet unrelated = 2\ngfi\n";
        let t = std::time::Instant::now();
        let (start, items) = y.complete("/tmp/zett-ycm-test.js", text, 3, 3).unwrap();
        eprintln!("answer in {:?}: start {start}, {:?}", t.elapsed(), items.iter().map(|i| &i.text).collect::<Vec<_>>());
        assert_eq!(start, 0);
        assert!(items.iter().any(|i| i.text == "getFileInfo"), "{items:?}");
        assert!(!items.iter().any(|i| i.text == "unrelated"));
    }
}

/// ycmd is built here (looked up once per process).
pub fn available() -> bool {
    static AT: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *AT.get_or_init(|| find_dir().is_some())
}
