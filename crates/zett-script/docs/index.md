# zett

An editor you live in: Vim's editing, Emacs's programmability, Doom's keys.
`zett` opens the editor in your terminal, `zett --gui` the notes window.

## Topics

- `editor`    the editor in the terminal: buffers, windows, the dashboard
- `keys`      the SPC tree and Vim's keys, how to find any key
- `vim`       Vim mode: search, marks, registers, macros, folds, blocks, spelling, windows
- `vim-keys`  every normal-mode key and what it does (`:help gJ` finds one)
- `emacs`     Emacs keys: moving, killing, M-%, rectangles, registers, macros, C-h
- `lsp`       language servers: errors, completion, definitions, rename
- `config`    init.lua: options, keys, commands, hooks, plugins
- `plugins`   the explorer, the undo tree, :set, writing your own
- `terminal`  zett with zeit and zash: terminals, compiling, web pages
- `cli`       zett on the command line and in scripts
- `query`     live query blocks in notes: tasks, TODOs, backlinks, recent files
- `commands`  every command and its keys (made from your config)
- `faq`       questions: getting zett, debugging, known problems, plans
- `news`      what is new: the latest changes
- `license`   zett's licence (MIT) and its no-warranty clause
- `hello`     hello in many scripts: a test of the font and the terminal
- `vim-api`   Neovim's Lua API (vim.*) and Vimscript: init.vim, .vim files, :let, :function

## Reference

Every command, option, key and function of Neovim and Emacs, one line each, with zett's state:
✓ zett has it, — decided otherwise (the line says why), · not yet.

- `ex-commands`     every Ex command (`:help :sort` goes to its line)
- `option-list`     every option: type, scope, default (`:help 'ts'` finds the setting itself)
- `normal-keys`     every normal-mode key as zett's Vim profile takes it
- `emacs-keys`      every Emacs default key and the command it runs
- `emacs-commands`  every Emacs command, by package
- `api-list`        every `vim.api` and `vim.fn` function

## Moving around

Open one: `:help TOPIC` in the editor, `SPC h d` for the list, or `zett doc TOPIC` in a shell.
In a help page, Enter or `CTRL-]` on a name follows it: a topic, a `` `key` ``, an `'option'`, a
`:command`; `CTRL-T` (or `CTRL-O`) goes back where it came from.
