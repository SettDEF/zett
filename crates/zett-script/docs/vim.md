# Vim mode

zett's Vim profile has every key of Neovim's normal-mode index (`:help vim-keys` lists them, with
what each does). The keys work as Neovim's do; where zett differs it says so below. Options that
change them are in `:help settings`.

## Moving

- `h j k l`, arrows; `w b e ge` words, `W B E gE` blank-separated words; `0 ^ $ g_` in the line
- `f t F T` a character, `;` `,` again; `%` the matching bracket (`matchpairs`), `N%` that far in
- `gg G` first / last line (`startofline` off: the column stays, as Neovim); `H M L` the window
- `( )` sentences, `{ }` paragraphs, `[[ ]] [] ][` sections, `[( [{ ]) ]}` unmatched brackets,
  `[m ]m [M ]M` methods, `[* ]/` comments, `[# ]#` #if blocks
- `gj gk g0 g^ g$ gm` by screen rows when lines wrap; `go` to a byte; `|` to a column
- `+ - _ <CR>` to the first non-blank of the line below / above / this one

## Search

- `/text<CR>` forward, `?text<CR>` backward; `n` `N` again either way, `wrapscan` goes round
- `*` `#` the word under the cursor (whole word), `g*` `g#` anywhere; `gn` `gN` select the match
- `ignorecase` and `smartcase` (both on: Emacs's way) decide the case; `hlsearch` keeps them shown
- `[i [I` the word through the file, `[ CTRL-I` to its first use, `gd gD` to its declaration

## Marks and jumps

- `ma` … `mz` set a mark, `'a` its line, `` `a `` its place; marks move with the text
- `''` and ` `` ` back to before the last jump; `'[ ']` the last change, `'< '>` the last selection
- `CTRL-O` `CTRL-I` (`Tab`) back and forth through the jump list; `g;` `g,` the change list
- `g'a` `` g`a `` to a mark without adding to the jump list; `['` `]'` the next / previous mark

## Registers

- `"a` … `"z` before a yank, delete or put names the register; `"A` (upper case) appends
- `"0` the last yank, `"1` … `"9` the last line deletes, `"-` small deletes, `"_` nothing
- `"+` `"*` the system clipboard; `:registers` lists them

## Changing

- operators: `d c y` (`dd cc yy` the line), `> < =` indent, `g~ gu gU g?` case and rot13,
  `gq gw` format to `textwidth`, `zf` fold, `zy` yank without trailing blanks, `!` filter,
  `g@` the `operatorfunc`; an operator twice means the line (`>>` `guu` `gUU` `g??`)
- `x X s S r R` replace, `~` case (an operator with `tildeop`), `J gJ` join (`joinspaces`,
  `formatoptions` j drops comment leaders), `p P gp gP ]p [p` put (`]p` matching the indent)
- `CTRL-A` `CTRL-X` add to the number under the cursor (`nrformats`: decimal, hex, binary,
  octal, letters); `.` repeats the last change; `u` `CTRL-R` `U` undo, redo, the line
- `i a I A o O gi gI` start typing; `autoindent`, `smartindent`, `copyindent`, `softtabstop`,
  `backspace` and `textwidth` (wrapping while typing) shape it

## Text objects

