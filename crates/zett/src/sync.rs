//! `zett sync`: the workspace folder between your devices, no server: peers on the network find each other
//! (UDP broadcast), prove they share the pairing key (HMAC), and exchange the files that changed since they last met.
//! Both sides changed a file: the local text stays, the other side's is kept beside it as NAME.conflict-HOST.EXT.
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream, UdpSocket};
use std::path::{Path, PathBuf};
use std::time::Duration;

type HmacSha256 = Hmac<Sha256>;
pub const PORT: u16 = 47123;
const MAX_FILE: u64 = 16 << 20;

/// Where this device keeps its key, its id and what it last agreed on with each peer.
pub struct Setup {
    pub dir: PathBuf,
    pub state: PathBuf,
    pub key: [u8; 32],
    pub id: String,
}

fn data_dir() -> PathBuf {
    std::env::var_os("XDG_DATA_HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".local/share")).join("zett")
}
fn state_dir() -> PathBuf {
    std::env::var_os("XDG_STATE_HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".local/state")).join("zett/sync")
}

/// The pairing code: what you type on the other device. The key is its hash.
pub fn key_of(code: &str) -> [u8; 32] {
    let c: String = code.chars().filter(|c| c.is_ascii_alphanumeric()).collect::<String>().to_ascii_lowercase();
    Sha256::digest(format!("zett-sync 1 {c}")).into()
}

fn random_hex(n: usize) -> String {
    let mut b = vec![0u8; n];
    if std::fs::File::open("/dev/urandom").and_then(|mut f| f.read_exact(&mut b)).is_err() {
        let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos();
        b = Sha256::digest(format!("{t}{}", std::process::id())).to_vec()[..n].to_vec();
    }
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// A new pairing code (shown once on the first device), or the one given (typed on the next).
pub fn pair(code: Option<&str>) -> std::io::Result<String> {
    let code = code.map(str::to_string).unwrap_or_else(|| {
        let h = random_hex(8);
        format!("{}-{}-{}-{}", &h[0..4], &h[4..8], &h[8..12], &h[12..16])
    });
    let d = data_dir();
    std::fs::create_dir_all(&d)?;
    std::fs::write(d.join("sync.code"), &code)?;
    Ok(code)
}

impl Setup {
    /// This device's setup, or None when it was never paired (`zett sync --pair`).
    pub fn load(dir: &Path) -> Option<Setup> {
        let code = std::fs::read_to_string(data_dir().join("sync.code")).ok()?;
        let idf = data_dir().join("sync.id");
        let id = std::fs::read_to_string(&idf).ok().filter(|s| !s.trim().is_empty()).unwrap_or_else(|| {
            let host = std::fs::read_to_string("/etc/hostname").unwrap_or_default().trim().to_string();
            let id = format!("{}-{}", if host.is_empty() { "device" } else { &host }, random_hex(3));
            let _ = std::fs::write(&idf, &id);
            id
        });
        Some(Setup { dir: dir.to_path_buf(), state: state_dir(), key: key_of(code.trim()), id: id.trim().to_string() })
    }
}

fn mac(key: &[u8], parts: &[&str]) -> String {
    let mut m = HmacSha256::new_from_slice(key).expect("any key length");
    for p in parts {
        m.update(p.as_bytes());
        m.update(b"\0");
    }
    m.finalize().into_bytes().iter().map(|x| format!("{x:02x}")).collect()
}

/// The files that sync: everything under the folder but hidden ones (.git, .zett) and big ones; path → hash.
fn manifest(dir: &Path) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    fn walk(base: &Path, d: &Path, out: &mut BTreeMap<String, String>) {
        let Ok(rd) = std::fs::read_dir(d) else { return };
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with('.') { continue }
            let p = e.path();
            let Ok(md) = e.metadata() else { continue };
            if md.is_dir() {
                walk(base, &p, out);
            } else if md.is_file() && md.len() <= MAX_FILE && !name.ends_with(".tmp") {
                if let (Ok(rel), Ok(bytes)) = (p.strip_prefix(base), std::fs::read(&p)) {
                    out.insert(rel.to_string_lossy().to_string(), hex(&Sha256::digest(&bytes)));
                }
            }
        }
    }
    walk(dir, dir, &mut out);
    out
}
fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// What two devices agreed on last time (path → hash), kept per peer.
fn base_file(s: &Setup, peer: &str) -> PathBuf {
    let tag = hex(&Sha256::digest(s.dir.to_string_lossy().as_bytes()))[..12].to_string();
    s.state.join(format!("{}-{tag}.tsv", peer.replace(['/', '\t', '\n'], "_")))
}
fn read_base(s: &Setup, peer: &str) -> BTreeMap<String, String> {
    std::fs::read_to_string(base_file(s, peer)).unwrap_or_default().lines().filter_map(|l| l.split_once('\t')).map(|(a, b)| (a.to_string(), b.to_string())).collect()
}
fn write_base(s: &Setup, peer: &str, m: &BTreeMap<String, String>) {
    let _ = std::fs::create_dir_all(&s.state);
    let body: String = m.iter().map(|(k, v)| format!("{k}\t{v}\n")).collect();
    let f = base_file(s, peer);
    let tmp = f.with_extension("tmp");
    if std::fs::write(&tmp, body).is_ok() {
        let _ = std::fs::rename(&tmp, &f);
    }
}

/// A path the other side sent: inside the folder, nothing hidden, no `..`.
fn safe(dir: &Path, rel: &str) -> Option<PathBuf> {
    let p = Path::new(rel);
    let ok = !rel.is_empty() && p.components().all(|c| matches!(c, std::path::Component::Normal(n) if !n.to_string_lossy().starts_with('.')));
    ok.then(|| dir.join(p))
}
fn put(dir: &Path, rel: &str, bytes: &[u8]) -> std::io::Result<()> {
    let f = safe(dir, rel).ok_or_else(|| std::io::Error::other(format!("refused path {rel}")))?;
    if let Some(p) = f.parent() { std::fs::create_dir_all(p)? }
    let tmp = f.with_extension("zett-sync.tmp");
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, &f)
}
fn del(dir: &Path, rel: &str) {
    if let Some(f) = safe(dir, rel) { let _ = std::fs::remove_file(f); }
}
fn conflict_name(rel: &str, host: &str) -> String {
    let p = Path::new(rel);
    let stem = p.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    let ext = p.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
    let name = format!("{stem}.conflict-{host}{ext}");
    p.parent().filter(|d| !d.as_os_str().is_empty()).map(|d| d.join(&name).to_string_lossy().to_string()).unwrap_or(name)
}

