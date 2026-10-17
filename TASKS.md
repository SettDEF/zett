# zett — tasks

See PLAN.md for the vision and milestones.

## M0 — foundation
- [x] PLAN.md, README, workspace layout
- [x] zett-core: rope buffer for plain files (open, save atomically, edit, positions) + tests
- [x] zett-core: block tree on Loro (ids, types, rich text, children, move, export/import, merge) + tests
- [x] Markdown mirror: page ⇄ .md (headings, paragraphs, bullets, todos, code, quotes, front matter) + tests
- [x] Markdown mirror: whole folder ⇄ workspace, watching both sides
- [ ] zett-edit: key parsing, normal/insert, first motions and edits over a block's text
- [x] CLI: ls, show, new, add, edit (Vim keys on a page), completions, --json
- [x] window (cxx-qt + QML): sidebar of pages, page view with blocks, Markdown shortcuts, Enter/Backspace/arrows between blocks, saved to .md; self-capture via ZETT_SHOT
- [x] tinct palette in the window; icon + launcher (install.sh)
- [ ] tests: core + mirror round-trips + headless window capture
- [x] file I/O behind the `fs` feature; core builds for wasm32 without it
- [ ] check `cargo build -p zett-core -p zett-edit --target wasm32-unknown-unknown` in the test script
- [x] performance budgets (docs/PERFORMANCE.md) + checker: `cargo run --release -p zett --example perf` — all met
- [x] operation-based undo (cost of the change, not of the file): x/dd/p/u 2–4 µs in 100k lines
- [x] Neovim comparison harness (scripts-nvim-compare.sh)
- [ ] M1: undo/redo cursor placement matches Neovim (diff: `abc` + `ofoo<Esc>Obar<Esc>uu<C-r>` → nvim 1:1, zett 1:3)

## Done the night of 2026-10-16/17
- [x] folder mirror: a workspace is a folder of .md files (~/zett or $ZETT_DIR)
- [x] CLI: ls, show, new, add, edit (Vim keys), completions, --json
- [x] the first window: pages sidebar, block page view, Markdown shortcuts, autosave, tinct colours
- [x] Vim normal mode in the window (Esc): zett-edit per block, block cursor, NORMAL/INSERT, j/k across blocks

## Next
- [x] watch the folder: reload pages changed by other editors (Neovim, git pull)
- [ ] undo history per page that survives a restart (through the CRDT)
- [x] [[links]] (click opens or creates the page), backlinks under each page, Ctrl+P quick open
- [~] slash menu (written, not verified on screen yet)
- [ ] drag to reorder blocks, block references ((id)) (M2)
- [ ] visual mode, text objects, search in the window (M1)
- [ ] M2½: `zett -t` terminal front end (ratatui), read-only first
- [x] Lua engine (zett-script): init.lua, plugins (require), commands, keymaps with leader + which-key + timeout, hooks, zett.* API
- [x] window: SPC tree with which-key panel, `:` command line (lua …, page …, capture …), help for keys/commands, notices; CLI run/lua/commands/keys
- [ ] plugin manifest (plugin.toml: permissions, version), lockfile, `zett sync`, `zett doctor`
- [ ] more API: blocks by id, cursor/selection, async jobs, timers, UI panels from Lua
