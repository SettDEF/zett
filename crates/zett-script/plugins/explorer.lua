-- explorer: the project's files in a side panel, as neo-tree shows them: the files, git status
-- or the open buffers; neo-tree's keys and action names. Settings: zett.opt.explorer = { … }

local z = zett
local defaults = {
  side = "left",            -- "left" or "right"
  width = 30,
  hidden = false,           -- dotfiles (neo-tree: hide_dotfiles = not hidden)
  hide_gitignored = false,  -- what .gitignore names
  ignore = { ".git", "node_modules", ".DS_Store" }, -- names never shown (hide_by_name)
  folders_first = true,
  follow = true,            -- show the file being edited (follow_current_file)
  quit_on_open = false,     -- close when a file opens
  trash = true,             -- delete into the trash (gio) rather than for good
  confirm_delete = true,
  focus = true,             -- take the keyboard when opened
  title = true,             -- the root folder's name on top
  icons = true,
  indent = { marker = "│", last_marker = "└", expanders = true, expander_collapsed = "\u{f460}", expander_expanded = "\u{f47c}" },
  icon = { folder_closed = "\u{f07b}", folder_open = "\u{f07c}", folder_empty = "\u{f07b}", folder_empty_open = "\u{f07c}" },
  git = true,               -- git status marks
  git_align = "right",      -- the mark at the panel's edge ("right") or after the name
  git_symbols = { added = "A", modified = "M", deleted = "D", renamed = "R", untracked = "U", conflict = "!" },
  -- neo-tree's default keys and action names; "none" or false turns a key off
  mappings = {
    ["<Enter>"] = "open", o = "open", s = "open_vsplit", S = "open_split", t = "open",
    a = "add", A = "add_directory", d = "delete", r = "rename",
    y = "copy_to_clipboard", x = "cut_to_clipboard", p = "paste_from_clipboard",
    c = "copy", m = "move",
    C = "close_node", z = "close_all_nodes", h = "close_node", l = "open",
    ["."] = "set_root", ["<BS>"] = "navigate_up",
    H = "toggle_hidden", ["/"] = "fuzzy_finder", f = "filter_on_submit", ["<C-x>"] = "clear_filter",
    ["[g"] = "prev_git_modified", ["]g"] = "next_git_modified",
    ["<"] = "prev_source", [">"] = "next_source",
    i = "show_file_details", Y = "copy_path",
    R = "refresh", q = "close_window", ["?"] = "show_help", ["<Esc>"] = "cancel",
  },
}

z.options("explorer", defaults, {
  side = "which side: left or right",
  width = "width in columns",
  hidden = "show dotfiles",
  hide_gitignored = "hide what .gitignore names",
  ignore = "names never shown",
  folders_first = "folders before files",
  follow = "show the file being edited",
  quit_on_open = "close when a file opens",
  trash = "delete into the trash",
  confirm_delete = "ask before deleting",
  focus = "take the keyboard when opened",
  title = "the root folder's name on top",
  icons = "file icons (Nerd Font)",
  indent = "indent markers and expanders",
  icon = "folder icons",
  git = "git status marks",
  git_align = "git marks: right or after the name",
  git_symbols = "the git marks",
})

local SOURCES = { "filesystem", "git_status", "buffers" }
local M = { root = nil, source = "filesystem", expanded = {}, hidden = nil, filter = nil, clip = nil, git = {}, ignored = {}, rows = {}, pending = nil }

