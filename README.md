# zett

An editor you live in: Vim's editing, Emacs's programmability, Notion's pages
and a canvas board — all views of the same plain Markdown files.

Rust core, QML window (cxx-qt), Lua scripting. Early days: see [PLAN.md](PLAN.md)
for the architecture and milestones, and [TASKS.md](TASKS.md) for what is next.

```sh
cargo run -p zett -- notes/        # open a folder (the window)
cargo run -p zett -- --exec 'ihello<Esc>' file.md   # headless
cargo test
```
