# The editor

`zett` in a terminal opens the editor; `zett FILE…` opens files, `zett DIR` the
file finder in that folder, plain `zett` the dashboard.

## Buffers and files

- `SPC SPC`   find a file in the project (fuzzy)
- `SPC .`     find a file from this file's folder
- `SPC ,`     switch buffer          `SPC b k`  kill buffer
- `SPC f r`   recent files           `SPC p p`  switch project
- `SPC f s`   save (`:w`)            `SPC f S`  save as
- `]b` `[b`   next / previous buffer `SPC ``   the last buffer

## Windows

- `SPC w v` / `SPC w s`   split right / below (`C-w v`, `C-w s`)
- `SPC w h j k l`         move between windows (`C-w h` …)
- `SPC w c`               close a window, `SPC w o` only this one

## Finding things

- `SPC /`     search the project (ripgrep), `SPC s s` this buffer
- `SPC s i`   symbols in this file (with a language server)
- `SPC o p`   the project tree (`SPC o P` shows this file in it)

## Editing

Vim's keys, checked against Neovim case by case: motions, operators, text
objects, visual mode, counts, registers, undo. `gcc` / `gc` comment lines.

## The command line

`:w` `:q` `:wq` `:wa` `:e FILE` `:bn` `:bp` `:ls` `:sp` `:vs` `:only` `:42`
`:lua CODE` `:compile CMD` `:rename NAME` `:help TOPIC`, and every command
by name (`SPC :` lists them).
