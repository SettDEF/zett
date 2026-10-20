//! zett — the command line. The window comes next (see TASKS.md); everything
//! here also works headless, for scripts and agents.
//!
//! A workspace is a folder of Markdown files: `--dir`, $ZETT_DIR, or ~/zett.

mod window;

use clap::{CommandFactory, Parser, Subcommand};
use clap::builder::styling::{AnsiColor, Effects, Styles};

const STYLES: Styles = Styles::styled()
    .header(AnsiColor::White.on_default().effects(Effects::BOLD))
    .usage(AnsiColor::White.on_default().effects(Effects::BOLD))
    .literal(AnsiColor::Cyan.on_default())
    .placeholder(AnsiColor::BrightBlack.on_default())
    .error(AnsiColor::Red.on_default().effects(Effects::BOLD));
use std::path::PathBuf;
use zett_core::folder::Folder;
use zett_core::{markdown, Kind};

const EXAMPLES: &str = "\x1b[1mExamples:\x1b[0m
  zett                              the editor in this terminal (its dashboard)
  zett src/main.rs                  open files; a folder opens its file finder
  zett --gui                        the notes window (pages, blocks)
  zett doc                          the documentation (also :help in the editor)
  zett new \"Ideas\"
  zett add ideas \"build zash\" --todo
  zett ls
  zett show ideas
  zett edit ideas 'jjdd'            Vim keys on the page's Markdown, saved back
  zett --dir ~/notes ls

A workspace is a folder of Markdown files: --dir, $ZETT_DIR, or ~/zett.";

#[derive(Parser)]
#[command(name = "zett", version, about = "An editor you live in: code with Vim's keys and Doom's, pages, Lua everywhere", after_help = EXAMPLES, args_conflicts_with_subcommands = true, styles = STYLES)]
struct Cli {
    /// files or a folder to open (in the terminal; with --gui in the window)
    paths: Vec<PathBuf>,
    /// the window instead of the terminal
    #[arg(long)]
    gui: bool,
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
    /// Run a command (the same ones keys and `:` run; `zett commands` lists them)
    Run {
        command: String,
        /// the page it works on (default: none)
        #[arg(long)]
        page: Option<String>,
    },
    /// Run Lua with the `zett` API (as `:lua` in the window)
    #[command(after_help = "Examples:\n  zett lua '#zett.pages()'\n  zett lua --page ideas 'zett.page.add(\"todo\", \"from a script\")'")]
    Lua {
        code: String,
        #[arg(long)]
        page: Option<String>,
    },
    /// List every command (built-in, plugins, init.lua)
    Commands,
    /// Read the documentation (a topic, or the list of them)
    #[command(after_help = "Examples:\n  zett doc            the topics\n  zett doc lsp        language servers\n  zett doc keys")]
    Doc { topic: Option<String> },
    /// List the key mappings of a mode (n = normal)
    Keys {
        #[arg(default_value = "n")]
        mode: String,
    },
    /// Shell completions
    Completions {
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}

/// `zett doc TOPIC`: a topic, headings bold and `code` in colour on a terminal.
fn doc(topic: &str) {
    use std::io::IsTerminal;
    let Some((_, text)) = zett_script::docs::get(topic) else {
        eprintln!("zett: no topic \"{topic}\"; the topics:");
        for (n, about, _) in zett_script::docs::TOPICS {
            eprintln!("  {n:<10} {about}");
        }
        std::process::exit(2);
    };
    let tty = std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none();
    for line in text.lines() {
        if !tty {
            println!("{line}");
        } else if let Some(h) = line.strip_prefix("# ").or_else(|| line.strip_prefix("## ")) {
            println!("\x1b[1m{h}\x1b[0m");
        } else if line.starts_with("    ") {
            println!("\x1b[36m{line}\x1b[0m");
        } else {
            // `code` in cyan
            let mut out = String::new();
            for (i, part) in line.split('`').enumerate() {
                if i % 2 == 1 { out.push_str(&format!("\x1b[36m{part}\x1b[0m")) } else { out.push_str(part) }
            }
            println!("{out}");
        }
    }
}

fn fail(msg: &str, code: i32) -> ! {
    eprintln!("zett: {msg}");
    std::process::exit(code);
}

fn main() {
    let cli = Cli::parse();
    let dir = cli.dir.clone().unwrap_or_else(|| std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default().join("zett"));
    let Some(cmd) = cli.cmd else {
        if cli.gui {
            // the window, on the workspace folder
            if std::env::var_os("ZETT_DIR").is_none() {
                unsafe { std::env::set_var("ZETT_DIR", &dir) };
            }
            window::run();
            return;
        }
        // no command: the editor in this terminal (files, a folder, or here)
        if let Err(e) = zett_tui::run(cli.paths) {
            fail(&e.to_string(), 1);
        }
        return;
    };
    if let Cmd::Completions { shell } = cmd {
        clap_complete::generate(shell, &mut Cli::command(), "zett", &mut std::io::stdout());
        return;
    }
    if let Cmd::Doc { topic } = &cmd {
        if topic.as_deref().is_some_and(|t| t.len() >= 3 && "commands".starts_with(t)) {
            let mut e = zett_script::Engine::new(None);
            e.set_frontend("editor");
            e.load_config(&zett_script::config_dir());
            print!("{}", zett_script::docs::commands_topic(&e));
            return;
        }
        return doc(topic.as_deref().unwrap_or(""));
    }
    let mut f = Folder::open(&dir).unwrap_or_else(|e| fail(&format!("{}: {e}", dir.display()), 1));
    if matches!(cmd, Cmd::Run { .. } | Cmd::Lua { .. } | Cmd::Commands | Cmd::Keys { .. }) {
        return script(cmd, f, cli.json);
    }
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
        Cmd::Completions { .. } | Cmd::Doc { .. } | Cmd::Run { .. } | Cmd::Lua { .. } | Cmd::Commands | Cmd::Keys { .. } => unreachable!(),
    }
}

