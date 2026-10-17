-- explorer: the project's files in a side panel (as nvim-tree / neo-tree), with git marks,
-- file operations and a filter. Settings: zett.opt.explorer = { width = 36, keys = { d = false } }

local z = zett
local defaults = {
  side = "left",          -- "left" or "right"
  width = 32,
  hidden = false,         -- dotfiles
  ignore = { ".git", "target", "node_modules", "__pycache__", ".cache", "dist" },
  folders_first = true,
  icons = true,
  indent_markers = true,  -- │ guides under open folders
  git = true,             -- M A D R U marks from git status
  git_symbols = { modified = "M", added = "A", deleted = "D", renamed = "R", untracked = "U", conflict = "!" },
  follow = true,          -- show the file being edited
  quit_on_open = false,   -- close the explorer when a file opens
  trash = true,           -- delete into the trash (gio) rather than for good
  confirm_delete = true,
  focus = true,
  keys = {
    ["<Enter>"] = "open", l = "open", o = "open", ["<Right>"] = "open",
    h = "close_node", ["<Left>"] = "close_node", P = "parent",
    s = "vsplit", i = "split",
    a = "add", r = "rename", d = "delete",
    c = "copy", x = "cut", p = "paste",
    y = "copy_name", Y = "copy_path",
    H = "toggle_hidden", ["."] = "toggle_hidden",
    R = "refresh", ["/"] = "filter", F = "clear_filter",
    ["-"] = "dir_up", ["<BS>"] = "dir_up", C = "cd",
    W = "collapse_all", q = "close", ["?"] = "help",
  },
}

z.options("explorer", defaults, {
  side = "which side: left or right",
  width = "width in columns",
  hidden = "show dotfiles",
  ignore = "names never shown",
  folders_first = "folders before files",
  icons = "file icons (Nerd Font)",
  indent_markers = "│ guides under open folders",
  git = "git status marks",
  follow = "show the file being edited",
  quit_on_open = "close when a file opens",
  trash = "delete into the trash",
  confirm_delete = "ask before deleting",
  focus = "take the keyboard when opened",
})

local M = { root = nil, expanded = {}, hidden = nil, filter = nil, clip = nil, git = {}, rows = {} }

local function cfg() return z.config("explorer", defaults) end
local function name_of(p) return p:match("([^/]+)/*$") or p end
local function parent_of(p) return p:match("^(.*)/[^/]+/*$") or "/" end

local function skipped(e, c)
  local show = M.hidden
  if show == nil then show = c.hidden end
  if not show and e.name:sub(1, 1) == "." then return true end
  for _, n in ipairs(c.ignore) do if e.name == n then return true end end
  return false
end

