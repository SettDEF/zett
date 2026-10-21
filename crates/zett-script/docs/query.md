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
