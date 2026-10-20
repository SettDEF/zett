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

The built-in layer is written with the same API (defaults.lua, editor.lua,
emacs.lua): anything there can be changed or removed from init.lua.

## Keys: Emacs or Vim

Emacs' keys by default: modeless, `C-x` chords, `M-x` for every command,
`C-h` for help. Keys point at commands; rebind any of them in mode `"e"`:

    zett.map("e", "<C-x><C-b>", "switch-buffer")
    zett.opt.keys = "vim"                         -- Vim's modes and Doom's SPC tree instead

## Starting up

Without a file zett opens the scratch buffer, as Emacs does. What it does
instead is yours to choose and to program:

    zett.opt.startup = "dashboard"                -- or "scratch", or any command:
    zett.opt.startup = "recent-files"             --   "find-file", one of your own
    zett.opt.scratch_message = "notes:\n"         -- the scratch buffer's text
    zett.on("start", function()                   -- anything else, in Lua
      zett.notify("good morning")
    end)

The dashboard is yours too; leave a part out to keep zett's own:

    zett.dashboard{
      banner = { "MY ZETT" },
      buttons = {
        { icon = "", label = "Find file", keys = "C-x C-f", command = "find-file" },
        { icon = "", label = "Recent files", keys = "C-x C-r", command = "recent-files" },
      },
      footer = false,                             -- the plugins / load-time line
    }

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
