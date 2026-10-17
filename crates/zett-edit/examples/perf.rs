//! Performance budgets (docs/PERFORMANCE.md), checked: `cargo run --release -p zett --example perf`.
//! Exits non-zero when a budget is broken.
use std::time::{Duration, Instant};
use zett_core::{markdown, Kind, Workspace};
use zett_edit::Editor;

struct Check { failed: usize }
impl Check {
    fn time<T>(&mut self, what: &str, budget: Duration, mut f: impl FnMut() -> T) -> T {
        // best of 3: the budget is about the code, not a background hiccup
        let mut best = Duration::MAX;
        let mut out = None;
        for _ in 0..3 {
            let t = Instant::now();
            out = Some(f());
            best = best.min(t.elapsed());
        }
        let ok = best <= budget;
        if !ok { self.failed += 1; }
        println!("{}  {:<48} {:>10.3?}  (budget {:?})", if ok { "ok  " } else { "SLOW" }, what, best, budget);
        out.unwrap()
    }
}

fn main() {
    let mut c = Check { failed: 0 };
    let ms = Duration::from_millis;
    let us = Duration::from_micros;

    // ── editing in a 100k-line buffer ──
    let text: String = (0..100_000).map(|i| format!("line {i} with some words in it\n")).collect();
    let mut e = Editor::from_str(&text);
    e.feed("50000G");
    c.time("one editing key (x), 100k lines", us(50), || e.feed("x"));
    c.time("motion (w), 100k lines", us(50), || e.feed("w"));
    c.time("dd, 100k lines", us(200), || e.feed("dd"));
    c.time("p, 100k lines", us(200), || e.feed("p"));
    c.time("u, 100k lines", us(200), || e.feed("u"));

    // ── the block tree ──
    let ws = c.time("create 100k blocks", ms(500), || {
        let ws = Workspace::new();
        for p in 0..100 {
            let page = ws.new_page(&format!("page {p}")).unwrap();
            for b in 0..1000 {
                ws.append(page, if b % 10 == 0 { Kind::Heading(2) } else { Kind::Paragraph }, "some text in a block").unwrap();
            }
        }
        ws
    });
    let bytes = c.time("export 100k blocks", ms(300), || ws.export().unwrap());
    println!("      snapshot size: {:.1} MB", bytes.len() as f64 / 1e6);
    c.time("import 100k blocks", ms(300), || Workspace::import(&bytes).unwrap());
    let page = ws.pages()[0];
    c.time("children of a 1k-block page", ms(2), || ws.children(page).len());

    // ── Markdown mirror ──
    let big = Workspace::new();
    let p = big.new_page("big").unwrap();
    for i in 0..10_000 { big.append(p, Kind::Paragraph, &format!("paragraph {i}")).unwrap(); }
    let md = c.time("Markdown of a 10k-block page", ms(20), || markdown::to_markdown(&big, p));
    c.time("read a 10k-block page from Markdown", ms(100), || markdown::from_markdown(&Workspace::new(), &md, "x").unwrap());

    println!("{}", if c.failed == 0 { "all budgets met".to_string() } else { format!("{} budget(s) broken", c.failed) });
    std::process::exit((c.failed > 0) as i32);
}
