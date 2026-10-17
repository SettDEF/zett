-- zett's built-in layer: the SPC tree, commands and their keys.
-- Written against the same API as any plugin; everything here can be
-- changed or removed from ~/.config/zett/init.lua.

local z = zett
z.opt.leader = " "

-- ── commands ──
local function cmd(name, desc, f) z.command(name, f, { desc = desc }) end

cmd("find-page", "Go to a page (or make one)", function() z.ui("quickOpen") end)
cmd("command-line", "Run a command by name", function() z.ui("commandLine") end)
cmd("lua", "Run Lua", function() z.ui("commandLine", "lua ") end)
cmd("new-page", "New page", function() z.ui("commandLine", "page ") end)
cmd("today", "Today's journal page", function()
  z.page.open("Journal " .. os.date("%Y-%m-%d"))
end)
cmd("capture", "Capture a thought into the Inbox", function() z.ui("commandLine", "capture ") end)
cmd("help-keys", "Show every key", function() z.ui("help", "keys") end)
cmd("help-commands", "Show every command", function() z.ui("help", "commands") end)
cmd("reload-config", "Reload init.lua and plugins", function() z.ui("reload") end)
cmd("open-config", "Open init.lua", function() z.ui("openConfig") end)
cmd("add-todo", "Add a to-do at the end of the page", function()
  local i = z.page.add("todo", "")
  z.ui("focusBlock", tostring(i))
end)
cmd("add-heading", "Add a heading at the end of the page", function()
  local i = z.page.add("h2", "")
  z.ui("focusBlock", tostring(i))
end)
cmd("toggle-zen", "Hide or show the sidebar", function() z.ui("toggleSidebar") end)
cmd("search", "Search every page", function() z.ui("search") end)

-- ── the SPC tree (which-key shows the group names) ──
local function map(keys, command, desc) z.map("n", keys, command, { desc = desc }) end

z.group("<leader>p", "pages")
map("<leader><leader>", "find-page", "Find page")
map("<leader>:", "command-line", "Command")
map("<leader>pp", "find-page", "Find page")
map("<leader>pn", "new-page", "New page")
map("<leader>pt", "today", "Today")
map("<leader>x", "capture", "Capture")
map("<leader>/", "search", "Search")

z.group("<leader>i", "insert")
map("<leader>it", "add-todo", "To-do")
map("<leader>ih", "add-heading", "Heading")

z.group("<leader>t", "toggle")
map("<leader>tz", "toggle-zen", "Zen (sidebar)")

z.group("<leader>h", "help")
map("<leader>hk", "help-keys", "Keys")
map("<leader>hc", "help-commands", "Commands")
map("<leader>hr", "reload-config", "Reload config")
map("<leader>hi", "open-config", "Open init.lua")

z.map("n", ":", "command-line", { desc = "Command" })