local function cfg() return z.config("explorer", defaults) end
local function name_of(p) return p:match("([^/]+)/*$") or p end
local function parent_of(p) return p:match("^(.*)/[^/]+/*$") or "/" end
local function rel(p) return p:sub(1, #M.root + 1) == M.root .. "/" and p:sub(#M.root + 2) or p end

local function skipped(e, c)
  local show = M.hidden
  if show == nil then show = c.hidden end
  if not show and e.name:sub(1, 1) == "." then return true end
  for _, n in ipairs(c.ignore) do if e.name == n then return true end end
  if c.hide_gitignored and M.ignored[e.path] then return true end
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

-- git status → path → "modified" | "added" | …; folders take their files' marks; what is ignored
function M.read_git()
  M.git, M.ignored = {}, {}
  local c = cfg()
  if not (c.git or c.hide_gitignored) then return end
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
  if c.hide_gitignored then
    local ign = z.fs.exec({ "git", "ls-files", "--others", "--ignored", "--exclude-standard", "--directory" }, top)
    for p in ign:gmatch("[^\n]+") do M.ignored[(top .. "/" .. p):gsub("/$", "")] = true end
  end
end

-- the paths to show and the folders they are in (a filter, the git status view, the buffers view)
local function kept(c)
  local keep = {}
  local function add(p)
    keep[p] = true
    local d = parent_of(p)
    while #d > #M.root and not keep[d] do keep[d] = true d = parent_of(d) end
  end
  if M.source == "git_status" then
    for p in pairs(M.git) do if p:sub(1, #M.root + 1) == M.root .. "/" and not z.fs.is_dir(p) then add(p) end end
    return keep
  elseif M.source == "buffers" then
    for _, p in ipairs(z.buffers()) do if p:sub(1, #M.root + 1) == M.root .. "/" then add(p) end end
    return keep
  elseif not M.filter then
    return nil
  end
  local q, n = M.filter:lower(), 0
  local function walk(dir)
    for _, e in ipairs(entries(dir, c)) do
      n = n + 1
      if n > 5000 then return end
      if e.dir then walk(e.path) end
      if e.name:lower():find(q, 1, true) then add(e.path) end
    end
  end
  walk(M.root)
  return keep
end

local function width(s) return utf8.len(s) or #s end

function M.render()
  local c = cfg()
  local keep = kept(c)
  local lines, rows = {}, {}
  local current = z.buf.path()
  local ind = c.indent
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
      local lead = guides .. (depth > 0 and ((last and ind.last_marker or ind.marker) .. " ") or "")
      local pieces = { { lead, "ExplorerIndent" } }
      if ind.expanders then
        pieces[#pieces + 1] = { e.dir and ((open and ind.expander_expanded or ind.expander_collapsed) .. " ") or "  ", "ExplorerExpander" }
      end
      local icon = ""
      if c.icons then
        if e.dir then
          local empty = #z.fs.list(e.path) == 0
          icon = (empty and (open and c.icon.folder_empty_open or c.icon.folder_empty) or (open and c.icon.folder_open or c.icon.folder_closed)) .. " "
        else
          icon = z.icon(e.name) .. " "
        end
      end
      local g = c.git and M.git[e.path]
      local gg = g and ("Git" .. g:sub(1, 1):upper() .. g:sub(2))
      local group = e.dir and "Directory" or (e.path == current and "Current") or gg or "File"
      pieces[#pieces + 1] = { icon .. e.name .. (e.link and " →" or ""), group }
      if M.clip and M.clip.path == e.path then pieces[#pieces + 1] = { M.clip.op == "cut" and " (cut)" or " (copied)", "Accent" } end
      if g and not e.dir then
        local mark = c.git_symbols[g] or "?"
        if c.git_align == "right" then
          local used = 1
          for _, p in ipairs(pieces) do used = used + width(p[1]) end
          pieces[#pieces + 1] = { string.rep(" ", math.max(1, c.width - used - width(mark) - 1)) .. mark, gg }
        else
          pieces[#pieces + 1] = { " " .. mark, gg }
        end
      end
      lines[#lines + 1] = pieces
      rows[#rows + 1] = { path = e.path, name = e.name, dir = e.dir, depth = depth, git = g }
      if open then
        walk(e.path, depth + 1, guides .. (depth > 0 and (last and "  " or (ind.marker .. " ")) or ""))
      end
    end
  end
  walk(M.root, 0, "")
  if #lines == 0 then
    local why = M.source == "git_status" and "nothing changed" or M.source == "buffers" and "no buffers here"
      or M.filter and ("nothing matches " .. M.filter) or "(empty)"
    lines = { { { why, "Muted" } } }
    rows = {}
  end
  M.rows = rows
  return lines
end

function M.title()
  if not cfg().title then return "" end
  local label = { filesystem = "\u{f07c} ", git_status = "\u{e702} ", buffers = "\u{f0219} " }
  local t = label[M.source] .. name_of(M.root)
  if M.source ~= "filesystem" then t = t .. "  " .. M.source:gsub("_", " ") end
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

local function make(dir, a, folder)
  if a == "" then return end
  local p = dir .. "/" .. a
  local ok, err
  if folder or a:sub(-1) == "/" then ok, err = z.fs.mkdir(p) p = p:gsub("/+$", "") else ok, err = z.fs.touch(p) end
  if done(ok, err, "add") then M.expanded[dir] = true M.reveal(p) end
end

-- copy or move to a path typed relative to the root
local function to_path(n, verb, f)
  z.input({ prompt = verb .. " to: ", default = rel(n.path) }, function(a)
    if a == "" or a == rel(n.path) then return end
    local to = a:sub(1, 1) == "/" and a or (M.root .. "/" .. a)
    if done(f(n.path, to)) then M.reveal(to) end
  end)
end

local function jump_git(step)
  local r = z.panel.cursor("explorer") or 1
  local i = r + step
  while M.rows[i] do
    if M.rows[i].git and not M.rows[i].dir then z.panel.set_cursor("explorer", i) return end
    i = i + step
  end
end

local function set_source(s)
  M.source = s
  M.filter = nil
  if s ~= "filesystem" then M.read_git() end
  z.panel.set_cursor("explorer", 1)
  M.refresh()
end

M.actions = {
  open = function() open_with(nil) end,
  open_vsplit = function() open_with("right") end,
  open_split = function() open_with("below") end,
  toggle_node = function() local n = M.node() if n and n.dir then open_with(nil) end end,
  close_node = function()
    local n = M.node()
    if not n then return end
    if n.dir and M.expanded[n.path] then
      M.expanded[n.path] = nil
      M.refresh(n.path)
    elseif n.depth > 0 then
      local up = parent_of(n.path)
      M.expanded[up] = nil
      M.refresh(up)
    end
  end,
  close_all_nodes = function() M.expanded = {} M.refresh() end,
  add = function()
    local dir = target_dir()
    z.input({ prompt = "New in " .. name_of(dir) .. "/ (end with / for a folder): " }, function(a) make(dir, a, false) end)
  end,
  add_directory = function()
    local dir = target_dir()
    z.input({ prompt = "New folder in " .. name_of(dir) .. "/: " }, function(a) make(dir, a, true) end)
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
  move = function() local n = M.node() if n then to_path(n, "Move", z.fs.rename) end end,
  copy = function() local n = M.node() if n then to_path(n, "Copy", z.fs.copy) end end,
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
  copy_to_clipboard = function() local n = M.node() if n then M.clip = { op = "copy", path = n.path } M.refresh() end end,
  cut_to_clipboard = function() local n = M.node() if n then M.clip = { op = "cut", path = n.path } M.refresh() end end,
  paste_from_clipboard = function()
    if not M.clip then z.notify("nothing copied (y copies, x cuts)") return end
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
  show_file_details = function()
    local n = M.node()
    local s = n and z.fs.stat(n.path)
    if not s then return end
    local size = s.size < 1024 and (s.size .. " B") or s.size < 1048576 and string.format("%.1f KB", s.size / 1024) or string.format("%.1f MB", s.size / 1048576)
    z.notify(rel(n.path) .. " · " .. (s.dir and "folder" or size) .. " · changed " .. os.date("%Y-%m-%d %H:%M", s.mtime) .. (n.git and (" · " .. n.git) or ""))
  end,
  toggle_hidden = function()
    local show = M.hidden
    if show == nil then show = cfg().hidden end
    M.hidden = not show
    M.refresh()
  end,
  refresh = function() M.read_git() M.refresh() end,
  fuzzy_finder = function()
    z.input({ prompt = "Filter: ", default = M.filter or "" }, function(a)
      M.filter = a ~= "" and a or nil
      z.panel.set_cursor("explorer", 1)
      M.refresh()
    end)
  end,
  clear_filter = function() M.filter = nil M.refresh() end,
  navigate_up = function()
    local old = M.root
    M.root = parent_of(M.root)
    M.expanded[old] = true
    M.read_git()
    M.refresh(old)
  end,
  set_root = function()
    local n = M.node()
    if n and n.dir then M.root = n.path M.read_git() z.panel.set_cursor("explorer", 1) M.refresh() end
  end,
  prev_git_modified = function() jump_git(-1) end,
  next_git_modified = function() jump_git(1) end,
  prev_source = function() for i, s in ipairs(SOURCES) do if s == M.source then set_source(SOURCES[(i - 2) % #SOURCES + 1]) return end end end,
  next_source = function() for i, s in ipairs(SOURCES) do if s == M.source then set_source(SOURCES[i % #SOURCES + 1]) return end end end,
  close_window = function() M.close() end,
  cancel = function() if M.filter then M.filter = nil M.refresh() else z.panel.focus(nil) end end,
  show_help = function()
    local ks = {}
    for k, a in pairs(cfg().mappings) do if type(a) == "string" and a ~= "none" then ks[#ks + 1] = k .. " " .. a end end
    table.sort(ks)
    z.notify("explorer: " .. table.concat(ks, " · "))
  end,
}
-- the names earlier versions used
M.actions.vsplit, M.actions.split = M.actions.open_vsplit, M.actions.open_split
M.actions.cut, M.actions.paste = M.actions.cut_to_clipboard, M.actions.paste_from_clipboard
M.actions.filter, M.actions.dir_up, M.actions.cd = M.actions.fuzzy_finder, M.actions.navigate_up, M.actions.set_root
M.actions.collapse_all, M.actions.close, M.actions.help = M.actions.close_all_nodes, M.actions.close_window, M.actions.show_help
M.actions.filter_on_submit, M.actions.parent = M.actions.fuzzy_finder, M.actions.close_node

function M.close() z.panel.close("explorer") end

-- opens on a source; reveal: the file being edited; cwd: the root is the project again
function M.open(o)
  o = o or {}
  local c = cfg()
  if not c then z.notify("the explorer is turned off (zett.opt.explorer = false)") return end
  local root = z.root()
  if o.cwd or (M.root ~= root and not (M.root and root:sub(1, #M.root) == M.root)) then
    M.root, M.expanded, M.filter = root, {}, nil
  end
  M.source = o.source or M.source
  z.panel.open({
    id = "explorer", side = c.side, width = c.width, title = M.title(), focus = c.focus,
    on_key = function(key)
      local maps = cfg().mappings
      -- two-key mappings ([g ]g): the first key waits for the second
      if M.pending then
        key, M.pending = M.pending .. key, nil
      elseif key == "[" or key == "]" then
        M.pending = key
        return true
      end
      local a = maps[key]
      if a == nil and cfg().keys then a = cfg().keys[key] end
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
  if o.reveal or c.follow then M.reveal(z.buf.path()) end
end

-- open, or close when open (neo-tree's toggle); on another source, switch to it
local function toggle(o)
  if z.panel.is_open("explorer") and (not o.source or o.source == M.source) then M.close()
  elseif z.panel.is_open("explorer") then set_source(o.source) z.panel.focus("explorer")
  else M.open(o) end
end

z.command("explorer", function() toggle({ source = "filesystem" }) end, { desc = "File explorer" })
z.command("explorer-reveal", function()
  if z.panel.is_open("explorer") and M.source == "filesystem" then M.close() else M.open({ source = "filesystem", reveal = true, cwd = true }) end
end, { desc = "File explorer at this file" })
z.command("explorer-git", function() toggle({ source = "git_status" }) end, { desc = "Changed files (git status)" })
z.command("explorer-buffers", function() toggle({ source = "buffers" }) end, { desc = "Open buffers" })
z.command("explorer-focus", function()
  if z.panel.is_open("explorer") then z.panel.focus("explorer") else M.open({}) end
end, { desc = "Focus the explorer" })
z.map("n", "<leader>op", "explorer", { desc = "Project sidebar" })
z.map("n", "<leader>oP", "explorer-reveal", { desc = "Find file in sidebar" })
z.map("n", "<leader>e", "explorer", { desc = "Explorer" })

z.on("buf_enter", function(e)
  local c = cfg()
  if c and c.follow and z.panel.is_open("explorer") and z.panel.focused() ~= "explorer" then
    if M.source == "buffers" then M.refresh(e.path) else M.reveal(e.path) end
  end
end)
z.on("write", function() if z.panel.is_open("explorer") then M.read_git() M.refresh() end end)
z.on("option_set", function() if z.panel.is_open("explorer") then M.refresh() end end)

-- for init.lua: require("explorer") gives this table (actions can be added or replaced)
package.loaded["explorer"] = M
