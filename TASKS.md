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

### Emacs parity (2026-10-21): what Emacs has and zett does not yet, in the order they are felt
A. editing (zett-edit textops.rs, pure, tested)
- [x] fill-paragraph M-q (zett.opt.fill_column; list and comment prefixes kept), sort-lines, delete-duplicate-lines,
  join-line M-^, transpose-words M-t / transpose-lines C-x C-t, delete-blank-lines C-x C-o,
  upcase/downcase-region C-x C-u / C-x C-l, mark-word M-@, dabbrev-expand M-/ (cycling), count-words M-=
B. interactive (zett-tui)
- [ ] keyboard macros: C-x ( · C-x ) · C-x e (e again repeats), F3 / F4
- [ ] query-replace M-% (y n ! q .), replace-string, isearch-forward-regexp C-M-s
- [ ] goto-line M-g g, zap-to-char M-z, shell-command M-!, shell-command-on-region M-| (C-u: replace)
- [ ] registers: point C-x r SPC / jump C-x r j, text C-x r s / insert C-x r i; bookmarks C-x r m / b
- [ ] rectangles: C-x r k / y / t (string-rectangle), C-x SPC
- [ ] occur M-s o (a buffer of matching lines, editable: changes go back, as wgrep)
C. files and sessions
- [ ] save-place (each file reopens where it was), auto-save + crash recovery, desktop (buffers and
  windows back after a restart), auto-revert (a file changed elsewhere reloads when untouched)
- [ ] dired: a folder as an editable buffer (rename by editing the names, mark, delete, copy)
- [ ] ibuffer (buffers as a buffer), TRAMP-like remote files over ssh
D. help and discovery
- [x] `:help X` as Neovim finds a tag (2026-10-21): a topic, a setting (`:help wrap`, `'ic'`) on a generated
  settings page, a command, a heading in any topic; the place found near the top, pulsing a moment;
  nothing found names the nearest ("maybe: smartcase")
- [ ] C-h k / f / v / m as buffers (key → command → its source), apropos, a preview while picking
  (consult), actions on any candidate (embark)
E. programming
- [ ] eldoc (signature in the echo area), tree-sitter, a debugger (DAP), send region to a REPL, ediff
F. org
- [ ] TODO states cycling, priorities, tags, SCHEDULED / DEADLINE, the agenda (with Dotos), tables
  (aligned, formulas), clocking, export (HTML / PDF)
G. the rest
- [ ] a terminal buffer (zash inside zett), spell checking, calc, calendar; zett server/client (one zett
  for every edit: zett-edit)
- skip: mail, RSS, IRC (other apps do it better)

### Parity with Neovim and Emacs: the plan to the end (2026-10-21)
Measured by scripts/parity.sh against the installed Neovim and Emacs (docs/parity/). Done for an
item means all three: it works in zett (a test, or a check in the real window), the parity page
counts it, and `:help` finds it. Nothing counts as done by being accepted and ignored.

Tonight (decided 2026-10-21): P1 documentation, then every option group (P2), then Ex commands (P3) as far as
the night goes, everyday groups first (X1 X3 X2). vim.api (P4) and Emacs (P5) are later nights.

Where it stands: normal-mode keys 477/477 ✓ · options 42/374 (the rest kept, ◐) · Ex commands
83/557 · vim.api + vim.fn 0/840 · Emacs keys 51/500 · Emacs commands 68/3164.

**P1 — documentation of what exists** (first: nothing undocumented stays behind)
- [ ] `:help vim`: search, marks, registers, macros and the dot, operators, text objects, folds,
  block selection, spelling, windows and scrolling, the jump list, counts and registers in front
- [ ] every normal-mode key findable: `:help gJ`, `:help CTRL-W_x`, `:help [s` open its entry
  (a generated key page from Neovim's index, with zett's own words where they differ)
- [ ] `:help options` per group, as the batches below land

**P2 — options, by what they act on** (each batch: behaviour + test + the option leaves ◐)
- [ ] O2 display: cursorcolumn cursorlineopt listchars showbreak breakindent(opt) breakat
  sidescroll sidescrolloff signcolumn numberwidth foldcolumn showmatch matchtime hlsearch
  laststatus showmode showcmd(loc) ruler rulerformat statusline cmdheight fillchars conceallevel
  concealcursor title titlestring titlelen display scroll scrolljump smoothscroll synmaxcol
  termguicolors background winbar showtabline tabline colorcolumn-done
