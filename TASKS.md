# zett — tasks

See PLAN.md for the vision and milestones.

## Direction (2026-10-20): Emacs by default, configurable like Neovim
- **Default feel = Emacs:** modeless editing, `C-x`/`C-c` chords, `M-x` for every command,
  self-documenting (`C-h k/f/v/m`), everything is a buffer, change it live while it runs.
- **Vim stays:** a built-in "evil" profile, one option away (`zett.opt.keys = "vim"`). Today's
  Vim work (95/95 = Neovim) becomes that profile, not the engine.
- **Configuration languages:** Lua (exists), Fennel (a Lisp that compiles to Lua: the Emacs
  flavour without a second VM), Rust (plugins as WASM components, same API as Lua), TOML for
  plain options (also what a settings page writes).
- Today the Vim modes live inside the editing core (`zett-edit`: `Mode`, `keys.rs`) and Lua
  keymaps hang off them. That is the first thing to change.

### P0 — the core everything else stands on
- [~] **Command system:** every action is a named command (name, docs, arguments, interactive
  prompts); keys only point at commands. `M-x` lists them all. Vim keys move from `keys.rs`
  into commands + a keymap.
  - [x] 38 editing commands by name (zett-edit `emacs.rs`), run from keys or M-x as `Effect::Edit`;
    listed with the Lua commands (source "editor")
  - [ ] numeric argument (`C-u 4`, `M-4`), interactive prompts for arguments
  - [ ] Vim's keys as commands + a keymap (today still the hard-wired grammar in `keys`)
- [x] **Startup is the config's:** the scratch buffer by default (as Emacs); `zett.opt.startup` =
  "dashboard" or any command; `zett.on("start")` effects reach the editor; the dashboard's
  banner, buttons and footer from `zett.dashboard{…}`
- [~] **Keymap engine, Emacs style:** (done: chords `C-x C-f`, which-key per prefix, C-g, `M-`
  as Alt, typed keys normalized; open: keymap precedence by mode) prefix chords (`C-x C-f`), keymaps with precedence
  (minor mode > major mode > global), which-key for every prefix, `C-g` cancels anything.
  Vim's modes become one keymap layer.
- [~] **Modeless editing:** `C-a C-e C-f C-b M-f M-b`, `C-k`/`C-y` with a kill ring, `M-y`,
  mark and region (`C-SPC`, `C-w`, `M-w`), `C-/` undo, `C-s`/`C-r` incremental search.
  (done: all but incremental search, which still opens the search picker; open: the region drawn
  on screen, the kill ring shared with the system clipboard)
- [ ] **Minibuffer:** one completion UI for `M-x`, find-file, switch-buffer and help (fuzzy,
  history, annotations: vertico + orderless + marginalia in one).
- [ ] **Major and minor modes:** one per file type (markdown, rust, lua…), toggles (line
  numbers, autosave, whitespace), hooks on enable/disable.
- [ ] **Config loading + safety:** `init.lua` / `init.fnl` / `config.toml` in a fixed order,
  live reload (exists), errors shown in a buffer, never a crash; `zett --safe` starts without
  any config.
- [ ] **Self-documentation:** `C-h k` (what does this key do), `C-h f` (command), `C-h v`
  (option), `C-h m` (active modes and their keys), jump to the definition. Builds on `:help`
  and docs.rs.

### P1 — configuration languages
- [ ] Lua API made stable and versioned: `zett.cmd`, `zett.map`, `zett.opt`, `zett.hook`,
  `zett.mode`; documented from the source.
- [ ] Fennel: bundle fennel.lua, load `init.fnl` and `plugins/*.fnl`, macros for `defcmd`,
  `defmode`, `map!`.
- [ ] Rust plugins: WASM components (wasmtime, WIT interface mirroring the Lua API),
  hot reload, permissions per plugin.
- [ ] `config.toml` for plain options; the settings page writes it.
- [ ] Plugin manifest + package manager (below): `plugin.toml`, lockfile, `zett sync`.

### P2 — what people come to Emacs for
- [ ] Everything a buffer: messages, help, explorer, terminal, undo tree (today panels).
- [ ] Org-like outlines on the block tree: fold, promote/demote, TODO states, an agenda across
  pages (and Dotos).
- [ ] Kill ring + registers + keyboard macros (`C-x (`, `C-x )`, `C-x e`).
- [ ] Windows the Emacs way: `C-x 2/3/o/0/1` (the window work exists; map it).
- [ ] isearch, `query-replace`, project-wide search.
- [ ] A Magit-style git buffer.

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
- [x] performance budgets (docs/PERFORMANCE.md) + checker: `cargo run --release -p zett-edit --example perf` — all met
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
- [x] slash menu (verified: /to → To-do)
- [ ] drag to reorder blocks, block references ((id)) (M2)
- [x] visual mode + text objects + f/t in zett-edit (95/95 = Neovim) and in the window (selection drawn, VISUAL / V-LINE)
- [ ] search in a block and across the page (/ ? n N), dot-repeat, registers, macros (M1)
- [ ] M2½: `zett -t` terminal front end (ratatui), read-only first
- [x] Lua engine (zett-script): init.lua, plugins (require), commands, keymaps with leader + which-key + timeout, hooks, zett.* API
- [x] window: SPC tree with which-key panel, `:` command line (lua …, page …, capture …), help for keys/commands, notices; CLI run/lua/commands/keys
- [ ] plugin manifest (plugin.toml: permissions, version), lockfile, `zett sync`, `zett doctor`
- [ ] more API: blocks by id, cursor/selection, async jobs, timers, UI panels from Lua
