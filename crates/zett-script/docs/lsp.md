# Language servers

zett starts a language server for a file when one is installed:
rust-analyzer, clangd, typescript-language-server, pyright, lua-language-server,
bash-language-server, gopls, taplo, marksman, qmlls. One per project.

## What it gives

- errors and warnings: a sign in the gutter, the range underlined, counts in
  the modeline, the line's message at the bottom
- completion while typing (Tab / C-n / C-p to choose, Enter or C-y to take)
- `gd` / `SPC c d`  go to the definition
- `K` / `SPC c k`   documentation at the cursor
- `gr` / `SPC c D`  references
- `SPC s i`         symbols in the file
- `SPC c r`         rename (in every file; unopened files open unsaved, `:wa`)
- `SPC c f`         format the buffer
- `]e` `[e`         next / previous error, `SPC c x` all of them

Trouble? `ZETT_LSP_LOG=/tmp/lsp.log zett FILE` keeps what servers print.
