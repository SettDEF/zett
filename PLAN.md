# zett — plan

**A programmable, local-first workspace where documents are alive.**
Notion's pages and databases, Neovim's editing, Emacs's "everything is
programmable", and a Quarry-board canvas — in one Rust app that is instant,
works offline, syncs between your devices without a server, and can run code.

## Why it beats Notion (and Neovim, and Emacs)

| | Notion | Neovim / Emacs | **zett** |
|---|---|---|---|
| Speed | web app, seconds to open a page | instant | **instant**: native Rust, 100k-block workspaces, < 8 ms keypress to screen |
| Offline / ownership | cloud, your data on their servers | local files | **local-first**: your files; sync device-to-device (CRDT), no server needed |
| Editing | mouse-first, few shortcuts | best in class | **Vim grammar in every block, table cell and canvas card**; `hjkl` moves cards |
| Programmable | almost nothing (API from outside) | everything (Lua / Lisp) | **everything**: Lua for config and commands, WASM plugins, and plugins can add new **block types with their own UI** |
| Live content | static blocks, slow formulas | none | **live blocks**: code that runs (Lua, shell, Rust, Python if you want it) with output kept in the page; **queries** over the workspace, files, git, your desktop |
| Data | databases with properties, relations, rollups | none | the same, plus **SQL over the whole workspace** and views over **any source**: a folder, git log, Dotos tasks, Kalends events, Vitals history, a JSON API |
| Reuse | synced blocks (limited) | none | **every block has an id**: embed it anywhere and edits flow both ways; backlinks for blocks, not only pages |
| History | page history (paid) | undo tree | **time travel on every block**: scrub history, branch a page, merge it back |
| Canvas | none | none | **canvas board**: pages, blocks, images, live widgets as cards, arrows, groups; a dashboard is just a canvas of live blocks |
| Desktop | a browser tab | a terminal | **part of your desktop**: Quarry images, Reel video timestamps, screenshots, Dotos tasks, Kalends events drop in as blocks; the shell can show any block as a widget |
| AI | Notion AI (cloud, paid) | plugins | **agents as collaborators**: Claude works in a page like a person, edits blocks with undo, asks inline; Dotos's agents land here |
| Interop | export only | plain text | **Markdown mirror**: every page is also a `.md` file, kept in sync both ways, so git, grep and Neovim keep working |

## The graph: see how everything connects

A **graph view** of the whole workspace (pages, blocks, tags, projects as
nodes; links, embeds and references as edges), force-directed, coloured by
project or tag, clusters you can zoom into. Like Obsidian's graph, but live,
filterable by any query, and it works **in the terminal too**: drawn with the
Kitty graphics protocol where the terminal has it (and zett's own terminal
does), Braille dots everywhere else. Click or `hjkl` to a node and open it.

## zett and zash: the terminal is a neighbour, not a copy

The terminal is **zash**, its own project (/mnt/storage/dev/projects/zash,
plan in its PLAN.md): Rust, GPU-drawn, tinct-themed, with shell marks, a
completion popup and (planned) native splits and a Neovim mode. zett does not
grow a second terminal; it **embeds zash's engine as a pane** (a `zash-core`
crate split out of zash: session, emulation, renderer), so both get every
improvement at once. What the pairing adds:

- **Commands are blocks.** zash already knows where each command, its output
  and its exit status are (OSC 133 marks). In zett a finished command becomes
  a block: link it from a page, embed it, search all past output, rerun it,
  diff two runs.
- **Vim in the scrollback**: zett-edit's grammar moves, selects and yanks
  through output; `/` searches it.
- **Live terminal blocks**: a page holds a running terminal (a dev server, a
  log tail) next to its notes, and keeps it across restarts.
- **Agents in the terminal**: an agent block runs commands in a pane you
  watch, each step approvable and undoable.
- **zett inside zash**: the terminal version of zett (below) runs in a zash
  pane and draws images, the graph and rich rows through zash's protocols.

## Two front ends on one engine: the window and the terminal

zett is **one engine with two faces**, like Neovim with a GUI. Everything
below the screen (blocks, editing, commands, Lua, plugins, sync) is shared
code; only drawing and input differ. A plugin written once works in both.