After an operator or in visual mode: `iw aw iW aW` words, `is as` sentences, `ip ap`
paragraphs, `i( a( ib` `i[ a[` `i{ a{ iB` `i< a<` brackets, `i" a" i' a' i` a` `` quotes
(`quoteescape`), `it at` tags.

## Macros and the dot

- `qa` … `q` records into a register, `@a` plays it, `@@` the last one again, a count repeats
- `Q` replays the register recorded last (Neovim's Q); `@:` the last `:` line
- `.` repeats the last change, with a count in front instead of its own

## Visual mode

- `v` characters, `V` lines, `CTRL-V` a block; `o` the other end, `O` the other corner (block)
- `gv` the last selection again; `selection=exclusive` leaves the cursor's character out
- in a block: `I` and `A` type on every line (`$A` at each line's own end), `c d y r ~ u U > <`,
  `p` puts a block register one line under another

## Folds

- `zf{motion}` `zF` make a fold, `zd zD zE` remove them; folds move with the text
- `zo zO zc zC za zA` open, close, toggle; `zv` show the cursor's line, `zx zX` close them again
- `zm zM zr zR` close or open by level, `zn zN zi` folding off, on, switched
- `zj zk` to the next / previous fold, `[z ]z` the start / end of the one the cursor is in
- where no manual fold is, `zc zo za` fold zett's outline section (a heading, a block)

## Spelling

- `]s [s` the next / previous misspelled word, `z=` suggestions (a count picks one at once)
- `zg zw` mark a word good / wrong (in `~/.local/share/zett/spell/LANG.add`, Vim's format),
  `zG zW` for this session, `zug zuw zuG zuW` take them back
- the dictionary: hunspell's (`sudo pacman -S hunspell-en_us`), or a word list in
  `~/.local/share/zett/spell/`

## Digraphs

- `CTRL-K {char1} {char2}` in insert mode and on the `:` line: one character for two, Vim's table
  (`a:` ä, `e'` é, `Eu` €, `->` →; either way round); `:digraphs` lists them, `:digraphs xy 9731`
  adds one; with `digraph` on, `{char1} <BS> {char2}` makes one too

## Mappings, folders, included files

- the mouse by its Vim names, keys held too: `:nnoremap <C-LeftMouse> <LeftMouse>gd`, `<M-LeftDrag>`,
  `<S-ScrollWheelUp>`, `<2-LeftMouse>` (a double click); `<LeftMouse>` in the keys clicks where it was
- `:cmap :cnoremap` and `:cabbrev` work on the `:` line, `:omap :onoremap` after an operator (`dL`
  with `:onoremap L $`); `:map` covers Normal, Visual and operator-pending, `:map!` and `:ab` insert
  mode and the `:` line
- `:cd` (`-` the folder before, `cdpath` for relative ones), `:tcd` for this buffer, `:lcd` for this
  window; `:pwd` the one that applies (`:verbose pwd` says which)
- `[i [I [ CTRL-I` the word under the cursor through the file and the files it includes (`]` from the
  line below); `:ilist :isearch :ijump :isplit`, `:dlist :dsearch :djump :dsplit` for definitions,
  `:checkpath` the included files not found (`include`, `define`, `path`, `suffixesadd`)

## Diff mode

- `:diffsplit FILE` compares this buffer with FILE beside it (`diffopt` `vertical` side by side),
  `:diffthis` in two windows does the same, `:diffoff` (`!` all) ends it; `:diffpatch PATCH` the
  file as the patch makes it
- changed lines `DiffChange` (the part that differs `DiffText`), added ones `DiffAdd`, and where the
  other side has more, filler lines (`DiffDelete`, `fillchars` `diff:`); the windows scroll together
- `]c [c` to the next / previous change, `do` (`:diffget`) takes the other side's version, `dp`
  (`:diffput`) gives it this one; `diffopt` `icase iwhite iwhiteall filler vertical`

## Sessions, views, runtime files

- `:mksession` (`Session.vim`; `!` over an old one) writes the folder, the buffers and the windows as
  split; `:source Session.vim` brings them back. `:mkview` `:loadview` keep a file's cursor and local
  options (`viewdir`, or zett's state folder)
- `:source` runs Lua, and a `.vim` file's lines as `:` commands (`"` comments, `\` continuation lines)
- `:runtime FILE` `:colorscheme NAME` (`colors/NAME.lua` or `.vim`) `:compiler NAME` (`compiler/`,
  then Neovim's own compiler files: makeprg and errorformat) `:packadd NAME` (`pack/*/opt/NAME`)
  `:packloadall` `:scriptnames`; the folders are `runtimepath`, zett's config folder by default

## Windows and scrolling

- `CTRL-W` and a key: `s v n` split / new, `w W p t b` and `h j k l` go to one, `H J K L` move
  it, `c q o` close / only, `x r R` exchange and rotate, `= + - < > _ |` sizes, `f gf ] ^` split
  and open; counts work (`3 CTRL-W +`)
- `CTRL-D CTRL-U` half a page, `CTRL-F CTRL-B` a page, `CTRL-E CTRL-Y` a line
- `zt zz zb` (and `z<CR> z. z-`) put the cursor's line at the top, middle, bottom; `zh zl zs ze`
  sideways; `z{N}<CR>` the window N lines high
- `gt gT` the next / previous buffer, `CTRL-^` the other one, `CTRL-]` `CTRL-T` to a definition
  and back (the language server), `K` help for the word

## The rest

- `ZZ` write and quit, `ZQ` quit; `q:` `q/` the command-line window (Enter runs a line)
- `CTRL-G` `g CTRL-G` where you are; `ga g8` the character's codes; `CTRL-L` redraw; `CTRL-Z`
  back to the shell (`fg` returns)
- `gx gf` open the address / file under the cursor; `gs` sleep; `&` `g&` repeat `:s`
- the mouse (`mouse`: the modes it works in): a click places the cursor, a drag selects, a double
  click the word, a triple click the line (`mousetime`); the right button moves the cursor, or with
  `mousemodel=extend` grows the selection to it; the middle button puts the selection there; the
  wheel scrolls `mousescroll` lines