// ── the wire: one JSON value per line, file bodies as hex inside it ──

struct Wire {
    r: BufReader<TcpStream>,
    w: TcpStream,
}
impl Wire {
    fn new(s: TcpStream) -> std::io::Result<Wire> {
        s.set_read_timeout(Some(Duration::from_secs(30)))?;
        Ok(Wire { r: BufReader::new(s.try_clone()?), w: s })
    }
    fn send(&mut self, v: &serde_json::Value) -> std::io::Result<()> {
        let mut s = v.to_string();
        s.push('\n');
        self.w.write_all(s.as_bytes())
    }
    fn recv(&mut self) -> std::io::Result<serde_json::Value> {
        let mut line = String::new();
        // a file is at most 16 MB: 32 MB of hex and a little
        (&mut self.r).take(40 << 20).read_line(&mut line)?;
        if line.is_empty() { return Err(std::io::Error::other("the other side closed")) }
        serde_json::from_str(&line).map_err(std::io::Error::other)
    }
}
fn unhex(s: &str) -> Option<Vec<u8>> {
    (s.len() % 2 == 0).then(|| (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok()).collect::<Option<Vec<u8>>>())?
}
fn read_rel(dir: &Path, rel: &str) -> Option<String> {
    safe(dir, rel).and_then(|f| std::fs::read(f).ok()).map(|b| hex(&b))
}

/// Both sides prove they hold the key; each learns the other's id. `first` speaks first (the one that connected).
fn handshake(w: &mut Wire, s: &Setup, first: bool) -> std::io::Result<String> {
    let mine = random_hex(16);
    let bad = || std::io::Error::other("the other device has a different pairing code");
    if first {
        w.send(&serde_json::json!({ "hello": "zett-sync 1", "id": s.id, "nonce": mine }))?;
        let v = w.recv()?;
        let (peer, theirs) = (v["id"].as_str().unwrap_or_default().to_string(), v["nonce"].as_str().unwrap_or_default().to_string());
        if v["proof"].as_str() != Some(&mac(&s.key, &["s", &mine, &peer])) { return Err(bad()) }
        w.send(&serde_json::json!({ "proof": mac(&s.key, &["c", &theirs, &s.id]) }))?;
        if w.recv()?["ok"] != true { return Err(bad()) }
        Ok(peer)
    } else {
        let v = w.recv()?;
        if v["hello"] != "zett-sync 1" { return Err(std::io::Error::other("not zett sync")) }
        let (peer, theirs) = (v["id"].as_str().unwrap_or_default().to_string(), v["nonce"].as_str().unwrap_or_default().to_string());
        w.send(&serde_json::json!({ "id": s.id, "nonce": mine, "proof": mac(&s.key, &["s", &theirs, &s.id]) }))?;
        let ok = w.recv()?["proof"].as_str() == Some(&mac(&s.key, &["c", &mine, &peer]));
        w.send(&serde_json::json!({ "ok": ok }))?;
        if !ok { return Err(bad()) }
        Ok(peer)
    }
}