| | **zett** (window) | **zett** in a terminal (`zett -t`, or automatic over SSH / without a display) |
|---|---|---|
| Drawn with | QML through cxx-qt, GPU | ratatui on any terminal; richer in zash (images, true sizes) |
| Pages | blocks with real typography, images, embeds | the same blocks as styled text: headings bold and coloured, tasks as `☐/☑`, code highlighted, images as Kitty graphics or a placeholder |
| Editing | Vim grammar in every block, mouse too | the identical Vim grammar (same zett-edit), keyboard first, mouse where the terminal reports it |
| Navigation | sidebar, tabs, splits, Ctrl+P, `SPC` | the same `SPC` tree and which-key popup, splits and tabs drawn with box lines |
| Canvas / graph | full canvas, zoom, drag | the graph in Braille dots or Kitty graphics; canvas as a read-mostly map with `hjkl` between cards |
| Databases | tables, boards, calendars | tables and boards in cells, sortable and editable |
| Where it shines | daily work on this desktop | SSH, servers, a tmux / zash pane next to code, a 10 ms start |

Both open the **same folder at the same time**: an edit in one shows up in
the other within a moment (the folder mirror already does this for the window
and Neovim).

**Browser**: the WASM build of the same engine (extension, web viewer).

## Plugins and extensions: as deep as Vim's, safer than Vim's

Vim and Emacs last because users can change anything. zett aims for that
reach without the usual cost (slow starts, plugins that break each other,
code that can read your whole disk).

**Three levels**, from a line of config to a full program:

1. **`init.lua`** — settings, keymaps, small commands, hooks. Reloads live.
   ```lua
   zett.map("n", "<leader>td", function() zett.page.add_block("todo", os.date("%F")) end)
   zett.on("page.saved", function(p) if p.tags.journal then zett.git.commit(p) end end)
   ```
2. **Lua plugins** — a folder with `plugin.toml` and Lua files. Commands,
   keymaps, block types (with a declarative layout), views, sources for
   queries, themes, snippets, language support. Lazy-loaded on first use.
3. **WASM plugins** — Rust (or anything that compiles to WASM) for heavy work:
   parsers, renderers, sync backends, data sources. Run in wasmtime with a
   fuel/memory limit, so a plugin cannot freeze the app.

**What a plugin can add** (each through the same API the built-ins use):

| Extension point | Example |
|---|---|
| Commands + keymaps | `SPC g b` blame the block's file, a `:Translate` command |
| Block types | a Mermaid diagram, a chess board, a stock chart, a Kanban card |
| Views | a timeline view, a mind map, a Gantt chart |
| Sources (for queries / tables) | GitHub issues, RSS, a SQLite file, Home Assistant |
| Text features | snippets, abbreviations, spell check, an LSP client for code blocks |
| Themes | colours (tinct is the default), fonts, spacing |
| Exporters / publishers | PDF, a blog, slides from a page |
| Agents | a review agent, a summariser, a research helper |

**Safety and order**, the things that make large Vim setups painful:

- **Permissions in the manifest** (`net = ["api.github.com"]`, `fs = ["~/Notes"]`,
  `run = ["git"]`), asked once and shown in the plugin page; WASM plugins
  cannot reach anything not granted. Lua plugins run with the same checks on
  the zett API (the raw `io`/`os` modules are off unless granted).
- **A lockfile** (`zett.lock`): exact versions and hashes; `zett sync`
  installs, updates and rolls back; a broken update is one command to undo.
- **Measured cost**: `zett doctor` and `SPC h p` show each plugin's start
  time and memory; a plugin that slows the start past the budget is named.
- **Sources**: a git URL, a local folder, or a plain registry (a git repo of
  manifests); no account needed.
- **Hot reload** while developing a plugin, a REPL (`SPC :` for Lua), and
  `zett plugin new` to scaffold one with tests.
- **Built-ins are plugins too**: tables, the canvas, the graph and the Doom
  layer are written against the public API, which keeps that API complete.

## Emacs's applications, the zett way

Emacs is more than an editor because everything is a buffer, so one set of
skills (search, motions, macros) works in mail, git, files and search results,
and editing a view changes the world behind it. In zett everything is a block,
which allows the same one level higher:

