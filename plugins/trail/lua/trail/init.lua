-- Trail: a file's changes in a side panel (saves and git commits), what changed since one of them shown in the file.
local M = {}

-- style: "boxes" (changes and entries in rounded boxes) or "plain"
M.config = { side = "right", width = 36, commits = 50, saves = 100, style = "boxes" }

local ns = vim.api.nvim_create_namespace("trail")
local state = { panel_buf = nil, panel_win = nil, file_buf = nil, path = nil, entries = {}, shown = nil, rows = {}, hunks = {}, hunk_rows = {}, removed = {}, removed_rows = {}, removed_filter = nil }

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

-- the function or heading a line names, or nil
local name_pats = { "^%s*local%s+function%s+([%w_.:]+)", "^%s*function%s+([%w_.:]+)", "^%s*fn%s+([%w_]+)", "^%s*def%s+([%w_]+)", "^#+%s+(.+)" }
local function pat_name(line)
  for _, p in ipairs(name_pats) do
    local n = line:match(p)
    if n then return n end
  end
end

-- the function or heading a hunk sits under: the closest line at or above `line` that names one
local function near(lines, line)
  for l = math.min(line, #lines), 1, -1 do
    local n = pat_name(lines[l])
    if n then return n end
  end
  return "top"
end

local function ago(t)
  local d = os.time() - t
  if d < 60 then return "now" end
  if d < 3600 then return ("%dm ago"):format(math.floor(d / 60)) end
  if d < 86400 then return os.date("%H:%M", t) end
  if d < 172800 then return "yesterday" end
  return os.date("%b %d", t)
end

-- ── removed code: blocks an older version had that a newer one (or the buffer) lost ──
-- a block's name: its first function or heading line, else its first line
local function block_name(lines)
  for _, l in ipairs(lines) do
    local n = pat_name(l)
    if n then return n end
  end
  for _, l in ipairs(lines) do
    local t = vim.trim(l)
    if t ~= "" then return vim.fn.strcharpart(t, 0, 30) end
  end
  return "(blank)"
end

-- blocks removed going from each entry to the next newer one, newest first, each text once
-- (the newer entry is where it went; its time is the removal's)
local function removed_blocks(entries, cur)
  local out, seen = {}, {}
  for i = 2, #entries do
    local old, newer = entries[i].cached or "", entries[i - 1].cached or ""
    local old_lines = vim.split(old, "\n", { plain = true })
    for _, h in ipairs(vim.diff(old, newer, { result_type = "indices" }) or {}) do
      local sa, ca = h[1], h[2]
      if ca > 0 then
        local lines = vim.list_slice(old_lines, sa, sa + ca - 1)
        while #lines > 0 and vim.trim(lines[#lines]) == "" do table.remove(lines) end
        while #lines > 0 and vim.trim(lines[1]) == "" do table.remove(lines, 1) end
        local text = table.concat(lines, "\n")
        if text:find("%S") and not seen[text] and not cur:find(text, 1, true) then
          seen[text] = true
          out[#out + 1] = { text = text, name = block_name(lines), time = entries[i - 1].time }
        end
      end
    end
  end
  return out
end

-- the search: case-insensitive unless the word has a capital
local function contains(text, q)
  if q:find("%u") then return text:find(q, 1, true) ~= nil end
  return text:lower():find(q:lower(), 1, true) ~= nil
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

-- text cut or padded to w cells (tabs as two spaces)
local function fit(s, w)
  s = s:gsub("\t", "  ")
  if vim.fn.strdisplaywidth(s) > w then
    local n = w - 1
    while n > 0 and vim.fn.strdisplaywidth(vim.fn.strcharpart(s, 0, n)) > w - 1 do n = n - 1 end
    s = vim.fn.strcharpart(s, 0, n) .. "…"
  end
  return s .. (" "):rep(math.max(0, w - vim.fn.strdisplaywidth(s)))
end

-- ── the file: what changed since an entry (added lines coloured, removed ones shown where they were) ──
function M.clear()
  if state.file_buf and vim.api.nvim_buf_is_valid(state.file_buf) then
    vim.api.nvim_buf_clear_namespace(state.file_buf, ns, 0, -1)
  end
  state.shown = nil
  state.hunks = {}
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
  local boxes = M.config.style == "boxes"
  local border = "FloatBorder"
  local new_lines = vim.api.nvim_buf_get_lines(b, 0, -1, false)
  -- a box's inner width: its longest line, within the file's window
  local win_w = state.file_win and vim.api.nvim_win_is_valid(state.file_win) and vim.api.nvim_win_get_width(state.file_win) or 80
  local function inner(sa, ca, sb, cb)
    local w = 20
    for l = sa, sa + ca - 1 do w = math.max(w, vim.fn.strdisplaywidth(old_lines[l] or "") + 2) end
    for l = sb, sb + cb - 1 do w = math.max(w, vim.fn.strdisplaywidth(new_lines[l] or "")) end
    return math.min(w, math.max(20, win_w - 14))
  end
  local function put(row, rows, above)
    vim.api.nvim_buf_set_extmark(b, ns, math.max(0, math.min(row, count) - 1), 0, { virt_lines = rows, virt_lines_above = above })
  end
  for _, h in ipairs(vim.diff(old, buf_text(b), { result_type = "indices" }) or {}) do
    local sa, ca, sb, cb = h[1], h[2], h[3], h[4]
    state.hunks[#state.hunks + 1] = { h = { old_start = sa, old_count = ca, new_start = sb, new_count = cb }, name = near(new_lines, math.max(sb, 1)) }
    local W = inner(sa, ca, sb, cb)
    for l = sb, sb + cb - 1 do
      if l >= 1 and l <= count then
        vim.api.nvim_buf_set_extmark(b, ns, l - 1, 0, { line_hl_group = "DiffAdd", sign_text = boxes and "│" or "+", sign_hl_group = boxes and border or "DiffAdd" })
      end
    end
    local rows = {}
    if boxes then
      local label = (" −%d +%d "):format(ca, cb)
      rows[1] = { { "╭─", border }, { label, "Title" }, { ("─"):rep(math.max(0, W + 1 - vim.fn.strdisplaywidth(label))) .. "╮", border } }
    end
    for l = sa, sa + ca - 1 do
      -- the code in the file's colours over the removed background ("@syntax": zett colours it as the buffer's language)
      local code = (old_lines[l] or ""):gsub("\t", "  ")
      local hl = { "DiffDelete", "@syntax" }
      rows[#rows + 1] = boxes and { { "│ ", border }, { "- ", "DiffDelete" }, { fit(code, W - 2), hl }, { " │", border } } or { { "- ", "DiffDelete" }, { code, hl } }
    end
    local bottom = { { "╰" .. ("─"):rep(W + 2) .. "╯", border } }
    if cb > 0 then
      -- the removed lines above the new ones; in boxes, the box closes under the last new line
      if boxes and ca > 0 then rows[#rows + 1] = { { "├" .. ("┄"):rep(W + 2) .. "┤", border } } end
      if #rows > 0 then put(sb, rows, true) end
      if boxes then put(sb + cb - 1, { bottom }, false) end
    elseif #rows > 0 then
      if boxes then rows[#rows + 1] = bottom end
      if sb == 0 then put(1, rows, true) else put(sb, rows, false) end
    end
  end
  M.render()
end

-- restore one hunk: its current lines become the entry's old lines (only that hunk changes)
function M.restore(i, h)
  local e, b = state.entries[i], state.file_buf
  if not e or not b or not vim.api.nvim_buf_is_valid(b) then return end
  -- the hunk must still be there: the file may have changed since the entry was shown
  local found = false
  for _, x in ipairs(vim.diff(e.cached or "", buf_text(b), { result_type = "indices" }) or {}) do
    if x[1] == h.old_start and x[2] == h.old_count and x[3] == h.new_start and x[4] == h.new_count then found = true end
  end
  if not found then return vim.notify("Trail: that change is gone, press Enter on the entry again", vim.log.levels.WARN) end
  local old_lines = vim.split(e.cached or "", "\n", { plain = true })
  local repl = {}
  for l = h.old_start, h.old_start + h.old_count - 1 do repl[#repl + 1] = old_lines[l] end
  -- a pure deletion (new_count 0) has its start as the line it goes after
  local first = h.new_count == 0 and h.new_start or h.new_start - 1
  vim.api.nvim_buf_set_lines(b, first, first + h.new_count, false, repl)
  M.show(i)
end

-- ── the panel: an entry a line or two (plain), or a rounded card (boxes) ──
local function entry_text(e)
  local what = e.kind == "unsaved" and "unsaved" or e.kind == "save" and "saved" or e.hash
  return ("%-9s %-7s +%d −%d"):format(e.kind == "unsaved" and "now" or ago(e.time), what, e.add or 0, e.del or 0), e.subject
end

local function kind_hl(e) return e.kind == "unsaved" and "WarningMsg" or e.kind == "commit" and "Identifier" or "Normal" end

function M.render()
  local pb = state.panel_buf
  if not pb or not vim.api.nvim_buf_is_valid(pb) then return end
  local boxes = M.config.style == "boxes"
  local lines = { "Trail: " .. vim.fn.fnamemodify(state.path or "", ":t"), "" }
  local hls = {} -- row → { group, from byte, to byte } pieces
  local function add(text, row_hls, entry, hunk, rem)
    lines[#lines + 1] = text
    hls[#lines] = row_hls
    if entry then state.rows[#lines] = entry end
    if hunk then state.hunk_rows[#lines] = hunk end
    if rem then state.removed_rows[#lines] = rem end
  end
  state.rows = {}
  state.hunk_rows = {}
  state.removed_rows = {}
  local cw = M.config.width - 6
  for i, e in ipairs(state.entries) do
    local head, sub = entry_text(e)
    if boxes then
      local bh = state.shown == i and "Special" or "FloatBorder"
      local function body(t, g)
        local s = "│ " .. fit(t, cw) .. " │"
        add(s, { { bh, 0, #s }, { g, 4, #s - 4 } }, i)
      end
      local top = "╭" .. ("─"):rep(cw + 2) .. "╮"
      add(top, { { bh, 0, #top } }, i)
      body(head, kind_hl(e))
      if sub then body(sub, "Comment") end
      local bot = "╰" .. ("─"):rep(cw + 2) .. "╯"
      add(bot, { { bh, 0, #bot } }, i)
    else
      local s = (state.shown == i and "▶ " or "  ") .. head
      add(s, { { kind_hl(e), 0, #s } }, i)
      if sub then add("    " .. sub, { { "Comment", 0, #sub + 4 } }, i) end
    end
    -- the shown entry's hunks, one row each: where it starts, what it sits under, its size
    if state.shown == i then
      for _, hk in ipairs(state.hunks) do
        local h = hk.h
        local s = ("    L%d %s −%d +%d"):format(math.max(1, h.new_start), hk.name, h.old_count, h.new_count)
        add(s, { { "Normal", 0, #s } }, nil, { entry = i, hunk = h })
      end
    end
  end
  if #state.entries == 0 then lines[#lines + 1] = "  no saves or commits yet" end
  -- the removed code: newest first, the search filters it, 15 rows
  lines[#lines + 1] = ""
  local head = "Removed" .. (state.removed_filter and (" (" .. state.removed_filter .. ")") or "")
  add(head, { { "Title", 0, #head } })
  local shown_rem = {}
  for _, r in ipairs(state.removed) do
    if not state.removed_filter or contains(r.text, state.removed_filter) then shown_rem[#shown_rem + 1] = r end
  end
  for i = 1, math.min(15, #shown_rem) do
    local r = shown_rem[i]
    local when = ago(r.time)
    local name = fit(r.name, M.config.width - 4 - vim.fn.strdisplaywidth(when))
    local s = "  " .. name .. "  " .. when
    add(s, { { "DiffDelete", 2, 2 + #name }, { "Comment", #s - #when, #s } }, nil, nil, r)
  end
  if #shown_rem == 0 then
    local none = state.removed_filter and "  no match" or "  none"
    add(none, { { "Comment", 0, #none } })
  end
  lines[#lines + 1] = ""
  local help = #lines + 1
  lines[help] = "⏎ show · r restore"
  lines[help + 1] = "m look · x clear · q close"
  lines[help + 2] = "/ search removed"
  vim.bo[pb].modifiable = true
  vim.api.nvim_buf_set_lines(pb, 0, -1, false, lines)
  vim.bo[pb].modifiable = false
  vim.api.nvim_buf_clear_namespace(pb, ns, 0, -1)
  vim.api.nvim_buf_set_extmark(pb, ns, 0, 0, { end_row = 0, end_col = #lines[1], hl_group = "Title" })
  for row, list in pairs(hls) do
    for p, h in ipairs(list) do
      vim.api.nvim_buf_set_extmark(pb, ns, row - 1, h[2], { end_row = row - 1, end_col = h[3], hl_group = h[1], priority = 100 + p })
    end
  end
  for r = help, #lines do
    vim.api.nvim_buf_set_extmark(pb, ns, r - 1, 0, { end_row = r - 1, end_col = #lines[r], hl_group = "Comment" })
  end
end

-- ── the preview of a removed block: a float with the block, y yanks it ──
function M.preview(rem)
  local lines = vim.split(rem.text, "\n", { plain = true })
  local width = math.max(20, math.floor(vim.o.columns * 0.7))
  local height = math.max(1, math.min(#lines, math.floor(vim.o.lines * 0.6)))
  local buf = vim.api.nvim_create_buf(false, true)
  vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)
  vim.bo[buf].bufhidden = "wipe"
  vim.bo[buf].modifiable = false
  if state.file_buf and vim.api.nvim_buf_is_valid(state.file_buf) then vim.bo[buf].filetype = vim.bo[state.file_buf].filetype end
  local win = vim.api.nvim_open_win(buf, true, {
    relative = "editor", width = width, height = height,
    row = math.floor((vim.o.lines - height) / 2), col = math.floor((vim.o.columns - width) / 2),
    style = "minimal", border = "rounded", title = " " .. rem.name .. " ", title_pos = "center",
  })
  local function close() if vim.api.nvim_win_is_valid(win) then vim.api.nvim_win_close(win, true) end end
  vim.keymap.set("n", "q", close, { buffer = buf, nowait = true, desc = "Trail: close the block" })
  vim.keymap.set("n", "<Esc>", close, { buffer = buf, nowait = true, desc = "Trail: close the block" })
  vim.keymap.set("n", "y", function()
    vim.fn.setreg("+", lines, "l")
    vim.fn.setreg('"', lines, "l")
    vim.notify("copied")
  end, { buffer = buf, nowait = true, desc = "Trail: copy the block" })
  return win
end

-- `/` in the panel: the Removed rows containing the word ("" shows all)
function M.search_removed(q)
  state.removed_filter = q ~= "" and q or nil
  M.render()
end

function M.refresh()
  if not state.path then return end
  state.entries = timeline(state.path, state.file_buf)
  local b = state.file_buf
  state.removed = (b and vim.api.nvim_buf_is_valid(b)) and removed_blocks(state.entries, buf_text(b)) or {}
  local shown = state.shown
  if shown and state.entries[shown] then M.show(shown) else M.clear() M.render() end
end

-- Enter on a hunk row: the file's window goes to the hunk's first line
local function jump(h)
  local w, b = state.file_win, state.file_buf
  if not (w and vim.api.nvim_win_is_valid(w) and b and vim.api.nvim_buf_is_valid(b)) then return end
  vim.api.nvim_win_set_cursor(w, { math.min(math.max(1, h.new_start), vim.api.nvim_buf_line_count(b)), 0 })
end

local function setup_panel(pb)
  vim.bo[pb].buftype = "nofile"
  vim.bo[pb].bufhidden = "wipe"
  vim.bo[pb].swapfile = false
  vim.bo[pb].filetype = "trail"
  pcall(vim.api.nvim_buf_set_name, pb, "trail://" .. (state.path or ""))
  local function map(k, f, desc) vim.keymap.set("n", k, f, { buffer = pb, nowait = true, desc = desc }) end
  local function at_cursor() return vim.api.nvim_win_get_cursor(0)[1] end
  local function enter()
    local row = at_cursor()
    if state.hunk_rows[row] then return jump(state.hunk_rows[row].hunk) end
    if state.removed_rows[row] then return M.preview(state.removed_rows[row]) end
    if state.rows[row] then M.show(state.rows[row]) end
  end
  map("<CR>", enter, "Trail: show the change, or jump to a hunk")
  map("p", enter, "Trail: show the change, or jump to a hunk")
  map("/", function()
    vim.ui.input({ prompt = "Removed code containing: " }, function(q) if q then M.search_removed(q) end end)
  end, "Trail: search the removed code")
  map("x", function() M.clear() M.render() end, "Trail: clear the marks")
  map("r", function()
    local hk = state.hunk_rows[at_cursor()]
    if hk then M.restore(hk.entry, hk.hunk) else M.refresh() end
  end, "Trail: restore a hunk, or reload")
  map("m", function() M.config.style = M.config.style == "boxes" and "plain" or "boxes" if state.shown then M.show(state.shown) else M.render() end end, "Trail: boxes or plain")
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
  state.file_win = vim.api.nvim_get_current_win()
  vim.cmd((M.config.side == "left" and "topleft" or "botright") .. " vnew")
  -- the split takes effect after this returns in zett (at once in Neovim): the panel is set up after it
  vim.schedule(function()
    state.panel_win = vim.api.nvim_get_current_win()
    state.panel_buf = vim.api.nvim_get_current_buf()
    vim.api.nvim_win_set_width(0, M.config.width)
    vim.wo.number, vim.wo.relativenumber, vim.wo.signcolumn = false, false, "no"
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
