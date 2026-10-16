//! zett — the command line. The window comes next (see TASKS.md); everything
//! here also works headless, for scripts and agents.
//!
//! A workspace is a folder of Markdown files: `--dir`, $ZETT_DIR, or ~/zett.

use clap::{CommandFactory, Parser, Subcommand};
use std::path::PathBuf;
use zett_core::folder::Folder;
use zett_core::{markdown, Kind};

const EXAMPLES: &str = "Examples:
  zett new \"Ideas\"
  zett add ideas \"build zash\" --todo
  zett ls
  zett show ideas
  zett edit ideas 'jjdd'            Vim keys on the page's Markdown, saved back
  zett --dir ~/notes ls

A workspace is a folder of Markdown files: --dir, $ZETT_DIR, or ~/zett.";

#[derive(Parser)]
#[command(name = "zett", version, about = "An editor you live in — pages, blocks, Vim everywhere", after_help = EXAMPLES)]
struct Cli {
    /// the workspace folder
    #[arg(long, global = true, env = "ZETT_DIR")]
    dir: Option<PathBuf>,
    /// machine-readable output
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    cmd: Option<Cmd>,
}

#[derive(Subcommand)]
enum Cmd {
    /// List the pages
    Ls,
    /// Print a page as Markdown
    Show { page: String },
    /// Make a new page
    New { title: String },
    /// Add a block to the end of a page
    Add {
        page: String,
        text: String,
        /// a task (with --done, a finished one)
        #[arg(long)]
        todo: bool,
        #[arg(long)]
        done: bool,
        /// a heading of this level (1–6)
        #[arg(long)]
        heading: Option<u8>,
        /// a bullet point
        #[arg(long)]
        bullet: bool,
    },
    /// Apply Vim keys to a page's Markdown and save it
    #[command(after_help = "Examples:\n  zett edit ideas 'Go- new idea<Esc>'\n  zett edit ideas 'gg/build<CR>dd'   (search lands in M1)")]
    Edit { page: String, keys: String },
    /// Shell completions
    Completions {
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}

fn fail(msg: &str, code: i32) -> ! {
    eprintln!("zett: {msg}");
    std::process::exit(code);
}

fn main() {
    let cli = Cli::parse();
    let dir = cli.dir.clone().unwrap_or_else(|| std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default().join("zett"));
    let Some(cmd) = cli.cmd else {
        println!("The zett window is not built yet — the command line works now.\n");
        let _ = Cli::command().print_help();
        return;
    };
    if let Cmd::Completions { shell } = cmd {
        clap_complete::generate(shell, &mut Cli::command(), "zett", &mut std::io::stdout());
        return;
    }
    let mut f = Folder::open(&dir).unwrap_or_else(|e| fail(&format!("{}: {e}", dir.display()), 1));
    let page_of = |f: &Folder, name: &str| f.find(name).unwrap_or_else(|| fail(&format!("no page \"{name}\" in {} (zett ls lists them)", dir.display()), 3));
    match cmd {
        Cmd::Ls => {
            let mut rows: Vec<(String, String, usize)> = f.ws.pages().into_iter().map(|p| {
                let file = f.path_of(p).and_then(|x| x.file_name()).map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
                (f.ws.text(p), file, f.ws.children(p).len())
            }).collect();
            rows.sort();
            if cli.json {
                let v: Vec<_> = rows.iter().map(|(t, file, n)| serde_json::json!({ "title": t, "file": file, "blocks": n })).collect();
                println!("{}", serde_json::Value::Array(v));
            } else if rows.is_empty() {
                println!("No pages yet in {} — try: zett new \"Ideas\"", dir.display());
            } else {
                for (t, file, n) in rows {
                    println!("{t:<32} {n:>4} blocks   {file}");
                }
            }
        }
        Cmd::Show { page } => {
            let p = page_of(&f, &page);
            print!("{}", markdown::to_markdown(&f.ws, p));
        }
        Cmd::New { title } => {
            let p = f.new_page(&title).unwrap_or_else(|e| fail(&e.to_string(), 1));
            f.save_page(p).unwrap_or_else(|e| fail(&e.to_string(), 1));
            println!("{}", f.path_of(p).map(|x| x.display().to_string()).unwrap_or_default());
        }
        Cmd::Add { page, text, todo, done, heading, bullet } => {
            let p = page_of(&f, &page);
            let kind = match (todo || done, heading, bullet) {
                (true, _, _) => Kind::Todo { checked: done },
                (_, Some(l), _) => Kind::Heading(l.clamp(1, 6)),
                (_, _, true) => Kind::Bullet,
                _ => Kind::Paragraph,
            };
            f.ws.append(p, kind, &text).unwrap_or_else(|e| fail(&e.to_string(), 1));
            f.save_page(p).unwrap_or_else(|e| fail(&e.to_string(), 1));
        }
        Cmd::Edit { page, keys } => {
            let p = page_of(&f, &page);
            let mut e = zett_edit::Editor::from_str(&markdown::to_markdown(&f.ws, p));
            e.feed(&keys);
            markdown::replace_page(&f.ws, p, &e.text()).unwrap_or_else(|e| fail(&e.to_string(), 1));
            f.save_page(p).unwrap_or_else(|e| fail(&e.to_string(), 1));
            print!("{}", markdown::to_markdown(&f.ws, p));
        }
        Cmd::Completions { .. } => unreachable!(),
    }
}
