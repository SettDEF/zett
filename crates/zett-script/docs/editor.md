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
- `SPC e`     the explorer (`SPC o P` shows this file in it); see `plugins`
- `SPC s u`   the undo tree

## Editing

Vim's keys, checked against Neovim case by case: motions, operators, text
objects, visual mode, counts, registers, undo. `gcc` / `gc` comment lines.

## The command line

`:w` `:q` `:wq` `:wa` `:e FILE` `:bn` `:bp` `:ls` `:sp` `:vs` `:only` `:42`
`:lua CODE` `:compile CMD` `:rename NAME` `:help TOPIC` `:set NAME VALUE`, and every command
by name. Typing after `:` lists what fits, with an example; Tab completes.

## Search everything

`C-c s` (`SPC s e`) searches the project's files as they are and what they once had: every file's
saved history (kept with each save) is searched for text a change deleted or replaced. Those hits
start with ⟲ and say when (`⟲ notes/plan.md@14 · 3 h ago: the old line`); Enter on one opens the
file in time travel at the state that still has it, to read it or take it back (`Enter` all, `b` the
section, `v`…`y` a copy).
