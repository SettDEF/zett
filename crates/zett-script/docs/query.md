# Query blocks

A fenced block named `query` in a Markdown note runs a search over the note's project and shows
what it finds under the block. The results stay current (looked up again every 10 s while it is on
screen); the block itself is plain text, so the note stays a plain file.

    ```query
    tasks
    ```

## Queries

- `tasks`          open tasks (`- [ ]`) in the project's Markdown files; `tasks done`, `tasks all`
- `todo`           TODO, FIXME and XXX in any text file
- `grep TEXT`      lines with TEXT (any case, unless TEXT has capitals)
- `files GLOB`     files named like GLOB (`*.md`, `notes-?.txt`), newest first
- `recent 7d`      files changed in the last 7 days (or `12h`), newest first
- `links NAME`     lines linking to the note NAME: `[[NAME]]` or `(NAME.md)`

Any query takes `in DIR` (a folder of the project) and `limit N` (50 unless said):
`tasks in notes limit 10`. A ` # …` ends a line's words.

## Keys

- `C-c C-o` (Emacs) / `SPC s q`: all the results of the block at the cursor, in the minibuffer;
  Enter opens one at its line. A block shows 12 under it, the rest are there.

Build output, dependencies and hidden folders (`.git`, `target`, `node_modules`) are not searched.

# Block references

A line that is only `![[target]]` shows the target's text under it, as it is now: a note, a file,
or one part of it with `#`: `![[plan#Next steps]]` (a heading; the exact name, else the first
containing the words), `![[src/main.rs#fn main]]` (a block of code, by its first line). The target is
a path from the note's folder or the project (`.md` may be left out), or a note's name anywhere in
the project. It is read again every 3 s; 15 lines show, `C-c C-o` (`SPC s q`) on the line opens it
at that part.

At the end of every note: what links to it, `[[name]]`, `[[name#part]]` or `(name.md)`, in notes and
in code comments (`// design: [[plan]]`). `C-c b` (`SPC s b`) lists them all.

# Code blocks that run

`C-c C-c` (`SPC m r`) in a fenced block of `sh`, `bash`, `zsh`, `fish`, `python`, `js`, `lua` or
`ruby` runs it in the note's folder, in the background (30 s at most; `C-g` stops waiting). What it
prints, and how it ended, is drawn under the block; it belongs to that code, so changing the code
takes it away until the block runs again. `C-c C-v` (`SPC m i`) writes it into the note as an
`output` block under the code (one undo step); write again after another run and it replaces that
block.