- [ ] O3 windows: splitbelow splitright equalalways eadirection winheight winwidth winminheight
  winminwidth winfixheight winfixwidth helpheight previewheight cmdwinheight splitkeep winfixbuf
- [ ] O4 files: autoread autowrite autowriteall backup backupcopy backupdir backupext backupskip
  writebackup swapfile directory undodir undolevels undoreload fileformat(s) fileencoding(s)
  encoding bomb endofline endoffile fixendofline binary readonly modified hidden confirm modeline
  modelines modelineexpr fsync updatecount updatetime write writeany path suffixesadd isfname
  autochdir cdpath cdhome fileignorecase exrc shada shadafile viewdir viewoptions sessionoptions
- [ ] O5 programs: makeprg errorformat makeef makeencoding grepprg grepformat shell shellcmdflag
  shellpipe shellquote shellredir shellxquote shellxescape shelltemp equalprg formatprg
  keywordprg formatexpr indentexpr includeexpr foldexpr (Lua expressions)
- [ ] O6 completion and the : line: complete completeopt completefunc omnifunc pumheight pumwidth
  pummaxwidth pumblend pumborder wildmenu wildmode wildoptions wildchar wildcharm wildignore
  wildignorecase history cedit inccommand infercase dictionary thesaurus(func) autocomplete*
- [ ] O7 spelling and folds: spell spelllang spellfile spellcapcheck spelloptions spellsuggest
  foldenable foldlevel foldlevelstart foldmethod (manual indent marker expr syntax) foldmarker
  foldminlines foldnestmax foldopen foldclose foldtext foldignore
- [ ] O8 keys and input: timeout timeoutlen ttimeout ttimeoutlen mouse mousemodel mousescroll
  mousetime mousefocus mousemoveevent keymodel selectmode langmap keymap remap maxmapdepth
  operatorfunc paste digraph allowrevins revins
- [ ] O9 the rest of Neovim's: cpoptions (the flags zett can honour), shortmess, belloff
  errorbells visualbell report-done more, verbose verbosefile, debug, eventignore(win), jumpoptions,
  switchbuf, tags tagcase tagfunc tagrelative tagstack taglength tagbsearch showfulltag, diff
  diffopt diffexpr patchexpr patchmode, comments commentstring formatlistpat, lisp lispwords
  lispoptions cinkeys cinoptions cinwords cinscopedecls indentkeys, define include isident
  isprint, highlight winhighlight winblend winborder, busy, channel, scrollback, ...
- [ ] O10 legacy and GUI options (Arabic, Hebrew, GUI fonts and tab tooltips, input methods): decided
  2026-10-21: as Neovim's terminal UI. What it ignores in a terminal (guifont, guitablabel, linespace…)
  zett ignores too and says so in :help; real features (rightleft, keymap, langmap, input methods) are built

**P3 — Ex commands** (the same parse: ranges, counts, !, registers; each a test)
- [ ] X1 buffers, args, windows, tabs: :ls :b :bn :bp :bd :bw :ball :sball :args :argadd
  :argdelete :next :prev :first :last :argdo :bufdo :windo :tabdo :split :vsplit :new :only
  :close :hide :resize :vertical :leftabove :aboveleft :rightbelow :belowright :topleft :botright
  :tab :tabnew :tabclose :tabonly :tabmove
- [ ] X2 the quickfix and location lists: :make :grep :vimgrep :cc :cn :cp :cfirst :clast :copen
  :cclose :cwindow :colder :cnewer :cdo :cfdo :cexpr :cgetexpr :caddexpr and the :l… twins
- [ ] X3 editing: :d :y :pu :m :t :co :j :> :< :normal :g :v :s variants :& :&& :~ :retab
  :sort(done?) :center :left :right :print :number :# :z :undo :redo :earlier :later :undolist
  :changes :jumps :marks :delmarks :registers :display :mark :k
- [ ] X4 keys and commands: :map family (n v x s o i c t l), :noremap, :unmap, :mapclear,
  :abbreviate family, :command :delcommand :comclear, :autocmd :augroup :doautocmd
  (onto zett.keymap, zett.command, zett.on)
- [ ] X5 files and sessions: :w variants :sav :up :x :wa :wqa :qa :e! :ene :r :checktime
  :mksession :mkview :loadview :oldfiles :browse :cd :lcd :tcd :pwd :find :sfind
