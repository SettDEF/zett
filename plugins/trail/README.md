# Trail

A file's history in a side panel: every save (Trail keeps a copy of each one) and every git commit that touched it.
Pick one and the file shows what changed since then: added lines are marked `+`, removed lines appear in red where they were.

Runs in zett and in Neovim.

- `:Trail` opens or closes the panel (right side by default)
- in the panel: `Enter` / `p` show the change since that point, `x` clear the marks, `r` reload, `q` close
- `:TrailClear` takes the marks off the file

```lua
require("trail").setup({ side = "right", width = 36, commits = 50, saves = 100 })
```

Saves are kept in `stdpath("state")/trail/`, one folder per file, the newest 100 per file.

Next: a change's hunks with their places, restoring one hunk, a list of removed code you can search, scrubbing through time.