/// What one sync did, for the report.
#[derive(Default, Debug)]
pub struct Report {
    pub peer: String,
    pub got: Vec<String>,
    pub sent: Vec<String>,
    pub deleted_here: Vec<String>,
    pub deleted_there: Vec<String>,
    pub conflicts: Vec<String>,
}

/// The connecting side decides; the other answers its asks. Returns what changed.
pub fn sync_with(s: &Setup, addr: &str) -> std::io::Result<Report> {
    let sock = TcpStream::connect_timeout(&resolve(addr)?, Duration::from_secs(3))?;
    let mut w = Wire::new(sock)?;
    let peer = handshake(&mut w, s, true)?;
    w.send(&serde_json::json!({ "manifest": true }))?;
    let theirs: BTreeMap<String, String> = serde_json::from_value(w.recv()?["manifest"].clone()).map_err(std::io::Error::other)?;
    let mine = manifest(&s.dir);
    let base = read_base(s, &peer);
    let mut rep = Report { peer: peer.clone(), ..Default::default() };
    let paths: BTreeSet<&String> = mine.keys().chain(theirs.keys()).chain(base.keys()).collect();
    for p in paths {
        let (l, r, b) = (mine.get(p), theirs.get(p), base.get(p));
        if l == r { continue }
        if l == b {
            // only they changed it
            match r {
                Some(_) => rep.got.push(p.clone()),
                None => rep.deleted_here.push(p.clone()),
            }
        } else if r == b {
            match l {
                Some(_) => rep.sent.push(p.clone()),
                None => rep.deleted_there.push(p.clone()),
            }
        } else if l.is_none() {
            // deleted here, changed there: their text comes back
            rep.got.push(p.clone());
        } else if r.is_none() {
            rep.sent.push(p.clone());
        } else {
            rep.conflicts.push(p.clone());
        }
    }
    // their side's files: the ones taken and the conflicting ones
    let want: Vec<&String> = rep.got.iter().chain(&rep.conflicts).collect();
    w.send(&serde_json::json!({ "get": want }))?;
    let files = w.recv()?;
    for p in &rep.got {
        let b = files["files"][p].as_str().and_then(unhex).ok_or_else(|| std::io::Error::other(format!("{p} did not come")))?;
        put(&s.dir, p, &b)?;
    }
    let mut conflict_copies = Vec::new();
    for p in &rep.conflicts {
        let b = files["files"][p].as_str().and_then(unhex).ok_or_else(|| std::io::Error::other(format!("{p} did not come")))?;
        let c = conflict_name(p, &peer);
        put(&s.dir, &c, &b)?;
        conflict_copies.push((c, hex(&b)));
    }
    for p in &rep.deleted_here {
        del(&s.dir, p);
    }
    // ours to them: changed files, the conflicts' local text (theirs kept there as a copy too), deletions
    let mut send = serde_json::Map::new();
    for p in rep.sent.iter().chain(&rep.conflicts) {
        if let Some(h) = read_rel(&s.dir, p) { send.insert(p.clone(), h.into()); }
    }
    for (c, h) in &conflict_copies {
        send.insert(c.clone(), h.clone().into());
    }
    w.send(&serde_json::json!({ "put": send, "delete": rep.deleted_there, "conflicts": rep.conflicts, "from": s.id }))?;
    let done = w.recv()?;
    if done["ok"] != true { return Err(std::io::Error::other(done["error"].as_str().unwrap_or("the other device refused").to_string())) }
    // both sides now hold the same files: that is the next base
    let now = manifest(&s.dir);
    write_base(s, &peer, &now);
    w.send(&serde_json::json!({ "base": now }))?;
    Ok(rep)
}