- [ ] X6 script: :let :unlet :if :else :endif :for :while :function :call :return :execute
  :echo :echom :echoerr :eval :try :catch :finally :throw :source :runtime :lua :luado
  :luafile (a small Vimscript evaluator for config files and plugins' simple use)
- [ ] X7 diff, tags, spelling, folds, the rest: :diffthis :diffoff :diffget :diffput
  :diffupdate :diffsplit :tag :tags :tselect :tjump :pop :spellgood :spellwrong :spellundo
  :spellrepall :fold :foldopen :foldclose :folddoopen :folddoclosed :highlight (done) :syntax
  :colorscheme :filetype :setfiletype :options :checkhealth :messages :history :redraw :sleep
- [ ] X8 providers and menus: decided 2026-10-21: :python3 :perl :ruby (and their do / file forms) run
  the real interpreters when installed (the code alone, no editor API inside; E319 when one is missing);
  menus (:menu family, :popup) as zett pickers

**P4 — vim.api and vim.fn** (Neovim plugins run unchanged): a `vim` table in zett-script over
zett's own API: buffers, windows, tabs, options (vim.o vim.bo vim.wo vim.opt), keymaps, autocmds,
user commands, extmarks and namespaces, highlights, vim.fn's everyday functions, vim.loop/uv
basics, vim.schedule, vim.notify. Measured per function; a real plugin (e.g. mini.nvim's
pieces, which-key, gitsigns) as the acceptance test.

**P5 — Emacs**: the default keys by command name, then the commands by package, the everyday
packages first (simple, files, window, buffer, isearch, replace, register, rect, kmacro,
dired, comint), the rest after.
### Settings, as Neovim's options and Emacs's variables (2026-10-21)
One table of core settings (name, Vim alias, type, default, what it does), `:set` / `:setlocal` /
`zett.opt`, values per file type (`zett.ft.markdown = { wrap = true }`), shown in `:set` and the
settings page.
- [x] the table (zett-tui settings.rs): :set, :setlocal, :set name? / noname, :set alone lists them,
  zett.opt, zett.ft.<type> (Markdown and plain text wrap); wrap, linebreak, cursorline, colorcolumn,
  list, scrolloff, tabstop, shiftwidth, expandtab, trim_trailing_whitespace, final_newline, fill_column
- [x] where `:` asks (2026-10-21): `cmdline` = bottom (Vim) / top / center (a box, like noice.nvim) /
  cursor (a box under the cursor), `cmdwidth`; suggestions open below a floating line. Next: a plugin
  replacing the command line altogether (a hook that draws it), and the same for `/` search
- [ ] display: listchars, signcolumn,
  signcolumn, number / relativenumber (have), conceal for Markdown
- [ ] indenting: tabstop, shiftwidth, expandtab, autoindent, smartindent; editorconfig
- [x] search: ignorecase, smartcase (2026-10-21; both on by default, Emacs's way; incsearch is how it works)
- [ ] search: hlsearch, wrapscan
- [ ] files: autosave (seconds), backup, autoread (reload a file changed elsewhere when untouched),
  trim trailing whitespace and final newline on save, fileformat / fileencoding
- [ ] per file type: settings, keys and hooks per language (ftplugin, Emacs mode hooks)

### Plugins: the words, and how far one can go (2026-10-21)
- core = Emacs × Neovim (what Neovim has built in stays built in); a **plugin** is anything installed,
  Lua (later WASM); **bundled plugins** ship with zett and can be switched off (git buffer, query
  blocks, notebook, agent, outline extras). No "extensions" or "add-ons": one word.
- have: commands, keys and groups, settings, events (zett.on), highlight groups, zett.buf (the current
  buffer, its undo states), zett.fs (read, write, stat, remove, exec to the end), zett.defer, zett.input,
  zett.open, zett.panel (floating panels), zett.notify, pages
- [ ] zett.job: processes that stream (stdout/stderr lines, stdin, kill, exit callback), timers that repeat
- [ ] buffers by id: create, hidden / scratch ones, per-buffer settings and variables, extmarks
  (marks that follow edits) and virtual text / lines (what query blocks and the agent draw)
- [ ] UI from Lua: the minibuffer / pickers (items, preview, actions), windows (split, focus, size),
  mode-line segments, the dashboard, menus, the echo area
- [ ] language: LSP requests from Lua, diagnostics, completion sources, tree-sitter queries
- [ ] network and data: HTTP, JSON, sockets, file watching, a key-value store per plugin
- [ ] modes: major (one per file type) and minor modes with their own keymaps
- [ ] safety and packaging: plugin.toml (version, what it may do: network, files, programs), a
  lockfile, `zett sync`, `zett doctor`; a plugin cannot freeze the editor (time limits, async calls)
- [ ] the bundled plugins (git, query, notebook, agent) rewritten on this API: proof that it is enough

### zett everywhere (2026-10-21, design)
- [x] zett-edit (~/.scripts/apps/zett): the clipboard (Super+Shift+E, the clipboard toast's zett chip),
  a file or stdin, each save written back; every edit kept in ~/.local/share/zett/edits (a git folder:
  C-c s finds them and their history); quit without saving changes nothing; secrets kept nowhere
- [ ] the shell's text fields (Ctrl+E), tela-zen's text boxes (the float over the box, live write-back),
  per-site modes and snippets, the agent with the page as context, drafts across devices (after sync)

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

### Animations and pictures in help and highlights (planned 2026-10-21, not built)
- [ ] zett renders Lottie itself (velato + vello_cpu, as zash's anim.rs) and shows it by what the
  terminal answers at startup: zash's OSC 7733, kitty graphics, sixel, iTerm2 images. Nothing is
  sent that the terminal did not confirm: an unknown terminal simply gets no animation, the text
  stays as it is, and the log says once why ("no graphics: TERM=…, no kitty/sixel reply").
  Never a half-drawn escape sequence, never garbage on screen. `zett --gui` plays them with Qt.
- [ ] uses: a short clip per key on `:help KEY`, pictures for windows/folds; glow on yank, a pulse
  where a jump lands, matching brackets fading in (each off with an option)

## Make zett the daily editor (picked 2026-10-23, in this order)

Rule from the user: zett builds in only what Neovim builds in; the rest stays plugins (z.plug).

1. **Tree-sitter, as Neovim's** (`vim.treesitter`): biggest gap.
   - Deps already in zett-tui/Cargo.toml: tree-sitter 0.25, libloading 0.8, streaming-iterator 0.1.
   - Parsers: `parser/<lang>.so` on the runtimepath, plus ~/.local/share/nvim/lazy/nvim-treesitter/parser
     (44 installed: rust lua qmljs bash python …) and /usr/lib/tree_sitter. Symbol `tree_sitter_<lang>`;
     load with `LanguageFn::from_raw(sym)` → `Language::from`.
   - Queries: `queries/<lang>/highlights.scm` on the runtimepath + nvim-treesitter/queries; follow
     `; inherits: a,b` (prepend parents), `; extends` from after/queries. On QueryError drop the
     offending pattern and retry.
   - Predicates: the crate does eq/match/any-of (+not-); zett must do `#lua-match?` (Lua pattern →
     regex), `#has-ancestor?`, `#has-parent?` (+not-), `#set! priority`; ignore `#offset!` etc. at first.
   - Highlight: capture → `@cap.lang`, fall back `@cap` → Neovim's default links (@keyword→Keyword,
     @function→Function, @string→String, @comment→Comment, @type→Type, @constant→Constant, …) to add
     in zett-script hl.rs (it has no @ groups yet). Hook: `Doc::spans()` in zett-tui lib.rs (syntect
     today); reparse when `buf.rev()` changes, cache captures per line.
   - Filetype → lang map as vim.treesitter.language.register: qml→qmljs, sh→bash, help→vimdoc, …
   - `vim.treesitter.start()/stop()`, then the user's init.lua starts it on FileType (as nvim-treesitter).
   - Then: folds (foldexpr), `vim.treesitter.get_node`, textobjects-ready API.
2. **LSP completion + `vim.snippet`** (both built into Neovim 0.11); nvim-cmp/LuaSnip stay plugins.
3. **Daily-use loop**: the user works in zett; every crash / slow spot / "not like nvim" goes into a
   log, fixed with a regression test each.
4. **Plugins through z.plug**, fixing the API gaps they hit: conform.nvim (format on save), nvim-lint,
   gitsigns (setup works; check the gutter marks), flash, which-key, persisted (or :mksession).

## GUI: everything VS Code has, in our style (planned 2026-10-28)

Rules: our rounded look (no sharp edges, no stripes), theme roles only, headless shots + scripts/gui-check.sh for each.

1. [x] Activity bar without the stripe: the active item is a soft filled pill
2. [~] Sidebar editor first: Explorer on top (folder name + branch as the header, Go to file, collapse all), the notes things hidden in the Explorer — done; left: new file / new folder buttons, the notes things in a folded section instead of hidden
3. [x] Quick open files (Ctrl P finds files too, fuzzy), recent files
4. [x] Editor: bracket pair highlight, trim trailing whitespace + final newline on save, auto save, word wrap (Alt Z), minimap setting; indent guides skipped (stripes); render whitespace left out (the coloured copy is HTML, dots would break its tags)
5. [x] Sticky scroll (the enclosing lines at the top); folding by indent: still open
6. [x] LSP: definition (F12), references (Shift F12), rename (F2), hover, signature help, code actions (Ctrl .), format (Shift Alt F)
7. [x] Outline: the file's declarations in the details card; Go to symbol (Ctrl Shift O)
8. [x] Split editor (Ctrl \ in a file): two files side by side, both editable
9. [x] Source control (Ctrl Shift G): stage/unstage, commit, diff view, discard with a confirm
10. [x] Panel under the editor: Terminal and Problems (every file, tasks' errors) tabs
11. [x] Multi-cursor: Alt click, Ctrl Alt Up/Down, column select (Shift Alt drag); line editing: Ctrl / comment, Alt Up/Down move, Shift Alt Up/Down copy, Ctrl Shift K delete
12. [~] Keyboard shortcuts page: the searchable list is in Settings; rebinding from it still open
13. [x] Zen mode (Ctrl K Z), minimap setting, breadcrumbs with the symbol the cursor is in
14. [x] Tasks from Cargo/npm/make in the palette, their errors in Problems
15. [ ] Debug: later (DAP) — planned only

## zett: the best editor out there (planned 2026-10-28)

1. [x] Editor core (stage a done 2026-10-28; stage b part 1: line keys, find/replace, git and problem marks, the server (sync, F12, references, symbols, format, edits); part 2: completion, signature, hover, rename, code actions, saving through the core; stage c: Ctrl+Alt cursors, bracket pairs, sticky scroll, a 100,000 line file opens in ~40 ms and types at ~8 ms a key (checked); stage d: word/line/column select, Tab indent, trim on save — the default since 2026-10-28 (Settings > Editor > Classic goes back); left: minimap, word wrap, snippets in the new core: CodeDoc + CodeView, cursors, undo, folds, colours, save, behind the setting): drawn by zett itself, not a Qt TextEdit. CodeDoc (Rust: rope, cursors, undo, folds, syntect
   colours per line with cached parse states) + CodeView.qml (only the lines on screen, cursors from the mono
   grid). Stages: a) view + edit + save behind Settings > Editor > New editor core; b) everything FileView has
   (LSP, find, minimap, git gutter, completion, line keys); c) folding, wrap, huge files; d) default, old one gone
