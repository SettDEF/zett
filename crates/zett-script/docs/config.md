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

## Colours: highlight groups

Everything zett draws has a group, as in Neovim: syntax (`Comment`, `String`,
`Keyword`, `Function`, `Type`, …), the editor (`LineNr`, `CursorLineNr`, `CursorLine` and its fade `CursorLineGlow`, `CursorLineGlow2`,
`StatusLine`, `ModeNormal`, `Pmenu`, `FloatBorder`, `DiagnosticError`, …) and
plugins (`Directory`, `GitModified`, `Title`, …). `:hi` lists them all.

    zett.hl("Comment", { fg = 8, italic = true })      -- a terminal colour (0-15)
    zett.hl("Keyword", { fg = "#c678dd", bold = true }) -- or any colour
    zett.hl("GitUntracked", { link = "Changed" })        -- or another group's look
    zett.hl("CursorLine", { bg = "+20" })                -- the theme's background lifted 20%
    zett.hl("Comment")                                    -- how it is now

Try one first: `:hi Comment fg=red italic` (adds to the group; `clear` resets).
Terminal colours follow the wallpaper in zash; `#hex` colours stay put.
