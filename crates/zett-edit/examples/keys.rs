//! `cargo run -p zett-edit --example keys -- TEXT KEYS`: prints the text after
//! the keys and the cursor (1-based line, 1-based column), as Neovim reports them.
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let text = a[1].replace("\\n", "\n");
    let mut e = zett_edit::Editor::from_str(&text);
    e.feed(&a[2]);
    let c = e.cursor();
    println!("{}|{}:{}", e.text().replace('\n', "\\n"), c.line + 1, c.col + 1);
}
