-- fidget: what the language servers are doing, in the bottom-right corner (as fidget.nvim):
-- a spinner while they work, a tick when done, then it fades away.
-- Settings: zett.opt.fidget = { done_ttl = 5, notifications = true }

local z = zett
local defaults = {
  anchor = "SE",          -- the corner: SE, NE, SW, NW
  spinner = { "⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏" },
  rate = 100,             -- ms per spinner frame
  done_icon = "✔",
  done_ttl = 3,           -- seconds a finished task stays
  notifications = false,  -- show messages (zett.notify) here too
  notify_ttl = 5,
  max_width = 48,
  max_tasks = 6,          -- per server
}
z.options("fidget", defaults, {
  anchor = "the corner: SE, NE, SW or NW",
  spinner = "the spinner's frames",
  rate = "milliseconds per spinner frame",
  done_icon = "shown when a task is done",
  done_ttl = "seconds a finished task stays",
  notifications = "show messages here too",
  notify_ttl = "seconds a message stays",
  max_width = "widest line",
  max_tasks = "tasks shown per server",
})

-- highlight groups, so a colour scheme can change them
local function default_hl(name, spec) if not z.hl(name) then z.hl(name, spec) end end
default_hl("FidgetTitle", { link = "Title" })
default_hl("FidgetTask", { link = "Muted" })
default_hl("FidgetDone", { link = "Added" })
default_hl("FidgetSpinner", { link = "Accent" })
default_hl("FidgetNotify", { link = "MsgArea" })

local M = { servers = {}, order = {}, notes = {}, frame = 1, ticking = false }

-- off when fidget.nvim itself is installed (zett.plug): the real one draws instead
local function cfg() if (z._replaced or {}).fidget then return nil end return z.config("fidget", defaults) end
local function now() return os.time() end

local function cut(s, n)
  if utf8.len(s) and utf8.len(s) > n then
    return s:sub(1, utf8.offset(s, n) - 1) .. "…"
  end
  return s
end

function M.render()
  local c = cfg()
  if not c then z.float.close("fidget") return false end
  local lines, busy, t = {}, false, now()
  for _, n in ipairs(M.notes) do
    if t - n.at < c.notify_ttl then lines[#lines + 1] = { { cut(n.text, c.max_width), "FidgetNotify" } } end
  end
  for _, name in ipairs(M.order) do
    local s = M.servers[name]
    local shown, working = {}, false
    for _, task in pairs(s) do
      if not task.done or t - task.done_at < c.done_ttl then shown[#shown + 1] = task end
      if not task.done then working = true end
    end
    table.sort(shown, function(a, b) return a.started < b.started end)
    if #shown > 0 then
      for i = math.max(1, #shown - c.max_tasks + 1), #shown do
        local task = shown[i]
        local text = task.message ~= "" and task.message or task.title
        if task.percent ~= "" then text = text .. " (" .. task.percent .. "%)" end
        if task.message ~= "" and task.title ~= "" then text = text .. "  " .. task.title end
        lines[#lines + 1] = {
          { cut(text, c.max_width) .. " ", "FidgetTask" },
          { task.done and c.done_icon or " ", "FidgetDone" },
        }
      end
      local icon = working and c.spinner[(M.frame - 1) % #c.spinner + 1] or c.done_icon
      lines[#lines + 1] = { { name .. " ", "FidgetTitle" }, { icon, working and "FidgetSpinner" or "FidgetDone" } }
      busy = busy or working
    end
  end
  if #lines == 0 then
    z.float.close("fidget")
    return false
  end
  z.float.open({ id = "fidget", anchor = c.anchor, lines = lines })
  return true
end

-- keeps turning while something works or is still to fade, then stops (no timer when idle)
function M.tick()
  M.ticking = M.render()
  if M.ticking then
    M.frame = M.frame + 1
    z.defer(cfg().rate, M.tick)
  end
end

local function wake()
  if not M.ticking then M.tick() end
end

z.on("lsp_progress", function(e)
  if not cfg() then return end
  local s = M.servers[e.server]
  if not s then
    s = {}
    M.servers[e.server] = s
    M.order[#M.order + 1] = e.server
  end
  local task = s[e.token] or { title = "", message = "", percent = "", started = os.clock() }
  if e.title ~= "" then task.title = e.title end
  task.message = e.message
  task.percent = e.percent
  if e.done == "true" then
    task.done, task.done_at = true, now()
    task.percent = ""
  else
    -- a token used again (rust-analyzer indexes in rounds): working again
    task.done = false
  end
  s[e.token] = task
  -- finished tasks that faded are forgotten
  for k, t in pairs(s) do if t.done and now() - t.done_at > cfg().done_ttl then s[k] = nil end end
  wake()
end)

z.on("notify", function(e)
  local c = cfg()
  if not (c and c.notifications) then return end
  M.notes[#M.notes + 1] = { text = e.message, at = now() }
  while #M.notes > 5 do table.remove(M.notes, 1) end
  wake()
end)

-- zett.fidget(text): a message in the corner from any script
function z.fidget(text)
  M.notes[#M.notes + 1] = { text = text, at = now() }
  wake()
end

M.zett_builtin = true
package.loaded["fidget"] = M