2. [x] (2026-10-28: Ctrl+I inline edits as a diff, quick fix/tests/document/simplify, explain to the panel; agent changes as a PR still open) Claude built in: inline edits as a diff to accept/reject, explain/fix/test on a selection, agent changes
   reviewed like a PR, the claude-ui sessions in one place
3. [x] (2026-10-28: [[code:…]] links, Ctrl+Alt+N notes beside the code, a file's notes in its outline; live results already in notebook blocks) Notes and code together: link a function to a note, live results in pages, a file's notes in its outline
4. [x] (2026-10-28: the drawer's Terminal tab: a shell in a pty on zash's engine, alacritty_terminal, its colours from tinct; drawn by zett as rich-text rows; keys, paste (Ctrl+Shift+V), wheel scrollback, resize; still to do: mouse selection, zash's graphics/effects) A real zash in the panel (zash as a library zett draws)
5. [x] (2026-10-29: gdb -i=dap; breakpoints by the line numbers or F9, F5/F10/F11/⇧F11/⇧F5, call stack, variables opened by a click, output; other adapters (debugpy, lldb-dap) still to wire) Debugging (DAP): breakpoints in the gutter, variables, call stack, debug console
6. [ ] (2026-10-29: blame of the cursor line, Keep current/incoming/both on conflicts, branch list + new branch, Stage/Discard per change; left: side-by-side diff, single lines inside a change) Git: blame per line, side-by-side diff, merge conflicts, branches, stage single lines
7. [x] (2026-10-29: gui-check corebig fails over budget; measured 38-54 ms open of 100,000 lines, 1-3 ms a key) Speed budgets enforced: open < 100 ms, typing latency < 8 ms, checked by gui-check
8. [ ] Extensions: Lua + WASM plugins, a small store

## Terminal zett: floats
- [ ] Glass floats (fidget, hover, completion): a blurred, see-through background under a float. A terminal cannot blur
      part of its cell grid, so zash draws it: zett tells zash the float's rectangle (an escape sequence zash knows),
      zash blurs what is under it; plain terminals keep the solid background
