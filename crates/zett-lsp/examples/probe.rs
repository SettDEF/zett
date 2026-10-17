// Starts a server on a file and prints what it says for a while: zett-lsp's smoke test.
fn main() {
    let file = std::path::PathBuf::from(std::env::args().nth(1).expect("a file"));
    let file = std::fs::canonicalize(file).unwrap();
    let secs: u64 = std::env::args().nth(2).and_then(|s| s.parse().ok()).unwrap_or(30);
    let root = file.parent().and_then(|p| p.parent()).unwrap().to_path_buf();
    let mut servers = zett_lsp::Servers::new();
    let text = std::fs::read_to_string(&file).unwrap();
    {
        let (c, lang) = servers.for_file(&file, &root).expect("no server");
        c.did_open(&file, lang, &text);
    }
    let start = std::time::Instant::now();
    let mut asked = false;
    while start.elapsed().as_secs() < secs {
        for e in servers.events() {
            println!("{:>5.1}s {e:?}", start.elapsed().as_secs_f32());
        }
        if !asked && start.elapsed().as_secs() > secs / 2 {
            asked = true;
            let (c, _) = servers.for_file(&file, &root).unwrap();
            println!("encoding: {}", c.encoding.lock().unwrap());
            c.hover(&file, 7, 20);
            c.definition(&file, 7, 20);
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
}
