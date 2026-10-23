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
- `C-x [ C-x ]` pages (form feeds, `^L`), `C-x C-p` marks one, `C-x n p` narrows to it, `C-x l` counts its lines
- `C-x C-n` keeps a goal column for `C-n C-p` (`C-u C-x C-n` drops it); `C-u C-SPC` back to the
  marks before, newest first (the mark ring)
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
- `Insert` overwrite mode; `C-x .` a fill prefix for `M-q` and `M-j`; `M-q` in code reindents the
  definition (a comment is filled); `M-$` spelling suggestions for the word

## Searching and replacing

- `C-s C-r` search as you type (again: the next match), `M-s o` the matching lines to pick from
- `M-s .` searches for the symbol at the cursor; `M-s h r` `M-s h p` `M-s h l` `M-s h .` highlight a
  regexp, a phrase, matching lines, the symbol (three at a time), `M-s h u` removes them; `M-s M-w`
  searches the web for the region
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
- the counter: `F3` (or `C-x C-k C-i`) while recording writes it and counts on, so each playback
  writes the next number; `C-x C-k C-c` sets it, `C-x C-k C-a` adds to it, `C-x C-k C-f` its
  format (`%d`, `%03d`, `%x`)
- `C-x q` while recording: a query point; playing the macro stops there to ask: `y` (or `SPC`)
  goes on, `n` skips the rest of this round, `q` stops
- `C-x C-k n` names the last macro (an `M-x` command of its own), `C-x C-k b` binds it to a key,
  `C-x C-k r` plays it once on each line of the region
- `C-x z` the last command again, then each `z` again

## Characters, encodings, the font

- `C-\` types through an input method (`TeX` at first: `\alpha` α, `\to` →, `\in` then space ∈,
  `x^2` x²); `C-x RET C-\` picks another (`german-prefix` `"a` ä, `german-postfix` `ae` ä,
  `latin-1-prefix` `'e` é), `C-x \` uses it for one character, `C-h I` lists what it makes
- `C-x RET f` the encoding and line ends to save with (`utf-8-dos`, `latin-1-unix`,
  `utf-8-with-signature`), `C-x RET r` reads the file again in another one (a file guessed wrong),
  `C-h C` says which the buffer has; zett reads and writes UTF-8 and Latin-1
- `C-x C-=` `C-x C--` `C-x C-0` the font bigger, smaller, back: zash changes it when asked (also
  through zeit); another terminal keeps its own keys for that
- `C-x x @` reads the file again as root (sudo asks its password on the terminal), `C-x C-s` then
  writes it as root; `M-x sudo-save` writes any buffer so

## ChangeLog

- `C-x 4 a` the ChangeLog (the nearest one up the folders, else a new one in the project) in
  another window, an entry for this file and its function under today's heading
- `C-x v a` the commits since the ChangeLog's newest day written into it as entries

## Lua, zett's Emacs Lisp

- `C-x C-e` evaluates the Lua before the cursor on the line, `C-j` writes the value in below,
  `C-M-x` the definition around the cursor, `C-c C-e` the region (or the buffer); `M-:` asks for one

## Bookmarks

- `C-x r m` a bookmark here (a name, or the file's), `C-x r b` jumps to one, `C-x r l` lists them;
  they stay between sessions (zett's state folder)

## Files, buffers, windows

- `C-x C-f` find a file, `C-x C-s` save, `C-x s` save all, `C-x C-w` write to another file, `C-x i`
  insert one, `C-x C-v` another in its place, `C-x x g` read it again, `C-x C-q` read-only
- `C-x b` switch buffer, `C-x C-b` list them, `C-x k` kill one, `C-x x r` rename it, `C-x ← →` the
  previous / next
- `C-x d` `C-x C-j` the folder (zett's explorer), `C-x x i` inserts another buffer, `C-M-v` and
  `M-PageUp` scroll the other window
- `C-x 2 C-x 3` split, `C-x o C-x O` the next / previous window, `C-x 0 C-x 1` close / keep only
  this one, `C-x +` balance, `C-x ^ C-x { C-x }` sizes; `C-x 4 C-f`, `C-x 4 b`, `C-x 4 .` in another
  window
- `C-x p f p g c s` the project: a file, another project, search, compile, a shell; `C-x v d` git
  status, `C-x v h` the commits of the section at the cursor
- `C-x C-c` quit (with unsaved buffers it says so: `C-x s` saves them), `C-z` back to the shell

## Tabs

- each tab keeps its own windows (the buffers are shared): `C-x t 2` a new one, `C-x t 0` close,
  `C-x t 1` the others, `C-x t o O` (and `C-Tab` `C-S-Tab`) next and previous, `C-x t r` a name,
  `C-x t RET` by name, `C-x t m M` move, `C-x t n` the same windows again, `C-x t u` the one closed
  last; `C-x t b C-f d p` a buffer, file, folder, project in a new tab
- from two tabs on the tab line lists them (the buffers otherwise); Vim's `:tabnew` and `gt` are
  the same tabs

## Version control (git)

- `C-x v =` the file's changes, `C-x v D` all of them (coloured: added, removed); `C-x v E =` and
  `C-x v T =` what is not pushed yet; `C-x v M D` since where the branch left another
- `C-x v l` the file's commits, `C-x v L` all, `C-x v O` not pushed, `C-x v I` not pulled; Enter
  shows one; `C-x v g` who changed each line; `C-x v ~` the file as it was in a commit
- `C-x v v` the next step: an untracked file is added, a changed one asks for the message and is
  committed (on its own); `C-x v i` add, `C-x v @` discard the changes, `C-x v x` delete, `C-x v R`
  rename, `C-x v G` ignore
- `C-x v b c` `C-x v b s` a new branch, another one; `C-x v m` merge; `C-x v s` `C-x v r` tags;
  `C-x v +` pull and `C-x v P` push in a terminal pane; `C-x v w c w x R` working trees
- `C-x v d` (and `C-x g`) the git buffer, as Magit; `C-x v h` the commits of the section at the cursor

## The mouse

- a click places the cursor, a drag selects, the middle button yanks there, the right button takes
  the region to the click (copied; again: killed); `C-click` the buffer list
- drag a status line or the line between windows to resize, `C-middle-click` on one to split there
  (in the Vim profile too); `F10` `S-F10` the menu, `F11` `M-F10` the terminal window full screen or
  maximized (Hyprland)

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
