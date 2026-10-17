# Plugins: the explorer, the undo tree, your own

zett comes with two plugins, written in Lua on the same API as yours:

- `explorer`  the project's files at the side: `SPC e` or `SPC o p`
  (`SPC o P` shows the file being edited)
- `undotree`  the undo history as a tree, with each change's diff: `SPC s u`

## Settings

Change one now with `:set`, which lists every setting and its value:

    :set explorer.width 40
    :set undotree.live true

Keep them in `~/.config/zett/init.lua` (only what differs from the default):

    zett.opt.explorer = {
      width = 36,
      hidden = true,
      quit_on_open = true,
      keys = { ["<C-v>"] = "vsplit", d = false },   -- false: the key does nothing
    }
    zett.opt.undotree = { side = "right", diff = false }
    zett.opt.undotree = false                       -- turn a plugin off

A key can run your own function; the explorer passes the node under the cursor:

    zett.opt.explorer = { keys = {
      ["<C-g>"] = function(node) zett.notify(node.path) end,
    } }

## The explorer's keys

`Enter` `l` open · `h` close the folder · `s` `i` open in a split · `a` new
(end with / for a folder) · `r` rename · `d` delete (to the trash) · `c` `x` `p`
copy, cut, paste · `y` `Y` copy name, path · `H` dotfiles · `/` filter ·
`F` clear it · `-` up a folder · `C` this folder as the root · `W` collapse all ·
`R` refresh · `q` close · `?` every key

## The undo tree's keys

`j` `k` choose a state (its diff shows below) · `Enter` go there · `u` `C-r`
undo, redo · `J` `K` older, newer in time · `d` the diff · `p` live: moving
moves the text · `q` close. In the text, `g-` and `g+` walk through time too.

## Your own

A folder in `~/.config/zett/plugins/NAME/` with an `init.lua`. One named
`explorer` or `undotree` replaces the built-in one. What a plugin can use:

    zett.options(name, defaults, about)   -- settings :set lists
    zett.config(name)                     -- them, with zett.opt.NAME laid over
    zett.panel.open{ id=, side=, width=, height=, title=, on_key=, on_move= }
    zett.panel.set_lines(id, { "text", { {"text", "Group"}, … } })
    zett.buf.path() / text() / undo_states() / undo_goto(n) / text_at(n)
    zett.fs.list(dir) / mkdir / touch / rename / copy / remove / exec
    zett.input({ prompt = "Name: " }, function(answer) end)
    zett.confirm("Sure?", function(yes) end)
    zett.on("buf_enter" | "text_changed" | "write" | "option_set", f)
    zett.open(path, "right" | "below")   zett.diff(a, b)   zett.clipboard(s)

Groups for colours: Title Directory File Current Accent Muted Added Removed
Changed GitModified GitAdded GitUntracked GitDeleted Error Warn Bold.
