-- undotree: the undo history as a tree, with a diff of each change (as mbbill/undotree).
-- Settings: zett.opt.undotree = { width = 34, diff = false, keys = { ["<CR>"] = "jump" } }

local z = zett
local defaults = {
  side = "left",       -- "left" or "right"
  width = 34,
  diff = true,         -- show the selected change's diff below the tree
  diff_height = 14,
  diff_context = 2,    -- unchanged lines around each change
  relative_time = true, -- "3m ago" rather than the clock time
  live = false,        -- moving in the tree moves the text there too
  focus = true,        -- the tree takes the keyboard when it opens
  keys = {
    ["<Enter>"] = "jump", o = "jump",
    u = "undo", ["<C-r>"] = "redo",
    J = "older", K = "newer",
    d = "toggle_diff", p = "toggle_live",
    q = "close", ["?"] = "help",
  },
}

z.options("undotree", defaults, {
  side = "which side: left or right",
  width = "width in columns",
  diff = "show the selected change's diff",
  diff_height = "rows for the diff",
  diff_context = "unchanged lines around a change",
  relative_time = "\"3m ago\" rather than the clock",
  live = "moving in the tree moves the text",
  focus = "take the keyboard when opened",
})

local M = { rows = {} }

local function ago(t)
  local cfg = z.config("undotree", defaults)
  if not cfg.relative_time then return os.date("%H:%M:%S", t) end
  local d = os.time() - t
  if d < 60 then return d .. "s ago" end
  if d < 3600 then return (d // 60) .. "m ago" end
  if d < 86400 then return (d // 3600) .. "h ago" end
  return (d // 86400) .. "d ago"
end

-- the tree, newest first, each branch in a column of its own (as git log --graph)
function M.render()
  local states = z.buf.undo_states()
  local cur = z.buf.undo_current()
  local lines, rows = {}, {}
  local lanes, n = {}, 0 -- lanes[l]: the state that column waits for, or false
  for i = #states, 1, -1 do
    local s = states[i]
    local hits = {}
    for l = 1, n do if lanes[l] == s.seq then hits[#hits + 1] = l end end
    local col = hits[1]
    if #hits > 1 then
      -- branches meet at their common state: ├─┴─╯
      local last, g = hits[#hits], {}
      for l = 1, n do
        if l == col then g[#g + 1] = "├─"
        elseif l == last then g[#g + 1] = "╯ "
        elseif l > col and l < last then g[#g + 1] = lanes[l] == s.seq and "┴─" or "┼─"
        else g[#g + 1] = lanes[l] and "│ " or "  " end
      end
      lines[#lines + 1] = { { table.concat(g), "Muted" } }
      rows[#rows + 1] = false
      for k = 2, #hits do lanes[hits[k]] = false end
    end
    while n > 0 and lanes[n] == false and n ~= col do lanes[n] = nil n = n - 1 end
    if not col then for l = 1, n do if lanes[l] == false then col = l break end end end
    if not col then n = n + 1 col = n end
    local g = {}
    for l = 1, n do
      if l == col then g[#g + 1] = s.seq == cur and "● " or "○ " else g[#g + 1] = lanes[l] and "│ " or "  " end
    end
    local here = s.seq == cur
    lines[#lines + 1] = {
      { table.concat(g), here and "Current" or "Accent" },
      { (s.seq == 0 and "original" or ("[" .. s.seq .. "]")) .. " ", here and "Current" or "File" },
      { ago(s.time), "Muted" },
    }
    rows[#rows + 1] = s.seq
    lanes[col] = s.seq ~= 0 and s.parent or false
    while n > 0 and lanes[n] == false do lanes[n] = nil n = n - 1 end
  end
  M.rows = rows
  return lines, cur
end

function M.selected()
  local r = z.panel.cursor("undotree")
  return r and M.rows[r]
end

function M.show_diff()
  local cfg = z.config("undotree", defaults)
  if not cfg.diff then z.panel.close("undotree-diff") return end
  local seq = M.selected()
  local lines = {}
  if seq and seq > 0 then
    local states = z.buf.undo_states()
    local parent = states[seq + 1].parent
    local d = z.diff(z.buf.text_at(parent), z.buf.text_at(seq))
    -- only the changes, with a little context
    local keep = {}
    for i, e in ipairs(d) do
      if e[1] ~= " " then
        for j = i - cfg.diff_context, i + cfg.diff_context do keep[j] = true end
      end
    end
    local last = 0
    for i, e in ipairs(d) do
      if keep[i] then
        if last > 0 and i > last + 1 then lines[#lines + 1] = { { "┈┈", "Muted" } } end
        local group = e[1] == "+" and "Added" or e[1] == "-" and "Removed" or "Muted"
        lines[#lines + 1] = { { e[1] .. " " .. e[2], group } }
        last = i
      end
    end
    if #lines == 0 then lines = { { { "no change in the text", "Muted" } } } end
  else
    lines = { { { "the text as opened", "Muted" } } }
  end
  z.panel.open({ id = "undotree-diff", side = cfg.side, width = cfg.width, height = cfg.diff_height, title = seq and seq > 0 and ("diff [" .. seq .. "]") or "diff", focus = false })
  z.panel.set_lines("undotree-diff", lines)
end

function M.refresh(keep_cursor)
  if not z.panel.is_open("undotree") then return end
  local lines, cur = M.render()
  z.panel.set_lines("undotree", lines)
  if not keep_cursor then
    for r, seq in ipairs(M.rows) do if seq == cur then z.panel.set_cursor("undotree", r) end end
  end
  M.show_diff()
end

M.actions = {
  jump = function()
    local seq = M.selected()
    if seq then z.buf.undo_goto(seq) end
    M.refresh()
  end,
  undo = function() z.buf.undo() M.refresh() end,
  redo = function() z.buf.redo() M.refresh() end,
  older = function() local c = z.buf.undo_current() if c > 0 then z.buf.undo_goto(c - 1) end M.refresh() end,
  newer = function() local c = z.buf.undo_current() if c + 1 < #z.buf.undo_states() then z.buf.undo_goto(c + 1) end M.refresh() end,
  toggle_diff = function()
    local o = type(z.opt.undotree) == "table" and z.opt.undotree or {}
    o.diff = not z.config("undotree", defaults).diff
    z.opt.undotree = o
    M.show_diff()
  end,
  toggle_live = function()
    local o = type(z.opt.undotree) == "table" and z.opt.undotree or {}
    o.live = not z.config("undotree", defaults).live
    z.opt.undotree = o
    z.notify("undotree: moving " .. (o.live and "moves the text" or "only selects"))
  end,
  close = function() M.close() end,
  help = function()
    local cfg = z.config("undotree", defaults)
    local ks = {}
    for k, a in pairs(cfg.keys) do if type(a) == "string" then ks[#ks + 1] = k .. " " .. a end end
    table.sort(ks)
    z.notify("undotree: " .. table.concat(ks, " · "))
  end,
}

function M.close()
  z.panel.close("undotree")
  z.panel.close("undotree-diff")
end

function M.open()
  local cfg = z.config("undotree", defaults)
  if not cfg then z.notify("undotree is turned off (zett.opt.undotree = false)") return end
  z.panel.open({
    id = "undotree", side = cfg.side, width = cfg.width, title = "Undo tree", focus = cfg.focus,
    on_key = function(key)
      local a = z.config("undotree", defaults).keys[key]
      if type(a) == "function" then a({ seq = M.selected() }) return true end
      if type(a) == "string" and M.actions[a] then M.actions[a]() return true end
      return false
    end,
    on_move = function()
      if z.config("undotree", defaults).live then
        local seq = M.selected()
        if seq then z.buf.undo_goto(seq) end
        M.refresh(true)
      else
        M.show_diff()
      end
    end,
    on_close = function() z.panel.close("undotree-diff") end,
  })
  M.refresh()
end

z.command("undotree", function()
  if z.panel.is_open("undotree") then M.close() else M.open() end
end, { desc = "Undo tree" })
z.map("n", "<leader>su", "undotree", { desc = "Undo history" })
z.map("n", "<leader>tu", "undotree", { desc = "Undo tree" })
z.map("n", "<leader>u", "undotree", { desc = "Undo tree" })
z.on("text_changed", function() M.refresh() end)
z.on("buf_enter", function() M.refresh() end)

-- for init.lua: require("undotree") gives this table (actions can be added or replaced)
package.loaded["undotree"] = M
z.on("option_set", function() M.refresh(true) end)
