# Emacs keys

zett's default profile is Emacs's (`zett.opt.keys = "vim"` for Vim). The keys are Emacs's own and
run commands by Emacs's names (`M-x save-buffer`); where Emacs has a package for something, zett's
own feature answers it (`C-x v d` is zett's git buffer, `M-.` the language server). Every key and
command is in `:help commands`; `C-h k` says what a key runs, `C-h w` which keys run a command.

## Moving

- `C-f C-b C-n C-p` (and the arrows), `C-a C-e` the line, `M-m` its first non-blank
- `M-f M-b` words, `M-a M-e` sentences, `M-{ M-}` paragraphs, `M-< M->` the buffer
- `C-M-f C-M-b` balanced expressions (a bracket group, a string, a symbol), `C-M-n C-M-p` bracket
  groups, `C-M-u` out to the opening bracket, `C-M-d` into the next group
- `C-M-a C-M-e` the start / end of the definition (zett's outline: `fn`, `def`, `class`, top-level
  blocks; headings in Markdown)
- `C-v M-v` a screen (two lines stay), `C-l` the line to the middle, top, bottom in turn, `M-r` the
  cursor to the window's middle, top, bottom; `C-x < C-x >` sideways
- `M-g g` a line, `M-g c` a character, `M-g TAB` a column (the argument's, or it asks); `M-g n M-g p`
  the next / previous error
- `C-u 4`, `M-5`, `M--` give the next command an argument (`C-u 3 C-n`, `M-- M-f` backwards)

## Killing and yanking

- `C-k` to the line's end (at the end: the line break; `C-u 3 C-k` three lines), `M-d M-DEL` words,
  `M-k C-x DEL` sentences, `C-M-k C-M-DEL` expressions, `M-z` through a character (`M-z ,`)
- `C-SPC` sets the mark, `C-x C-x` swaps it with the cursor; `C-w` kills the region, `M-w` copies it
- `C-y` yanks, `M-y` right after: the kill before it; kills in a row add up, `C-M-w` makes the next
  one add too; the kill ring is the system clipboard (both ways)
- `M-@ M-h C-M-@ C-M-h C-x h` mark a word, a paragraph, an expression, a definition, the buffer

## Changing

- `M-u M-l M-c` a word's case, `C-x C-u C-x C-l` the region's; `C-t M-t C-x C-t C-M-t` transpose
  characters, words, lines, expressions
- `M-SPC` one space, again none, again as it was; `M-\` none; `M-^` this line onto the one before;
  `C-x C-o` blank lines down to one; `C-o C-M-o` open and split a line; `M-j` a line break that
  continues the comment
- `M-q` fills the paragraph (`C-x f` sets the column), `M-;` comments, `M-/` completes from the
  buffer, `M-(` puts brackets around, `M-)` moves past the closing one onto a new line
- `C-x TAB` shifts the region (`C-u -2 C-x TAB` two left), `C-M-\` reindents it; `C-q TAB` inserts a
  key as it is
- `C-/ C-_ C-x u` undo, `C-? C-M-_` redo

## Searching and replacing

- `C-s C-r` search as you type (again: the next match), `M-s o` the matching lines to pick from
- `M-%` replaces, asking at each match: `y` or `SPC` replace, `n` or `DEL` skip, `!` all the rest,
  `.` this one and stop, `q` or `RET` stop; `C-M-%` with an Emacs regexp (`\(…\)` groups, `\1` in
  the replacement); in a region, only there. An all-lower-case text matches any case and the
  replacement takes the match's (`foo` → `bar` makes `Foo` `Bar`, `FOO` `BAR`)

## Rectangles and registers

- the rectangle is the columns between the mark and the cursor, on every line between them: `C-x r
  k` kill, `C-x r y` yank, `C-x r d` delete, `C-x r o` open, `C-x r c` clear, `C-x r t` replace each
  line's part with a text, `C-x r N` number the lines, `C-x r M-w` copy
- registers are Vim's: `C-x r s a` copies the region to `"a` (`"ap` puts it in Vim mode), `C-x r i
  a` inserts it, `C-x r SPC a` remembers the place as mark `a`, `C-x r j a` jumps there, `C-x r n` a
  number, `C-x r +` adds to it, `C-x r r` a rectangle (a block register)

## Keyboard macros

- `C-x (` starts recording, `C-x )` stops, `C-x e` plays it (then each `e` again; `C-u 5 C-x e`
  five times); `F3` `F4` the same

## Files, buffers, windows

- `C-x C-f` find a file, `C-x C-s` save, `C-x s` save all, `C-x C-w` write to another file, `C-x i`
  insert one, `C-x C-v` another in its place, `C-x x g` read it again, `C-x C-q` read-only
- `C-x b` switch buffer, `C-x C-b` list them, `C-x k` kill one, `C-x x r` rename it, `C-x ← →` the
  previous / next
- `C-x 2 C-x 3` split, `C-x o C-x O` the next / previous window, `C-x 0 C-x 1` close / keep only
  this one, `C-x +` balance, `C-x ^ C-x { C-x }` sizes; `C-x 4 C-f`, `C-x 4 b`, `C-x 4 .` in another
  window
- `C-x p f p g c s` the project: a file, another project, search, compile, a shell; `C-x v d` git
  status, `C-x v h` the commits of the section at the cursor
- `C-x C-c` quit (with unsaved buffers it says so: `C-x s` saves them), `C-z` back to the shell

## Help

- `C-h k` what a key runs (`C-h c` in one line), `C-h f` a command, `C-h v` a setting, `C-h w` a
  command's keys, `C-h b` every key, `C-h a` every command, `C-h l` the last keys typed, `C-h e`
  the messages, `C-h ?` every topic
- `M-x` any command by name, `M-:` a Lua expression (zett's Emacs Lisp), `M-!` a shell command,
  `M-&` one in a terminal pane

## Code

- `M-.` the definition, `M-,` back, `M-?` the references, `M-g i` the file's symbols; `C-x 4 .` the
  definition in another window
- `C-c C-n C-c C-p` and `C-c TAB` move between and fold sections (Org's keys, for Markdown and code
  alike); `M-↑ M-↓ M-← M-→` move, promote, demote a whole section; `C-x n s n d w` narrow