/// The commands that go through the Lua engine (with the user's init.lua and plugins).
fn script(cmd: Cmd, f: Folder, json: bool) {
    use zett_script::{Effect, Engine};
    let folder = std::rc::Rc::new(std::cell::RefCell::new(f));
    let mut e = Engine::new(Some(folder.clone()));
    e.load_config(&zett_script::config_dir());
    for err in &e.errors {
        eprintln!("zett: {err}");
    }
    let set_page = |e: &Engine, page: &Option<String>| {
        if let Some(name) = page {
            let p = folder.borrow().find(name).unwrap_or_else(|| fail(&format!("no page \"{name}\""), 3));
            e.set_current(Some(p));
        }
    };
    let report = |fx: Vec<Effect>| {
        for x in fx {
            match x {
                Effect::Notify(m) => println!("{m}"),
                Effect::Open(p) => println!("opened: {}", folder.borrow().ws.text(p)),
                Effect::Ui(name, _) => eprintln!("zett: that opens {name} in the window (run `zett` for it)"),
                Effect::Edit(name, _) => eprintln!("zett: {name} edits text in the editor (run `zett -t`)"),
                Effect::Refresh => {}
            }
        }
    };
    match cmd {
        Cmd::Run { command, page } => {
            set_page(&e, &page);
            match e.run(&command) {
                Ok(fx) => report(fx),
                Err(err) => fail(&err, 2),
            }
        }
        Cmd::Lua { code, page } => {
            set_page(&e, &page);
            match e.eval(&code) {
                Ok((out, fx)) => {
                    report(fx);
                    if out != "nil" {
                        println!("{out}");
                    }
                }
                Err(err) => fail(&err, 2),
            }
        }
        Cmd::Commands => {
            let list = e.commands();
            if json {
                let v: Vec<_> = list.iter().map(|(n, d, s)| serde_json::json!({ "name": n, "desc": d, "source": s })).collect();
                println!("{}", serde_json::Value::Array(v));
            } else {
                for (n, d, s) in list {
                    println!("{n:<20} {d:<44} {s}");
                }
            }
        }
        Cmd::Keys { mode } => {
            let leader = e.leader();
            for (k, d) in e.bindings(&mode) {
                // shown as typed: <leader> spelled out as the key it is
                let shown = k.replace("<leader>", if leader == "<Space>" { "SPC " } else { &leader });
                println!("{shown:<16} {d}");
            }
        }
        _ => unreachable!(),
    }
}