/// Answers one connection (the other side decides).
fn answer(s: &Setup, sock: TcpStream) -> std::io::Result<String> {
    let mut w = Wire::new(sock)?;
    let peer = handshake(&mut w, s, false)?;
    w.recv()?;
    w.send(&serde_json::json!({ "manifest": manifest(&s.dir) }))?;
    let ask = w.recv()?;
    let mut files = serde_json::Map::new();
    for p in ask["get"].as_array().into_iter().flatten().filter_map(|v| v.as_str()) {
        if let Some(h) = read_rel(&s.dir, p) { files.insert(p.to_string(), h.into()); }
    }
    w.send(&serde_json::json!({ "files": files }))?;
    let put_ = w.recv()?;
    let res = (|| -> std::io::Result<()> {
        // the conflicting files: ours kept as a copy before theirs replaces it
        for p in put_["conflicts"].as_array().into_iter().flatten().filter_map(|v| v.as_str()) {
            if let Some(b) = read_rel(&s.dir, p).and_then(|h| unhex(&h)) { put(&s.dir, &conflict_name(p, &s.id), &b)?; }
        }
        for (p, h) in put_["put"].as_object().into_iter().flatten() {
            let b = h.as_str().and_then(unhex).ok_or_else(|| std::io::Error::other("bad file"))?;
            // the copy of our own text that the other side made is already here under our name
            if p.contains(&format!(".conflict-{}", s.id)) { continue }
            put(&s.dir, p, &b)?;
        }
        for p in put_["delete"].as_array().into_iter().flatten().filter_map(|v| v.as_str()) {
            del(&s.dir, p);
        }
        Ok(())
    })();
    match &res {
        Ok(()) => w.send(&serde_json::json!({ "ok": true }))?,
        Err(e) => w.send(&serde_json::json!({ "ok": false, "error": e.to_string() }))?,
    }
    res?;
    let _ = w.recv();
    write_base(s, &peer, &manifest(&s.dir));
    Ok(peer)
}

fn resolve(addr: &str) -> std::io::Result<std::net::SocketAddr> {
    use std::net::ToSocketAddrs;
    let a = if addr.contains(':') { addr.to_string() } else { format!("{addr}:{PORT}") };
    a.to_socket_addrs()?.next().ok_or_else(|| std::io::Error::other(format!("{addr}: no address")))
}

/// Waits for peers: answers each connection and each "who is there" on the network. `once`: one sync, then back.
pub fn serve(s: &Setup, port: u16, once: bool, mut log: impl FnMut(String)) -> std::io::Result<()> {
    let l = TcpListener::bind(("0.0.0.0", port))?;
    let port = l.local_addr()?.port();
    // the answer to discovery: a proof we hold the key, so strangers learn nothing useful
    if let Ok(u) = UdpSocket::bind(("0.0.0.0", PORT)) {
        let key = s.key;
        let id = s.id.clone();
        std::thread::spawn(move || {
            let mut buf = [0u8; 512];
            while let Ok((n, from)) = u.recv_from(&mut buf) {
                let q = String::from_utf8_lossy(&buf[..n]).to_string();
                if let Some(nonce) = q.strip_prefix("zett-sync? ") {
                    let _ = u.send_to(format!("zett-sync! {port} {id} {}", mac(&key, &["here", nonce.trim()])).as_bytes(), from);
                }
            }
        });
    }
    log(format!("waiting for your other devices on port {port} (Ctrl+C stops)"));
    for c in l.incoming() {
        let Ok(c) = c else { continue };
        let from = c.peer_addr().map(|a| a.ip().to_string()).unwrap_or_default();
        match answer(s, c) {
            Ok(peer) => log(format!("synced with {peer} ({from})")),
            Err(e) => log(format!("{from}: {e}")),
        }
        if once { break }
    }
    Ok(())
}