local function entries(dir, c)
  local out = {}
  for _, e in ipairs(z.fs.list(dir)) do
    if not skipped(e, c) then out[#out + 1] = e end
  end
  table.sort(out, function(a, b)
    if c.folders_first and a.dir ~= b.dir then return a.dir end
    return a.name:lower() < b.name:lower()
  end)
  return out
end

-- git status → path → "modified" | "added" | …; folders take their files' marks
function M.read_git()
  M.git = {}
  if not cfg().git then return end
  local top, code = z.fs.exec({ "git", "rev-parse", "--show-toplevel" }, M.root)
  if code ~= 0 then return end
  top = top:gsub("%s+$", "")
  local out = z.fs.exec({ "git", "status", "--porcelain=v1", "--untracked-files=all" }, M.root)
  for line in out:gmatch("[^\n]+") do
    local xy, path = line:sub(1, 2), line:sub(4)
    path = path:match("-> (.+)$") or path
    path = path:gsub('^"(.*)"$', "%1")
    local kind = xy == "??" and "untracked" or xy:find("U") and "conflict" or xy:find("A") and "added"
      or xy:find("D") and "deleted" or xy:find("R") and "renamed" or "modified"
    local abs = top .. "/" .. path
    M.git[abs] = kind
    local d = parent_of(abs)
    while #d >= #top and not M.git[d] do M.git[d] = "modified" d = parent_of(d) end
  end
end

-- with a filter: every file under the root whose name matches, and the folders they are in
local function matches(c)
  local q, keep, n = M.filter:lower(), {}, 0
  local function walk(dir)
    for _, e in ipairs(entries(dir, c)) do
      n = n + 1
      if n > 5000 then return end
      if e.dir then walk(e.path) end
      if e.name:lower():find(q, 1, true) then
        keep[e.path] = true
        local d = parent_of(e.path)
        while #d > #M.root and not keep[d] do keep[d] = true d = parent_of(d) end
      end
    end
  end
  walk(M.root)
  return keep
end

function M.render()
  local c = cfg()
  local keep = M.filter and matches(c)
  local lines, rows = {}, {}
  local current = z.buf.path()
  local function walk(dir, depth, guides)
    local list = entries(dir, c)
    if keep then
      local f = {}
      for _, e in ipairs(list) do if keep[e.path] then f[#f + 1] = e end end
      list = f
    end
    for i, e in ipairs(list) do
      local last = i == #list
      local open = e.dir and (keep ~= nil or M.expanded[e.path])
      local pieces = {}
      if c.indent_markers then
        pieces[#pieces + 1] = { guides .. (depth > 0 and (last and "└ " or "│ ") or ""), "Muted" }
      else
        pieces[#pieces + 1] = { string.rep("  ", depth), "Muted" }
      end
      local icon = ""
      if c.icons then
        icon = (e.dir and (open and "\u{f07c}" or "\u{f07b}") or z.icon(e.name)) .. " "
      end
      local g = M.git[e.path]
      local group = e.dir and "Directory" or (e.path == current and "Current") or (g and "Git" .. g:sub(1, 1):upper() .. g:sub(2)) or "File"
      pieces[#pieces + 1] = { icon .. e.name .. (e.link and " →" or ""), group }
      if g and not e.dir then
        pieces[#pieces + 1] = { " " .. (c.git_symbols[g] or "?"), "Git" .. g:sub(1, 1):upper() .. g:sub(2) }
      elseif g then
        pieces[#pieces + 1] = { " •", "Muted" }
      end
      if M.clip and M.clip.path == e.path then pieces[#pieces + 1] = { M.clip.op == "cut" and "  (cut)" or "  (copied)", "Accent" } end
      lines[#lines + 1] = pieces
      rows[#rows + 1] = { path = e.path, name = e.name, dir = e.dir, depth = depth }
      if open then
        walk(e.path, depth + 1, guides .. (depth > 0 and (last and "  " or "│ ") or ""))
      end
    end
  end
  walk(M.root, 0, "")
  if #lines == 0 then
    lines = { { { M.filter and ("nothing matches " .. M.filter) or "(empty)", "Muted" } } }
    rows = {}
  end
  M.rows = rows
  return lines
end

function M.title()
  local t = "\u{f07c} " .. name_of(M.root)
  if M.filter then t = t .. "  /" .. M.filter end
  return t
end

function M.refresh(select_path)
  if not z.panel.is_open("explorer") then return end
  local c = cfg()
  z.panel.open({ id = "explorer", side = c.side, width = c.width, title = M.title(), focus = false })
  z.panel.set_lines("explorer", M.render())
  if select_path then
    for r, row in ipairs(M.rows) do if row.path == select_path then z.panel.set_cursor("explorer", r) end end
  end
end

function M.node()
  local r = z.panel.cursor("explorer")
  return r and M.rows[r]
end

-- the folder an action works in: the folder under the cursor, else the one the file is in
local function target_dir()
  local n = M.node()
  if not n then return M.root end
  return n.dir and n.path or parent_of(n.path)
end

-- opens the folders down to `path` and selects it
function M.reveal(path)
  if not path or path == "" or path:sub(1, #M.root + 1) ~= M.root .. "/" then return end
  local d = parent_of(path)
  while #d > #M.root do M.expanded[d] = true d = parent_of(d) end
  M.refresh(path)
end

local function open_with(how)
  local n = M.node()
  if not n then return end
  if n.dir then
    if how then return end
    M.expanded[n.path] = not M.expanded[n.path] or nil
    M.refresh(n.path)
    return
  end
  z.open(n.path, how)
  if cfg().quit_on_open then M.close() else z.panel.focus(nil) end
end

local function done(ok, err, what)
  if not ok then z.notify(what .. ": " .. (err or "failed")) end
  M.read_git()
  return ok
end

M.actions = {
  open = function() open_with(nil) end,
  vsplit = function() open_with("right") end,
  split = function() open_with("below") end,
  close_node = function()
    local n = M.node()
    if not n then return end
    if n.dir and M.expanded[n.path] then
      M.expanded[n.path] = nil
      M.refresh(n.path)
    else
      M.actions.parent()
    end
  end,
  parent = function()
    local n = M.node()
    if n and n.depth > 0 then M.refresh(parent_of(n.path)) end
  end,
  add = function()
    local dir = target_dir()
    z.input({ prompt = "New in " .. name_of(dir) .. "/ (end with / for a folder): " }, function(a)
      if a == "" then return end
      local p = dir .. "/" .. a
      local ok, err
      if a:sub(-1) == "/" then ok, err = z.fs.mkdir(p) p = p:gsub("/+$", "") else ok, err = z.fs.touch(p) end
      if done(ok, err, "add") then M.expanded[dir] = true M.reveal(p) end
    end)
  end,
  rename = function()
    local n = M.node()
    if not n then return end
    z.input({ prompt = "Rename to: ", default = n.name }, function(a)
      if a == "" or a == n.name then return end
      local p = a:sub(1, 1) == "/" and a or (parent_of(n.path) .. "/" .. a)
      if done(z.fs.rename(n.path, p)) then M.reveal(p) end
    end)
  end,
  delete = function()
    local n = M.node()
    if not n then return end
    local c = cfg()
    local function go()
      if done(z.fs.remove(n.path, c.trash), nil, "delete") then
        z.notify((c.trash and "to the trash: " or "deleted: ") .. n.name)
        M.refresh()
      end
    end
    if c.confirm_delete then
      z.confirm((c.trash and "Trash " or "Delete ") .. n.name .. (n.dir and "/ and all in it?" or "?"), function(yes) if yes then go() end end)
    else
      go()
    end
  end,
  copy = function() local n = M.node() if n then M.clip = { op = "copy", path = n.path } M.refresh() end end,
  cut = function() local n = M.node() if n then M.clip = { op = "cut", path = n.path } M.refresh() end end,
  paste = function()
    if not M.clip then z.notify("nothing copied (c copies, x cuts)") return end
    local dir, clip = target_dir(), M.clip
    local function put(to)
      local ok, err
      if clip.op == "cut" then ok, err = z.fs.rename(clip.path, to) else ok, err = z.fs.copy(clip.path, to) end
      if done(ok, err, clip.op == "cut" and "move" or "copy") then
        if clip.op == "cut" then M.clip = nil end
        M.expanded[dir] = true
        M.reveal(to)
      end
    end
    local to = dir .. "/" .. name_of(clip.path)
    if z.fs.exists(to) then
      z.input({ prompt = name_of(to) .. " exists; name: ", default = name_of(to) .. "_copy" }, function(a)
        if a ~= "" then put(dir .. "/" .. a) end
      end)
    else
      put(to)
    end
  end,
  copy_name = function() local n = M.node() if n then z.clipboard(n.name) z.notify("copied " .. n.name) end end,
  copy_path = function() local n = M.node() if n then z.clipboard(n.path) z.notify("copied " .. n.path) end end,
  toggle_hidden = function()
    local show = M.hidden
    if show == nil then show = cfg().hidden end
    M.hidden = not show
    M.refresh()
  end,
  refresh = function() M.read_git() M.refresh() end,
  filter = function()
    z.input({ prompt = "Filter: ", default = M.filter or "" }, function(a)
      M.filter = a ~= "" and a or nil
      z.panel.set_cursor("explorer", 1)
      M.refresh()
    end)
  end,
  clear_filter = function() M.filter = nil M.refresh() end,
  dir_up = function()
    local old = M.root
    M.root = parent_of(M.root)
    M.expanded[old] = true
    M.read_git()
    M.refresh(old)
  end,
  cd = function()
    local n = M.node()
    if n and n.dir then M.root = n.path M.read_git() z.panel.set_cursor("explorer", 1) M.refresh() end
  end,
  collapse_all = function() M.expanded = {} M.refresh() end,
  close = function() M.close() end,
  help = function()
    local ks = {}
    for k, a in pairs(cfg().keys) do if type(a) == "string" then ks[#ks + 1] = k .. " " .. a end end
    table.sort(ks)
    z.notify("explorer: " .. table.concat(ks, " · "))
  end,
}

function M.close() z.panel.close("explorer") end

function M.open(reveal)
  local c = cfg()
  if not c then z.notify("the explorer is turned off (zett.opt.explorer = false)") return end
  local root = z.root()
  if M.root ~= root and not (M.root and root:sub(1, #M.root) == M.root) then
    M.root, M.expanded, M.filter = root, {}, nil
  end
  z.panel.open({
    id = "explorer", side = c.side, width = c.width, title = M.title(), focus = c.focus,
    on_key = function(key)
      local a = cfg().keys[key]
      if type(a) == "function" then
        local n = M.node()
        a(n and { path = n.path, name = n.name, dir = n.dir, depth = n.depth, root = M.root } or { root = M.root })
        M.refresh()
        return true
      end
      if type(a) == "string" and M.actions[a] then M.actions[a]() return true end
      return false
    end,
  })
  M.read_git()
  M.refresh()
  if reveal or c.follow then M.reveal(z.buf.path()) end
end

z.command("explorer", function()
  if z.panel.is_open("explorer") and z.panel.focused() == "explorer" then M.close()
  elseif z.panel.is_open("explorer") then z.panel.focus("explorer")
  else M.open(false) end
end, { desc = "File explorer" })
z.command("explorer-reveal", function()
  if not z.panel.is_open("explorer") then M.open(true) else M.reveal(z.buf.path()) z.panel.focus("explorer") end
end, { desc = "This file in the explorer" })
z.map("n", "<leader>op", "explorer", { desc = "Project sidebar" })
z.map("n", "<leader>oP", "explorer-reveal", { desc = "Find file in sidebar" })
z.map("n", "<leader>e", "explorer", { desc = "Explorer" })

z.on("buf_enter", function(e)
  local c = cfg()
  if c and c.follow and z.panel.is_open("explorer") and z.panel.focused() ~= "explorer" then M.reveal(e.path) end
end)
z.on("write", function() if z.panel.is_open("explorer") then M.read_git() M.refresh() end end)

-- for init.lua: require("explorer") gives this table (actions can be added or replaced)
package.loaded["explorer"] = M
z.on("option_set", function() if z.panel.is_open("explorer") then M.refresh() end end)
