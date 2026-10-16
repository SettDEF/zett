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
  zett-app      cxx-qt bridge and the QML window (sidebar, tabs, splits,
                page / canvas / table / text views), tinct-themed
  zett          binary: `zett [path]`, `zett --exec`, `zett serve` (agents, scripts)
```

## Milestones

Each one is usable on its own; order chosen so the data model is right first.

1. **M0 — foundation.** Block tree on Loro with the Markdown mirror; a window
   that shows a workspace as pages; `zett --exec` headless; tests.
2. **M1 — editing.** Vim grammar in blocks (normal / insert / visual / `:`),
   block-level motions (move, indent, fold, zoom into a block), slash menu,
   drag to reorder; undo through the CRDT. The `SPC` leader tree with the
   which-key popup and `SPC h` help.
3. **M2 — linking.** `[[pages]]`, `((block refs))`, embeds that edit both
   ways, backlinks, tags, quick open, full-text search.
4. **M3 — databases.** Properties, table / kanban / calendar / gallery views,
   relations, rollups, formulas; SQL over the workspace.
5. **M4 — canvas.** Cards, arrows, groups, zoom, live widgets; keyboard-driven.
6. **M5 — programmable.** `init.lua` with Doom-style modules and flags,
   `zett doctor`, `zett sync` with a lockfile; commands, keymaps, hooks;
   custom block types in Lua; live code blocks with stored output; a REPL.
7. **M6 — live data.** Sources (folder, git, HTTP, Dotos, Kalends, Vitals,
   Quarry, Reel), charts, dashboards; shell widgets from blocks.
8. **M7 — sync and history.** Device-to-device sync, time-travel slider,
   branches and merge.
9. **M8 — agents and plugins.** Agents as collaborators (Claude via `zett
   serve`), WASM plugins.
10. **M9 — everywhere.** Firefox extension (modal editing in every text field,
    link hints, clipper, native-messaging bridge), tela-zen integration,
    Neovim plugin, CLI verbs.
11. **M10 — on the web.** Project website with docs; publish pages and canvases
    as static sites; the WASM web viewer.

The Firefox extension's editing part can start early (after M1): it needs only
`zett-edit` compiled to WASM, not the rest of the app.

## Decided

- Rust everywhere; QML window through cxx-qt (one binary).
- Lua for configuration and quick extensions; WASM for heavy plugins.
- Loro as the CRDT; Markdown files mirror pages for interop.
- `zett-core` and `zett-edit` stay WASM-clean: no file system, threads or
  native-only crates in them; I/O lives in the app and the CLI.
- Name "zett" is a working title until the EU/DPMA registers are checked.

## Ask before (cannot be changed after release)

- The domain for the website (zett.dev and zett.app looked free on 2026-10-16).
- The Firefox extension id (`gecko.id`) and the add-ons store listing name.
- The app id / desktop file name and any package names.