- **Edit the view, change the world.** A folder block where renaming a line
  renames the file (dired), search results you edit to change every file
  (wgrep), a git status block where deleting a line unstages it.
- **A git view as good as Magit**: stage lines and hunks with Vim keys,
  commit, rebase, history as blocks you can link.
- **Capture and time tracking**: `SPC x` capture templates (also from the
  desktop through Dotos's Super+R), a clock on any task, reports as queries.
- **Agenda**: every task with a date across the workspace, plus Dotos and
  Kalends, in one view (Org agenda).
- **Export**: a page as slides, PDF, or a website (with "publish").
- **Remote workspaces**: open a zett folder over SSH as if it were local
  (TRAMP), naturally with `zett -t` on the other side.
- **Tables with formulas** (Org tables) as part of the databases.
- **Mail and feeds as sources**: a query block shows "unread from X" or new
  posts next to the notes about them.

Not needed: a window manager (the quickshell desktop is that, and blocks can
be shell widgets) and a built-in browser (tela-zen).

## How a workspace is stored

- **A folder of Markdown files** stays the truth you can see: one `.md` per
  page, readable and editable by anything (git, grep, Neovim). Already working.
- **`.zett/` next to it** keeps what Markdown cannot: block ids, the CRDT
  history (Loro snapshots + updates), canvas layouts, database schemas, the
  search index. Deleting it loses history, never text.
- **Block ids in Markdown** only where something points at a block (a short
  `^id` at the end of the line, as in other Markdown tools), so files stay clean.
- **Edits from outside** are merged into the CRDT as a change by "disk", so
  undo and history cover them too.
- **Attachments** (images, PDFs) live in `assets/` with content-hash names;
  the same file is stored once.

## The Doom Emacs layer: great out of the box

Emacs is powerful but empty; Doom Emacs made it loved by shipping an
opinionated, beautiful, fast setup on top. zett ships that from day one:

- **`SPC` leader with a which-key popup.** Every command is two or three
  mnemonic keys away (`SPC f f` find file, `SPC s p` search the workspace,
  `SPC p c` new page, `SPC w v` split), and the popup shows what comes next,
  so nothing has to be memorised. Same tree in every view, including the
  canvas and the Firefox extension.
- **Modules, toggled in one file.** `~/.config/zett/init.lua` lists modules
  (`lang.rust`, `ui.zen`, `tools.git`, `tools.agents`, `desktop.quarry`,
  `canvas`, `sync`…) with flags (`+lsp`, `+inline`). Each module is a bundle
  of commands, keymaps, blocks and settings. Turn one off and it is gone,
  including its keys.
- **Curated defaults.** tinct theme, Vim keys, good fonts, sensible splits,
  autosave, relative numbers in text view. A fresh install feels finished.
- **`zett doctor`** explains what is wrong (missing LSP server, broken
  module, slow plugin), and **`zett sync`** installs, updates and pins modules
  and plugins from a lockfile.
- **Fast start.** Modules load lazily; cold start to an open page in under
  150 ms, measured on every release.
- **Popups, not windows.** Search, help, the REPL, agents, and terminals open
  as dismissable popups (`Esc` closes), like Doom's popup rules.
- **Help everywhere.** `SPC h k` describes a key, `SPC h c` a command, `SPC h b`
  a block type, with the source one key away. Like Emacs's self-documenting
  help, which is the thing that makes it learnable.

## zett everywhere

The same engine, not a copy: `zett-core` and `zett-edit` build for native **and
WebAssembly**, so the browser pieces edit with exactly the app's Vim grammar.

| Piece | What it does |
|---|---|
| **Firefox extension** (like Firenvim, but better) | zett's modal editing in **every text field on the web**: GitHub, Reddit, mail, chat. Inline in the field, or pop the field out into a zett floating editor. Your `init.lua` keymaps apply. Plus Vim-style page navigation (link hints, scroll, tab keys) |
| **Web clipper** (same extension) | Clip a page, a selection, an image or a video timestamp into a zett page as blocks, with the source kept as a property |
| **Browser ⇄ app bridge** | The extension talks to the running app through `zett serve` (native messaging): edit a text field in the full app, search your workspace from the address bar, open the page you are reading next to your notes |
| **tela-zen integration** | The same features built into your own browser, through its agent socket |
| **Website** | The project site (what it is, docs, download), **and published pages**: any page or canvas can be published as a static site with one command, styled like the app |
| **Web viewer** | Read and lightly edit a workspace in a browser tab (the WASM build), for devices without the app |
| **Neovim plugin** | Edit zett pages and blocks from Neovim, with links and search |
| **Shell widgets** | Any block or query can be a widget in the quickshell desktop |
| **CLI** | `zett add`, `zett find`, `zett run`, pipes in and out, for scripts |

## The core ideas

1. **Everything is a block in one tree.** A workspace is a tree of blocks
   (pages are blocks with children). Each block has an id, a type, content and
   properties. Text, tasks, code, tables, databases, embeds, canvas cards:
   all blocks. Views (text, page, canvas, table, kanban, calendar) are lenses
   on the same tree.
2. **The tree is a CRDT** (Loro: rich text, movable trees, maps, history).
   That gives offline editing, device-to-device sync, collaboration, time
   travel and branches from one data model, instead of bolting each on later.
3. **Commands all the way down.** Every action is a command with an id and
   params. Keys, menus, the palette, Lua, plugins and agents all call the same
   commands, so anything you can do, a script or an agent can do, with undo.
4. **Blocks can be programs.** A block type is: data schema + render (QML or a
   declarative layout) + commands. Built-in types use the same API as plugins.
5. **Live data.** A query block (`from tasks where due < today`) or a source
   block (git, folder, HTTP, Dotos, Vitals) produces rows; tables, charts and
   boards render them and update when the data changes.

## Architecture

```
crates/
  zett-core     block tree on Loro (CRDT): ids, types, properties, rich text,
                history, branches; the Markdown mirror (both directions)
  zett-edit     modal engine: Vim grammar over any text (block, cell, file):
                counts, operators, motions, text objects, registers, macros
  zett-cmd      command registry, keymaps per mode and context, palette, `:`
  zett-query    the query language and sources (workspace, files, git,
                desktop apps, HTTP); incremental results
  zett-run      live code blocks: Lua in-process, others as sandboxed jobs
  zett-script   Lua 5.4 (mlua) API + WASM plugins (wasmtime): commands,
                block types, views, hooks
  zett-sync     device-to-device sync of the CRDT (LAN discovery, encrypted)
  zett-search   full-text index (tantivy) over blocks, kept incremental
  zett-app      cxx-qt bridge and the QML window (sidebar, tabs, splits,
                page / canvas / table / text views), tinct-themed
  zett-tui      the terminal front end (ratatui) over the same core
  zett          binary: `zett [path]`, `zett -t`, `zett --exec`, `zett serve` (agents, scripts)
  (zash-core)   from the zash project: terminal panes inside zett
```

Today `zett-core`, `zett-edit` and `zett` (CLI + window) exist; the rest are
split out when their milestone starts, not before.

## Milestones

Each one is usable on its own; order chosen so the data model is right first.
✅ done · 🟡 started · ⬜ not yet (status 2026-10-17)

1. ✅ **M0 — foundation.** Block tree on Loro with the Markdown mirror; the
   folder mirror (both directions, edits from other programs picked up); a
   window that shows a workspace as pages; the CLI (`ls show new add edit`);
   performance budgets measured and met.
2. 🟡 **M1 — editing.** Vim grammar in blocks (normal ✅ / insert ✅ /
   visual ✅ / `:` ⬜), block-level motions (move, indent, fold, zoom into a
   block), slash menu ✅, drag to reorder;
   undo through the CRDT. The `SPC` leader tree with the which-key popup and
   `SPC h` help.
3. 🟡 **M2 — linking.** `[[pages]]` ✅ (click opens or creates), backlinks ✅,
   quick open ✅ (Ctrl+P), `((block refs))`, embeds that edit both ways, tags,
   full-text search.
4. ⬜ **M2½ — zett in the terminal.** `zett -t`: the same pages, blocks, Vim
   keys and `SPC` tree in ratatui; images and true sizes inside zash. Early on
   purpose: it keeps the engine honest (nothing may depend on QML), and it is
   what runs over SSH.
5. **M3 — databases.** Properties, table / kanban / calendar / gallery views,
   relations, rollups, formulas; SQL over the workspace.
6. **M4 — canvas.** Cards, arrows, groups, zoom, live widgets; keyboard-driven.
7. **M5 — programmable.** `init.lua` with Doom-style modules and flags,
   `zett doctor`, `zett sync` with a lockfile; commands, keymaps, hooks;
   custom block types in Lua; live code blocks with stored output; a REPL.
8. **M6 — live data.** Sources (folder, git, HTTP, Dotos, Kalends, Vitals,
   Quarry, Reel), charts, dashboards; shell widgets from blocks.
9. **M7 — sync and history.** Device-to-device sync, time-travel slider,
   branches and merge.
10. **M8 — agents and plugins.** Agents as collaborators (Claude via `zett
   serve`), WASM plugins.
11. **M8½ — terminal panes and graph.** zash panes inside zett (command
    blocks, scrollback with Vim keys, live terminal blocks), the graph view in
    the window and in the terminal.
12. **M9 — everywhere.** Firefox extension (modal editing in every text field,
    link hints, clipper, native-messaging bridge), tela-zen integration,
    Neovim plugin, CLI verbs.
13. **M10 — on the web.** Project website with docs; publish pages and canvases
    as static sites; the WASM web viewer.

The Firefox extension's editing part can start early (after M1): it needs only
`zett-edit` compiled to WASM, not the rest of the app.

## Next up (in order)

1. Visual mode (`v`, `V`) and text objects (`iw`, `ap`, `i"`…) in zett-edit,
   checked against Neovim with the comparison harness.
2. Block motions: move a block (`Alt+j/k`), indent / outdent (`>>` `<<`),
   fold, zoom into a block.
3. Undo that survives a restart (history from the CRDT in `.zett/`).
4. `zett -t`: the terminal front end, first read-only, then editing.
5. Plugin manifests (permissions), a lockfile, `zett sync`, `zett doctor`.

## Quality: how we know it is good

- **Neovim as the referee.** Every editing feature runs the same keys in
  zett-edit and headless Neovim (`scripts-nvim-compare.sh`); a difference is a
  bug unless written down as deliberate. Today: 38/38 cases equal.
- **Budgets, measured** (docs/PERFORMANCE.md): keypress to screen < 8 ms,
  cold start < 150 ms, 100k-block workspaces; `examples/perf.rs` fails a
  release that misses one.
- **Headless captures** of the window (`ZETT_SHOT`) for every visual change,
  read before anything is called done; the same for the terminal front end.
- **Round-trip tests**: Markdown → blocks → Markdown is byte-identical for
  everything zett writes; CRDT merges fuzzed with random concurrent edits.
- **The WASM check**: `zett-core` and `zett-edit` build for
  `wasm32-unknown-unknown` in the test script, so the browser pieces never rot.

## Decided

- Rust everywhere; QML window through cxx-qt (one binary).
- The terminal is zash (own project); zett embeds its engine, never a copy.
- Two front ends (window, terminal) on one engine; plugins work in both.
- Plugins: `init.lua` → Lua plugins → WASM plugins, with permissions and a lockfile.
- Lua for configuration and quick extensions; WASM for heavy plugins.
- Loro as the CRDT; Markdown files mirror pages for interop.
- `zett-core` and `zett-edit` stay WASM-clean: no file system, threads or
  native-only crates in them; I/O lives in the app and the CLI.
- Name "zett" is a working title until the EU/DPMA registers are checked.

## Ask before (cannot be changed after release)

- The domain for the website (zett.dev and zett.app looked free on 2026-10-16).
- The Firefox extension id (`gecko.id`) and the add-ons store listing name.
- The app id / desktop file name and any package names.

## The terminal GUI in zash (planned 2026-10-28)

Rule (the user): keep it very simple. A small core that draws the workbench from zash's layout
library (zash `ui-panels`: tokens from the shell, layout engine, components, one description for
zash hints and plain text), and everything else plugs in later through hooks, not built into the core.

- Core first: the workbench (activity bar, Dotos-style sidebar with the tree, tabs + breadcrumb,
  editor with gutter, optional bottom panel, status bar), from zett's state, nothing else.
- Hooks next: a component can be added at a named place (`sidebar`, `panel`, `popup`, `statusbar`,
  `gutter`, `overlay`) by zett itself, Lua or a plugin, e.g. `zett.ui.on('popup', fn)`. The
  which-key popup is the first user of the popup hook; nothing in the core knows about it.
- Then, each as its own hook user: settings menu (search, categories, typed controls, live preview,
  writes readable lines to init.lua, changed filter + reset), command palette, which-key, code popups
  (completion, hover, signature, diagnostics), git gutter + diff panel, search panel, toasts, theme
  picker, minimap, pictures in the editor, agent panel.
- Mouse (drag tabs, resize the sidebar, click the tree) waits for zash's interaction step (stage 3).

### The core's four parts (all small)
1. Places (hooks): `sidebar`, `panel`, `popup`, `statusbar`, `gutter`, `overlay`.
2. Read-only state for hooks: `zett.ui.state()` (file, cursor, selection, mode, tabs, project),
   updated on change; no hook reaches into zett's internals.
3. A focus stack for keys: the top layer gets the keys, what it does not take falls to the one
   below (Escape closes a popup, the editor gets them back); popups and editor never fight.
4. Registries, one clean list each: commands (description + keys), options (type, default,
   description), key bindings per mode. Palette, which-key, settings and the shell's keybinds
   menu only show these lists.

### Performance rules (many plugins must not make it slow)
1. Registering costs nothing; a component is built the first time it opens (lazy, built in).
2. One render path: every component, from zett, Lua or a plugin, gives the same UI tree; layout
   and drawing once per frame, in Rust, for all together.
3. Only what changed is redone: a component names the state it reads; unchanged, its last tree
   is kept. Nothing changes, nothing is computed.
4. No Lua per frame: Lua runs on events only (key, file change, popup opened).
5. A time budget per hook (e.g. 1 ms per event), measured; over it, `:checkhealth` names it with
   its time and it can be switched off.
Checked with Vitals (that is what it is for): zett's CPU and memory at rest, while typing and with
popups open, before and after each hook user is added (`vitals top`, its stream, a recorded
history); the numbers go into this plan next to each step, and a step that costs more than its
budget is fixed before the next one starts.

## The window in the shell's look (planned 2026-10-28)

The window (qml/Main.qml) is restyled to the desktop's design: layered surfaces from tinct, the
content as one rounded card on the darker background, a Dotos-style sidebar (sections with headers,
pill rows with icons, mono counts), chips in the status line. Simple core, extras via hooks.

1. ✅ Frame: background, the page in a card, Dotos sidebar (search pill, PAGES + button, icon rows), status chips.
2. ✅ Empty page offers ways to begin; open pages as pill tabs (Ctrl+W, Ctrl+Tab); details card on the
   right (outline, linked from, info; Ctrl+\\).
3. Home page: recent pages, today's journal, open to-dos from every page; the window is never empty.
4. Sidebar places: Recent, Pinned, Tags, Files (the workspace folder), Journal; collapsible sections.
5. Command palette (Ctrl+K): every command with its key; quick open stays Ctrl+P.
6. Search everything (Ctrl+Shift+F) in the details card: results grouped by page, history too.
7. Settings page in the same look: theme, keys (Vim/Emacs), fonts, the config folder; live.
8. Splits: two pages side by side (drag a tab), each its own card.
9. Bottom drawer: a zash terminal, problems/agent output; toggled like the details card.
10. Split Main.qml into Theme / Sidebar / Tabs / PageView / Details / StatusBar / Palette (cut line ranges).
11. Hooks: Lua adds sidebar sections, details cards, status chips, palette entries into named slots.

Not VS Code: no activity rail, no square tabs or file-explorer look; pill tabs, cards, the shell's
layers and rounding, Dotos rows. Everything reachable by key, with the key shown next to it.

Measured 2026-10-28 (ZETT_SHOT_ENGINE=timing, offscreen, release): first frame 237-242 ms in 3 of 5 runs, 325 and 424 ms in two (machine busy); target < 300 ms.
Typing (ZETT_SHOT_ENGINE=typing:N, 60 letters, letter -> frame): paragraph median 6 ms, worst 8 ms; 120-line Rust block (live colours) median 6 ms, worst 8 ms; target < 8 ms.
