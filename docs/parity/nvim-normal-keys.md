# Neovim's Normal-mode keys in zett

Generated 2026-10-21 by scripts/parity.sh from NVIM v0.12.6, GNU Emacs 31.1 and zett 0.1.0.

Each key is written out (`d{motion}` as `dw`) and fed into zett's Vim profile with text, a yank, a change and
a search to act on: ✓ something happened (or zett's key map binds it), · nothing yet.

| | key | fed as | what it does |
|---|---|---|---|
| ✓ | `CTRL-A` | `<C-a>` | add N to number at/after cursor |
| ✓ | `CTRL-B` | `<C-b>` | scroll N screens Backwards |
| · | `CTRL-C` | `<C-c>` |    interrupt current (search) command |
| ✓ | `CTRL-D` | `<C-d>` |    scroll Down N lines (default: half a screen) |
| ✓ | `CTRL-E` | `<C-e>` |    scroll N lines upwards (N lines Extra) |
| ✓ | `CTRL-F` | `<C-f>` | scroll N screens Forward |
| ✓ | `CTRL-G` | `<C-g>` |    display current file name and position |
| ✓ | `<BS>` | `<BS>` | same as "h" |
| ✓ | `CTRL-H` | `<C-h>` | same as "h" |
| · | `<Tab>` | `<Tab>` | go to N newer entry in jump list |
| · | `CTRL-I` | `<C-i>` | same as <Tab> |
| ✓ | `<NL>` | `<C-j>` | same as "j" |
| · | `<S-NL>` | `<S-NL>` | same as CTRL-F |
| ✓ | `CTRL-J` | `<C-j>` | same as "j" |
| ✓ | `CTRL-L` | `<C-l>` |    redraw screen |
| ✓ | `<CR>` | `<CR>` | cursor to the first CHAR N lines lower |
| ✓ | `<S-CR>` | `<S-CR>` | same as CTRL-F |
| ✓ | `CTRL-M` | `<C-m>` | same as <CR> |
| ✓ | `CTRL-N` | `<C-n>` | same as "j" |
| · | `CTRL-O` | `<C-o>` | go to N older entry in jump list |
| ✓ | `CTRL-P` | `<C-p>` | same as "k" |
| ✓ | `CTRL-R` | `<C-r>` | redo changes which were undone with 'u' |
| ✓ | `CTRL-T` | `<C-t>` |    jump to N older Tag in tag list |
| ✓ | `CTRL-U` | `<C-u>` |    scroll N lines Upwards (default: half a |
| · | `CTRL-V` | `<C-v>` |    start blockwise Visual mode |
| ✓ | `CTRL-W {char}` | `<C-w>a` |    window commands, see \|CTRL-W\| |
| ✓ | `CTRL-X` | `<C-x>` | subtract N from number at/after cursor |
| ✓ | `CTRL-Y` | `<C-y>` |    scroll N lines downwards |
| ✓ | `CTRL-Z` | `<C-z>` |    suspend program (or start new shell) |
| · | `CTRL-\ CTRL-N` | `<C-\><C-n>` |    go to Normal mode (no-op) |
| · | `CTRL-\ CTRL-G` | `<C-\><C-g>` |    go to Normal mode (no-op) |
| ✓ | `CTRL-]` | `<C-]>` |    :ta to ident under cursor |
| ✓ | `CTRL-^` | `<C-^>` |    edit Nth alternate file (equivalent to |
| · | `CTRL-<Tab>` | `<C-<>Tab>` |    same as `g<Tab>` : go to last accessed tab |
| ✓ | `<Space>` | `<Space>` | same as "l" |
| · | `!{motion}{filter}` | `` | 1 |
| ✓ | `!!{filter}` | `!!sort<CR>` | filter N lines through the {filter} command |
| ✓ | `"{register}` | `"a` |    use {register} for next delete, yank or put |
| ✓ | `#` | `#` | search backward for the Nth occurrence of |
| ✓ | `$` | `$` | cursor to the end of Nth next line |
| ✓ | `%` | `%` | find the next (curly/square) bracket on |
| ✓ | `{count}%` | `2%` | go to N percentage in the file |
| ✓ | `&` | `&` | repeat last :s |
| ✓ | `'{a-zA-Z0-9}` | `'a` | cursor to the first CHAR on the line with |
| ✓ | `''` | `''` | cursor to the first CHAR of the line where |
| ✓ | `'(` | `'(` | cursor to the first CHAR on the line of the |
| ✓ | `')` | `')` | cursor to the first CHAR on the line of the |
| ✓ | `'<` | `'<` | cursor to the first CHAR of the line where |
| ✓ | `'>` | `'>` | cursor to the first CHAR of the line where |
| ✓ | `'[` | `'[` | cursor to the first CHAR on the line of the |
| ✓ | `']` | `']` | cursor to the first CHAR on the line of the |
| ✓ | `'{` | `'{` | cursor to the first CHAR on the line of the |
| ✓ | `'}` | `'}` | cursor to the first CHAR on the line of the |
| ✓ | `(` | `(` | cursor N sentences backward |
| ✓ | `)` | `)` | cursor N sentences forward |
| ✓ | `*` | `*` | search forward for the Nth occurrence of |
| ✓ | `+` | `+` | same as <CR> |
| ✓ | `<S-+>` | `<S-+>` | same as CTRL-F |
| ✓ | `,` | `,` | repeat latest f, t, F or T in opposite |
| ✓ | `-` | `-` | cursor to the first CHAR N lines higher |
| · | `<S-->` | `<S-->` | same as CTRL-B |
| ✓ | `.` | `.` | repeat last change with count replaced with |
| ✓ | `/{pattern}<CR>` | `/beta<CR>` | search forward for the Nth occurrence of |
| ✓ | `/<CR>` | `/<CR>` | search forward for {pattern} of last search |
| · | `0` | `0` | cursor to the first char of the line |
| ✓ | `1` | `1` |    prepend to command to give a count |
| ✓ | `2` | `2` | " |
| ✓ | `3` | `3` | " |
| ✓ | `4` | `4` | " |
| ✓ | `5` | `5` | " |
| ✓ | `6` | `6` | " |
| ✓ | `7` | `7` | " |
| ✓ | `8` | `8` | " |
| ✓ | `9` | `9` | " |
| ✓ | `:` | `:` | start entering an Ex command |
| · | `{count}:` | `2:` |    start entering an Ex command with range |
| ✓ | `;` | `;` | repeat latest f, t, F or T N times |
| ✓ | `<{motion}` | `<w` | shift Nmove lines one 'shiftwidth' |
| ✓ | `<<` | `<<` | shift N lines one 'shiftwidth' leftwards |
| ✓ | `={motion}` | `=w` | filter Nmove lines through "indent" |
| ✓ | `==` | `==` | filter N lines through "indent" |
| ✓ | `>{motion}` | `>w` | shift Nmove lines one 'shiftwidth' |
| ✓ | `>>` | `>>` | shift N lines one 'shiftwidth' rightwards |
| ✓ | `?{pattern}<CR>` | `?beta<CR>` | search backward for the Nth previous |
| ✓ | `?<CR>` | `?<CR>` | search backward for {pattern} of last search |
| · | `@{a-z}` | `@a` | execute the contents of register {a-z} |
| ✓ | `@:` | `@:` |    repeat the previous ":" command N times |
| ✓ | `@@` | `@@` | repeat the previous @{a-z} N times |
| ✓ | `A` | `A` | append text after the end of the line N times |
| ✓ | `B` | `B` | cursor N WORDS backward |
| ✓ | `["x]C` | `C` | change from the cursor position to the end |
| ✓ | `["x]D` | `D` | delete the characters under the cursor |
| ✓ | `E` | `E` | cursor forward to the end of WORD N |
| ✓ | `F{char}` | `Fa` | cursor to the Nth occurrence of {char} to |
| ✓ | `G` | `G` | cursor to line N, default last line |
| ✓ | `H` | `H` | cursor to line N from top of screen |
| ✓ | `I` | `I` | insert text before the first CHAR on the |
| ✓ | `J` | `J` | Join N lines; default is 2 |
| ✓ | `K` | `K` |    lookup Keyword under the cursor with |
| ✓ | `L` | `L` | cursor to line N from bottom of screen |
| ✓ | `M` | `M` | cursor to middle line of screen |
| ✓ | `N` | `N` | repeat the latest '/' or '?' N times in |
| ✓ | `O` | `O` | begin a new line above the cursor and |
| ✓ | `["x]P` | `P` | put the text [from register x] before the |
| ✓ | `R` | `R` | enter replace mode: overtype existing |
| ✓ | `["x]S` | `S` | delete N lines [into register x] and start |
| ✓ | `T{char}` | `Ta` | cursor till after Nth occurrence of {char} |
| · | `U` | `U` | undo all latest changes on one line |
| ✓ | `V` | `V` |    start linewise Visual mode |
| ✓ | `W` | `W` | cursor N WORDS forward |
| ✓ | `["x]X` | `X` | delete N characters before the cursor [into |
| ✓ | `["x]Y` | `Y` |    yank N lines [into register x]; synonym for |
| ✓ | `ZZ` | `ZZ` |    write if buffer changed and close window |
| ✓ | `ZQ` | `ZQ` |    close window without writing |
| · | `[{char}` | `[a` |    square bracket command (see \|[\| below) |
| · | `]{char}` | `]a` |    square bracket command (see \|]\| below) |
| ✓ | `^` | `^` | cursor to the first CHAR of the line |
| ✓ | `_` | `_` | cursor to the first CHAR N - 1 lines lower |
| ✓ | ``{a-zA-Z0-9}` | ``a` | cursor to the mark {a-zA-Z0-9} |
| ✓ | ``(` | ``(` | cursor to the start of the current sentence |
| ✓ | ``)` | ``)` | cursor to the end of the current sentence |
| ✓ | ``<` | ``<` | cursor to the start of the highlighted area |
| ✓ | ``>` | ``>` | cursor to the end of the highlighted area |
| ✓ | ``[` | ``[` | cursor to the start of last operated text |
| ✓ | ``]` | ``]` | cursor to the end of last operated text or |
| · | `"``"` | `"``"` | cursor to the position before latest jump |
| ✓ | ``{` | ``{` | cursor to the start of the current paragraph |
| ✓ | ``}` | ``}` | cursor to the end of the current paragraph |
| ✓ | `a` | `a` | append text after the cursor N times |
| ✓ | `b` | `b` | cursor N words backward |
| ✓ | `["x]c{motion}` | `cw` | delete Nmove text [into register x] and |
| ✓ | `["x]cc` | `cc` | delete N lines [into register x] and start |
| ✓ | `["x]d{motion}` | `dw` | delete Nmove text [into register x] |
| ✓ | `["x]dd` | `dd` | delete N lines [into register x] |
| · | `do` | `do` | same as ":diffget" |
| · | `dp` | `dp` | same as ":diffput" |
| ✓ | `e` | `e` | cursor forward to the end of word N |
| ✓ | `f{char}` | `fa` | cursor to Nth occurrence of {char} to the |
| ✓ | `g{char}` | `ga` |    extended commands, see \|g\| below |
| ✓ | `h` | `h` | cursor N chars to the left |
| ✓ | `i` | `i` | insert text before the cursor N times |
| ✓ | `j` | `j` | cursor N lines downward |
| ✓ | `k` | `k` | cursor N lines upward |
| ✓ | `l` | `l` | cursor N chars to the right |
| ✓ | `m{A-Za-z}` | `ma` |    set mark {A-Za-z} at cursor position |
| ✓ | `n` | `n` | repeat the latest '/' or '?' N times |
| ✓ | `o` | `o` | begin a new line below the cursor and |
| ✓ | `["x]p` | `p` | put the text [from register x] after the |
| ✓ | `q{0-9a-zA-Z"}` | `qa` |    record typed characters into named register |
| · | `q` | `q` |    (while recording) stops recording |
| ✓ | `Q` | `Q` | replay last recorded register |
| ✓ | `q:` | `q:` |    edit : command-line in command-line window |
| ✓ | `q/` | `q/` |    edit / command-line in command-line window |
| ✓ | `q?` | `q?` |    edit ? command-line in command-line window |
| ✓ | `r{char}` | `ra` | replace N chars with {char} |
| ✓ | `["x]s` | `s` | (substitute) delete N characters [into |
| ✓ | `t{char}` | `ta` | cursor till before Nth occurrence of {char} |
| ✓ | `u` | `u` | undo changes |
| ✓ | `v` | `v` |    start charwise Visual mode |
| ✓ | `w` | `w` | cursor N words forward |
| ✓ | `["x]x` | `x` | delete N characters under and after the |
| ✓ | `["x]y{motion}` | `yw` |    yank Nmove text [into register x] |
| ✓ | `["x]yy` | `yy` |    yank N lines [into register x] |
| · | `z{char}` | `za` |    commands starting with 'z', see \|z\| below |
| ✓ | `{` | `{` | cursor N paragraphs backward |
| ✓ | `\|` | `\|` | cursor to column N |
| ✓ | `}` | `}` | cursor N paragraphs forward |
| ✓ | `~` | `~` | 'tildeop' off: switch case of N characters |
| ✓ | `~{motion}` | `~w` |    'tildeop' on: switch case of Nmove text |
| · | `<C-End>` | `<C-End>` | same as "G" |
| · | `<C-Home>` | `<C-Home>` | same as "gg" |
| · | `<C-Left>` | `<C-Left>` | same as "b" |
| · | `<C-LeftMouse>` | `<C-LeftMouse>` |    ":ta" to the keyword at the mouse click |
| · | `<C-Right>` | `<C-Right>` | same as "w" |
| · | ` <C-RightMouse>` | `<C-RightMouse>` |    same as "CTRL-T" |
| · | `<C-Tab>` | `<C-Tab>` |    same as "g<Tab>" |
| ✓ | `["x]<Del>` | `<Del>` | same as "x" |
| ✓ | `{count}<Del>` | `2<Del>` |    remove the last digit from {count} |
| ✓ | `<Down>` | `<Down>` | same as "j" |
| ✓ | `<End>` | `<End>` | same as "$" |
| · | `<F1>` | `<F1>` |    same as <Help> |
| · | `<Help>` | `<Help>` |    open a help window |
| ✓ | `<Home>` | `<Home>` | same as "0" |
| · | `<Insert>` | `<Insert>` | same as "i" |
| ✓ | `<Left>` | `<Left>` | same as "h" |
| · | `<LeftMouse>` | `<LeftMouse>` | move cursor to the mouse click position |
| · | `<MiddleMouse>` | `<MiddleMouse>` | same as "gP" at the mouse click position |
| · | `<PageDown>` | `<PageDown>` |    same as CTRL-F |
| · | `<PageUp>` | `<PageUp>` |    same as CTRL-B |
| ✓ | `<Right>` | `<Right>` | same as "l" |
| · | `<RightMouse>` | `<RightMouse>` |    start Visual mode, move cursor to the mouse |
| ✓ | `<S-Down>` | `<S-Down>` | same as CTRL-F |
| ✓ | `<S-Left>` | `<S-Left>` | same as "b" |
| · | `<S-LeftMouse>` | `<S-LeftMouse>` |    same as "*" at the mouse click position |
| ✓ | `<S-Right>` | `<S-Right>` | same as "w" |
| · | ` <S-RightMouse>` | `<S-RightMouse>` |    same as "#" at the mouse click position |
| ✓ | `<S-Up>` | `<S-Up>` | same as CTRL-B |
| · | `<Undo>` | `<Undo>` | same as "u" |
| ✓ | `<Up>` | `<Up>` | same as "k" |
| ✓ | `a"` | `da"` |    double quoted string |
| ✓ | `a'` | `da'` |    single quoted string |
| ✓ | `a(` | `da(` |    same as ab |
| ✓ | `a)` | `da)` |    same as ab |
| · | `a<` | `da<` |    "a <>" from '<' to the matching '>' |
| · | `a>` | `da>` |    same as a< |
| · | `aB` | `daB` |    "a Block" from `[{` to `]}` (with brackets) |
| ✓ | `aW` | `daW` |    "a WORD" (with white space) |
| · | `a[` | `da[` |    "a []" from '[' to the matching ']' |
| · | `a]` | `da]` |    same as a[ |
| · | `a`` | `da`` |    string in backticks |
| ✓ | `ab` | `dab` |    "a block" from "[(" to "])" (with braces) |
| ✓ | `ap` | `dap` |    "a paragraph" (with white space) |
| · | `as` | `das` |    "a sentence" (with white space) |
| · | `at` | `dat` |    "a tag block" (with white space) |
| ✓ | `aw` | `daw` |    "a word" (with white space) |
| · | `a{` | `da{` |    same as aB |
| · | `a}` | `da}` |    same as aB |
| ✓ | `i"` | `di"` |    double quoted string without the quotes |
| ✓ | `i'` | `di'` |    single quoted string without the quotes |
| ✓ | `i(` | `di(` |    same as ib |
| ✓ | `i)` | `di)` |    same as ib |
| · | `i<` | `di<` |    "inner <>" from '<' to the matching '>' |
| · | `i>` | `di>` |    same as i< |
| · | `iB` | `diB` |    "inner Block" from `[{` and `]}` |
| ✓ | `iW` | `diW` |    "inner WORD" |
| · | `i[` | `di[` |    "inner []" from '[' to the matching ']' |
| · | `i]` | `di]` |    same as i[ |
| · | `i`` | `di`` |    string in backticks without the backticks |
| ✓ | `ib` | `dib` |    "inner block" from "[(" to "])" |
| ✓ | `ip` | `dip` |    "inner paragraph" |
| · | `is` | `dis` |    "inner sentence" |
| · | `it` | `dit` |    "inner tag block" |
| ✓ | `iw` | `diw` |    "inner word" |
| · | `i{` | `di{` |    same as iB |
| · | `i}` | `di}` |    same as iB |
| ✓ | `CTRL-W CTRL-B` | `<C-w><C-b>` |    same as "CTRL-W b" |
| · | `CTRL-W CTRL-C` | `<C-w><C-c>` |    no-op |
| ✓ | `CTRL-W CTRL-D` | `<C-w><C-d>` |    same as "CTRL-W d" |
| ✓ | `CTRL-W CTRL-F` | `<C-w><C-f>` |    same as "CTRL-W f" |
| ✓ | `CTRL-W CTRL-H` | `<C-w><C-h>` |    same as "CTRL-W h" |
| ✓ | `CTRL-W CTRL-I` | `<C-w><C-i>` |    same as "CTRL-W i" |
| ✓ | `CTRL-W CTRL-J` | `<C-w><C-j>` |    same as "CTRL-W j" |
| ✓ | `CTRL-W CTRL-K` | `<C-w><C-k>` |    same as "CTRL-W k" |
| ✓ | `CTRL-W CTRL-L` | `<C-w><C-l>` |    same as "CTRL-W l" |
| ✓ | `CTRL-W CTRL-N` | `<C-w><C-n>` |    same as "CTRL-W n" |
| ✓ | `CTRL-W CTRL-O` | `<C-w><C-o>` |    same as "CTRL-W o" |
| ✓ | `CTRL-W CTRL-P` | `<C-w><C-p>` |    same as "CTRL-W p" |
| ✓ | `CTRL-W CTRL-Q` | `<C-w><C-q>` |    same as "CTRL-W q" |
| · | `CTRL-W CTRL-R` | `<C-w><C-r>` |    same as "CTRL-W r" |
| ✓ | `CTRL-W CTRL-S` | `<C-w><C-s>` |    same as "CTRL-W s" |
| ✓ | `CTRL-W CTRL-T` | `<C-w><C-t>` |    same as "CTRL-W t" |
| ✓ | `CTRL-W CTRL-V` | `<C-w><C-v>` |    same as "CTRL-W v" |
| ✓ | `CTRL-W CTRL-W` | `<C-w><C-w>` |    same as "CTRL-W w" |
| ✓ | `CTRL-W CTRL-X` | `<C-w><C-x>` |    same as "CTRL-W x" |
| ✓ | `CTRL-W CTRL-Z` | `<C-w><C-z>` |    same as "CTRL-W z" |
| ✓ | `CTRL-W CTRL-]` | `<C-w><C-]>` |    same as "CTRL-W ]" |
| ✓ | `CTRL-W CTRL-^` | `<C-w><C-^>` |    same as "CTRL-W ^" |
| ✓ | `CTRL-W CTRL-_` | `<C-w><C-_>` |    same as "CTRL-W _" |
| ✓ | `CTRL-W +` | `<C-w>+` |    increase current window height N lines |
| ✓ | `CTRL-W -` | `<C-w>-` |    decrease current window height N lines |
| ✓ | `CTRL-W <` | `<C-w><` |    decrease current window width N columns |
| ✓ | `CTRL-W =` | `<C-w>=` |    make all windows the same height & width |
| ✓ | `CTRL-W >` | `<C-w>>` |    increase current window width N columns |
| ✓ | `CTRL-W H` | `<C-w>H` |    move current window to the far left |
| ✓ | `CTRL-W J` | `<C-w>J` |    move current window to the very bottom |
| ✓ | `CTRL-W K` | `<C-w>K` |    move current window to the very top |
| ✓ | `CTRL-W L` | `<C-w>L` |    move current window to the far right |
| ✓ | `CTRL-W P` | `<C-w>P` |    go to preview window |
| ✓ | `CTRL-W R` | `<C-w>R` |    rotate windows upwards N times |
| ✓ | `CTRL-W S` | `<C-w>S` |    same as "CTRL-W s" |
| ✓ | `CTRL-W T` | `<C-w>T` |    move current window to a new tabpage |
| ✓ | `CTRL-W W` | `<C-w>W` |    go to N previous window (wrap around) |
| ✓ | `CTRL-W ]` | `<C-w>]` |    split window and jump to tag under cursor |
| ✓ | `CTRL-W ^` | `<C-w>^` |    split current window and edit alternate |
| ✓ | `CTRL-W _` | `<C-w>_` |    set current window height to N (default: |
| ✓ | `CTRL-W b` | `<C-w>b` |    go to bottom window |
| ✓ | `CTRL-W c` | `<C-w>c` |    close current window (like \|:close\|) |
| ✓ | `CTRL-W d` | `<C-w>d` |    split window and jump to definition under |
| ✓ | `CTRL-W f` | `<C-w>f` |    split window and edit file name under the |
| ✓ | `CTRL-W F` | `<C-w>F` |    split window and edit file name under the |
| · | ` CTRL-W g CTRL-]  split window and do \|:tjump\| to tag under` | `` | 1 |
| ✓ | `CTRL-W g ]` | `<C-w>g]` |    split window and do \|:tselect\| for tag |
| ✓ | `CTRL-W g }` | `<C-w>g}` |    do a \|:ptjump\| to the tag under the cursor |
| ✓ | `CTRL-W g f` | `<C-w>gf` |    edit file name under the cursor in a new |
| ✓ | `CTRL-W g F` | `<C-w>gF` |    edit file name under the cursor in a new |
| ✓ | `CTRL-W g t` | `<C-w>gt` |    same as `gt`: go to next tabpage |
| ✓ | `CTRL-W g T` | `<C-w>gT` |    same as `gT`: go to previous tabpage |
| · | `CTRL-W g <Tab>` | `<C-w>g<Tab>` |    same as \|g<Tab>\|: go to last accessed tab |
| ✓ | `CTRL-W h` | `<C-w>h` |    go to Nth left window (stop at first window) |
| ✓ | `CTRL-W i` | `<C-w>i` |    split window and jump to declaration of |
| ✓ | `CTRL-W j` | `<C-w>j` |    go N windows down (stop at last window) |
| ✓ | `CTRL-W k` | `<C-w>k` |    go N windows up (stop at first window) |
| ✓ | `CTRL-W l` | `<C-w>l` |    go to Nth right window (stop at last window) |
| ✓ | `CTRL-W n` | `<C-w>n` |    open new window, N lines high |
| ✓ | `CTRL-W o` | `<C-w>o` |    close all but current window (like \|:only\|) |
| ✓ | `CTRL-W p` | `<C-w>p` |    go to previous (last accessed) window |
| ✓ | `CTRL-W q` | `<C-w>q` |    quit current window (like \|:quit\|) |
| ✓ | `CTRL-W r` | `<C-w>r` |    rotate windows downwards N times |
| ✓ | `CTRL-W s` | `<C-w>s` |    split current window in two parts, new |
| ✓ | `CTRL-W t` | `<C-w>t` |    go to top window |
| ✓ | `CTRL-W v` | `<C-w>v` |    split current window vertically, new window |
| ✓ | `CTRL-W w` | `<C-w>w` |    go to N next window (wrap around) |
| ✓ | `CTRL-W x` | `<C-w>x` |    exchange current window with window N |
| ✓ | `CTRL-W z` | `<C-w>z` |    close preview window |
| ✓ | `CTRL-W \|` | `<C-w>\|` |    set window width to N columns |
| ✓ | `CTRL-W }` | `<C-w>}` |    show tag under cursor in preview window |
| ✓ | `CTRL-W <Down>` | `<C-w><Down>` |    same as "CTRL-W j" |
| ✓ | `CTRL-W <Up>` | `<C-w><Up>` |    same as "CTRL-W k" |
| ✓ | `CTRL-W <Left>` | `<C-w><Left>` |    same as "CTRL-W h" |
| ✓ | ` CTRL-W <Right>` | `<C-w><Right>` |    same as "CTRL-W l" |
| · | `[ CTRL-D` | `[<C-d>` |    jump to first #define found in current and |
| · | `[ CTRL-I` | `[<C-i>` |    jump to first line in current and included |
| · | `[#` | `[#` | cursor to N previous unmatched #if, #else |
| · | `['` | `['` | cursor to previous lowercase mark, on first |
| · | `[(` | `[(` | cursor N times back to unmatched '(' |
| · | `[*` | `[*` | same as "[/" |
| · | `[`` | `[`` | cursor to previous lowercase mark |
| · | `[/` | `[/` | cursor to N previous start of a C comment |
| · | `[D` | `[D` |    list all defines found in current and |
| · | `[I` | `[I` |    list all lines found in current and |
| · | `[P` | `[P` | same as "[p" |
| · | `[[` | `[[` | cursor N sections backward |
| · | `[]` | `[]` | cursor N SECTIONS backward |
| · | `[c` | `[c` | cursor N times backwards to start of change |
| · | `[d` | `[d` |    show first #define found in current and |
| · | `[f` | `[f` |    same as "gf" |
| · | `[i` | `[i` |    show first line found in current and |
| · | `[m` | `[m` | cursor N times back to start of member |
| · | `[p` | `[p` | like "P", but adjust indent to current line |
| · | `[s` | `[s` | move to the previous misspelled word |
| · | `[z` | `[z` | move to start of open fold |
| · | `[{` | `[{` | cursor N times back to unmatched '{' |
| · | ` [<MiddleMouse>` | `[<MiddleMouse>` | same as "[p" |
| · | `] CTRL-D` | `]<C-d>` |    jump to first #define found in current and |
| · | `] CTRL-I` | `]<C-i>` |    jump to first line in current and included |
| · | `]#` | `]#` | cursor to N next unmatched #endif or #else |
| · | `]'` | `]'` | cursor to next lowercase mark, on first |
| · | `])` | `])` | cursor N times forward to unmatched ')' |
| · | `]*` | `]*` | same as "]/" |
| · | `]`` | `]`` | cursor to next lowercase mark |
| · | `]/` | `]/` | cursor to N next end of a C comment |
| · | `]D` | `]D` |    list all #defines found in current and |
| · | `]I` | `]I` |    list all lines found in current and |
| · | `]P` | `]P` | same as "[p" |
| · | `][` | `][` | cursor N SECTIONS forward |
| · | `]]` | `]]` | cursor N sections forward |
| · | `]c` | `]c` | cursor N times forward to start of change |
| · | `]d` | `]d` |    show first #define found in current and |
| · | `]f` | `]f` |    same as "gf" |
| · | `]i` | `]i` |    show first line found in current and |
| · | `]m` | `]m` | cursor N times forward to end of member |
| · | `]p` | `]p` | like "p", but adjust indent to current line |
| · | `]s` | `]s` | move to next misspelled word |
| · | `]z` | `]z` | move to end of open fold |
| · | `]}` | `]}` | cursor N times forward to unmatched '}' |
| · | ` ]<MiddleMouse>` | `]<MiddleMouse>` | same as "]p" |
| ✓ | `g CTRL-G` | `g<C-g>` |    show information about current cursor |
| ✓ | `g CTRL-H` | `g<C-h>` |    start Select block mode |
| · | `g CTRL-]` | `g<C-]>` |    \|:tjump\| to the tag under the cursor |
| · | `g#` | `g#` | like "#", but without using "\<" and "\>" |
| · | `g$` | `g$` | when 'wrap' off go to rightmost character of |
| ✓ | `g&` | `g&` | repeat last ":s" on all lines |
| · | `g'{mark}` | `g'a` | like \|'\| but without changing the jumplist |
| · | `g`{mark}` | `g`a` | like \|`\| but without changing the jumplist |
| · | `g*` | `g*` | like "*", but without using "\<" and "\>" |
| ✓ | `g+` | `g+` |    go to newer text state N times |
| ✓ | `g,` | `g,` | go to N newer position in change list |
| ✓ | `g-` | `g-` |    go to older text state N times |
| · | `g0` | `g0` | when 'wrap' off go to leftmost character of |
| ✓ | `g8` | `g8` |    print hex value of bytes used in UTF-8 |
| ✓ | `g;` | `g;` | go to N older position in change list |
| ✓ | `g<` | `g<` |    display previous command output |
| · | `g?` | `g?` | Rot13 encoding operator |
| ✓ | `g??` | `g??` | Rot13 encode current line |
| ✓ | `g?g?` | `g?g?` | Rot13 encode current line |
| ✓ | `gD` | `gD` | go to definition of word under the cursor |
| ✓ | `gE` | `gE` | go backwards to the end of the previous |
| ✓ | `gH` | `gH` |    start Select line mode |
| ✓ | `gI` | `gI` | like "I", but always start in column 1 |
| ✓ | `gJ` | `gJ` | join lines without inserting space |
| ✓ | `gN` | `gN` |       1,2  find the previous match with the last used |
| ✓ | `["x]gP` | `gP` | put the text [from register x] before the |
| · | `gQ` | `gQ` |    switch to "Ex" mode with Vim editing |
| · | `gR` | `gR` | enter Virtual Replace mode |
| ✓ | `gT` | `gT` |    go to the previous tabpage |
| ✓ | `gU{motion}` | `gUw` | make Nmove text uppercase |
| ✓ | `gV` | `gV` |    don't reselect the previous Visual area |
| · | `g]` | `g]` |    :tselect on the tag under the cursor |
| · | `g^` | `g^` | when 'wrap' off go to leftmost non-white |
| ✓ | `g_` | `g_` | cursor to the last CHAR N - 1 lines lower |
| ✓ | `ga` | `ga` |    print ascii value of character under the |
| ✓ | `gd` | `gd` | go to definition of word under the cursor |
| ✓ | `ge` | `ge` | go backwards to the end of the previous |
| ✓ | `gf` | `gf` |    start editing the file whose name is under |
| ✓ | `gF` | `gF` |    start editing the file whose name is under |
| ✓ | `gg` | `gg` | cursor to line N, default first line |
| ✓ | `gh` | `gh` |    start Select mode |
| ✓ | `gi` | `gi` | like "i", but first move to the \|'^\| mark |
| · | `gj` | `gj` | like "j", but when 'wrap' on go N screen |
| · | `gk` | `gk` | like "k", but when 'wrap' on go N screen |
| ✓ | `gm` | `gm` | go to character at middle of the screenline |
| ✓ | `gM` | `gM` | go to character at middle of the text line |
| ✓ | `gn` | `gn` |       1,2  find the next match with the last used |
| ✓ | `go` | `go` | cursor to byte N in the buffer |
| ✓ | `["x]gp` | `gp` | put the text [from register x] after the |
| ✓ | `gq{motion}` | `gqw` | format Nmove text |
| · | `gr{char}` | `gra` | virtual replace N chars with {char} |
| ✓ | `gs` | `gs` |    go to sleep for N seconds (default 1) |
| ✓ | `gt` | `gt` |    go to the next tabpage |
| · | `gu{motion}` | `guw` | make Nmove text lowercase |
| ✓ | `gv` | `gv` |    reselect the previous Visual area |
| ✓ | `gw{motion}` | `gww` | format Nmove text and keep cursor |
| ✓ | `gx` | `gx` |    execute application for filepath at cursor |
| ✓ | `g@{motion}` | `g@w` |    call 'operatorfunc' |
| ✓ | `g~{motion}` | `g~w` | swap case for Nmove text |
| · | `g<Down>` | `g<Down>` | same as "gj" |
| · | `g<End>` | `g<End>` | same as "g$" but go to the rightmost |
| · | `g<Home>` | `g<Home>` | same as "g0" |
| · | `g<LeftMouse>` | `g<LeftMouse>` |    same as <C-LeftMouse> |
| · | `g<RightMouse>` | `g<RightMouse>` |    same as <C-RightMouse> |
| · | `g<Tab>` | `g<Tab>` |    go to last accessed tabpage |
| · | `g<Up>` | `g<Up>` | same as "gk" |
| ✓ | `z<CR>` | `z<CR>` |    redraw, cursor line to top of window, |
| · | `z{height}<CR>` | `z5<CR>` |    redraw, make window {height} lines high |
| ✓ | `z+` | `z+` |    cursor on line N (default line below |
| ✓ | `z-` | `z-` |    redraw, cursor line at bottom of window, |
| ✓ | `z.` | `z.` |    redraw, cursor line to center of window, |
| · | `z=` | `z=` |    give spelling suggestions |
| · | `zA` | `zA` |    open a closed fold or close an open fold |
| · | `zC` | `zC` |    close folds recursively |
| · | `zD` | `zD` |    delete folds recursively |
| · | `zE` | `zE` |    eliminate all folds |
| · | `zF` | `zF` |    create a fold for N lines |
| · | `zG` | `zG` |    temporarily mark word as correctly spelled |
| ✓ | `zH` | `zH` |    when 'wrap' off scroll half a screenwidth |
| ✓ | `zL` | `zL` |    when 'wrap' off scroll half a screenwidth |
| · | `zM` | `zM` |    set 'foldlevel' to zero |
| · | `zN` | `zN` |    set 'foldenable' |
| · | `zO` | `zO` |    open folds recursively |
| · | `zR` | `zR` |    set 'foldlevel' to the deepest fold |
| · | `zW` | `zW` |    temporarily mark word as incorrectly spelled |
| · | `zX` | `zX` |    re-apply 'foldlevel' |
| ✓ | `z^` | `z^` |    cursor on line N (default line above |
| · | `za` | `za` |    open a closed fold, close an open fold |
| ✓ | `zb` | `zb` |    redraw, cursor line at bottom of window |
| · | `zc` | `zc` |    close a fold |
| · | `zd` | `zd` |    delete a fold |
| ✓ | `ze` | `ze` |    when 'wrap' off scroll horizontally to |
| · | `zf{motion}` | `zfw` |    create a fold for Nmove text |
| · | `zg` | `zg` |    permanently mark word as correctly spelled |
| ✓ | `zh` | `zh` |    when 'wrap' off scroll screen N characters |
| · | `zi` | `zi` |    toggle 'foldenable' |
| · | `zj` | `zj` | move to the start of the next fold |
| · | `zk` | `zk` | move to the end of the previous fold |
| ✓ | `zl` | `zl` |    when 'wrap' off scroll screen N characters |
| · | `zm` | `zm` |    subtract one from 'foldlevel' |
| · | `zn` | `zn` |    reset 'foldenable' |
| · | `zo` | `zo` |    open fold |
| · | `zp` | `zp` |    paste in block-mode without trailing spaces |
| · | `zP` | `zP` |    paste in block-mode without trailing spaces |
| · | `zr` | `zr` |    add one to 'foldlevel' |
| ✓ | `zs` | `zs` |    when 'wrap' off scroll horizontally to |
| ✓ | `zt` | `zt` |    redraw, cursor line at top of window |
| · | `zuw` | `zuw` |    undo \|zw\| |
| · | `zug` | `zug` |    undo \|zg\| |
| · | `zuW` | `zuW` |    undo \|zW\| |
| · | `zuG` | `zuG` |    undo \|zG\| |
| · | `zv` | `zv` |    open enough folds to view the cursor line |
| · | `zw` | `zw` |    permanently mark word as incorrectly spelled |
| · | `zx` | `zx` |    re-apply 'foldlevel' and do "zv" |
| · | `zy` | `zy` |    yank without trailing spaces |
| ✓ | `zz` | `zz` |    redraw, cursor line at center of window |
| ✓ | `z<Left>` | `z<Left>` |    same as "zh" |
| ✓ | `z<Right>` | `z<Right>` |    same as "zl" |
| ✓ | `v` | `v` | force operator to work charwise |
| ✓ | `V` | `V` | force operator to work linewise |
| · | `CTRL-V` | `<C-v>` | force operator to work blockwise |
