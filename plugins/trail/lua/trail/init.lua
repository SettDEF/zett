-- Trail: a file's changes in a side panel (saves and git commits), what changed since one of them shown in the file.
local M = {}

M.config = { side = "right", width = 36, commits = 50, saves = 100 }

local ns = vim.api.nvim_create_namespace("trail")
local state = { panel_buf = nil, panel_win = nil, file_buf = nil, path = nil, entries = {}, shown = nil }

-- ── where saves are kept: one folder per file, a snapshot per save ──
local function hash(s)
  local h = 5381
  for i = 1, #s do h = (h * 33 + s:byte(i)) % 4294967296 end
  return ("%08x"):format(h)
end

local function save_dir(path)
  return vim.fn.stdpath("state") .. "/trail/" .. hash(path)
end

local function read(path)
  local f = io.open(path, "rb")
  if not f then return nil end
  local s = f:read("*a")
  f:close()
  return s
end

local function write(path, s)
  local f = io.open(path, "wb")
  if not f then return end
  f:write(s)
  f:close()
end

local function saves(path)
  local dir = save_dir(path)
  local out = {}
  for _, n in ipairs(vim.fn.isdirectory(dir) == 1 and vim.fn.readdir(dir) or {}) do
    local t, seq = n:match("^(%d+)%-?(%d*)%.txt$")
    if t then out[#out + 1] = { time = tonumber(t), seq = tonumber(seq) or 0, file = dir .. "/" .. n } end
  end
  table.sort(out, function(a, b) if a.time ~= b.time then return a.time > b.time end return a.seq > b.seq end)
  return out
end

-- after a save: a snapshot, when the text differs from the last one (the oldest go past `saves`)
function M.snapshot(path)
  local text = read(path)
  if not text then return end
  local list = saves(path)
  if list[1] and read(list[1].file) == text then return end
  local dir = save_dir(path)
  vim.fn.mkdir(dir, "p")
  -- saves in the same second: numbered after the time
  local t, seq = os.time(), 0
  while vim.fn.filereadable(("%s/%d-%d.txt"):format(dir, t, seq)) == 1 do seq = seq + 1 end
  write(("%s/%d-%d.txt"):format(dir, t, seq), text)
  write(dir .. "/path", path)
  for i = M.config.saves + 1, #list do os.remove(list[i].file) end
end

-- ── git: the commits that touched the file, and its text at one ──
local function git(dir, args)
  local out = vim.fn.system(vim.list_extend({ "git", "-C", dir }, args))
  if vim.v.shell_error ~= 0 then return nil end
  return out
end

local function commits(path)
  local dir = vim.fn.fnamemodify(path, ":h")
  local out = git(dir, { "log", "-n", tostring(M.config.commits), "--format=%h%x09%ct%x09%s", "--", vim.fn.fnamemodify(path, ":t") })
  local list = {}
  for line in (out or ""):gmatch("[^\n]+") do
    local h, t, s = line:match("^(%S+)\t(%d+)\t(.*)$")
    if h then list[#list + 1] = { hash = h, time = tonumber(t), subject = s } end
  end
  return list
end

local function at_commit(path, h)
  return git(vim.fn.fnamemodify(path, ":h"), { "show", h .. ":./" .. vim.fn.fnamemodify(path, ":t") }) or ""
end

-- ── the timeline: unsaved edits, saves, commits; newest first ──
local function buf_text(b)
  local lines = vim.api.nvim_buf_get_lines(b, 0, -1, false)
  return table.concat(lines, "\n") .. "\n"
end

local function stat(old, new)
  local add, del = 0, 0
  for _, h in ipairs(vim.diff(old or "", new or "", { result_type = "indices" }) or {}) do
    del, add = del + h[2], add + h[4]
  end
  return add, del
end

local function ago(t)
  local d = os.time() - t
  if d < 60 then return "now" end
  if d < 3600 then return ("%dm ago"):format(math.floor(d / 60)) end
  if d < 86400 then return os.date("%H:%M", t) end
  if d < 172800 then return "yesterday" end
  return os.date("%b %d", t)
end

local function timeline(path, b)
  local list = {}
  for _, s in ipairs(saves(path)) do list[#list + 1] = { kind = "save", time = s.time, text = function() return read(s.file) end } end
  for _, c in ipairs(commits(path)) do
    list[#list + 1] = { kind = "commit", time = c.time, hash = c.hash, subject = c.subject, text = function() return at_commit(path, c.hash) end }
  end
  table.sort(list, function(a, b2) return a.time > b2.time end)
  -- each one's size: what it changed from the one before it
  local texts = {}
  local function text_of(i) if texts[i] == nil then texts[i] = list[i].text() or "" end return texts[i] end
  for i, e in ipairs(list) do
    e.add, e.del = stat(list[i + 1] and text_of(i + 1) or "", text_of(i))
    e.cached = text_of(i)
  end
  local disk = read(path) or ""
  local now = buf_text(b)
  if now ~= disk then
    local add, del = stat(disk, now)
    table.insert(list, 1, { kind = "unsaved", time = os.time(), add = add, del = del, cached = disk })
  end
  return list
end

-- ── the file: what changed since an entry (added lines coloured, removed ones shown where they were) ──
function M.clear()
  if state.file_buf and vim.api.nvim_buf_is_valid(state.file_buf) then
    vim.api.nvim_buf_clear_namespace(state.file_buf, ns, 0, -1)
  end
  state.shown = nil
end

function M.show(i)
  local e = state.entries[i]
  local b = state.file_buf
  if not e or not b or not vim.api.nvim_buf_is_valid(b) then return end
  M.clear()
  state.shown = i
  local old = e.cached or ""
  local old_lines = vim.split(old, "\n", { plain = true })
  local count = vim.api.nvim_buf_line_count(b)
  for _, h in ipairs(vim.diff(old, buf_text(b), { result_type = "indices" }) or {}) do
    local sa, ca, sb, cb = h[1], h[2], h[3], h[4]
    for l = sb, sb + cb - 1 do
      if l >= 1 and l <= count then
        vim.api.nvim_buf_set_extmark(b, ns, l - 1, 0, { line_hl_group = "DiffAdd", sign_text = "+", sign_hl_group = "DiffAdd" })
      end
    end
    if ca > 0 then
      local rows = {}
      for l = sa, sa + ca - 1 do rows[#rows + 1] = { { "- " .. (old_lines[l] or ""), "DiffDelete" } } end
      -- above the first new line of the change; after line sb when the change only removed
      if cb > 0 then
        vim.api.nvim_buf_set_extmark(b, ns, math.min(sb, count) - 1, 0, { virt_lines = rows, virt_lines_above = true })
      elseif sb == 0 then
        vim.api.nvim_buf_set_extmark(b, ns, 0, 0, { virt_lines = rows, virt_lines_above = true })
      else
        vim.api.nvim_buf_set_extmark(b, ns, math.min(sb, count) - 1, 0, { virt_lines = rows })
      end
    end
  end
  M.render()
end

-- ── the panel ──
local function entry_line(e, i)
  local mark = state.shown == i and "▶ " or "  "
  local what = e.kind == "unsaved" and "unsaved" or e.kind == "save" and "saved" or e.hash
  local head = ("%s%-9s %-7s +%d −%d"):format(mark, e.kind == "unsaved" and "now" or ago(e.time), what, e.add or 0, e.del or 0)
  if e.subject then return { head, "    " .. e.subject } end
  return { head }
end

function M.render()
  local pb = state.panel_buf
  if not pb or not vim.api.nvim_buf_is_valid(pb) then return end
  local lines = { "Trail: " .. vim.fn.fnamemodify(state.path or "", ":t"), "" }
  state.rows = {}
  for i, e in ipairs(state.entries) do
    for _, l in ipairs(entry_line(e, i)) do
      lines[#lines + 1] = l
      state.rows[#lines] = i
    end
  end
  if #state.entries == 0 then lines[#lines + 1] = "  no saves or commits yet" end
  lines[#lines + 1] = ""
  lines[#lines + 1] = "⏎ show · x clear · q close"
  vim.bo[pb].modifiable = true
  vim.api.nvim_buf_set_lines(pb, 0, -1, false, lines)
  vim.bo[pb].modifiable = false
  vim.api.nvim_buf_clear_namespace(pb, ns, 0, -1)
  vim.api.nvim_buf_set_extmark(pb, ns, 0, 0, { end_row = 0, end_col = #lines[1], hl_group = "Title" })
  for row, i in pairs(state.rows) do
    local e = state.entries[i]
    local text = lines[row]
    local hl = text:sub(1, 4) == "    " and "Comment" or (e.kind == "unsaved" and "WarningMsg" or e.kind == "commit" and "Identifier" or "Normal")
    vim.api.nvim_buf_set_extmark(pb, ns, row - 1, 0, { end_row = row - 1, end_col = #text, hl_group = hl })
  end
  vim.api.nvim_buf_set_extmark(pb, ns, #lines - 1, 0, { end_row = #lines - 1, end_col = #lines[#lines], hl_group = "Comment" })
end

function M.refresh()
  if not state.path then return end
  state.entries = timeline(state.path, state.file_buf)
  local shown = state.shown
  if shown and state.entries[shown] then M.show(shown) else M.clear() M.render() end
end

local function under_cursor()
  local row = vim.api.nvim_win_get_cursor(0)[1]
  return state.rows and state.rows[row]
end

local function setup_panel(pb)
  vim.bo[pb].buftype = "nofile"
  vim.bo[pb].bufhidden = "wipe"
  vim.bo[pb].swapfile = false
  vim.bo[pb].filetype = "trail"
  pcall(vim.api.nvim_buf_set_name, pb, "trail://" .. (state.path or ""))
  local function map(k, f, desc) vim.keymap.set("n", k, f, { buffer = pb, nowait = true, desc = desc }) end
  map("<CR>", function() local i = under_cursor() if i then M.show(i) end end, "Trail: show the change")
  map("p", function() local i = under_cursor() if i then M.show(i) end end, "Trail: show the change")
  map("x", function() M.clear() M.render() end, "Trail: clear the marks")
  map("r", M.refresh, "Trail: refresh")
  map("q", M.close, "Trail: close")
end

function M.close()
  M.clear()
  if state.panel_win and vim.api.nvim_win_is_valid(state.panel_win) then
    pcall(vim.api.nvim_win_close, state.panel_win, true)
  end
  state.panel_buf, state.panel_win, state.path = nil, nil, nil
end

function M.open()
  local b = vim.api.nvim_get_current_buf()
  local path = vim.api.nvim_buf_get_name(b)
  if path == "" then return vim.notify("Trail: this buffer has no file", vim.log.levels.WARN) end
  state.file_buf, state.path = b, vim.fn.fnamemodify(path, ":p")
  vim.cmd((M.config.side == "left" and "topleft" or "botright") .. " vnew")
  -- the split takes effect after this returns in zett (at once in Neovim): the panel is set up after it
  vim.schedule(function()
    state.panel_win = vim.api.nvim_get_current_win()
    state.panel_buf = vim.api.nvim_get_current_buf()
    vim.api.nvim_win_set_width(0, M.config.width)
    setup_panel(state.panel_buf)
    M.refresh()
  end)
end

function M.toggle()
  if state.panel_buf and vim.api.nvim_buf_is_valid(state.panel_buf) then M.close() else M.open() end
end

function M.setup(opts)
  M.config = vim.tbl_extend("force", M.config, opts or {})
end

return M