/// The devices on this network that hold the same key: their addresses.
pub fn discover(s: &Setup, wait: Duration) -> Vec<(String, String)> {
    let Ok(u) = UdpSocket::bind(("0.0.0.0", 0)) else { return vec![] };
    let _ = u.set_broadcast(true);
    let _ = u.set_read_timeout(Some(Duration::from_millis(150)));
    let nonce = random_hex(8);
    let q = format!("zett-sync? {nonce}");
    let _ = u.send_to(q.as_bytes(), ("255.255.255.255", PORT));
    let _ = u.send_to(q.as_bytes(), ("127.0.0.1", PORT));
    let end = std::time::Instant::now() + wait;
    let mut out: Vec<(String, String)> = Vec::new();
    let mut buf = [0u8; 512];
    while std::time::Instant::now() < end {
        let Ok((n, from)) = u.recv_from(&mut buf) else { continue };
        let a = String::from_utf8_lossy(&buf[..n]).to_string();
        let f: Vec<&str> = a.split(' ').collect();
        if f.len() == 4 && f[0] == "zett-sync!" && f[3] == mac(&s.key, &["here", &nonce]) && f[2] != s.id && !out.iter().any(|(_, id)| id == f[2]) {
            out.push((format!("{}:{}", from.ip(), f[1]), f[2].to_string()));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup(name: &str, id: &str, code: &str) -> Setup {
        let root = std::env::temp_dir().join(format!("zett-sync-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("ws")).unwrap();
        Setup { dir: root.join("ws"), state: root.join("state"), key: key_of(code), id: id.into() }
    }
    fn once(a: &Setup, b: &Setup) -> Result<Report, String> {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = l.local_addr().unwrap().port();
        let bs = Setup { dir: b.dir.clone(), state: b.state.clone(), key: b.key, id: b.id.clone() };
        let t = std::thread::spawn(move || { let c = l.accept().unwrap().0; answer(&bs, c).map_err(|e| e.to_string()) });
        let r = sync_with(a, &format!("127.0.0.1:{port}")).map_err(|e| e.to_string());
        let _ = t.join().unwrap();
        r
    }
    fn read(s: &Setup, p: &str) -> Option<String> {
        std::fs::read_to_string(s.dir.join(p)).ok()
    }

    #[test]
    fn two_devices_end_with_the_same_files() {
        let (a, b) = (setup("a1", "laptop", "abcd-1234"), setup("b1", "desk", "ABCD 1234"));
        std::fs::write(a.dir.join("ideas.md"), "# Ideas\n").unwrap();
        std::fs::create_dir_all(b.dir.join("notes")).unwrap();
        std::fs::write(b.dir.join("notes/plan.md"), "plan\n").unwrap();
        std::fs::create_dir_all(a.dir.join(".git")).unwrap();
        std::fs::write(a.dir.join(".git/HEAD"), "x").unwrap();
        let r = once(&a, &b).unwrap();
        assert_eq!((r.sent.len(), r.got.len()), (1, 1));
        assert_eq!(read(&b, "ideas.md").as_deref(), Some("# Ideas\n"));
        assert_eq!(read(&a, "notes/plan.md").as_deref(), Some("plan\n"));
        assert!(!b.dir.join(".git").exists(), "hidden folders stay home");

        // one side edits, the other deletes another file: both carried over, nothing conflicts
        std::fs::write(b.dir.join("ideas.md"), "# Ideas\nmore\n").unwrap();
        std::fs::remove_file(a.dir.join("notes/plan.md")).unwrap();
        let r = once(&a, &b).unwrap();
        assert!(r.conflicts.is_empty());
        assert_eq!(read(&a, "ideas.md").as_deref(), Some("# Ideas\nmore\n"));
        assert!(read(&b, "notes/plan.md").is_none(), "the deletion reached the other side");

        // both edit the same file: the local text wins on both, the other kept beside it on both
        std::fs::write(a.dir.join("ideas.md"), "from laptop\n").unwrap();
        std::fs::write(b.dir.join("ideas.md"), "from desk\n").unwrap();
        let r = once(&a, &b).unwrap();
        assert_eq!(r.conflicts, vec!["ideas.md".to_string()]);
        for s in [&a, &b] {
            assert_eq!(read(s, "ideas.md").as_deref(), Some("from laptop\n"));
            assert_eq!(read(s, "ideas.conflict-desk.md").as_deref(), Some("from desk\n"), "in {}", s.id);
        }
        // and the next sync has nothing to do
        let r = once(&a, &b).unwrap();
        assert!(r.got.is_empty() && r.sent.is_empty() && r.conflicts.is_empty(), "{r:?}");
    }

    #[test]
    fn a_device_with_another_code_gets_nothing() {
        let (a, b) = (setup("a2", "laptop", "abcd-1234"), setup("b2", "desk", "other-code"));
        std::fs::write(b.dir.join("secret.md"), "x").unwrap();
        assert!(once(&a, &b).is_err());
        assert!(read(&a, "secret.md").is_none());
    }

    #[test]
    fn paths_from_the_other_side_stay_inside_the_folder() {
        let d = std::env::temp_dir();
        assert!(safe(&d, "../x").is_none() && safe(&d, "/etc/passwd").is_none() && safe(&d, ".git/config").is_none());
        assert!(safe(&d, "notes/a.md").is_some());
        assert_eq!(conflict_name("notes/a.md", "desk"), "notes/a.conflict-desk.md");
    }
}
