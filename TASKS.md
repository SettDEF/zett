# zett — tasks

See PLAN.md for the vision and milestones.

## M0 — foundation
- [x] PLAN.md, README, workspace layout
- [x] zett-core: rope buffer for plain files (open, save atomically, edit, positions) + tests
- [ ] zett-core: block tree on Loro (ids, types, properties, rich text, children, move)
- [ ] Markdown mirror: workspace folder ⇄ block tree (headings, paragraphs, lists, tasks, code, quotes, front matter)
- [ ] zett-edit: key parsing, normal/insert, first motions and edits over a block's text
- [ ] zett: `--exec` headless (open workspace, run keys/commands, print or save)
- [ ] zett-app: cxx-qt bridge; QML window with sidebar (pages) and page view
- [ ] tinct palette in the window; app id `zett`
- [ ] tests: core + mirror round-trips + headless window capture
- [ ] move file I/O (Buffer::open/save) out of zett-core into the app/CLI, so the core stays WASM-clean
- [ ] check `cargo build -p zett-core -p zett-edit --target wasm32-unknown-unknown` in the test script
