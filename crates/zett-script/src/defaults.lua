-- zett's built-in layer: the SPC tree, commands and their keys.
-- Written against the same API as any plugin; everything here can be
-- changed or removed from ~/.config/zett/init.lua.

local z = zett
z.opt.leader = " "

-- ── commands ──
local function cmd(name, desc, f) z.command(name, f, { desc = desc }) end

-- the notes window's page commands (not in the code editor)
local function pcmd(name, desc, f) if z.frontend ~= "editor" then cmd(name, desc, f) end end
pcmd("find-page", "Go to a page (or make one)", function() z.ui("quickOpen") end)
cmd("command-line", "Run a command by name", function() z.ui("commandLine") end)
cmd("lua", "Run Lua", function() z.ui("commandLine", "lua ") end)
pcmd("new-page", "New page", function() z.ui("commandLine", "page ") end)
pcmd("today", "Today's journal page", function()
  z.page.open("Journal " .. os.date("%Y-%m-%d"))
end)
pcmd("capture", "Capture a thought into the Inbox", function() z.ui("commandLine", "capture ") end)
cmd("help-keys", "Show every key", function() z.ui("help", "keys") end)
cmd("help-commands", "Show every command", function() z.ui("help", "commands") end)
cmd("reload-config", "Reload init.lua and plugins", function() z.ui("reload") end)
cmd("open-config", "Open init.lua", function() z.ui("openConfig") end)
pcmd("add-todo", "Add a to-do at the end of the page", function()
  local i = z.page.add("todo", "")
  z.ui("focusBlock", tostring(i))
end)
pcmd("add-heading", "Add a heading at the end of the page", function()
  local i = z.page.add("h2", "")
  z.ui("focusBlock", tostring(i))
end)
pcmd("toggle-zen", "Hide or show the sidebar", function() z.ui("toggleSidebar") end)
pcmd("search", "Search every page", function() z.ui("search") end)

-- ── the SPC tree (which-key shows the group names) ──
local function map(keys, command, desc) z.map("n", keys, command, { desc = desc }) end

-- the notes window's tree; the code editor (zett in a terminal) loads editor.lua instead
if z.frontend ~= "editor" then
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
end

z.group("<leader>h", "help")
map("<leader>hk", "help-keys", "Keys")
map("<leader>hc", "help-commands", "Commands")
map("<leader>hr", "reload-config", "Reload config")
map("<leader>hi", "open-config", "Open init.lua")

z.map("n", ":", "command-line", { desc = "Command" })

-- ── helpers for plugins ──

-- zett.options(name, defaults, about): a plugin's settings, their defaults and a line about
-- each; `:set NAME.KEY VALUE` lists and changes them.
z._options = {}
function z.options(name, defaults, about)
  z._options[name] = { defaults = defaults, about = about or {} }
end

-- zett.config(name): the plugin's settings, zett.opt[name] laid over the defaults
-- (tables merge, keys too); nil when the user turned it off (zett.opt[name] = false).
function z.config(name, defaults)
  defaults = defaults or (z._options[name] or {}).defaults or {}
  local user = z.opt[name]
  if user == false then return nil end
  -- key tables take any spelling (<CR>, <cr>, <Enter>)
  local function merge(a, b, keyed)
    local out = {}
    local function name(k) return keyed and type(k) == "string" and z.key_name(k) or k end
    for k, v in pairs(a) do out[name(k)] = v end
    for k, v in pairs(b or {}) do
      k = name(k)
      if type(v) == "table" and type(out[k]) == "table" and #v == 0 then out[k] = merge(out[k], v, k == "keys") else out[k] = v end
    end
    return out
  end
  return merge(defaults, type(user) == "table" and user or {})
end

-- zett.confirm(question, function(yes) end)
function z.confirm(question, f)
  z.input({ prompt = question .. " [y/N] " }, function(a) f(a:lower():sub(1, 1) == "y") end)
end
