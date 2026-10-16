# zett — tasks

See PLAN.md for the vision and milestones.

## M0 — foundation
- [x] PLAN.md, README, workspace layout
- [x] zett-core: rope buffer for plain files (open, save atomically, edit, positions) + tests
- [x] zett-core: block tree on Loro (ids, types, rich text, children, move, export/import, merge) + tests
- [x] Markdown mirror: page ⇄ .md (headings, paragraphs, bullets, todos, code, quotes, front matter) + tests
- [ ] Markdown mirror: whole folder ⇄ workspace, watching both sides
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
