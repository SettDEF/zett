//! Debugging: a debug adapter (gdb -i=dap by default) as a child process speaking the Debug Adapter
//! Protocol; requests go out framed, everything it says comes back through `poll` for QML to act on.
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{channel, Receiver};

struct Session {
    child: Child,
    stdin: ChildStdin,
    rx: Receiver<String>,
    seq: i64,
}

thread_local! {
    static SESSIONS: RefCell<HashMap<i32, Session>> = RefCell::new(HashMap::new());
    static NEXT: RefCell<i32> = const { RefCell::new(1) };
}

/// Starts an adapter (argv as JSON, e.g. ["gdb", "-i=dap"]): its id, -1 when it does not start.
pub fn start(argv: &str, cwd: &str) -> i32 {
    let argv: Vec<String> = serde_json::from_str(argv).unwrap_or_else(|_| vec!["gdb".into(), "-i=dap".into()]);
    let Some((bin, args)) = argv.split_first() else { return -1 };
    let mut cmd = Command::new(bin);
    cmd.args(args).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null());
    if std::path::Path::new(cwd).is_dir() {
        cmd.current_dir(cwd);
    }
    let Ok(mut child) = cmd.spawn() else { return -1 };
    let (Some(stdin), Some(out)) = (child.stdin.take(), child.stdout.take()) else { return -1 };
    let (tx, rx) = channel();
    std::thread::spawn(move || {
        let mut r = BufReader::new(out);
        loop {
            // Content-Length: N, a blank line, N bytes of JSON
            let mut len = 0usize;
            loop {
                let mut line = String::new();
                if r.read_line(&mut line).unwrap_or(0) == 0 { return }
                let l = line.trim();
                if l.is_empty() { break }
                if let Some(v) = l.strip_prefix("Content-Length:") { len = v.trim().parse().unwrap_or(0) }
            }
            let mut body = vec![0u8; len];
            if r.read_exact(&mut body).is_err() { return }
            if tx.send(String::from_utf8_lossy(&body).into_owned()).is_err() { return }
        }
    });
    let id = NEXT.with(|n| { let mut n = n.borrow_mut(); *n += 1; *n - 1 });
    SESSIONS.with(|s| s.borrow_mut().insert(id, Session { child, stdin, rx, seq: 0 }));
    id
}

/// A request: its sequence number (the response carries it as request_seq), -1 when the session is gone.
pub fn request(id: i32, command: &str, args: &str) -> i32 {
    SESSIONS.with(|s| {
        let mut s = s.borrow_mut();
        let Some(ses) = s.get_mut(&id) else { return -1 };
        ses.seq += 1;
        let args: serde_json::Value = serde_json::from_str(args).unwrap_or(serde_json::json!({}));
        let msg = serde_json::json!({ "seq": ses.seq, "type": "request", "command": command, "arguments": args }).to_string();
        let framed = format!("Content-Length: {}\r\n\r\n{msg}", msg.len());
        if ses.stdin.write_all(framed.as_bytes()).and_then(|_| ses.stdin.flush()).is_err() { return -1 }
        ses.seq as i32
    })
}

/// Everything the adapter said since the last call, as a JSON array; `{"gone": true}` once it has exited.
pub fn poll(id: i32) -> String {
    SESSIONS.with(|s| {
        let mut s = s.borrow_mut();
        let Some(ses) = s.get_mut(&id) else { return "[]".into() };
        let mut out: Vec<String> = ses.rx.try_iter().collect();
        if matches!(ses.child.try_wait(), Ok(Some(_))) && out.is_empty() {
            out.push(r#"{"type":"gone"}"#.into());
        }
        format!("[{}]", out.join(","))
    })
}

pub fn stop(id: i32) {
    SESSIONS.with(|s| if let Some(mut ses) = s.borrow_mut().remove(&id) { let _ = ses.child.kill(); let _ = ses.child.wait(); });
}

#[cfg(test)]
mod tests {
    // gdb's own DAP: initialize answers with its capabilities
    #[test]
    fn gdb_speaks_dap() {
        if std::process::Command::new("gdb").arg("--version").output().is_err() { return }
        let id = super::start(r#"["gdb","-i=dap"]"#, ".");
        assert!(id > 0);
        super::request(id, "initialize", r#"{"adapterID":"zett","linesStartAt1":true,"columnsStartAt1":true}"#);
        let end = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let mut got = String::new();
        while std::time::Instant::now() < end && !got.contains("\"initialize\"") {
            got.push_str(&super::poll(id));
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        super::stop(id);
        assert!(got.contains("supportsConfigurationDoneRequest"), "{got}");
    }
}
