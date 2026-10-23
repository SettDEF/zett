# zett: questions

## Getting zett

zett is one program. From its source folder, `./install.sh` builds it and puts `zett` in
`~/.local/bin`. It needs Rust (cargo), and Qt 6 for `zett --gui`.

## Vim keys instead of Emacs keys?

In `~/.config/zett/init.lua`: `zett.opt.keys = "vim"`. Vim's modes and Doom's SPC tree come
instead of the Emacs keys. `:help vim` and `:help keys` tell the rest.

## Where are my settings, and how do they come back after a mistake?

`~/.config/zett/init.lua`, then `~/.config/zett/plugins/NAME/init.lua`. `SPC h r` (or
`M-x reload-config`) reads them again. A file with an error is reported and the rest of that file
is skipped; zett's defaults and the other plugins still load, so zett always starts.

## Debugging

- `~/.local/state/zett/zett.log`: what zett noted for itself (for one, why a help page's
  places pulse instead of being drawn: the terminal is not zash).
- `:messages` (`C-h e`): the messages shown so far.
- `C-h l`: the last keys typed. `C-h k` then a key: the command it runs.

## Known problems

- In a terminal other than zash, help pages pulse their places instead of drawing them, and
  pictures do not show.
- `zett --gui` is younger than the editor in the terminal.

## What is planned

zett's plan (PLAN.md in its source) goes on from the editor: databases (table, board, calendar
views), a canvas, sync between devices without a server, live data from other programs, and
agents working in a page beside you.
