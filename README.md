# zett

An editor you live in: Vim's editing, Emacs's programmability, Notion's pages
and a canvas board — all views of the same plain Markdown files.

Rust core, Lua scripting, two front ends: the code editor in the terminal and
the notes window (QML, cxx-qt). See [PLAN.md](PLAN.md) for the architecture and
milestones, and [TASKS.md](TASKS.md) for what is next.

```sh
zett                 # the editor in the terminal: the dashboard
zett src/main.rs     # files; a folder opens the file finder
zett --gui notes/    # the notes window
./install.sh         # to ~/.local/bin
```

## The editor (in a terminal), Doom-style

- **Vim's editing**, checked against Neovim case by case (`scripts-nvim-compare.sh`).
- **SPC** with which-key, Doom's tree and keys: `SPC SPC` find file, `SPC ,`
  buffers, `SPC .` files here, `SPC /` search the project (ripgrep), `SPC :` M-x,
  `SPC f r` recent files, `SPC p p` projects, `SPC w v/s/h/j/k/l` windows,
  `SPC e` explorer, `SPC s u` undo tree, `SPC o t` a terminal below, `SPC g g` lazygit,
  `SPC c c` compile, `SPC t l` line numbers, `SPC h k` every key, `gcc`/`gc` comments.
- **Language servers** (rust-analyzer, clangd, typescript, … when installed):
  errors in the gutter and the modeline, the line's message at the bottom,
  completion while typing, `gd` definition, `K` documentation, `]e` `[e`, `SPC c x`.
- **Undo as a tree** (`u` `C-r` `g-` `g+`, nothing lost when you undo and change).
- **Built-in plugins in Lua**: the explorer (git marks, add/rename/trash/copy,
  filter) and the undo tree (branches, diffs), set with `:set` or `zett.opt.NAME`.
- **`:` shows what fits as you type**, with an example each; `:help` for the docs.
- **The dashboard, the doom-modeline**; colours from the
  terminal's 16, so in zash it follows the wallpaper.
- **With zeit and zash**: the terminal opens as a zeit pane, a compile is a zeit
  job (its line in the bar, a note when done).

Everything is Lua: `~/.config/zett/init.lua` changes keys, commands and options
(`zett.frontend` says which front end loads it); plugins in `plugins/`.
