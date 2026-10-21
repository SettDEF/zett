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
  - [x] numeric argument (zett-edit `prefix.rs`): `C-u` (4, ×4), `C-u 12`, `M-0…9`, `M--` /
    `C-u -` as the commands universal-, digit-, negative-argument; negative runs the opposite
    (M-- M-f = M-b), `C-u N C-k` kills N lines, `C-u 5 a` types five; Lua reads `zett.arg`
  - [ ] interactive prompts for arguments (through the minibuffer)
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
  (done, incremental search too: C-s / C-r as Emacs's isearch, the keys in the bundled isearch
  plugin, the search in zett-edit `search.rs`, the matcher Buffer::find over the rope; M-s o is the
  line search C-s used to open. The region is drawn and
  transient: C-SPC activates it, an edit, M-w or C-g ends it; M-< / M-> leave a mark without one.
  Kills go to the system clipboard and a yank takes in what was copied elsewhere: wl-copy/wl-paste,
  OSC 52 without Wayland.)
  - [x] one kill ring for every buffer (zett-edit `killring.rs`, shared by the App through
    sync_profile): a kill in one buffer yanks in another, also without a system clipboard
- [x] **Minibuffer:** one completion UI for `M-x`, find-file, switch-buffer and help (fuzzy,
  history, annotations: vertico + orderless + marginalia in one). zett-tui `minibuf.rs`: words in
  any order, `!word` excludes, notes and keys in columns, choices remembered per kind (M-p / M-n),
  Emacs editing keys, Tab takes the candidate; at the bottom (`zett.opt.minibuffer = "bottom"`,
  the Emacs default) or a float (Vim's).
  - [ ] Lua prompts (`zett.input`) and find-file's path walking through it
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

### Beyond Notion and Emacs (2026-10-21: the ten ideas, in this order: 5 1 3 2 4 6 7 8 9 10)
- [x] 5 structure keys for notes and code (outline.rs)
- [x] 1 time travel (travel.rs; the window's pages through Loro still open)
- [x] 3 the agent as a collaborator (zett-edit suggest.rs, zett-tui agent.rs): C-c a / SPC a a asks
  (Haiku by default, `claude -p`, no tools: zett.opt.agent_model, agent_command) about the region or
  the section; the answer comes back as suggestions per changed block, the old lines struck through
  and the new ones drawn under them with the agent's tag; C-c y / n take or drop one, C-c ] [ move,
  C-c Y / N all; each taken one is its own undo step; they follow their lines through edits
  - [ ] streaming (the API), several questions at once, the window (pages)
- [x] 2 live query blocks (zett-tui query.rs): a ```query block in a Markdown note (tasks [done|all], todo,
  grep TEXT, files GLOB, recent 7d, links NAME; in DIR, limit N) draws its results under the closing
  fence, looked up again every 10 s; C-c C-o / SPC s q lists them all to jump to one; `:help query`
  - [ ] the window (pages): the same blocks through Loro; a query over block properties once blocks have them
- [x] 4 block references (zett-tui refs.rs): a line `![[target#part]]` shows the note, file, heading or
  code block (by outline section) under it, read again every 3 s; C-c C-o follows it; every note ends
  with what links to it, from notes and code comments; C-c b / SPC s b lists them
  - [ ] references by block id (once blocks have ids, M2), editing through a reference, the window
- [x] 6 notebook blocks (zett-tui notebook.rs): C-c C-c / SPC m r runs a fenced sh, bash, zsh, fish,
  python, js, lua or ruby block in the note's folder (background, 30 s, C-g stops waiting); its output
  and exit under the block, tied to that code; C-c C-v / SPC m i writes it in as an ```output block
  (replacing the last one)
  - [ ] sessions (a REPL kept between blocks), variables from one block to the next, the window
- [x] 7 search everything (zett-tui everything.rs, zett-edit history_search): C-c s / SPC s e, the files
  (ripgrep) and every saved history's deleted or replaced text (⟲ file@state · when), Enter on a past
  hit opens time travel at the state that has it; histories get an index (hash → path) with each save
  - [ ] the window's pages (Loro history), closed buffers' unsaved changes
- [x] 8 git buffer (zett-tui git.rs): C-x g / SPC g s, Magit's status: s u x on files and groups, Tab
  diff inline, Enter opens, c commits (echo area), g, q; Enter on a commit shows it; C-c g b / SPC g b:
  the commits that changed the section at the cursor (git log -L), Enter shows one
  - [ ] hunks (stage part of a file), push / pull / branches, a blame column
- [ ] 9 sync without a server
- [ ] 10 canvas

### P2 — what people come to Emacs for
- [ ] Everything a buffer: messages, help, explorer, terminal, undo tree (today panels).
- [~] Org-like outlines on the block tree: fold, promote/demote, TODO states, an agenda across
  pages (and Dotos).
  - [x] one set of structure keys for notes and code (zett-edit `outline.rs`): Markdown headings,
    code blocks by indentation and closing brackets; next/previous/same level/up (C-c C-n …),
    move a subtree (M-↑/↓), promote/demote it (M-←/→: # count, or one indent step), fold (Tab on
    a heading, C-c Tab, S-Tab for all; Vim: za zM zR zj zk), narrow (C-x n s / n / w), mark
    (C-c @); folds remember their heading and follow it through any edit
  - [ ] tree-sitter as a third source (exact blocks per language), LSP symbols as a fourth
  - [ ] TODO states and the agenda
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
- [~] undo history per page that survives a restart (through the CRDT)
  - [x] files in the code editor: the undo tree saved with each save (state/zett/history/undo,
    only the changes, with the text's fingerprint: a file changed elsewhere starts fresh)
  - [x] time travel (C-c t, M-x time-travel, SPC U): ←→ through every state with a slider and
    when, the whole buffer as it was; Enter goes back (later states kept), b brings one section
    back as it was (the rest stays), v…y copies from the past
  - [ ] pages in the window: the same through Loro's history
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
