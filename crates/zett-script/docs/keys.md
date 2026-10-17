# Keys

`SPC` is the leader. Press it and wait: which-key shows what comes next,
groups marked with `+`. The tree follows Doom Emacs:

- `SPC f`  file      `SPC b`  buffer    `SPC w`  window    `SPC p`  project
- `SPC s`  search    `SPC o`  open      `SPC g`  git       `SPC c`  code
- `SPC t`  toggle    `SPC h`  help      `SPC q`  quit

`SPC h k` lists every key, `SPC h c` every command, `SPC :` runs one by name.

Change keys in `~/.config/zett/init.lua`:

    zett.map("n", "<leader>gs", "git", { desc = "Git" })
    zett.map("n", "<C-s>", "save-file")

A key can run a command name or a Lua function. See `config`.
