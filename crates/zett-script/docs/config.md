# Configuration

Everything is Lua: `~/.config/zett/init.lua` (`SPC f p` opens it), then
plugins in `~/.config/zett/plugins/NAME/init.lua`. `SPC h r` reloads.

## The API

    zett.opt.leader = " "                         -- options
    zett.map("n", "<leader>x", "command-name")    -- keys: a command or a function
    zett.map("n", "<leader>y", function() zett.notify("hi") end)
    zett.command("hello", function() zett.notify("hello") end, { desc = "Say hello" })
    zett.group("<leader>m", "mine")               -- a which-key group
    zett.on("start", function() end)              -- hooks (start, page events)
    zett.frontend                                 -- "editor" or "window"

The built-in layer is written with the same API (defaults.lua, editor.lua):
anything there can be changed or removed from init.lua.
