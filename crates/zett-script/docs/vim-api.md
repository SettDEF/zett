# Neovim's Lua API and Vimscript

A Neovim config or plugin runs in zett unchanged where zett has the thing: `vim.*` in Lua is
Neovim's API on zett's own (`zett.*`), and Vimscript is translated to Lua and run. What is there
and what is not yet is counted in docs/parity (`nvim-api.md`, `nvim-ex-commands.md`).

## Lua: vim.*

- `vim.o` `vim.go` `vim.opt` read and set options by long or short name (`vim.o.ts = 4`,
  `vim.opt.tabstop = 4`, `vim.opt.shortmess:append("c")`); `vim.bo` `vim.wo` `vim.opt_local` set
  them for the buffer (`:setlocal`)
- `vim.g` `vim.b` `vim.w` `vim.t` `vim.v` `vim.env`: variables; `vim.g.mapleader` is zett's leader
- `vim.cmd("…")`, `vim.cmd.edit("x")`, `vim.cmd({ cmd = "edit", args = { "x" } })`: Ex commands;
  several lines and Vimscript's own commands go through the translator below
- `vim.keymap.set(mode, lhs, rhs, opts)`: a function, or keys (as `:noremap`); `vim.keymap.del`
- `vim.api.nvim_*`: the buffer's lines and text (`nvim_buf_get_lines`, `nvim_buf_set_lines`,
  `nvim_buf_set_text`, `nvim_get_current_line`), the cursor, options, variables, keymaps,
  `nvim_create_user_command`, `nvim_create_autocmd` and `nvim_create_augroup`, `nvim_set_hl`,
  `nvim_echo`, `nvim_feedkeys`, `nvim_eval`, `nvim_call_function`
- `vim.fn.*`: Vimscript's functions: files (`expand`, `fnamemodify`, `glob`, `readfile`,
  `writefile`, `stdpath`, `executable`), `system`, the buffer (`getline`, `setline`, `append`,
  `line`, `col`, `cursor`, `search`), strings with Vim's patterns (`substitute`, `matchstr`,
  `split`), lists and dictionaries, `json_encode`, `has('nvim')`, `exists()`, timers
- `vim.tbl_*`, `vim.split`, `vim.trim`, `vim.inspect`, `vim.deepcopy`, `vim.json`, `vim.fs`,
  `vim.notify`, `vim.schedule`, `vim.defer_fn`, `vim.uv` (time, timers, `fs_stat`),
  `vim.ui.select` and `vim.ui.input` (zett's prompt)

There is one buffer to a script: the one being edited (handle 0 or its number). An Ex command a
script asks for runs when the script returns; `vim.o` is set at once, so a later read sees it.

## Vimscript

`init.vim` in zett's config folder is read after `init.lua`; `:source x.vim` and `:runtime` run
.vim files; the `:` line takes Vimscript too (`:let g:x = 1`, `:echo &ts . " " . g:x`, a
`:function` typed line by line until `:endfunction`).

- `let` (`+=` `-=` `.=` `..=`, `let [a, b] = list`, `let d.k =`, `let &opt =`, `let $VAR =`),
  `unlet`, `const`
- `if` `elseif` `else` `endif`, `while` `endwhile`, `for x in list` / `for [k, v] in items(d)`
  `endfor`, `break`, `continue`
- `function[!] Name(a, b, ...) [abort] [dict]` … `endfunction` (`a:name`, `a:0`, `a:000`, `l:`),
  `s:` functions and variables per script (`<SID>` and `s:Name()` in its mappings and commands
  reach them), `return`, `call`, `defer`, `delfunction`
- `try` `catch /pattern/` `finally` `endtry`, `throw`, `v:exception`
- `echo` `echon` `echomsg` `echoerr`, `execute`, `finish`; `set` runs at once
- expressions: numbers, `'…'` and `"…"` strings, `[lists]`, `{'dicts': 1}`, `#{literal: 1}`,
  `{a -> a * 2}` lambdas, `x[i]` `x[a:b]` `d.key`, `?:`, `|| &&`, `== != > < >= <= =~ !~ is isnot`
  with `#` and `?`, `+ - * / %`, `.` `..`, `x->method()`, `function('Name')`, `&opt`, `$VAR`,
  `g: b: s: l: a: v:` (and `g:` alone as the dictionary), `map()` and `filter()` with
  `'v:val'` strings or lambdas

Vim patterns become Lua patterns (`\s \d \w \+ \= \( \) \< \>`); what Lua patterns cannot say
(`\|`, `\{n,m}`) is not there. Vim's debugger and profiler are not either: an error names the
line it came from.
