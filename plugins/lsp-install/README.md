# lsp-install

Helps install the language servers a filetype needs, when they are missing.

- `:LspInstall [filetype]` (default: this buffer's) asks `Install` / `Cancel` and shows the exact command first; nothing runs without Install
- `:LspInstallStatus` lists each known server as `✓` (installed) or `·` (missing, with its install command)
- the first time a filetype with a missing server opens, one INFO line says so (once per filetype per session)
- `vim.g.lsp_install_no_hint = true` turns that hint off
- commands starting with `sudo` are not run (no password prompt here): the command is copied to the `+` register and shown to run in a terminal

Servers: lua, qml, python, sh/bash/zsh, go, rust, c/cpp, javascript/typescript, json. Edit the `servers` table in `lua/lsp-install/init.lua` to change them.
