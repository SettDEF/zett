-- Neovim's Lua API (vim.*) on zett's own: what a Neovim config or plugin calls works unchanged
-- where zett has the thing. Options are zett.opt (global) and :setlocal (buffer, window); Ex commands
-- run after the Lua that asked for them returns; the buffer is zett.buf, the one being edited.
local z = zett
vim = vim or {}
local unpack = table.unpack or unpack
-- LuaJIT's (Lua 5.1) globals that Neovim plugins use: unpack, loadstring, math.pow, the bit library
_G.unpack = _G.unpack or table.unpack
_G.loadstring = _G.loadstring or load
-- function environments (5.1): one global table here, so getfenv is _G and setfenv returns the function as is
_G.getfenv = _G.getfenv or function() return _G end
_G.setfenv = _G.setfenv or function(f) return f end
-- newproxy(true): an object with its own metatable, its __gc set later (gitsigns.util.gc_proxy). Lua 5.4
-- only marks it for finalizing when __gc exists at setmetatable, so a placeholder is there from the start
_G.newproxy = _G.newproxy or function(mt) return mt and setmetatable({}, { __gc = function() end }) or {} end
math.pow = math.pow or function(a, b) return a ^ b end
table.getn = table.getn or function(t) return #t end
-- the largest positive whole-number key (gitsigns counts varargs with it, holes included)
table.maxn = table.maxn or function(t) local n = 0 for k in pairs(t) do if type(k) == "number" and k > n and k == math.floor(k) then n = k end end return n end
if not package.loaded.bit then
  local function u(x) x = math.tointeger(x) or math.floor(x) return x & 0xffffffff end
  local function s(x) x = u(x) return x >= 0x80000000 and x - 0x100000000 or x end
  local function fold(op) return function(a, ...) a = u(a) for _, b in ipairs({ ... }) do a = op(a, u(b)) end return s(a) end end
  package.loaded.bit = {
    band = fold(function(a, b) return a & b end),
    bor = fold(function(a, b) return a | b end),
    bxor = fold(function(a, b) return a ~ b end),
    bnot = function(a) return s(~u(a)) end,
    lshift = function(a, n) return s(u(a) << n) end,
    rshift = function(a, n) return s(u(a) >> n) end,
    arshift = function(a, n) return s(s(a) >> n | (s(a) < 0 and ~(0xffffffff >> n) or 0)) end,
    tobit = function(a) return s(a) end,
    tohex = function(a, n) return string.format("%0" .. (n or 8) .. "x", u(a)) end,
  }
  _G.bit = package.loaded.bit
end

-- ── tables and strings (vim.tbl_*, vim.split, …): plain Lua, as Neovim's shared.lua ──

function vim.deepcopy(o)
  if type(o) ~= "table" then return o end
  local r = {}
  for k, v in pairs(o) do r[vim.deepcopy(k)] = vim.deepcopy(v) end
  return setmetatable(r, getmetatable(o))
end
function vim.tbl_keys(t) local r = {} for k in pairs(t) do r[#r + 1] = k end return r end
function vim.tbl_values(t) local r = {} for _, v in pairs(t) do r[#r + 1] = v end return r end
function vim.tbl_contains(t, v) for _, x in pairs(t) do if x == v then return true end end return false end
function vim.tbl_isempty(t) return next(t) == nil end
function vim.tbl_count(t) local n = 0 for _ in pairs(t) do n = n + 1 end return n end
function vim.tbl_map(f, t) local r = {} for k, v in pairs(t) do r[k] = f(v) end return r end
function vim.tbl_filter(f, t) local r = {} for _, v in ipairs(t) do if f(v) then r[#r + 1] = v end end return r end
function vim.islist(t)
  if type(t) ~= "table" then return false end
  local n = 0
  for k in pairs(t) do
    if type(k) ~= "number" then return false end
    n = n + 1
  end
  return n == #t
end
vim.tbl_islist = vim.islist
local function extend(how, deep, ...)
  local r = {}
  for _, t in ipairs({ ... }) do
    for k, v in pairs(t or {}) do
      if deep and type(v) == "table" and type(r[k]) == "table" and not vim.islist(v) then
        r[k] = extend(how, true, r[k], v)
      elseif r[k] == nil or how == "force" then
        r[k] = deep and vim.deepcopy(v) or v
      elseif how == "error" then
        error("key found in more than one map: " .. tostring(k))
      end
    end
  end
  return r
end
function vim.tbl_extend(how, ...) return extend(how, false, ...) end
function vim.tbl_deep_extend(how, ...) return extend(how, true, ...) end
function vim.tbl_get(t, ...)
  for _, k in ipairs({ ... }) do
    if type(t) ~= "table" then return nil end
    t = t[k]
  end
  return t
end
function vim.tbl_flatten(t)
  local r = {}
  local function go(x) for _, v in ipairs(x) do if type(v) == "table" then go(v) else r[#r + 1] = v end end end
  go(t)
  return r
end
function vim.list_extend(dst, src, a, b) for i = a or 1, b or #src do dst[#dst + 1] = src[i] end return dst end
function vim.list_slice(t, a, b) local r = {} for i = a or 1, b or #t do r[#r + 1] = t[i] end return r end
function vim.pesc(s) return (s:gsub("[%(%)%.%%%+%-%*%?%[%]%^%$]", "%%%0")) end
function vim.trim(s) return (s:gsub("^%s+", ""):gsub("%s+$", "")) end
function vim.startswith(s, p) return s:sub(1, #p) == p end
function vim.endswith(s, p) return p == "" or s:sub(-#p) == p end
function vim.gsplit(s, sep, opts)
  local plain = type(opts) == "table" and opts.plain or opts == true
  local parts = {}
  local at = 1
  while true do
    local a, b = s:find(sep, at, plain)
    if not a or sep == "" then parts[#parts + 1] = s:sub(at) break end
    parts[#parts + 1] = s:sub(at, a - 1)
    at = b + 1
  end
  if type(opts) == "table" and opts.trimempty then
    while parts[1] == "" do table.remove(parts, 1) end
    while parts[#parts] == "" do table.remove(parts) end
  end
  local i = 0
  return function() i = i + 1 return parts[i] end
end
function vim.split(s, sep, opts) local r = {} for p in vim.gsplit(s, sep, opts) do r[#r + 1] = p end return r end
function vim.is_callable(f)
  if type(f) == "function" then return true end
  local m = getmetatable(f)
  return m ~= nil and type(m.__call) == "function"
end
function vim.validate() end
vim.F = { if_nil = function(...) for i = 1, select("#", ...) do local v = select(i, ...) if v ~= nil then return v end end end }
vim.nonnil = vim.F.if_nil

function vim.inspect(v, _, depth)
  depth = depth or 0
  if type(v) == "string" then return string.format("%q", v) end
  if type(v) ~= "table" then return tostring(v) end
  if depth > 8 then return "{...}" end
  local parts, n = {}, #v
  for i = 1, n do parts[#parts + 1] = vim.inspect(v[i], nil, depth + 1) end
  local keys = {}
  for k in pairs(v) do if not (type(k) == "number" and k >= 1 and k <= n and k % 1 == 0) then keys[#keys + 1] = k end end
  table.sort(keys, function(a, b) return tostring(a) < tostring(b) end)
  for _, k in ipairs(keys) do
    local key = type(k) == "string" and k:match("^[%a_][%w_]*$") and k or "[" .. vim.inspect(k, nil, depth + 1) .. "]"
    parts[#parts + 1] = key .. " = " .. vim.inspect(v[k], nil, depth + 1)
  end
  return "{ " .. table.concat(parts, ", ") .. " }"
end
function vim.print(...)
  local out = {}
  for _, v in ipairs({ ... }) do out[#out + 1] = type(v) == "string" and v or vim.inspect(v) end
  z.notify(table.concat(out, " "))
  return ...
end

-- ── JSON (vim.json, vim.fn.json_encode / json_decode) ──
vim.NIL = setmetatable({}, { __tostring = function() return "vim.NIL" end })
local function json_encode(v)
  local t = type(v)
  if v == nil or v == vim.NIL then return "null" end
  if t == "boolean" or t == "number" then return tostring(v) end
  if t == "string" then
    return '"' .. v:gsub('[%c"\\]', function(c)
      local m = { ['"'] = '\\"', ["\\"] = "\\\\", ["\n"] = "\\n", ["\t"] = "\\t", ["\r"] = "\\r" }
      return m[c] or string.format("\\u%04x", c:byte())
    end) .. '"'
  end
  if t == "table" then
    local parts = {}
    if vim.islist(v) and (next(v) ~= nil or getmetatable(v) ~= vim._empty_dict_mt) then
      for _, x in ipairs(v) do parts[#parts + 1] = json_encode(x) end
      return "[" .. table.concat(parts, ",") .. "]"
    end
    local keys = vim.tbl_keys(v)
    table.sort(keys, function(a, b) return tostring(a) < tostring(b) end)
    for _, k in ipairs(keys) do parts[#parts + 1] = json_encode(tostring(k)) .. ":" .. json_encode(v[k]) end
    return "{" .. table.concat(parts, ",") .. "}"
  end
  error("json: cannot encode a " .. t)
end
local function json_decode(s)
  local i = 1
  local function ws() i = s:find("[^ \t\r\n]", i) or #s + 1 end
  local value
  local function str()
    local out = {}
    i = i + 1
    while true do
      local c = s:sub(i, i)
      if c == "" then error("json: unterminated string") end
      if c == '"' then i = i + 1 break end
      if c == "\\" then
        local e = s:sub(i + 1, i + 1)
        local m = { n = "\n", t = "\t", r = "\r", b = "\b", f = "\f", ["/"] = "/", ["\\"] = "\\", ['"'] = '"' }
        if e == "u" then
          out[#out + 1] = utf8.char(tonumber(s:sub(i + 2, i + 5), 16))
          i = i + 6
        else
          out[#out + 1] = m[e] or e
          i = i + 2
        end
      else
        out[#out + 1] = c
        i = i + 1
      end
    end
    return table.concat(out)
  end
  function value()
    ws()
    local c = s:sub(i, i)
    if c == "{" then
      local r = {}
      i = i + 1
      ws()
      if s:sub(i, i) == "}" then i = i + 1 return r end
      while true do
        ws()
        local k = str()
        ws()
        i = i + 1 -- :
        r[k] = value()
        ws()
        local d = s:sub(i, i)
        i = i + 1
        if d == "}" then return r end
      end
    elseif c == "[" then
      local r = {}
      i = i + 1
      ws()
      if s:sub(i, i) == "]" then i = i + 1 return r end
      while true do
        r[#r + 1] = value()
        ws()
        local d = s:sub(i, i)
        i = i + 1
        if d == "]" then return r end
      end
    elseif c == '"' then
      return str()
    elseif s:sub(i, i + 3) == "true" then i = i + 4 return true
    elseif s:sub(i, i + 4) == "false" then i = i + 5 return false
    elseif s:sub(i, i + 3) == "null" then i = i + 4 return vim.NIL
    else
      local n = s:match("^-?%d+%.?%d*[eE]?[-+]?%d*", i)
      if not n or n == "" then error("json: unexpected " .. c .. " at " .. i) end
      i = i + #n
      return tonumber(n)
    end
  end
  return value()
end
vim._empty_dict_mt = {}
function vim.empty_dict() return setmetatable({}, vim._empty_dict_mt) end
vim.json = { encode = json_encode, decode = json_decode }

-- ── messages, timers ──
vim.log = { levels = { TRACE = 0, DEBUG = 1, INFO = 2, WARN = 3, ERROR = 4, OFF = 5 } }
function vim.notify(msg, level)
  local tag = ({ [3] = "W: ", [4] = "E: " })[level or 2] or ""
  z.notify(tag .. tostring(msg))
end
function vim.notify_once(msg, level)
  vim._notified = vim._notified or {}
  if vim._notified[msg] then return false end
  vim._notified[msg] = true
  vim.notify(msg, level)
  return true
end
function vim.schedule(f) z.defer(0, f) end
function vim.defer_fn(f, ms) z.defer(ms, f) end
function vim.schedule_wrap(f) return function(...) local a = { ... } vim.schedule(function() f(unpack(a)) end) end end
function vim.in_fast_event() return false end
vim.uv = vim.uv or {}
function vim.uv.hrtime() return z.hrtime() end
function vim.uv.now() return math.floor(z.hrtime() / 1e6) end
function vim.uv.cwd() return z.root() end
function vim.uv.os_homedir() return os.getenv("HOME") end
function vim.uv.os_getenv(n) return os.getenv(n) end
function vim.uv.fs_stat(p)
  if not z.fs.exists(p) then return nil end
  return { type = z.fs.is_dir(p) and "directory" or "file", size = z.fs.size and z.fs.size(p) or 0, mtime = { sec = z.fs.mtime and z.fs.mtime(p) or 0 } }
end
function vim.uv.new_timer()
  local t = { on = false }
  function t:start(ms, rep, f)
    self.on = true
    local function tick()
      if not self.on then return end
      f()
      if rep and rep > 0 then z.defer(rep, tick) else self.on = false end
    end
    z.defer(ms, tick)
  end
  function t:stop() self.on = false end
  function t:close() self.on = false end
  function t:is_active() return self.on end
  return t
end
-- vim.uv.new_fs_event: a watch on a file or a folder (what changed in it), checked a few times a
-- second; cb(err, filename, events) as libuv calls it (tinct.nvim's live palette, auto-reloaders)
function vim.uv.new_fs_event()
  local h = { on = false }
  local function snap(path)
    local st = z.fs.stat(path)
    if not st then return {} end
    if not st.dir then return { [""] = st.mtime_ns .. ":" .. st.size } end
    local s = {}
    for _, e in ipairs(z.fs.list(path)) do
      local x = z.fs.stat(e.path)
      s[e.name] = x and (x.mtime_ns .. ":" .. x.size) or "gone"
    end
    return s
  end
  function h:start(path, _, cb)
    self.on, self.path = true, path
    local last = snap(path)
    local is_dir = z.fs.is_dir(path)
    local base = path:match("([^/]+)/*$")
    local function tick()
      if not self.on then return end
      local now = snap(path)
      for k, v in pairs(now) do
        if last[k] ~= v then cb(nil, k ~= "" and k or base, { change = true }) end
      end
      for k in pairs(last) do
        if now[k] == nil then cb(nil, k ~= "" and k or base, { rename = true }) end
      end
      last = now
      z.defer(is_dir and 400 or 300, tick)
    end
    z.defer(400, tick)
    return 0
  end
  function h:stop() self.on = false return 0 end
  function h:close() self.on = false end
  function h:is_active() return self.on end
  function h:getpath() return self.path end
  return h
end
vim.loop = vim.uv

-- ── variables: vim.g, vim.b, vim.w, vim.t, vim.v, vim.env ──
-- vim.g.mapleader is zett's leader (zett.opt.leader)
vim.g = vim.g or setmetatable({}, {
  __newindex = function(t, k, v)
    if k == "mapleader" then z.opt.leader = v end
    rawset(t, k, v)
  end,
})
vim.b = vim.b or setmetatable({}, { __index = function(t, k) if type(k) == "number" then return t end end })
vim.w = vim.w or setmetatable({}, { __index = function(t, k) if type(k) == "number" then return t end end })
vim.t = vim.t or {}
vim.v = vim.v or { shell_error = 0, ["true"] = true, ["false"] = false, null = vim.NIL, count = 0, count1 = 1, errmsg = "", version = 1000, progpath = "zett" }
-- v:oldfiles: zett's recent files (newest first), read when asked
setmetatable(vim.v, { __index = function(_, k)
  if k ~= "oldfiles" then return nil end
  local dir = (os.getenv("XDG_STATE_HOME") or ((os.getenv("HOME") or "") .. "/.local/state")) .. "/zett/recent"
  local out, f = {}, io.open(dir)
  if f then
    for l in f:lines() do if l ~= "" then out[#out + 1] = l end end
    f:close()
  end
  return out
end })
vim.env = setmetatable({}, {
  __index = function(_, k) return rawget(vim._env, k) or os.getenv(k) end,
  __newindex = function(_, k, v) vim._env[k] = v end,
})
vim._env = {}

-- ── Ex commands: vim.cmd("set ts=4"), vim.cmd.set("ts=4"), vim.cmd({ cmd = "edit", args = { "x" } }) ──
local function ex(line)
  -- a command that makes a new empty buffer the current one: that buffer is current from now on in this Lua
  -- (cur_buf), its lines kept until zett makes it
  local words = {}
  for w in tostring(line):gmatch("%S+") do words[#words + 1] = w end
  local mods = { botright = 1, bo = 1, topleft = 1, to = 1, vertical = 1, vert = 1, rightbelow = 1, belowright = 1, bel = 1, leftabove = 1, aboveleft = 1, abo = 1, silent = 1, ["silent!"] = 1, keepalt = 1, noautocmd = 1, noa = 1, tab = 1, keepjumps = 1 }
  local i = 1
  while words[i] and mods[words[i]] do i = i + 1 end
  local c = words[i] and words[i]:gsub("^%d+", "") or ""
  local makes = { enew = 1, ene = 1, new = 1, vnew = 1, vne = 1, tabnew = 1, tabe = 1, tabedit = 1 }
  if makes[c] and not words[i + 1] then
    local base = vim._pending_buf and vim._pending_buf.nr or #z.buffers()
    vim._pending_buf = { nr = base + 1, lines = { "" } }
  end
  z.ui("ex", line)
end
local function cmd_string(c)
  if type(c) == "string" then return c end
  local s = c.cmd .. (c.bang and "!" or "")
  if c.args and #c.args > 0 then s = s .. " " .. table.concat(c.args, " ") end
  if c.range then s = table.concat(c.range, ",") .. s end
  return s
end
vim.cmd = setmetatable({}, {
  __call = function(_, c)
    if type(c) == "table" then return ex(cmd_string(c)) end
    -- several lines: Vimscript (if/for/let/function) goes through the translator (vimscript.lua)
    if vim._vimscript and c:find("\n") then return vim._vimscript.run(c) end
    for line in c:gmatch("[^\n]+") do
      if vim._vimscript and vim._vimscript.handles(line) then vim._vimscript.run(line) else ex(line) end
    end
  end,
  __index = function(_, name)
    return function(...)
      local a = { ... }
      if type(a[1]) == "table" then return ex(cmd_string(vim.tbl_extend("force", { cmd = name }, a[1]))) end
      ex(name .. (#a > 0 and (" " .. table.concat(vim.tbl_map(tostring, a), " ")) or ""))
    end
  end,
})

-- ── options: vim.o / vim.go global (zett.opt), vim.bo / vim.wo through :setlocal, vim.opt ──
local function info(name)
  local i = z.option_info and z.option_info(name)
  return i
end
local function typed(i, v)
  if v == nil then return nil end
  if i and i.kind == "bool" then return v == true or v == "true" or v == 1 or v == "1" end
  if i and i.kind == "int" then return tonumber(v) or 0 end
  return v
end
local function opt_get(name)
  local i = info(name)
  local key = i and i.name or name
  -- the current buffer's own value first (filetype, commentstring, :setlocal), as Neovim's vim.o / vim.bo give it
  local v = z.local_opt and z.local_opt(key)
  if v == nil then v = z.opt[key] end
  if v == nil then v = i and i.default end
  return typed(i, v)
end
local function as_text(v)
  if type(v) == "table" then return table.concat(vim.islist(v) and v or vim.tbl_map(function(x) return x end, vim.tbl_keys(v)), ",") end
  return v
end
local function opt_set(name, v, scope)
  local i = info(name)
  local key = i and i.name or name
  v = as_text(v)
  if scope == "local" then
    if not vim._pending_buf then pcall(z.buf.set_opt, key, tostring(v)) end
    if type(v) == "boolean" then ex("setlocal " .. (v and "" or "no") .. key) else ex("setlocal " .. key .. "=" .. tostring(v):gsub(" ", "\\ ")) end
  else
    z.opt[key] = typed(i, v)
    -- the window drawn and buffers already open take it up as :set would
    if type(v) == "boolean" then ex("set " .. (v and "" or "no") .. key) else ex("set " .. key .. "=" .. tostring(v):gsub(" ", "\\ ")) end
  end
end
local function opt_table(scope)
  return setmetatable({}, {
    __index = function(t, k) if type(k) == "number" then return t end return opt_get(k) end,
    __newindex = function(_, k, v) opt_set(k, v, scope) end,
  })
end
vim.o = opt_table("global")
vim.go = vim.o
vim.bo = opt_table("local")
vim.wo = opt_table("local")
-- vim.bo[n] / vim.wo[n]: that buffer's or window's own options, never the current one's (telescope sets
-- buftype=prompt and nomodifiable on its scratch buffers this way). A scratch buffer and a float keep theirs;
-- another file's are kept for it and set when it becomes current
do
  local held = {}
  vim._zett_held_opts = held
  local function current_buf() return vim.api.nvim_get_current_buf() end
  local function buf_view(b)
    return setmetatable({}, {
      __index = function(_, name) return vim.api.nvim_buf_get_option(b, name) end,
      __newindex = function(_, name, v)
        if vim._zett_is_vbuf and vim._zett_is_vbuf(b) then return vim.api.nvim_buf_set_option(b, name, v) end
        held[b] = held[b] or {}
        held[b][name] = v
      end,
    })
  end
  local bo_plain = vim.bo
  vim.bo = setmetatable({}, {
    __index = function(t, k)
      if type(k) == "number" then
        if (k == 0 or k == current_buf()) and not vim._cur_float then return t end
        return buf_view(k == 0 and current_buf() or k)
      end
      if vim._cur_float then return vim.api.nvim_buf_get_option(0, k) end
      return bo_plain[k]
    end,
    __newindex = function(_, k, v)
      if vim._cur_float then return vim.api.nvim_buf_set_option(0, k, v) end
      bo_plain[k] = v
    end,
  })
  local wo_plain = vim.wo
  vim.wo = setmetatable({}, {
    __index = function(t, k)
      if type(k) == "number" then
        if vim._zett_is_float and vim._zett_is_float(k) then return vim._zett_float_wo(k) end
        return t
      end
      if vim._cur_float and vim._zett_float_wo then return vim._zett_float_wo(vim._cur_float)[k] end
      return wo_plain[k]
    end,
    __newindex = function(_, k, v)
      if vim._cur_float and vim._zett_float_wo then vim._zett_float_wo(vim._cur_float)[k] = v return end
      wo_plain[k] = v
    end,
  })
end
local function opt_object(scope)
  return setmetatable({}, {
    __index = function(_, name)
      local o = {}
      function o:get()
        local v = opt_get(name)
        local i = info(name)
        if i and i.kind == "text" and type(v) == "string" and v:find(",") then return vim.split(v, ",", { trimempty = true }) end
        if i and i.kind == "text" and type(v) == "string" and (name == "shortmess" or name == "formatoptions" or name == "cpoptions") then
          local r = {}
          for c in v:gmatch(".") do r[c] = true end
          return r
        end
        return v
      end
      function o:append(x) opt_set(name, (opt_get(name) or "") .. (tostring(opt_get(name) or "") ~= "" and "," or "") .. as_text(x), scope) end
      function o:prepend(x) opt_set(name, as_text(x) .. "," .. (opt_get(name) or ""), scope) end
      function o:remove(x)
        local parts = vim.tbl_filter(function(p) return p ~= x end, vim.split(tostring(opt_get(name) or ""), ",", { trimempty = true }))
        opt_set(name, table.concat(parts, ","), scope)
      end
      return setmetatable(o, { __add = function(a, b) a:append(b) return a end, __sub = function(a, b) a:remove(b) return a end, __pow = function(a, b) a:prepend(b) return a end })
    end,
    __newindex = function(_, k, v) opt_set(k, v, scope) end,
  })
end
vim.opt = opt_object("global")
vim.opt_global = vim.opt
vim.opt_local = opt_object("local")

-- ── keymaps: vim.keymap.set / del ──
local zett_mode = { n = "n", v = "v", x = "v", s = "v", i = "i", c = "c", o = "n", t = nil, [""] = "n", ["!"] = "i" }
vim.keymap = {}
function vim.keymap.set(modes, lhs, rhs, opts)
  opts = opts or {}
  if type(modes) == "string" then modes = { modes } end
  -- expr: the function gives keys to type (autopairs, completion, snippets); they go in as typed, remapped
  -- only with remap = true, as in Neovim
  if opts.expr and type(rhs) == "function" then
    local f = rhs
    rhs = function()
      local keys = f()
      -- <Plug> keys are only mappings: always remapped, as Neovim does
      if type(keys) == "string" and keys ~= "" then z.feed_keys(keys, opts.remap == true or keys:lower():find("<plug>", 1, true) ~= nil) end
    end
  end
  for _, m in ipairs(modes) do
    if type(rhs) == "function" then
      local zm = zett_mode[m]
      if zm then z.map(zm, lhs, rhs, { desc = opts.desc }) end
    else
      local flags = (opts.silent and "<silent>" or "") .. (opts.buffer and "<buffer>" or "") .. (opts.expr and "<expr>" or "") .. (opts.nowait and "<nowait>" or "")
      ex(m .. ((opts.remap or tostring(rhs):lower():find("<plug>", 1, true)) and "map " or "noremap ") .. (flags ~= "" and flags .. " " or "") .. lhs .. " " .. rhs)
    end
  end
end
function vim.keymap.del(modes, lhs)
  if type(modes) == "string" then modes = { modes } end
  for _, m in ipairs(modes) do
    local zm = zett_mode[m]
    if zm then pcall(z.unmap, zm, lhs) end
    ex(m .. "unmap " .. lhs)
  end
end

-- ── callbacks Ex commands call back (:command, :autocmd, :lua) ──
vim._cb = vim._cb or {}
local function keep(f)
  vim._cb[#vim._cb + 1] = f
  return #vim._cb
end

-- ── buffers and windows: the one being edited (handle 0, or its number) ──
local function cur_buf()
  -- a buffer vim.cmd("vnew") makes is current at once (zett makes it after this Lua returns): its number
  -- until it exists
  if vim._pending_buf then
    if #z.buffers() >= vim._pending_buf.nr then vim._pending_buf = nil else return vim._pending_buf.nr end
  end
  -- the host says which one (its place in zett.buffers); else found by path
  local n = z.cur_buffer and z.cur_buffer() or 0
  if n > 0 then return n end
  local p = z.buf.path()
  for i, b in ipairs(z.buffers()) do if b == p then return i end end
  return 1
end
local function lines(a, b) local ok, r = pcall(z.buf.lines, a, b) return ok and r or {} end

-- ── vim.api ──
local api = vim.api or {}
-- a buffer that is not the current one: read from the copies the editor gives, written by a queue it applies after
local function other(b) return type(b) == "number" and b > 0 and b ~= cur_buf() and z.buf_lines_of and z.buf_line_count_of(b) ~= nil end
local other_writes = {}
function vim._zett_apply_writes(nr)
  for _, w in ipairs(other_writes[nr] or {}) do pcall(z.buf.set_lines, w[1], w[2], w[3]) end
  other_writes[nr] = nil
end
-- the buffer vim.cmd("vnew") is making: its lines kept here (read back at once), its writes queued for it
local function pending(b)
  local p = vim._pending_buf
  if not p then return nil end
  if b == nil or b == 0 or b == p.nr then return p end
end
local function set_range(L, a, e, l)
  local n = #L
  a = a < 0 and n + a + 1 or a
  e = e < 0 and n + e + 1 or e
  local out = {}
  for i = 1, a do out[#out + 1] = L[i] end
  for _, x in ipairs(l or {}) do out[#out + 1] = x end
  for i = e + 1, n do out[#out + 1] = L[i] end
  if #out == 0 then out = { "" } end
  return out
end
api.nvim_buf_line_count = function(b) local p = pending(b) if p then return #p.lines end if other(b) then return z.buf_line_count_of(b) end return z.buf.line_count() end
api.nvim_buf_get_lines = function(b, a, e, _)
  local p = pending(b)
  if p then local n = #p.lines a = a < 0 and n + a + 1 or a e = e < 0 and n + e + 1 or e local out = {} for i = a + 1, e do out[#out + 1] = p.lines[i] end return out end
  if other(b) then return z.buf_lines_of(b, a, e) end
  return lines(a, e)
end
api.nvim_buf_set_lines = function(b, a, e, _, l)
  local p = pending(b)
  if p then
    p.lines = set_range(p.lines, a, e, l)
    other_writes[p.nr] = other_writes[p.nr] or {}
    table.insert(other_writes[p.nr], { a, e, l })
    z.note_other_write(p.nr)
    return
  end
  if other(b) then
    other_writes[b] = other_writes[b] or {}
    table.insert(other_writes[b], { a, e, l })
    z.note_other_write(b)
    return
  end
  z.buf.set_lines(a, e, l)
end
vim.api = api
api.nvim_command = ex
api.nvim_exec = function(src, output) vim.cmd(src) return output and "" or nil end
api.nvim_exec2 = function(src, _) vim.cmd(src) return { output = "" } end
api.nvim_cmd = function(c, _) ex(cmd_string(c)) return "" end
api.nvim_get_current_buf = cur_buf
api.nvim_get_current_win = function() return 1000 end
api.nvim_get_current_tabpage = function() return 1 end
api.nvim_list_bufs = function() local r = {} for i in ipairs(z.buffers()) do r[i] = i end return r end
api.nvim_list_wins = function() return { 1000 } end
api.nvim_list_tabpages = function() return { 1 } end
api.nvim_buf_is_valid = function(b) return b == 0 or (b >= 1 and b <= #z.buffers()) end
api.nvim_buf_is_loaded = api.nvim_buf_is_valid
api.nvim_win_is_valid = function(w) return w == 0 or w == 1000 end
api.nvim_win_get_buf = function() return cur_buf() end
api.nvim_buf_get_name = function(b)
  if b == 0 or b == cur_buf() then return z.buf.path() end
  return z.buffers()[b] or ""
end
api.nvim_buf_set_name = function(_, name) ex("file " .. name) end
api.nvim_buf_get_text = function(_, sr, sc, er, ec, _)
  local l = lines(sr, er + 1)
  if #l == 0 then return {} end
  if #l == 1 then return { l[1]:sub(sc + 1, ec) } end
  l[1] = l[1]:sub(sc + 1)
  l[#l] = l[#l]:sub(1, ec)
  return l
end
api.nvim_buf_set_text = function(_, sr, sc, er, ec, repl)
  local l = lines(sr, er + 1)
  local head, tail = (l[1] or ""):sub(1, sc), (l[#l] or ""):sub(ec + 1)
  local new = vim.deepcopy(repl)
  if #new == 0 then new = { "" } end
  new[1] = head .. new[1]
  new[#new] = new[#new] .. tail
  z.buf.set_lines(sr, er + 1, new)
end
api.nvim_get_current_line = function() local r, _ = z.buf.cursor() return lines(r - 1, r)[1] or "" end
api.nvim_set_current_line = function(s) local r, _ = z.buf.cursor() z.buf.set_lines(r - 1, r, { s }) end
api.nvim_del_current_line = function() local r, _ = z.buf.cursor() z.buf.set_lines(r - 1, r, {}) end
api.nvim_win_get_cursor = function() local r, c = z.buf.cursor() return { r, c } end
api.nvim_win_set_cursor = function(_, pos) z.buf.set_cursor(pos[1], pos[2]) end
api.nvim_put = function(l, _, after, _)
  local r, _ = z.buf.cursor()
  local at = after and r or r - 1
  z.buf.set_lines(at, at, l)
end
api.nvim_get_var = function(k) return vim.g[k] end
api.nvim_set_var = function(k, v) vim.g[k] = v end
api.nvim_del_var = function(k) vim.g[k] = nil end
api.nvim_buf_get_var = function(_, k) return vim.b[k] end
api.nvim_buf_set_var = function(_, k, v) vim.b[k] = v end
api.nvim_buf_del_var = function(_, k) vim.b[k] = nil end
api.nvim_win_get_var = function(_, k) return vim.w[k] end
api.nvim_win_set_var = function(_, k, v) vim.w[k] = v end
api.nvim_get_vvar = function(k) return vim.v[k] end
api.nvim_set_vvar = function(k, v) vim.v[k] = v end
api.nvim_get_option_value = function(name, _) return opt_get(name) end
api.nvim_set_option_value = function(name, v, o) opt_set(name, v, o and (o.scope == "local" or o.buf or o.win) and "local" or "global") end
api.nvim_get_option = function(name) return opt_get(name) end
api.nvim_set_option = function(name, v) opt_set(name, v, "global") end
api.nvim_buf_get_option = function(_, name) return opt_get(name) end
api.nvim_buf_set_option = function(_, name, v) opt_set(name, v, "local") end
api.nvim_win_get_option = function(_, name) return opt_get(name) end
api.nvim_win_set_option = function(_, name, v) opt_set(name, v, "local") end
api.nvim_get_option_info2 = function(name, _)
  local i = info(name)
  if not i then error("Invalid option name: '" .. name .. "'") end
  return { name = i.name, type = ({ bool = "boolean", int = "number", text = "string" })[i.kind], default = typed(i, i.default) }
end
api.nvim_set_keymap = function(m, lhs, rhs, o)
  o = o or {}
  if o.callback then return vim.keymap.set(m, lhs, o.callback, o) end
  vim.keymap.set(m, lhs, rhs, vim.tbl_extend("force", o, { remap = not o.noremap }))
end
api.nvim_buf_set_keymap = function(_, m, lhs, rhs, o) api.nvim_set_keymap(m, lhs, rhs, vim.tbl_extend("force", o or {}, { buffer = true })) end
api.nvim_del_keymap = function(m, lhs) vim.keymap.del(m, lhs) end
api.nvim_buf_del_keymap = function(_, m, lhs) vim.keymap.del(m, lhs) end
api.nvim_create_user_command = function(name, run, o)
  o = o or {}
  local nargs = o.nargs and ("-nargs=" .. tostring(o.nargs) .. " ") or ""
  local flags = nargs .. (o.bang and "-bang " or "") .. (o.range and "-range " or "") .. (o.count and "-count " or "")
  if type(run) == "string" then return ex("command! " .. flags .. name .. " " .. run) end
  local n = keep(run)
  ex("command! " .. flags .. name .. " lua vim._cb[" .. n .. "]({ name = '" .. name .. "', args = <q-args>, fargs = vim.split(<q-args>, '%s+', { trimempty = true }), bang = ('<bang>' == '!'), line1 = <line1>, line2 = <line2>, range = <range>, count = <count> })")
  -- an M-x / command-palette name of its own, as zett's commands have
  pcall(z.command, name, function() run({ name = name, args = "", fargs = {}, bang = false }) end, { desc = o.desc or name })
end
api.nvim_buf_create_user_command = function(_, name, run, o) api.nvim_create_user_command(name, run, o) end
api.nvim_del_user_command = function(name) ex("delcommand " .. name) end
api.nvim_buf_del_user_command = function(_, name) ex("delcommand " .. name) end
local groups, group_names = {}, {}
api.nvim_create_augroup = function(name, o)
  if not groups[name] then
    group_names[#group_names + 1] = name
    groups[name] = #group_names
  end
  if not o or o.clear ~= false then ex("autocmd! " .. name) end
  ex("augroup " .. name .. " | augroup END")
  return groups[name]
end
api.nvim_del_augroup_by_name = function(name) ex("augroup! " .. name) end
api.nvim_del_augroup_by_id = function(id) if group_names[id] then ex("augroup! " .. group_names[id]) end end
local autocmds = {}
api.nvim_create_autocmd = function(events, o)
  o = o or {}
  if type(events) == "string" then events = { events } end
  local group = type(o.group) == "number" and group_names[o.group] or o.group
  local pattern = o.pattern or (o.buffer and "<buffer>") or "*"
  if type(pattern) == "table" then pattern = table.concat(pattern, ",") end
  local id = #autocmds + 1
  local run
  if o.callback then
    local cb = o.callback
    if type(cb) == "string" then local name = cb cb = function(a) return vim.fn[name](a) end end
    local n = keep(function(ev)
      if autocmds[id] == false then return end
      if o.once then autocmds[id] = false end
      ev.id, ev.group = id, o.group
      if cb(ev) == true then autocmds[id] = false end
    end)
    run = "lua vim._cb[" .. n .. "]({ event = '%s', match = vim.fn.expand('<amatch>'), file = vim.fn.expand('<afile>'), buf = vim.api.nvim_get_current_buf() })"
  else
    run = o.command
  end
  autocmds[id] = true
  for _, e in ipairs(events) do
    ex("autocmd " .. (group and group .. " " or "") .. e .. " " .. pattern .. " " .. (o.callback and run:format(e) or run))
  end
  return id
end
api.nvim_del_autocmd = function(id) autocmds[id] = false end
api.nvim_clear_autocmds = function(o)
  o = o or {}
  local group = type(o.group) == "number" and group_names[o.group] or o.group
  local ev = type(o.event) == "table" and table.concat(o.event, ",") or o.event
  ex("autocmd! " .. (group or "") .. (ev and (" " .. ev) or ""))
end
api.nvim_exec_autocmds = function(events, o)
  if type(events) == "string" then events = { events } end
  for _, e in ipairs(events) do ex("doautocmd " .. ((o or {}).group and (o.group .. " ") or "") .. e) end
end
api.nvim_echo = function(chunks, _, _)
  z.notify(table.concat(vim.tbl_map(function(c) return c[1] end, chunks), ""))
end
api.nvim_out_write = function(s) z.notify((s:gsub("\n$", ""))) end
api.nvim_err_write = function(s) z.notify("E: " .. s:gsub("\n$", "")) end
api.nvim_err_writeln = function(s) z.notify("E: " .. s) end
api.nvim_notify = function(msg, level, _) vim.notify(msg, level) end
api.nvim_set_hl = function(_, name, v)
  if v.link then return ex("highlight! link " .. name .. " " .. v.link) end
  local attrs = {}
  for _, a in ipairs({ "bold", "italic", "underline", "undercurl", "strikethrough", "reverse" }) do if v[a] then attrs[#attrs + 1] = a end end
  local function col(c) return type(c) == "number" and string.format("#%06x", c) or c end
  local parts = { "highlight", name }
  if v.fg or v.foreground then parts[#parts + 1] = "guifg=" .. col(v.fg or v.foreground) end
  if v.bg or v.background then parts[#parts + 1] = "guibg=" .. col(v.bg or v.background) end
  if v.sp or v.special then parts[#parts + 1] = "guisp=" .. col(v.sp or v.special) end
  if #attrs > 0 then parts[#parts + 1] = "gui=" .. table.concat(attrs, ",") end
  if v.ctermfg then parts[#parts + 1] = "ctermfg=" .. v.ctermfg end
  if v.ctermbg then parts[#parts + 1] = "ctermbg=" .. v.ctermbg end
  if #parts == 2 then parts[#parts + 1] = "NONE" end
  ex(table.concat(parts, " "))
end
api.nvim_create_namespace = function(name)
  vim._ns = vim._ns or {}
  if not vim._ns[name] then vim._ns[name] = vim.tbl_count(vim._ns) + 1 end
  return vim._ns[name]
end
api.nvim_get_namespaces = function() return vim._ns or {} end
api.nvim_replace_termcodes = function(s, _, _, _) return s end
-- nvim_feedkeys: typed in the mode the editor is in ("n" in mode: no mappings)
api.nvim_feedkeys = function(keys, mode, _) if keys ~= "" then z.feed_keys(keys, not tostring(mode or "m"):find("n")) end end
api.nvim_input = function(keys) ex("normal " .. keys) return #keys end
api.nvim_get_mode = function() return { mode = vim.fn.mode(), blocking = false } end
api.nvim_eval = function(e) return vim._vimscript and vim._vimscript.eval(e) or nil end
api.nvim_call_function = function(f, args) return vim.fn[f](unpack(args or {})) end
api.nvim_strwidth = function(s) return utf8.len(s) or #s end
api.nvim_set_current_dir = function(d) ex("cd " .. d) end
api.nvim_list_runtime_paths = function() return { z.config_dir() } end
api.nvim_get_runtime_file = function(name, all)
  local p = z.config_dir() .. "/" .. name
  if z.fs.exists(p) then return { p } end
  return {}
end
api.nvim_get_chan_info = function() return {} end
api.nvim_list_uis = function() return { { width = tonumber(z.opt.columns) or 80, height = tonumber(z.opt.lines) or 24, rgb = true } } end   -- the real size, as the terminal draws it
api.nvim_get_color_by_name = function(n)
  local h = n:match("^#(%x%x%x%x%x%x)$")
  return h and tonumber(h, 16) or -1
end

-- ── vim.fn: Vimscript's functions ──
local fn = {}
local function bool(b) return b and 1 or 0 end
local function home(p) return (p:gsub("^~", os.getenv("HOME") or "~")) end
local function modify(p, mods)
  for m in mods:gmatch(":(%a)") do
    if m == "p" then p = p:sub(1, 1) == "/" and p or (z.root() .. "/" .. p)
    elseif m == "h" then p = p:match("^(.*)/[^/]*$") or "."
    elseif m == "t" then p = p:match("([^/]*)$")
    elseif m == "r" then p = p:match("^(.*)%.[^./]*$") or p
    elseif m == "e" then p = p:match("%.([^./]*)$") or ""
    elseif m == "~" then local h = os.getenv("HOME") or "" if vim.startswith(p, h) then p = "~" .. p:sub(#h + 1) end
    end
  end
  return p
end
function fn.fnamemodify(p, mods) return modify(home(p), mods) end
function fn.expand(s, _, list)
  local r
  local base, mods = s:match("^([%%#]<?%a*)(.*)$")
  if s:match("^<cword>") or s:match("^<cWORD>") then
    local row, col = z.buf.cursor()
    local l = lines(row - 1, row)[1] or ""
    local pat = s:match("^<cword>") and "[%w_]" or "%S"
    local a, b = col + 1, col + 1
    while a > 1 and l:sub(a - 1, a - 1):match(pat) do a = a - 1 end
    while b <= #l and l:sub(b, b):match(pat) do b = b + 1 end
    r = l:sub(a, b - 1)
  elseif s:match("^<sfile>") or s:match("^<script>") then r = ""
  elseif s:match("^<amatch>") or s:match("^<afile>") then r = z.buf.path()
  elseif base then
    r = base:sub(1, 1) == "%" and z.buf.path() or ""
    r = modify(r, (s:match("^[%%#](.*)$")))
  else
    r = home(s):gsub("%$(%w+)", function(v) return os.getenv(v) or "" end)
    if r:find("[*?]") then return fn.glob(r, false, list) end
  end
  if list then return { r } end
  return r
end
function fn.glob(pat, _, list)
  local out = {}
  local p = io.popen("for f in " .. pat:gsub("([^%w*?./~_-])", "\\%1") .. "; do [ -e \"$f\" ] && echo \"$f\"; done 2>/dev/null")
  if p then for l in p:lines() do out[#out + 1] = l end p:close() end
  if list then return out end
  return table.concat(out, "\n")
end
function fn.globpath(dirs, pat, _, list)
  local out = {}
  for d in tostring(dirs):gmatch("[^,]+") do vim.list_extend(out, fn.glob(d .. "/" .. pat, false, true)) end
  if list then return out end
  return table.concat(out, "\n")
end
function fn.filereadable(p) p = home(p) local f = io.open(p, "r") if f then f:close() return bool(not z.fs.is_dir(p)) end return 0 end
function fn.filewritable(p) return bool(z.fs.exists(home(p)) and not z.fs.is_dir(home(p))) end
function fn.isdirectory(p) return bool(z.fs.is_dir(home(p))) end
function fn.mkdir(p, _, _) z.fs.mkdir(home(p)) return 1 end
function fn.delete(p, _) local ok = pcall(z.fs.remove, home(p)) return ok and 0 or -1 end
function fn.rename(a, b) local ok = pcall(z.fs.rename, home(a), home(b)) return ok and 0 or -1 end
function fn.readfile(p, _, max)
  local out = {}
  local f = io.open(home(p), "r")
  if not f then error("E484: Can't open file " .. p) end
  for l in f:lines() do
    out[#out + 1] = l
    if max and max > 0 and #out >= max then break end
  end
  f:close()
  return out
end
function fn.writefile(l, p, flags)
  local f = io.open(home(p), (flags or ""):find("a") and "a" or "w")
  if not f then return -1 end
  if type(l) == "string" then l = { l } end
  f:write(table.concat(l, "\n") .. (((flags or ""):find("b")) and "" or "\n"))
  f:close()
  return 0
end
function fn.getcwd() return z.root() end
function fn.chdir(d) local old = z.root() ex("cd " .. d) return old end
function fn.executable(name)
  if name:find("/") then return bool(z.fs.exists(name)) end
  for d in (os.getenv("PATH") or ""):gmatch("[^:]+") do if z.fs.exists(d .. "/" .. name) and not z.fs.is_dir(d .. "/" .. name) then return 1 end end
  return 0
end
function fn.exepath(name)
  for d in (os.getenv("PATH") or ""):gmatch("[^:]+") do if z.fs.exists(d .. "/" .. name) then return d .. "/" .. name end end
  return ""
end
function fn.resolve(p) return p end
function fn.simplify(p) return (p:gsub("/%./", "/"):gsub("//+", "/")) end
function fn.tempname() return os.tmpname() end
function fn.getftime(p) return z.fs.mtime and (z.fs.mtime(home(p)) or -1) or -1 end
function fn.getfsize(p) if not z.fs.exists(home(p)) then return -1 end return z.fs.size and z.fs.size(home(p)) or 0 end
function fn.stdpath(what)
  local h = os.getenv("HOME") or ""
  local x = function(v, d) return (os.getenv(v) or (h .. d)) .. "/zett" end
  local cfg = type(z.config_dir) == "function" and z.config_dir() or z.config_dir
  local m = { config = cfg, data = x("XDG_DATA_HOME", "/.local/share"), state = x("XDG_STATE_HOME", "/.local/state"), cache = x("XDG_CACHE_HOME", "/.cache"), run = (os.getenv("XDG_RUNTIME_DIR") or "/tmp") .. "/zett" }
  if what == "config_dirs" or what == "data_dirs" then return {} end
  return m[what] or ""
end
function fn.getenv(n) local v = vim.env[n] if v == nil then return vim.NIL end return v end
function fn.setenv(n, v) vim.env[n] = v end
function fn.environ()
  local r = {}
  local p = io.popen("env")
  if p then for l in p:lines() do local k, v = l:match("^([^=]+)=(.*)$") if k then r[k] = v end end p:close() end
  return r
end
function fn.hostname() local p = io.popen("hostname") local h = p and p:read("l") or "" if p then p:close() end return h end
function fn.system(cmd, input)
  if type(cmd) == "table" then cmd = table.concat(vim.tbl_map(fn.shellescape, cmd), " ") end
  local tmp
  if input then
    tmp = os.tmpname()
    local f = io.open(tmp, "w")
    f:write(type(input) == "table" and table.concat(input, "\n") or input)
    f:close()
    cmd = cmd .. " < " .. tmp
  end
  local p = io.popen(cmd .. " 2>&1")
  local out = p and p:read("a") or ""
  local _, _, code = p:close()
  vim.v.shell_error = code or 0
  if tmp then os.remove(tmp) end
  return out
end
function fn.systemlist(cmd, input) return vim.split(fn.system(cmd, input):gsub("\n$", ""), "\n", { plain = true }) end
function fn.shellescape(s) return "'" .. tostring(s):gsub("'", "'\\''") .. "'" end
function fn.fnameescape(s) return (tostring(s):gsub("([ \\%%#|\"'*?%[{}])", "\\%1")) end
function fn.escape(s, chars) return (s:gsub("[" .. vim.pesc(chars) .. "]", "\\%0")) end

-- buffer and cursor
function fn.line(what)
  local r, _ = z.buf.cursor()
  if what == "." then return r end
  if what == "$" then return z.buf.line_count() end
  if what == "w0" then return 1 end
  if what == "w$" then return z.buf.line_count() end
  return 0
end
function fn.col(what)
  local r, c = z.buf.cursor()
  if what == "." then return c + 1 end
  if what == "$" then return #(lines(r - 1, r)[1] or "") + 1 end
  return 0
end
fn.virtcol = fn.col
function fn.getline(a, b)
  local function n(x) if type(x) == "string" then return fn.line(x) end return x end
  if b == nil then return lines(n(a) - 1, n(a))[1] or "" end
  return lines(n(a) - 1, n(b))
end
function fn.setline(a, t)
  if type(a) == "string" then a = fn.line(a) end
  if type(t) ~= "table" then t = { t } end
  local count = z.buf.line_count()
  for i, s in ipairs(t) do
    local l = a + i - 1
    if l <= count then z.buf.set_lines(l - 1, l, { s }) else z.buf.set_lines(-1, -1, { s }) end
  end
  return 0
end
function fn.append(a, t)
  if type(a) == "string" then a = fn.line(a) end
  if type(t) ~= "table" then t = { t } end
  z.buf.set_lines(a, a, t)
  return 0
end
function fn.cursor(l, c) if type(l) == "table" then l, c = l[1], l[2] end z.buf.set_cursor(l, math.max((c or 1) - 1, 0)) return 0 end
function fn.getpos(what)
  local r, c = z.buf.cursor()
  if what == "." then return { 0, r, c + 1, 0 } end
  return { 0, 0, 0, 0 }
end
fn.getcurpos = function() local p = fn.getpos(".") p[5] = p[3] return p end
function fn.setpos(what, p) if what == "." then z.buf.set_cursor(p[2], math.max(p[3] - 1, 0)) return 0 end return -1 end
function fn.bufnr(_) return cur_buf() end
function fn.bufname(b) if b == nil or b == "%" or b == "" then return z.buf.path() end return z.buffers()[tonumber(b) or 0] or "" end
function fn.bufexists(b) return bool(api.nvim_buf_is_valid(tonumber(b) or 0)) end
function fn.buflisted(b) return fn.bufexists(b) end
function fn.winnr() return 1 end
function fn.winheight() return 24 end
function fn.winwidth() return 80 end
function fn.win_getid() return 1000 end
function fn.tabpagenr() return 1 end
function fn.mode() return z.mode and z.mode() or "n" end
function fn.wordcount()
  local text = table.concat(lines(0, -1), "\n")
  local w = 0
  for _ in text:gmatch("%S+") do w = w + 1 end
  return { words = w, chars = utf8.len(text) or #text, bytes = #text }
end
function fn.search(pat, _)
  local l = lines(0, -1)
  local r, _ = z.buf.cursor()
  local lp = vim._lua_pattern(pat)
  for i = r + 1, #l do if l[i]:find(lp) then z.buf.set_cursor(i, (l[i]:find(lp)) - 1) return i end end
  for i = 1, r do if l[i]:find(lp) then z.buf.set_cursor(i, (l[i]:find(lp)) - 1) return i end end
  return 0
end

-- strings
-- A Vim pattern (magic) as a Lua pattern: . * ^ $ [..] \s \d \w \a \l \u \x \+ \= \? \< \>, \(..\)
-- groups; what Lua patterns cannot say (\| \{n,m}) is kept literally
function vim._lua_pattern(p)
  local out, i = {}, 1
  local classes = { s = "%s", S = "%S", d = "%d", D = "%D", w = "[%w_]", W = "[^%w_]", a = "%a", A = "%A", l = "%l", L = "%L", u = "%u", U = "%U", x = "%x", X = "%X", h = "[%a_]", k = "[%w_]", i = "[%w_]", f = "[%w_./-]", t = "\t", e = "\27", r = "\r", n = "\n" }
  while i <= #p do
    local c = p:sub(i, i)
    if c == "\\" then
      local n = p:sub(i + 1, i + 1)
      i = i + 2
      if classes[n] then out[#out + 1] = classes[n]
      elseif n == "+" then out[#out + 1] = "+"
      elseif n == "=" or n == "?" then out[#out + 1] = "?"
      elseif n == "(" then out[#out + 1] = "("
      elseif n == ")" then out[#out + 1] = ")"
      elseif n == "<" then out[#out + 1] = "%f[%w_]"
      elseif n == ">" then out[#out + 1] = "%f[^%w_]"
      elseif n == "{" then
        -- \{-} is Vim's lazy *: Lua's -
        local close = p:find("}", i, true)
        if close and p:sub(i, close - 1) == "-" then out[#out + 1] = "-" i = close + 1 else out[#out + 1] = "{" end
      elseif n:match("%p") then out[#out + 1] = "%" .. n
      else out[#out + 1] = n end
    elseif c == "[" then
      local close = p:find("]", i + 2, true)
      if not close then out[#out + 1] = "%[" i = i + 1
      else
        local body = p:sub(i + 1, close - 1):gsub("%%", "%%%%")
        out[#out + 1] = "[" .. body .. "]"
        i = close + 1
      end
    elseif c == "~" then out[#out + 1] = "~" i = i + 1
    elseif c:match("[%(%)%%%+%-%?]") then out[#out + 1] = "%" .. c i = i + 1
    else out[#out + 1] = c i = i + 1 end
  end
  return table.concat(out)
end
local function vpat(p) return vim._lua_pattern(p) end
function fn.len(v) if type(v) == "table" then return vim.tbl_count(v) end return #tostring(v) end
fn.strlen = function(s) return #s end
fn.strchars = function(s) return utf8.len(s) or #s end
fn.strwidth = fn.strchars
fn.strdisplaywidth = fn.strchars
fn.toupper = string.upper
fn.tolower = string.lower
function fn.trim(s, mask, dir)
  if mask and mask ~= "" then
    local m = "[" .. vim.pesc(mask) .. "]"
    if dir ~= 2 then s = s:gsub("^" .. m .. "+", "") end
    if dir ~= 1 then s = s:gsub(m .. "+$", "") end
    return s
  end
  if dir == 1 then return (s:gsub("^%s+", "")) end
  if dir == 2 then return (s:gsub("%s+$", "")) end
  return vim.trim(s)
end
function fn.split(s, pat, keep)
  local r = {}
  if not pat or pat == "" then for w in s:gmatch("%S+") do r[#r + 1] = w end return r end
  r = vim.split(s, vpat(pat))
  if not keep then r = vim.tbl_filter(function(x) return x ~= "" end, r) end
  return r
end
function fn.join(l, sep) return table.concat(vim.tbl_map(tostring, l), sep == nil and " " or sep) end
function fn.substitute(s, pat, rep, flags)
  local lp = vpat(pat)
  local lr = rep:gsub("%%", "%%%%"):gsub("\\(%d)", "%%%1"):gsub("\\&", "\1"):gsub("&", "%%0"):gsub("\1", "&")
  if (flags or ""):find("g") then return (s:gsub(lp, lr)) end
  return (s:gsub(lp, lr, 1))
end
function fn.matchstr(s, pat) return s:match(vpat(pat)) or "" end
function fn.match(s, pat) local a = s:find(vpat(pat)) return a and a - 1 or -1 end
function fn.matchend(s, pat) local _, b = s:find(vpat(pat)) return b or -1 end
function fn.matchlist(s, pat)
  local caps = { s:match("(" .. vpat(pat) .. ")") }
  if #caps == 0 then return {} end
  for i = #caps + 1, 10 do caps[i] = "" end
  return caps
end
function fn.stridx(s, x, start) local a = s:find(x, (start or 0) + 1, true) return a and a - 1 or -1 end
function fn.strridx(s, x)
  local last = -1
  local at = 1
  while true do local a = s:find(x, at, true) if not a then break end last = a - 1 at = a + 1 end
  return last
end
function fn.strpart(s, a, n) return n and s:sub(a + 1, a + n) or s:sub(a + 1) end
function fn.strcharpart(s, a, n)
  local chars = {}
  for _, c in utf8.codes(s) do chars[#chars + 1] = utf8.char(c) end
  return table.concat(chars, "", a + 1, math.min(#chars, n and a + n or #chars))
end
fn["repeat"] = function(s, n)
  if type(s) == "table" then local r = {} for _ = 1, n do vim.list_extend(r, s) end return r end
  return s:rep(n)
end
fn.printf = function(f, ...) return string.format(f, ...) end
function fn.string(v) return vim._vimscript.V.str(v, true) end
function fn.str2nr(s, base) return tonumber(s:match("^%s*([-+]?%w+)") or "", base or 10) or 0 end
function fn.str2float(s) return tonumber(s:match("^%s*([-+]?[%d.eE+-]+)") or "") or 0.0 end
function fn.nr2char(n) return utf8.char(n) end
function fn.char2nr(s) return utf8.codepoint(s, 1) end
function fn.tr(s, from, to)
  local m = {}
  local f, t = {}, {}
  for _, c in utf8.codes(from) do f[#f + 1] = utf8.char(c) end
  for _, c in utf8.codes(to) do t[#t + 1] = utf8.char(c) end
  for i, c in ipairs(f) do m[c] = t[i] end
  return (s:gsub(utf8.charpattern, function(c) return m[c] end))
end
function fn.count(v, x)
  local n = 0
  if type(v) == "string" then local at = 1 while true do local a, b = v:find(x, at, true) if not a then break end n = n + 1 at = b + 1 end return n end
  for _, y in pairs(v) do if y == x then n = n + 1 end end
  return n
end
function fn.toupper(s) return s:upper() end
function fn.tolower(s) return s:lower() end
fn.strtrans = function(s) return s end
fn.iconv = function(s) return s end

-- lists and dictionaries
function fn.empty(v)
  if v == nil or v == vim.NIL or v == false or v == 0 or v == "" then return 1 end
  if type(v) == "table" then return bool(next(v) == nil) end
  return 0
end
function fn.has_key(d, k) return bool(d[k] ~= nil) end
fn.keys = vim.tbl_keys
fn.values = vim.tbl_values
function fn.items(d) local r = {} for k, v in pairs(d) do r[#r + 1] = { k, v } end return r end
function fn.get(t, k, default)
  if type(t) == "table" and vim.islist(t) and type(k) == "number" then k = k + 1 end
  local v = t[k]
  if v == nil then return default end
  return v
end
function fn.add(l, x) l[#l + 1] = x return l end
function fn.insert(l, x, at) table.insert(l, (at or 0) + 1, x) return l end
function fn.remove(l, i, e)
  if not vim.islist(l) then local v = l[i] l[i] = nil return v end
  if e then local r = {} for _ = i, e do r[#r + 1] = table.remove(l, i + 1) end return r end
  return table.remove(l, i + 1)
end
function fn.extend(a, b)
  if vim.islist(a) then return vim.list_extend(a, b) end
  for k, v in pairs(b) do a[k] = v end
  return a
end
fn.copy = function(v) if type(v) ~= "table" then return v end local r = {} for k, x in pairs(v) do r[k] = x end return r end
fn.deepcopy = vim.deepcopy
function fn.filter(l, f)
  if type(f) ~= "function" then error("vim.fn.filter: zett takes a Lua function") end
  if vim.islist(l) then
    local r = {}
    for i, v in ipairs(l) do if f(i - 1, v) then r[#r + 1] = v end end
    for i = #l, 1, -1 do l[i] = nil end
    for i, v in ipairs(r) do l[i] = v end
  else
    for k, v in pairs(l) do if not f(k, v) then l[k] = nil end end
  end
  return l
end
function fn.map(l, f)
  if type(f) ~= "function" then error("vim.fn.map: zett takes a Lua function") end
  for k, v in pairs(l) do l[k] = f(vim.islist(l) and k - 1 or k, v) end
  return l
end
function fn.sort(l, how)
  if type(how) == "function" then table.sort(l, function(a, b) return how(a, b) < 0 end)
  elseif how == "n" or how == "N" then table.sort(l, function(a, b) return tonumber(a) < tonumber(b) end)
  elseif how == "i" or how == 1 then table.sort(l, function(a, b) return tostring(a):lower() < tostring(b):lower() end)
  else table.sort(l, function(a, b) return tostring(a) < tostring(b) end) end
  return l
end
function fn.uniq(l)
  local r = {}
  for _, v in ipairs(l) do if r[#r] ~= v then r[#r + 1] = v end end
  for i = #l, 1, -1 do l[i] = nil end
  for i, v in ipairs(r) do l[i] = v end
  return l
end
function fn.reverse(l) local n = #l for i = 1, math.floor(n / 2) do l[i], l[n - i + 1] = l[n - i + 1], l[i] end return l end
function fn.index(l, x) for i, v in ipairs(l) do if v == x then return i - 1 end end return -1 end
function fn.range(a, b, step)
  if b == nil then a, b = 0, a - 1 end
  local r = {}
  for i = a, b, step or 1 do r[#r + 1] = i end
  return r
end
function fn.min(l) local m for _, v in pairs(l) do if m == nil or v < m then m = v end end return m or 0 end
function fn.max(l) local m for _, v in pairs(l) do if m == nil or v > m then m = v end end return m or 0 end
fn.abs = math.abs
fn.floor = math.floor
fn.ceil = math.ceil
function fn.round(x) return x >= 0 and math.floor(x + 0.5) or math.ceil(x - 0.5) end
fn.sqrt = math.sqrt
fn.pow = function(a, b) return a ^ b end
function fn.float2nr(x) return math.tointeger(x >= 0 and math.floor(x) or math.ceil(x)) or 0 end
function fn.type(v)
  local t = type(v)
  if t == "number" then return math.type(v) == "integer" and 0 or 5 end
  return ({ string = 1, ["function"] = 2, table = vim.islist(v) and 3 or 4, boolean = 6 })[t] or 7
end
function fn.flatten(l) return vim.tbl_flatten(l) end

-- time
function fn.localtime() return os.time() end
function fn.strftime(f, t) return os.date(f, t) end
-- reltime: wall-clock seconds as { s }; reltime(a) is the time since a, reltime(a, b) from a to b
local function wall() return z.hrtime() / 1e9 end
function fn.reltime(a, b)
  if a == nil then return { wall() } end
  return { (b and b[1] or wall()) - (a[1] or 0) }
end
function fn.reltimefloat(t) return t and t[1] or 0 end
function fn.reltimestr(t) return string.format("%10.6f", fn.reltimefloat(t)) end

-- the editor itself
local features = { nvim = 1, zett = 1, unix = 1, linux = 1, termguicolors = 1, lua = 1, ["nvim-0.10"] = 1, ["nvim-0.9"] = 1, ["nvim-0.8"] = 1, ["nvim-0.7"] = 1, ["nvim-0.5"] = 1, syntax = 1, autocmd = 1, clipboard = 1, ["patch-8.1.0000"] = 1, folding = 1, timers = 1, ["vim_starting"] = 0, ttyin = 1, ttyout = 1 }
-- the Neovim level zett speaks (vim.system, vim.lsp.completion, vim.snippet are 0.11's):
-- has("nvim-0.11.2") compares against it, as Neovim does
local NVIM = { 0, 11, 2 }
function fn.has(f)
  if features[f] then return features[f] end
  local a, b, c = tostring(f):match("^nvim%-(%d+)%.(%d+)%.?(%d*)$")
  if a then
    local want = { tonumber(a), tonumber(b), tonumber(c) or 0 }
    for i = 1, 3 do
      if NVIM[i] ~= want[i] then return NVIM[i] > want[i] and 1 or 0 end
    end
    return 1
  end
  return 0
end
function fn.exists(e)
  local kind, name = e:sub(1, 1), e:sub(2)
  if kind == "&" or kind == "+" then return bool(info(name) ~= nil) end
  if kind == "*" then return bool(rawget(fn, name) ~= nil or (vim._vimfn or {})[name] ~= nil) end
  if kind == ":" then return 2 end
  local scope, var = e:match("^([gbwtv]):(.+)$")
  if scope then return bool(vim[scope][var] ~= nil) end
  if kind == "$" then return bool(os.getenv(name) ~= nil) end
  return 0
end
function fn.json_encode(v) return json_encode(v) end
function fn.json_decode(s) return json_decode(s) end
function fn.input(prompt, default) return default or "" end
function fn.confirm() return 1 end
function fn.inputlist() return 0 end
function fn.getchar() return 0 end
function fn.visualmode() return "" end
function fn.function_(name) return function(...) return vim.fn[name](...) end end
fn["function"] = fn.function_
fn.funcref = fn.function_
function fn.call(f, args) if type(f) == "string" then return vim.fn[f](unpack(args)) end return f(unpack(args)) end
function fn.execute(c) if type(c) == "table" then for _, x in ipairs(c) do vim.cmd(x) end else vim.cmd(c) end return "" end
function fn.eval(e) return api.nvim_eval(e) end
function fn.luaeval(e) return load("return " .. e)() end
function fn.timer_start(ms, f, o)
  local t = vim.uv.new_timer()
  local rep = o and o["repeat"] or 0
  local n = 0
  t:start(ms, rep ~= 0 and ms or 0, function()
    n = n + 1
    f(1)
    if rep > 0 and n >= rep then t:stop() end
  end)
  vim._timers = vim._timers or {}
  vim._timers[#vim._timers + 1] = t
  return #vim._timers
end
function fn.timer_stop(id) if vim._timers and vim._timers[id] then vim._timers[id]:stop() end end
function fn.hlexists(_) return 1 end
function fn.hlID(_) return 1 end
function fn.synID() return 0 end
function fn.foldlevel() return 0 end
function fn.foldclosed() return -1 end
-- foldtext(): Vim's own, "+--  N lines: the first line" (blanks, comment marks and {{{ taken off)
function fn.foldtext()
  local s, e = vim.v.foldstart or 1, vim.v.foldend or 1
  local t = (fn.getline(s) or ""):gsub("{{{%d*", ""):gsub("^%s*/?%*+%s*", ""):gsub("%s*%*+/%s*$", ""):gsub("^%s*%-%-%s*", ""):gsub("^%s*#%s*", ""):gsub("^%s+", "")
  return string.format("+-%s%3d lines: %s", vim.v.folddashes or "", e - s + 1, t)
end
function fn.indent(l) local s = fn.getline(l) return #(s:match("^%s*")) end
function fn.shiftwidth() local sw = opt_get("shiftwidth") if sw == 0 then return opt_get("tabstop") end return sw end
function fn.pumvisible() return 0 end
function fn.getreg(_) return "" end
function fn.setreg() return 0 end
function fn.serverlist() return {} end
function fn.jobstart() return -1 end
function fn.argv(i)
  local a = vim.g._zett_argv or {}
  if i == nil then return a end
  return a[i + 1] or ""
end
function fn.argc() return #(vim.g._zett_argv or {}) end

vim.fn = setmetatable(vim.fn or {}, {
  __index = function(_, name)
    local f = fn[name]
    if f then return f end
    -- a Vimscript function defined with :function (vimscript.lua)
    local user = vim._vimfn and vim._vimfn[name]
    if user then return user end
    return function() error("zett: vim.fn." .. name .. " is not there yet (:help vim-api)") end
  end,
})
-- for listing (pairs over vim.fn names what zett has)
for k, v in pairs(fn) do rawset(vim.fn, k, v) end

-- ── vim.fs ──
vim.fs = vim.fs or {}
function vim.fs.normalize(p) return (home(p):gsub("//+", "/"):gsub("(.)/$", "%1")) end
function vim.fs.basename(p) return p:match("([^/]*)/*$") end
function vim.fs.dirname(p) return p:match("^(.*)/[^/]*$") or "." end
function vim.fs.joinpath(...) return (table.concat({ ... }, "/"):gsub("//+", "/")) end
function vim.fs.root(_, markers)
  if type(markers) == "string" then markers = { markers } end
  local d = z.root()
  while d and d ~= "" do
    for _, m in ipairs(markers or {}) do if z.fs.exists(d .. "/" .. m) then return d end end
    d = d:match("^(.*)/[^/]*$")
  end
  return nil
end

-- ── the rest ──
function vim.version() return { major = NVIM[1], minor = NVIM[2], patch = NVIM[3], api_level = 13, prerelease = false } end
vim.has = function(f) return fn.has(f) == 1 end
function vim.region() return {} end
vim.lsp = vim.lsp or { buf = {}, util = {}, protocol = {}, handlers = {}, config = function() end, enable = function() end }
-- vim.lsp.buf: zett's own LSP actions (format waits, so a BufWritePre format is in the write)
vim.lsp.buf.format = function(o) z.ui("lspFormat", tostring((o or {}).timeout_ms or 1000)) end
vim.lsp.buf.hover = function() z.ui("hover", "") end
vim.lsp.buf.definition = function() z.ui("definition", "") end
vim.lsp.buf.references = function() z.ui("references", "") end
vim.lsp.buf.document_symbol = function() z.ui("symbols", "") end
vim.lsp.buf.rename = function(name)
  if name and name ~= "" then z.ui("ex", "rename " .. name) else z.ui("rename", "") end
end
vim.diagnostic = vim.diagnostic or { config = function() end, severity = { ERROR = 1, WARN = 2, INFO = 3, HINT = 4 }, get = function() return {} end }
vim.diagnostic.goto_next = function() z.ui("diag", "next") end
vim.diagnostic.goto_prev = function() z.ui("diag", "prev") end
vim.diagnostic.jump = function(o) z.ui("diag", (o and (o.count or 1) < 0) and "prev" or "next") end
-- vim.snippet (Neovim 0.10+): zett's own snippet session; its stops move with every edit
vim.snippet = {
  expand = function(body) z.ui("snippetExpand", body) end,
  jump = function(dir) z.ui("snippetJump", tostring(dir or 1)) end,
  active = function(f)
    if not vim.g._zett_snippet then return false end
    return f == nil or f.direction == nil or true
  end,
  stop = function() z.ui("snippetStop", "") end,
}
-- vim.lsp.completion (0.11): zett's popup is this; enable() turns 'autocomplete' on, get() asks now
vim.lsp.completion = {
  enable = function(on, _, _, o)
    if on ~= false and (o == nil or o.autotrigger ~= false) then vim.o.autocomplete = true end
  end,
  get = function() z.ui("compNow", "") end,
}
vim.treesitter = vim.treesitter or { start = function() end, stop = function() end }
vim.filetype = vim.filetype or { add = function() end, match = function() return nil end }
vim.ui = vim.ui or {}
function vim.ui.select(items, o, on_choice)
  o = o or {}
  local fmt = o.format_item or tostring
  local by = {}
  for i, it in ipairs(items) do by[i .. ": " .. fmt(it)] = it end
  z.input({ prompt = (o.prompt or "Select") .. " (" .. #items .. "): " }, function(answer)
    local n = tonumber((answer or ""):match("^%s*(%d+)"))
    if n and items[n] then on_choice(items[n], n) else on_choice(nil, nil) end
  end)
end
function vim.ui.input(o, on_confirm)
  o = o or {}
  z.input({ prompt = o.prompt or "", default = o.default or "" }, function(a) on_confirm(a) end)
end
function vim.ui.open(p) return fn.system({ "xdg-open", p }) end

-- every vim.api function is a vim.fn one too (Neovim: vim.fn.nvim_buf_get_lines), the same function
for k, v in pairs(vim.api) do
  if type(v) == "function" and rawget(vim.fn, k) == nil then rawset(vim.fn, k, v) end
end

-- ── extmarks: highlights, signs and virtual text plugins put on a buffer (zett draws them) ──
-- marks[buf][ns][id] = { row, col, opts } (0-based, columns in bytes, as Neovim); they stay where
-- they were put (a plugin that wants them to follow the text sets them again after a change)
local marks, mark_ids, mark_seq = {}, {}, 0
local function mbuf(b) if b == nil or b == 0 then return cur_buf() end return b end
local function bump() mark_seq = mark_seq + 1 end

api.nvim_buf_set_extmark = function(b, ns, row, col, o)
  o = o or {}
  b = mbuf(b)
  marks[b] = marks[b] or {}
  marks[b][ns] = marks[b][ns] or {}
  local id = o.id
  if not id then mark_ids[ns] = (mark_ids[ns] or 0) + 1 id = mark_ids[ns] end
  marks[b][ns][id] = { row = row, col = col, o = o }
  bump()
  return id
end

local function details(m, ns)
  local d = {}
  for k, v in pairs(m.o) do if k ~= "id" then d[k] = v end end
  d.ns_id = ns
  d.end_row = m.o.end_row or m.o.end_line
  return d
end

api.nvim_buf_get_extmark_by_id = function(b, ns, id, o)
  local m = ((marks[mbuf(b)] or {})[ns] or {})[id]
  if not m then return {} end
  if o and o.details then return { m.row, m.col, details(m, ns) } end
  return { m.row, m.col }
end

-- start / end: a row, or { row, col }; -1 the end. ns -1: every namespace
api.nvim_buf_get_extmarks = function(b, ns, s, e, o)
  o = o or {}
  local function pos(p, last)
    if type(p) == "table" then return p[1], p[2] or 0 end
    if p == -1 then return math.huge, math.huge end
    return p, last and math.huge or 0
  end
  local sr, sc = pos(s, false)
  local er, ec = pos(e, true)
  local out = {}
  for n, list in pairs(marks[mbuf(b)] or {}) do
    if ns == -1 or n == ns then
      for id, m in pairs(list) do
        local after = m.row > sr or (m.row == sr and m.col >= sc)
        local before = m.row < er or (m.row == er and m.col <= ec)
        if after and before and (not o.type or (o.type == "highlight" and m.o.hl_group) or (o.type == "sign" and m.o.sign_text) or (o.type == "virt_text" and m.o.virt_text)) then
          out[#out + 1] = o.details and { id, m.row, m.col, details(m, n) } or { id, m.row, m.col }
        end
      end
    end
  end
  table.sort(out, function(x, y) return x[2] < y[2] or (x[2] == y[2] and (x[3] < y[3] or (x[3] == y[3] and x[1] < y[1]))) end)
  if o.limit and o.limit >= 0 then while #out > o.limit do table.remove(out) end end
  return out
end

api.nvim_buf_del_extmark = function(b, ns, id)
  local list = (marks[mbuf(b)] or {})[ns]
  if not (list and list[id]) then return false end
  list[id] = nil
  bump()
  return true
end

-- the namespace's marks on lines l1..l2 (l2 -1: to the end); ns -1: every namespace
api.nvim_buf_clear_namespace = function(b, ns, l1, l2)
  l1, l2 = l1 or 0, (l2 == nil or l2 < 0) and math.huge or l2
  for n, list in pairs(marks[mbuf(b)] or {}) do
    if ns == -1 or n == ns then
      for id, m in pairs(list) do
        if m.row >= l1 and m.row < l2 then list[id] = nil end
      end
    end
  end
  bump()
end
api.nvim_buf_clear_highlight = api.nvim_buf_clear_namespace

-- the old way to highlight a range (col_end -1: to the end of the line); ns 0: a new namespace
api.nvim_buf_add_highlight = function(b, ns, group, line, cs, ce)
  if ns == 0 or ns == nil then ns = api.nvim_create_namespace("") end
  api.nvim_buf_set_extmark(b, ns, line, cs, { end_row = line, end_col = (ce == nil or ce < 0) and 2147483647 or ce, hl_group = group })
  return ns
end

api.nvim_buf_set_virtual_text = function(b, ns, line, chunks, _)
  if ns == 0 or ns == nil then ns = api.nvim_create_namespace("") end
  api.nvim_buf_set_extmark(b, ns, line, 0, { virt_text = chunks })
  return ns
end

-- for the window: how often marks changed, and buffer b's marks on rows a..e as lines of
-- row col end_row end_col priority hl line_hl sign_text sign_hl num_hl virt_pos virt (text\31hl\31…)
function vim._marks_seq() return mark_seq end
-- virt_lines as one field: lines by \29, each line's (text, group) pieces by \31
local function mark_vlines(v)
  if not v then return "" end
  local ls = {}
  for _, line in ipairs(v) do
    local cs = {}
    for _, c in ipairs(line) do
      local hl = type(c[2]) == "table" and table.concat(c[2], "+") or c[2]
      cs[#cs + 1] = tostring(c[1] or "") .. "\31" .. tostring(hl or "")
    end
    ls[#ls + 1] = #cs > 0 and table.concat(cs, "\31") or " \31"
  end
  return table.concat(ls, "\29")
end

function vim._marks_for(b, a, e)
  local out = {}
  for _, list in pairs(marks[b] or {}) do
    for _, m in pairs(list) do
      local o = m.o
      local er = o.end_row or o.end_line or m.row
      if er >= a and m.row <= e then
        local vt = {}
        for _, c in ipairs(o.virt_text or {}) do
          local hl = type(c[2]) == "table" and c[2][#c[2]] or c[2]
          vt[#vt + 1] = tostring(c[1] or "") .. "\31" .. tostring(hl or "")
        end
        out[#out + 1] = table.concat({ m.row, m.col, er, o.end_col or (o.end_row and 0) or -1, o.priority or (o.sign_text and 10 or 4096), o.hl_group or "", o.line_hl_group or "", o.sign_text or "", o.sign_hl_group or "", o.number_hl_group or "", o.virt_text_pos or "eol", table.concat(vt, "\31"), o.conceal and ("=" .. o.conceal) or "", mark_vlines(o.virt_lines), o.virt_lines_above and "1" or "" }, "\t")
      end
    end
  end
  return table.concat(out, "\30")
end

-- vim.highlight.range / vim.hl.range: a range in a group, through an extmark
vim.hl = vim.hl or {}
vim.hl.range = vim.hl.range or function(b, ns, group, s, e, o)
  o = o or {}
  return api.nvim_buf_set_extmark(b, ns, s[1], s[2], { end_row = e[1], end_col = e[2] < 0 and 2147483647 or e[2], hl_group = group, priority = o.priority })
end
vim.highlight = vim.highlight or vim.hl

for _, k in ipairs({ "nvim_buf_set_extmark", "nvim_buf_get_extmark_by_id", "nvim_buf_get_extmarks", "nvim_buf_del_extmark", "nvim_buf_clear_namespace", "nvim_buf_clear_highlight", "nvim_buf_add_highlight", "nvim_buf_set_virtual_text" }) do
  rawset(vim.fn, k, api[k])
end

-- ── more of vim.fn: arithmetic, bits, lists, buffers and lines (only what is not there yet) ──
local function def(name, f)
  if rawget(vim.fn, name) == nil then
    fn[name] = f
    rawset(vim.fn, name, f)
  end
end
for _, n in ipairs({ "acos", "asin", "atan", "cos", "cosh", "exp", "log", "log10", "sin", "sinh", "tan", "tanh", "sqrt" }) do
  local m = math[n] or (n == "cosh" and function(x) return (math.exp(x) + math.exp(-x)) / 2 end)
    or (n == "sinh" and function(x) return (math.exp(x) - math.exp(-x)) / 2 end)
    or (n == "tanh" and function(x) local a, b = math.exp(x), math.exp(-x) return (a - b) / (a + b) end)
  if m then def(n, function(x) return m(tonumber(x) or 0) end) end
end
def("atan2", function(y, x) return math.atan(tonumber(y) or 0, tonumber(x) or 0) end)
def("fmod", function(a, b) return math.fmod(tonumber(a) or 0, tonumber(b) or 1) end)
def("isinf", function(x) return (x == math.huge and 1) or (x == -math.huge and -1) or 0 end)
def("isnan", function(x) return x ~= x and 1 or 0 end)
def("and", function(a, b) return (math.tointeger(a) or 0) & (math.tointeger(b) or 0) end)
def("or", function(a, b) return (math.tointeger(a) or 0) | (math.tointeger(b) or 0) end)
def("xor", function(a, b) return (math.tointeger(a) or 0) ~ (math.tointeger(b) or 0) end)
def("invert", function(a) return ~(math.tointeger(a) or 0) end)
def("list2str", function(l) local t = {} for i, c in ipairs(l or {}) do t[i] = utf8.char(c) end return table.concat(t) end)
def("str2list", function(s) local t = {} for _, c in utf8.codes(s or "") do t[#t + 1] = c end return t end)
def("indexof", function(l, expr) for i, v in ipairs(l or {}) do if (type(expr) == "function" and expr(i - 1, v)) or v == expr then return i - 1 end end return -1 end)
def("foreach", function(l, f) for i, v in ipairs(l or {}) do f(i - 1, v) end return l end)
def("mapnew", function(l, f) local t = {} for i, v in ipairs(l or {}) do t[i] = f(i - 1, v) end return t end)
def("extendnew", function(a, b) local t = {} for k, v in pairs(a or {}) do t[k] = v end if #(a or {}) > 0 then for _, v in ipairs(b or {}) do t[#t + 1] = v end else for k, v in pairs(b or {}) do t[k] = v end end return t end)
def("flattennew", function(l, depth)
  local out = {}
  local function go(x, d) for _, v in ipairs(x) do if type(v) == "table" and (depth == nil or d < depth) then go(v, d + 1) else out[#out + 1] = v end end end
  go(l or {}, 0)
  return out
end)
local next_id = 0
def("id", function(_) next_id = next_id + 1 return string.format("0x%x", next_id) end)
def("getpid", function() local f = io.open("/proc/self/stat") local s = f and f:read("*l") or "" if f then f:close() end return tonumber(s:match("^(%d+)")) or 0 end)
def("isabsolutepath", function(p) return (p or ""):sub(1, 1) == "/" and 1 or 0 end)
def("glob2regpat", function(g)
  local r = (g or ""):gsub("[%.%^%$%[%]\\~]", "\\%0"):gsub("%*%*", "\1"):gsub("%*", "[^/]*"):gsub("\1", ".*"):gsub("%?", ".")
  return "^" .. r .. "$"
end)
def("garbagecollect", function() collectgarbage() return 0 end)
def("foreground", function() return 0 end)
def("keytrans", function(s) return s end)
def("buffer_exists", function(b) return vim.fn.bufexists(b) end)
def("buffer_name", function(b) return vim.fn.bufname(b) end)
def("buffer_number", function(b) return vim.fn.bufnr(b) end)
def("file_readable", function(f) return vim.fn.filereadable(f) end)
def("last_buffer_nr", function() return #z.buffers() end)
def("bufloaded", function(b) return api.nvim_buf_is_valid(type(b) == "number" and b or vim.fn.bufnr(b)) and 1 or 0 end)
def("bufwinnr", function(b) return (vim.fn.bufnr(b) == cur_buf()) and 1 or -1 end)
def("bufwinid", function(b) return (vim.fn.bufnr(b) == cur_buf()) and 1000 or -1 end)
-- the first / last line from lnum on (back) that is not blank, 0 when none
def("nextnonblank", function(l)
  l = vim.fn.line(l) > 0 and vim.fn.line(l) or tonumber(l) or 1
  for n = l, api.nvim_buf_line_count(0) do if (vim.fn.getline(n) or ""):match("%S") then return n end end
  return 0
end)
-- lines and bytes: line2byte (the byte where a line starts, 1-based) and byte2line
def("line2byte", function(l)
  l = tonumber(l) or vim.fn.line(l)
  local n = api.nvim_buf_line_count(0)
  if l < 1 or l > n + 1 then return -1 end
  local b = 1
  for i, s in ipairs(api.nvim_buf_get_lines(0, 0, l - 1, false)) do b = b + #s + 1 end
  return b
end)
def("byte2line", function(byte)
  byte = tonumber(byte) or 0
  local b = 1
  for i, s in ipairs(api.nvim_buf_get_lines(0, 0, -1, false)) do
    if byte < b + #s + 1 then return i end
    b = b + #s + 1
  end
  return -1
end)
-- buffer `b`'s lines a..e (the current buffer's; another's when zett can read it)
def("getbufline", function(b, a, e)
  if vim.fn.bufnr(b) ~= cur_buf() and b ~= "%" and b ~= 0 then return {} end
  local n = api.nvim_buf_line_count(0)
  a = a == "$" and n or tonumber(a) or 1
  e = e == nil and a or (e == "$" and n or tonumber(e) or a)
  return api.nvim_buf_get_lines(0, a - 1, e, false)
end)
def("getbufoneline", function(b, l) return (vim.fn.getbufline(b, l)[1]) or "" end)
def("deletebufline", function(b, a, e)
  local n = api.nvim_buf_line_count(0)
  a = a == "$" and n or tonumber(a) or 1
  e = e == nil and a or (e == "$" and n or tonumber(e) or a)
  api.nvim_buf_set_lines(0, a - 1, e, false, {})
  return 0
end)
def("appendbufline", function(b, l, text)
  l = l == "$" and api.nvim_buf_line_count(0) or tonumber(l) or 0
  api.nvim_buf_set_lines(0, l, l, false, type(text) == "table" and text or { text })
  return 0
end)
def("getbufvar", function(b, name, default)
  if name == "" then return vim.b end
  if name:sub(1, 1) == "&" then local ok, v = pcall(function() return vim.bo[name:sub(2)] end) return ok and v or default end
  local v = vim.b[name]
  if v == nil then return default == nil and "" or default end
  return v
end)
def("getwinvar", function(w, name, default)
  if name:sub(1, 1) == "&" then local ok, v = pcall(function() return vim.wo[name:sub(2)] end) return ok and v or default end
  local v = vim.w and vim.w[name]
  if v == nil then return default == nil and "" or default end
  return v
end)
def("gettabvar", function(_, name, default) local v = vim.t and vim.t[name] if v == nil then return default == nil and "" or default end return v end)
def("gettabwinvar", function(_, w, name, default) return vim.fn.getwinvar(w, name, default) end)
-- assert_*: the results go to v:errors (as in Vim), 0 when it held, 1 when not
vim.v.errors = vim.v.errors or {}
local function fail(msg) table.insert(vim.v.errors, msg) return 1 end
def("assert_equal", function(a, b, msg) if vim.deep_equal and vim.deep_equal(a, b) or a == b then return 0 end return fail(msg or ("Expected " .. vim.inspect(a) .. " but got " .. vim.inspect(b))) end)
def("assert_notequal", function(a, b, msg) if not (vim.deep_equal and vim.deep_equal(a, b) or a == b) then return 0 end return fail(msg or ("Expected not equal to " .. vim.inspect(a))) end)
def("assert_true", function(v, msg) if v == true or (type(v) == "number" and v ~= 0) then return 0 end return fail(msg or "Expected True but got " .. vim.inspect(v)) end)
def("assert_false", function(v, msg) if v == false or v == 0 then return 0 end return fail(msg or "Expected False but got " .. vim.inspect(v)) end)
def("assert_match", function(p, s, msg) if vim.fn.match(s, p) >= 0 then return 0 end return fail(msg or ("Pattern '" .. p .. "' does not match '" .. s .. "'")) end)
def("assert_notmatch", function(p, s, msg) if vim.fn.match(s, p) < 0 then return 0 end return fail(msg or ("Pattern '" .. p .. "' does match '" .. s .. "'")) end)
def("assert_inrange", function(lo, hi, v, msg) if v >= lo and v <= hi then return 0 end return fail(msg or ("Expected range " .. lo .. " - " .. hi .. ", but got " .. v)) end)
def("assert_report", function(msg) return fail(msg) end)
def("assert_fails", function(cmd, err) local ok, e = pcall(vim.cmd, cmd) if not ok and (not err or tostring(e):find(err, 1, true)) then return 0 end return fail("command did not fail: " .. tostring(cmd)) end)
def("assert_exception", function(_) return 0 end)
def("assert_beeps", function(_) return 0 end)
def("assert_nobeep", function(_) return 0 end)
def("assert_equalfile", function(a, b, msg) local function rd(p) local f = io.open(p) local s = f and f:read("*a") if f then f:close() end return s end if rd(a) == rd(b) and rd(a) ~= nil then return 0 end return fail(msg or ("files differ: " .. a .. " " .. b)) end)

-- ── mappings as Neovim reports them (zett.maps) ──
local function modes_of(m) if m == nil or m == "" or m == " " then return { "n", "v", "o" } end return { m } end
local function map_dict(e, mode)
  return { lhs = e.lhs, lhsraw = e.lhs, rhs = e.rhs, desc = e.desc ~= "" and e.desc or nil, noremap = e.noremap and 1 or 0, mode = mode, buffer = 0, silent = 0, expr = 0, nowait = 0, script = 0, sid = 0, callback = e.lua and function() end or nil }
end
api.nvim_get_keymap = function(mode)
  local out = {}
  for _, m in ipairs(modes_of(mode)) do
    for _, e in ipairs(z.maps(m)) do out[#out + 1] = map_dict(e, m) end
  end
  return out
end
api.nvim_buf_get_keymap = function(_, mode) return api.nvim_get_keymap(mode) end
rawset(vim.fn, "nvim_get_keymap", api.nvim_get_keymap)
rawset(vim.fn, "nvim_buf_get_keymap", api.nvim_buf_get_keymap)
local function same_keys(a, b) return z.key_name(a) == z.key_name(b) end
-- maparg(lhs, mode, abbr, dict): the rhs ("" none), or the mapping as a dict
def("maparg", function(lhs, mode, _, dict)
  for _, m in ipairs(modes_of(mode)) do
    for _, e in ipairs(z.maps(m)) do
      if same_keys(e.lhs, lhs) then return (dict == 1 or dict == true) and map_dict(e, m) or e.rhs end
    end
  end
  return (dict == 1 or dict == true) and {} or ""
end)
-- mapcheck(name, mode): a mapping that name starts or that starts with name (its rhs)
def("mapcheck", function(name, mode)
  local k = z.key_name(name)
  for _, m in ipairs(modes_of(mode)) do
    for _, e in ipairs(z.maps(m)) do
      local l = z.key_name(e.lhs)
      if l:sub(1, #k) == k or k:sub(1, #l) == l then return e.rhs end
    end
  end
  return ""
end)
-- hasmapto(what, mode): some mapping's rhs has what in it
def("hasmapto", function(what, mode)
  for _, m in ipairs(modes_of(mode or "nvo")) do
    for _, e in ipairs(z.maps(m == "nvo" and "n" or m)) do
      if (e.rhs or ""):find(what, 1, true) then return 1 end
    end
  end
  return 0
end)
def("maplist", function()
  local out = {}
  for _, m in ipairs({ "n", "v", "o", "i", "c", "x", "s", "t" }) do
    for _, e in ipairs(z.maps(m)) do out[#out + 1] = map_dict(e, m) end
  end
  return out
end)

-- ── windows as zett has them (zett.windows: as last drawn); handles from 1000 as Neovim's ──
local function wins() return z.windows and z.windows() or {} end
local function cur_nr() local n = z.cur_window and z.cur_window() or 0 return n > 0 and n or 1 end
local function handle(nr) return 999 + nr end
local function win_of(w)
  local nr = (w == nil or w == 0) and cur_nr() or (w - 999)
  for i, x in ipairs(wins()) do if x.nr == nr then return x, i end end
  if nr == cur_nr() then return { nr = nr, buf = cur_buf(), width = 80, height = 24, row = 0, col = 0 }, 1 end
end
api.nvim_get_current_win = function() return handle(cur_nr()) end
api.nvim_list_wins = function()
  local r = {}
  for _, x in ipairs(wins()) do r[#r + 1] = handle(x.nr) end
  if #r == 0 then r[1] = handle(cur_nr()) end
  return r
end
api.nvim_win_is_valid = function(w) return win_of(w) ~= nil end
api.nvim_win_get_buf = function(w) local x = win_of(w) return x and x.buf or cur_buf() end
api.nvim_win_get_width = function(w) local x = win_of(w) return x and x.width or 0 end
api.nvim_win_get_height = function(w) local x = win_of(w) return x and x.height or 0 end
api.nvim_win_get_position = function(w) local x = win_of(w) return x and { x.row, x.col } or { 0, 0 } end
api.nvim_win_get_number = function(w) local _, i = win_of(w) return i or 0 end
api.nvim_win_get_tabpage = function() return 1 end
-- the focus to window w (a command after this Lua returns, as the others)
local function focus(w) if w == nil or w == 0 or w == handle(cur_nr()) then return end local _, i = win_of(w) if i then ex(i .. "wincmd w") end end
api.nvim_set_current_win = function(w) focus(w) end
api.nvim_set_current_buf = function(b) ex("buffer " .. b) end
api.nvim_win_set_buf = function(w, b) focus(w) ex("buffer " .. b) end
api.nvim_win_close = function(w, force) focus(w) ex(force and "close!" or "close") end
api.nvim_win_hide = function(w) focus(w) ex("hide") end
api.nvim_win_set_height = function(w, h) focus(w) ex("resize " .. h) end
api.nvim_win_set_width = function(w, n) focus(w) ex("vertical resize " .. n) end
-- f run with window w current: the focus goes there and comes back around it
api.nvim_win_call = function(w, f)
  local back = api.nvim_get_current_win()
  focus(w)
  local r = f()
  focus(back)
  return r
end
api.nvim_buf_call = function(_, f) return f() end
api.nvim_buf_delete = function(b, o) o = o or {} ex((o.unload and "bunload" or "bdelete") .. (o.force and "!" or "") .. " " .. (b == 0 and cur_buf() or b)) end
api.nvim_buf_get_number = function(b) return (b == nil or b == 0) and cur_buf() or b end
api.nvim_tabpage_list_wins = function() return api.nvim_list_wins() end
api.nvim_tabpage_get_win = function() return api.nvim_get_current_win() end
api.nvim_tabpage_is_valid = function(t) return t == 0 or t == 1 end
api.nvim_tabpage_get_number = function() return 1 end
api.nvim_tabpage_set_win = function(_, w) focus(w) end
for _, k in ipairs({ "nvim_get_current_win", "nvim_list_wins", "nvim_win_is_valid", "nvim_win_get_buf", "nvim_win_get_width", "nvim_win_get_height", "nvim_win_get_position", "nvim_win_get_number", "nvim_win_get_tabpage", "nvim_set_current_win", "nvim_set_current_buf", "nvim_win_set_buf", "nvim_win_close", "nvim_win_hide", "nvim_win_set_height", "nvim_win_set_width", "nvim_win_call", "nvim_buf_call", "nvim_buf_delete", "nvim_buf_get_number", "nvim_tabpage_list_wins", "nvim_tabpage_get_win", "nvim_tabpage_is_valid", "nvim_tabpage_get_number", "nvim_tabpage_set_win" }) do
  rawset(vim.fn, k, api[k])
end
-- winnr(), winwidth(), winheight(), win_getid() as these
def("winwidth", function(n) local x = win_of(n == 0 and 0 or (n and handle(n))) return x and x.width or -1 end)
def("winheight", function(n) local x = win_of(n == 0 and 0 or (n and handle(n))) return x and x.height or -1 end)
-- the stubs these were (one window of 80 x 24) replaced by the real windows
local function by_index(n) local l = wins() return n and l[n] end
local function set(name, f) fn[name] = f rawset(vim.fn, name, f) end
set("winnr", function(a)
  if a == "$" then return math.max(#wins(), 1) end
  local _, i = win_of(0)
  return i or 1
end)
set("win_getid", function(n) local x = n and n > 0 and by_index(n) or win_of(0) return x and handle(x.nr) or 0 end)
set("winwidth", function(n) local x = (n == nil or n == 0) and win_of(0) or by_index(n) return x and x.width or -1 end)
set("winheight", function(n) local x = (n == nil or n == 0) and win_of(0) or by_index(n) return x and x.height or -1 end)
set("winbufnr", function(n) local x = (n == nil or n == 0) and win_of(0) or (n >= 1000 and win_of(n) or by_index(n)) return x and x.buf or -1 end)
set("win_id2win", function(w) local _, i = win_of(w) return i or 0 end)

-- ── highlight groups as Neovim reports them (zett.hl_get); ids are places in the sorted names ──
local function hl_id(name)
  for i, n in ipairs(z.hl_names()) do if n == name then return i end end
  return 0
end
local function hl_name(id) return z.hl_names()[id] end
-- nvim_get_hl(ns, { name =, id =, link = }): one group, or every group by name
api.nvim_get_hl = function(_, o)
  o = o or {}
  local name = o.name or (o.id and hl_name(o.id))
  if name then return z.hl_get(name, o.link == false) or {} end
  local all = {}
  for _, n in ipairs(z.hl_names()) do all[n] = z.hl_get(n, o.link == false) or {} end
  return all
end
-- the old ones: foreground / background, links followed
local function old(h)
  local r = {}
  for k, v in pairs(h or {}) do
    if k == "fg" then r.foreground = v elseif k == "bg" then r.background = v elseif k ~= "link" then r[k] = v end
  end
  return r
end
api.nvim_get_hl_by_name = function(name, _) return old(z.hl_get(name, true)) end
api.nvim_get_hl_by_id = function(id, _) return old(z.hl_get(hl_name(id) or "", true)) end
api.nvim_get_hl_id_by_name = function(name)
  local id = hl_id(name)
  if id == 0 then ex("highlight " .. name .. " NONE") return #z.hl_names() + 1 end
  return id
end
for _, k in ipairs({ "nvim_get_hl", "nvim_get_hl_by_name", "nvim_get_hl_by_id", "nvim_get_hl_id_by_name" }) do rawset(vim.fn, k, api[k]) end
set("hlexists", function(name) return z.hl_get(name, false) and 1 or 0 end)
set("hlID", function(name) return hl_id(name) end)
def("highlightID", function(name) return hl_id(name) end)
def("highlight_exists", function(name) return z.hl_get(name, false) and 1 or 0 end)

-- ── scratch buffers and floating windows (nvim_create_buf, nvim_open_win) ──
-- a scratch buffer lives here (its lines, name, options, its own mappings); a float shows one in
-- a zett float panel placed as Neovim places it. Handles: buffers from 10000, floats from 20000.
local vbufs, vnext, floats, fnext = {}, 0, {}, 0
vim._cur_float = nil
local function is_vbuf(b) return type(b) == "number" and vbufs[b] ~= nil end
local function is_float(w) return type(w) == "number" and floats[w] ~= nil end
local function vlines(b) return vbufs[b].lines end
-- what a float shows: its buffer's lines (a real buffer's as they are now)
-- a float's lines with its buffer's extmarks: hl_group ranges and virtual text (eol, right
-- aligned, overlay), as styled pieces { {text, group}, … } for the panel
local function vt_text(vt) local s = {} for _, c in ipairs(vt) do s[#s + 1] = c[1] or "" end return table.concat(s) end
local function vt_group(c) local g = c[2] if type(g) == "table" then g = g[#g] end return g end
local function styled(text, row, list, width)
  local cuts = {}
  local eol, right, over = {}, {}, {}
  for _, ns in pairs(list) do
    for _, m in pairs(ns) do
      if m.row == row then
        local o = m.o
        if o.hl_group and (o.end_col or o.end_row) then
          local e = (o.end_row and o.end_row > row) and #text or (o.end_col or #text)
          cuts[#cuts + 1] = { m.col, e, o.hl_group, o.priority or 4096 }
        end
        if o.virt_text then
          local pos = o.virt_text_pos or "eol"
          if pos == "overlay" then over[#over + 1] = { m.col, o.virt_text }
          elseif pos == "right_align" or pos == "eol_right_align" then right[#right + 1] = o.virt_text
          else eol[#eol + 1] = o.virt_text end
        end
      end
    end
  end
  if #cuts == 0 and #eol == 0 and #right == 0 and #over == 0 then return text end
  -- per byte: the group of the highest-priority range over it
  local grp = {}
  table.sort(cuts, function(a, b) return a[4] < b[4] end)
  for _, c in ipairs(cuts) do for i = c[1] + 1, math.min(c[2], #text) do grp[i] = c[3] end end
  -- overlays replace the text under them
  for _, ov in ipairs(over) do
    local at = ov[1]
    for _, c in ipairs(ov[2]) do
      local s = c[1] or ""
      if #text < at then text = text .. string.rep(" ", at - #text) end
      text = text:sub(1, at) .. s .. text:sub(at + #s + 1)
      for i = at + 1, at + #s do grp[i] = vt_group(c) end
      at = at + #s
    end
  end
  local out, start = {}, 1
  for i = 2, #text + 1 do
    if i > #text or grp[i] ~= grp[start] then
      out[#out + 1] = { text:sub(start, i - 1), grp[start] or "" }
      start = i
    end
  end
  local used = vim.fn.strdisplaywidth(text)
  for _, vt in ipairs(eol) do
    out[#out + 1] = { " ", "" }
    used = used + 1
    for _, c in ipairs(vt) do out[#out + 1] = { c[1] or "", vt_group(c) or "" } used = used + vim.fn.strdisplaywidth(c[1] or "") end
  end
  for _, vt in ipairs(right) do
    local w = vim.fn.strdisplaywidth(vt_text(vt))
    local pad = (width or 0) - used - w
    if pad > 0 then out[#out + 1] = { string.rep(" ", pad), "" } used = used + pad end
    for _, c in ipairs(vt) do out[#out + 1] = { c[1] or "", vt_group(c) or "" } end
    used = used + w
  end
  return out
end
local function float_lines(h)
  local b = floats[h].buf
  local l
  if is_vbuf(b) then l = vlines(b) else
    local ok, r = pcall(api.nvim_buf_get_lines, b, 0, -1, false)
    l = ok and r or {}
  end
  local list = marks[b]
  if not list or next(list) == nil then return l end
  local width = tonumber(floats[h].config.width) or 40
  local out = {}
  for i, text in ipairs(l) do out[i] = styled(text, i - 1, list, width) end
  return out
end
local function refresh(b)
  for h, f in pairs(floats) do if f.buf == b then z.panel.set_lines(f.panel, float_lines(h)) if f.cursor then z.panel.set_cursor(f.panel, f.cursor[1]) end end end
end
-- extmarks on a buffer a float shows: the float drawn again
for _, name in ipairs({ "nvim_buf_set_extmark", "nvim_buf_del_extmark", "nvim_buf_clear_namespace" }) do
  local f = api[name]
  api[name] = function(b, ...)
    local r = f(b, ...)
    for _, fl in pairs(floats) do if fl.buf == b then refresh(b) break end end
    return r
  end
end
local function range(n, s, e)
  s = s < 0 and n + s + 1 or s
  e = e < 0 and n + e + 1 or e
  return math.max(s, 0), math.min(e, n)
end

api.nvim_create_buf = function(_, _)
  vnext = vnext + 1
  local h = 10000 + vnext
  vbufs[h] = { lines = { "" }, name = "", opts = {}, vars = {}, maps = {} }
  return h
end

local orig = {}
for _, k in ipairs({ "nvim_buf_get_lines", "nvim_buf_set_lines", "nvim_buf_line_count", "nvim_buf_get_name", "nvim_buf_set_name", "nvim_buf_is_valid", "nvim_buf_is_loaded", "nvim_buf_delete", "nvim_buf_set_option", "nvim_buf_get_option", "nvim_buf_set_var", "nvim_buf_get_var", "nvim_get_current_win", "nvim_get_current_buf", "nvim_win_close", "nvim_win_is_valid", "nvim_win_get_buf", "nvim_win_hide" }) do orig[k] = api[k] end
local function cur(b) if (b == 0 or b == nil) and vim._cur_float then return floats[vim._cur_float].buf end return b end

api.nvim_buf_get_lines = function(b, s, e, strict)
  b = cur(b)
  if not is_vbuf(b) then return orig.nvim_buf_get_lines(b, s, e, strict) end
  local L = vlines(b)
  s, e = range(#L, s, e)
  local out = {}
  for i = s + 1, e do out[#out + 1] = L[i] end
  return out
end
api.nvim_buf_set_lines = function(b, s, e, strict, repl)
  b = cur(b)
  if not is_vbuf(b) then return orig.nvim_buf_set_lines(b, s, e, strict, repl) end
  local L = vlines(b)
  s, e = range(#L, s, e)
  local new = {}
  for i = 1, s do new[#new + 1] = L[i] end
  for _, l in ipairs(repl or {}) do new[#new + 1] = l end
  for i = e + 1, #L do new[#new + 1] = L[i] end
  if #new == 0 then new = { "" } end
  vbufs[b].lines = new
  refresh(b)
end
api.nvim_buf_line_count = function(b) b = cur(b) if is_vbuf(b) then return #vlines(b) end return orig.nvim_buf_line_count(b) end
api.nvim_buf_get_name = function(b) b = cur(b) if is_vbuf(b) then return vbufs[b].name end return orig.nvim_buf_get_name(b) end
api.nvim_buf_set_name = function(b, n) b = cur(b) if is_vbuf(b) then vbufs[b].name = n return end return orig.nvim_buf_set_name(b, n) end
api.nvim_buf_is_valid = function(b) if is_vbuf(b) then return true end return orig.nvim_buf_is_valid(b) end
api.nvim_buf_is_loaded = function(b) if is_vbuf(b) then return true end return orig.nvim_buf_is_loaded(b) end
api.nvim_buf_set_option = function(b, k, v) b = cur(b) if is_vbuf(b) then vbufs[b].opts[k] = v return end return orig.nvim_buf_set_option(b, k, v) end
api.nvim_buf_get_option = function(b, k) b = cur(b) if is_vbuf(b) then return vbufs[b].opts[k] end return orig.nvim_buf_get_option(b, k) end
api.nvim_buf_set_var = function(b, k, v) b = cur(b) if is_vbuf(b) then vbufs[b].vars[k] = v return end return orig.nvim_buf_set_var(b, k, v) end
api.nvim_buf_get_var = function(b, k) b = cur(b) if is_vbuf(b) then return vbufs[b].vars[k] end return orig.nvim_buf_get_var(b, k) end

local function close_float(h)
  local f = floats[h]
  if not f then return end
  floats[h] = nil
  if vim._float_focus == h then vim._float_focus = nil end
  pcall(z.panel.close, f.panel)
  if vim._cur_float == h then vim._cur_float = nil end
end
api.nvim_buf_delete = function(b, o)
  b = cur(b)
  if not is_vbuf(b) then return orig.nvim_buf_delete(b, o) end
  for h, f in pairs(floats) do if f.buf == b then close_float(h) end end
  vbufs[b] = nil
end

-- ── keys in a focused float: its own normal and insert mode, its buffer's mappings in each, typing into its
-- buffer (prompt buffers start the line with their prompt), the change events plugins filter on ──
vim._float_focus = nil
local function fl_line(h)
  local f = floats[h]
  local L = vlines(f.buf)
  local c = f.cursor or { #L, #(L[#L] or "") }
  local row = math.max(1, math.min(c[1], #L))
  return L, row, math.min(c[2], #(L[row] or ""))
end
local function fl_prompt(h)
  local b = floats[h].buf
  if (vbufs[b].opts.buftype or "") ~= "prompt" then return "" end
  return (vim.fn.prompt_getprompt and vim.fn.prompt_getprompt(b)) or ""
end
local function fl_changed(h)
  local f = floats[h]
  vbufs[f.buf].tick = (vbufs[f.buf].tick or 0) + 1
  refresh(f.buf)
  if f.cursor then pcall(z.panel.set_cursor, f.panel, f.cursor[1]) end
  vim._cur_float = h
  pcall(api.nvim_exec_autocmds, f.mode == "i" and "TextChangedI" or "TextChanged", { buffer = f.buf, modeline = false })
  vim._cur_float = vim._float_focus
end
local function fl_set(h, L, row, line, col)
  L[row] = line
  vbufs[floats[h].buf].lines = L
  floats[h].cursor = { row, col }
  fl_changed(h)
end
local function fl_insert_mode(h, where)
  local f = floats[h]
  local L, row, col = fl_line(h)
  local p = fl_prompt(h)
  if p ~= "" and (L[row] or ""):sub(1, #p) ~= p then L[row] = p .. (L[row] or "") vbufs[f.buf].lines = L end
  local line = L[row] or ""
  if where == "end" then col = #line elseif where == "start" then col = #p elseif where == "after" then col = math.min(#line, col + 1) end
  f.cursor = { row, math.max(col, #p) }
  f.mode = "i"
  pcall(api.nvim_exec_autocmds, "InsertEnter", { buffer = f.buf, modeline = false })
end

local function run_rhs(h, rhs)
  local ok, err = true, nil
  if type(rhs) == "function" then
    ok, err = pcall(rhs)
  else
    local cmd = rhs:match("^<[Cc][Mm][Dd]>(.-)<[Cc][Rr]>$") or rhs:match("^:(.-)<[Cc][Rr]>$")
    if cmd and (cmd == "q" or cmd == "quit" or cmd == "close" or cmd == "bd" or cmd == "bdelete") then close_float(h)
    elseif cmd then ok, err = pcall(vim.cmd, cmd)
    else for _, k in ipairs(vim._zett_key_tokens(rhs)) do vim._zett_float_key(h, k, true) end end
  end
  if ok == false then vim.notify(tostring(err), vim.log.levels.ERROR) end
end

-- one key: true when the float took it (a focused float takes them all but its window keys)
function vim._zett_float_key(h, key, noremap)
  local f = floats[h]
  if not f or not is_vbuf(f.buf) then return false end
  local mode = f.mode or "n"
  vim._cur_float = h
  if not noremap then
    for lhs, rhs in pairs(vbufs[f.buf].maps[mode] or {}) do
      if z.key_name(lhs) == z.key_name(key) then run_rhs(h, rhs) vim._cur_float = vim._float_focus return true end
    end
  end
  local L, row, col = fl_line(h)
  local line = L[row] or ""
  local took = true
  if mode == "i" then
    local p = fl_prompt(h)
    if key == "<Esc>" or key == "<C-c>" or key == "<C-[>" then
      f.mode = "n"
      f.cursor = { row, math.max(#p, col - 1) }
      pcall(api.nvim_exec_autocmds, "InsertLeave", { buffer = f.buf, modeline = false })
    elseif key == "<BS>" or key == "<C-h>" then
      if col > #p then
        local start = vim.str_utf_start and (col + vim.str_utf_start(line, col)) or col
        start = math.max(#p + 1, start)
        fl_set(h, L, row, line:sub(1, start - 1) .. line:sub(col + 1), start - 1)
      end
    elseif key == "<C-u>" then fl_set(h, L, row, p .. line:sub(col + 1), #p)
    elseif key == "<Left>" then f.cursor = { row, math.max(#p, col - 1) }
    elseif key == "<Right>" then f.cursor = { row, math.min(#line, col + 1) }
    elseif key == "<Home>" then f.cursor = { row, #p }
    elseif key == "<End>" then f.cursor = { row, #line }
    else
      local ch = key == "<Space>" and " " or key == "<Tab>" and "\t" or ((not key:match("^<.+>$")) and key or nil)
      if ch then fl_set(h, L, row, line:sub(1, col) .. ch .. line:sub(col + 1), col + #ch) else took = false end
    end
  else
    if key == "i" then fl_insert_mode(h, "here")
    elseif key == "a" then fl_insert_mode(h, "after")
    elseif key == "A" then fl_insert_mode(h, "end")
    elseif key == "I" then fl_insert_mode(h, "start")
    elseif key == "<Esc>" then -- stays in normal mode
    elseif key == "j" or key == "<Down>" then f.cursor = { math.min(#L, row + 1), 0 } pcall(z.panel.set_cursor, f.panel, f.cursor[1])
    elseif key == "k" or key == "<Up>" then f.cursor = { math.max(1, row - 1), 0 } pcall(z.panel.set_cursor, f.panel, f.cursor[1])
    elseif key == "0" then f.cursor = { row, 0 }
    elseif key == "$" then f.cursor = { row, math.max(0, #line - 1) }
    else took = false end
  end
  vim._cur_float = vim._float_focus
  return took
end

-- "<Esc>A" → { "<Esc>", "A" }; Vim's internal key bytes (\27 …) as their names
function vim._zett_key_tokens(keys)
  local out, i = {}, 1
  local names = { ["\27"] = "<Esc>", ["\r"] = "<CR>", ["\n"] = "<CR>", ["\t"] = "<Tab>", ["\8"] = "<BS>", ["\127"] = "<BS>" }
  while i <= #keys do
    local c = keys:sub(i, i)
    local tok = c == "<" and keys:match("^<[^<>]+>", i) or nil
    if tok then out[#out + 1] = tok i = i + #tok
    elseif names[c] then out[#out + 1] = names[c] i = i + 1
    else
      local n = c:byte() >= 0xF0 and 4 or c:byte() >= 0xE0 and 3 or c:byte() >= 0xC0 and 2 or 1
      out[#out + 1] = keys:sub(i, i + n - 1) i = i + n
    end
  end
  return out
end

local function float_key(h, key) return vim._zett_float_key(h, key) end
function vim._zett_is_float(w) return type(w) == "number" and floats[w] ~= nil end
function vim._zett_is_vbuf(b) return is_vbuf(b) end
function vim._zett_float_wo(h) floats[h].wo = floats[h].wo or {} return floats[h].wo end
function vim._zett_float_panel(h) return floats[h] and floats[h].panel end
function vim._zett_float_mode(h) return floats[h] and floats[h].mode or "n" end

-- stacking among floats of one zindex: the one opened or reconfigured last on top (as Neovim)
local stack_n = 0
local function float_stack() stack_n = stack_n + 1 return stack_n end

-- a float's border as zett draws it: "none", a style name, or 8 characters joined by \31 (Neovim's order:
-- top-left, top, top-right, right, bottom-right, bottom, bottom-left, left); a list of 1, 2 or 4 repeats, as Neovim's
local function float_border(b)
  if b == nil or b == "" then b = (vim.o.winborder ~= nil and vim.o.winborder ~= "") and vim.o.winborder or "none" end
  if type(b) == "string" then return b end
  if type(b) ~= "table" or #b == 0 then return "none" end
  local ch = {}
  for i, x in ipairs(b) do ch[i] = type(x) == "table" and (x[1] or "") or tostring(x) end
  local all = {}
  for i = 1, 8 do all[i] = ch[((i - 1) % #ch) + 1] end
  return table.concat(all, "\31")
end

-- a float's place and size as Neovim reports them (plenary's borders, telescope's layout ask): its top-left
-- cell on the screen (anchor taken into account, a border outside it), its size inside the border
local function float_geom(h)
  local c = floats[h].config
  local w, hgt = math.floor(tonumber(c.width) or 40), math.floor(tonumber(c.height) or 1)
  local row, col = math.floor(tonumber(c.row) or 0), math.floor(tonumber(c.col) or 0)
  local a = (c.anchor or "NW"):upper()
  local b = float_border(c.border)
  local bd = (b ~= "none" and b ~= "shadow") and 1 or 0
  if a:find("S") then row = row - hgt - 2 * bd end
  if a:find("E") then col = col - w - 2 * bd end
  if c.relative == "win" or c.relative == "cursor" then
    local base = c.relative == "cursor" and { vim.fn.screenrow() - 1, vim.fn.screencol() - 1 } or orig.nvim_win_get_position(c.win or 0)
    row, col = row + (base[1] or 0), col + (base[2] or 0)
  end
  return math.max(0, row) + bd, math.max(0, col) + bd, w, hgt
end
for _, k in ipairs({ "nvim_win_get_position", "nvim_win_get_width", "nvim_win_get_height" }) do orig[k] = orig[k] or api[k] end
api.nvim_win_get_position = function(w) w = (w == 0 or w == nil) and vim._cur_float or w if is_float(w) then local r, c = float_geom(w) return { r, c } end return orig.nvim_win_get_position(w) end
api.nvim_win_get_width = function(w) w = (w == 0 or w == nil) and vim._cur_float or w if is_float(w) then local _, _, wd = float_geom(w) return wd end return orig.nvim_win_get_width(w) end
api.nvim_win_get_height = function(w) w = (w == 0 or w == nil) and vim._cur_float or w if is_float(w) then local _, _, _, ht = float_geom(w) return ht end return orig.nvim_win_get_height(w) end
for _, k in ipairs({ "nvim_win_get_position", "nvim_win_get_width", "nvim_win_get_height" }) do rawset(vim.fn, k, api[k]) end
-- a float's cursor: kept with it, and its panel scrolled so that line is in view (telescope's results sit at the bottom)
orig.nvim_win_get_cursor = orig.nvim_win_get_cursor or api.nvim_win_get_cursor
orig.nvim_win_set_cursor = orig.nvim_win_set_cursor or api.nvim_win_set_cursor
api.nvim_win_get_cursor = function(w)
  w = (w == 0 or w == nil) and vim._cur_float or w
  if is_float(w) then local c = floats[w].cursor or { 1, 0 } return { c[1], c[2] } end
  return orig.nvim_win_get_cursor(w)
end
api.nvim_win_set_cursor = function(w, pos)
  w = (w == 0 or w == nil) and vim._cur_float or w
  if is_float(w) then
    floats[w].cursor = { pos[1], pos[2] }
    pcall(z.panel.set_cursor, floats[w].panel, pos[1])
    return
  end
  return orig.nvim_win_set_cursor(w, pos)
end

-- a float's place in the stack: zindex first, then the one opened or reconfigured last on top; a float that
-- frames another of its zindex (holds it with at most a cell around it: plenary's borders) sits just under it
local function float_stack_value(h)
  local z = (tonumber(floats[h].config.zindex) or 50) * 1000000
  local v = z + float_stack()
  local r, c, w, ht = float_geom(h)
  for g, other in pairs(floats) do
    if g ~= h and other.stackval and math.floor(other.stackval / 1000000) * 1000000 == z then
      local r2, c2, w2, h2 = float_geom(g)
      local frames = r <= r2 and c <= c2 and r + ht >= r2 + h2 and c + w >= c2 + w2 and r2 - r <= 1 and c2 - c <= 1
        and (r + ht) - (r2 + h2) <= 1 and (c + w) - (c2 + w2) <= 1 and (w * ht > w2 * h2)
      if frames then v = math.min(v, other.stackval - 1) end
    end
  end
  floats[h].stackval = v
  return v
end

-- for :checkhealth and tests: each float's config and its first lines
function vim._zett_floats()
  local out = {}
  for h, f in pairs(floats) do
    local L = float_lines(h)
    local first = {}
    for i = 1, math.min(2, #L) do first[i] = type(L[i]) == "table" and table.concat(vim.tbl_map(function(p) return p[1] end, L[i])) or L[i] end
    out[#out + 1] = ("%d %s r%s c%s %sx%s border=%s z=%s focusable=%s | %s"):format(h, f.config.relative or "", f.config.row, f.config.col, f.config.width, f.config.height, (vim.inspect(f.config.border):gsub("%s+", "")), f.config.zindex, tostring(f.config.focusable), table.concat(first, " / "))
  end
  table.sort(out)
  return table.concat(out, "\n")
end

local function place(h)
  local f = floats[h]
  local c = f.config
  local L = float_lines(h)
  -- Neovim's row/col are where the `anchor` corner goes (fidget: "SE", the bottom right); the panel takes
  -- its top left, so the window is moved back by its size
  local w, hgt = math.floor(tonumber(c.width) or 40), math.floor(tonumber(c.height) or #L)
  local row, col = math.floor(tonumber(c.row) or 0), math.floor(tonumber(c.col) or 0)
  local a = (c.anchor or "NW"):upper()
  local border = float_border(c.border)
  local bd = (border ~= "none" and border ~= "shadow") and 2 or 0
  if a:find("S") then row = row - hgt - bd end
  if a:find("E") then col = col - w - bd end
  row, col = math.max(0, row), math.max(0, col)
  local title = c.title
  if type(title) == "table" then local t = {} for _, x in ipairs(title) do t[#t + 1] = type(x) == "table" and x[1] or tostring(x) end title = table.concat(t) end
  z.panel.open({
    id = f.panel, side = "float",
    relative = (c.relative and c.relative ~= "") and c.relative or "editor",
    row = row, col = col, width = w, height = hgt,
    title = title or "", focus = f.enter and true or false, border = border, zindex = float_stack_value(h),
    on_key = function(key) return float_key(h, key) end,
    on_close = function() floats[h] = nil end,
  })
  z.panel.set_lines(f.panel, L)
  if f.cursor then z.panel.set_cursor(f.panel, f.cursor[1]) end
end
api.nvim_open_win = function(b, enter, config)
  b = (b == 0) and cur_buf() or b
  config = config or {}
  -- a split (relative empty, split = / vertical =): a real window next to this one
  if (not config.relative or config.relative == "") and (config.split or config.vertical ~= nil) and not is_vbuf(b) then
    ex((config.vertical or config.split == "left" or config.split == "right") and "vsplit" or "split")
    ex("buffer " .. b)
    return api.nvim_get_current_win()
  end
  fnext = fnext + 1
  local h = 20000 + fnext
  floats[h] = { buf = b, config = config, enter = enter, panel = "nvim-float-" .. h }
  place(h)
  return h
end
api.nvim_win_set_config = function(w, config)
  if not is_float(w) then return end
  for k, v in pairs(config or {}) do floats[w].config[k] = v end
  place(w)
end
api.nvim_win_get_config = function(w)
  w = (w == 0 or w == nil) and (vim._cur_float or api.nvim_get_current_win()) or w
  if is_float(w) then local c = vim.deepcopy and vim.deepcopy(floats[w].config) or floats[w].config c.relative = c.relative or "editor" return c end
  return { relative = "", focusable = true, external = false }
end
api.nvim_win_close = function(w, force) w = (w == 0 and vim._cur_float) or w if is_float(w) then return close_float(w) end return orig.nvim_win_close(w, force) end
api.nvim_win_hide = function(w) w = (w == 0 and vim._cur_float) or w if is_float(w) then return close_float(w) end return orig.nvim_win_hide(w) end
api.nvim_win_is_valid = function(w) if is_float(w) then return true end return orig.nvim_win_is_valid(w) end
api.nvim_win_get_buf = function(w) w = (w == 0 and vim._cur_float) or w if is_float(w) then return floats[w].buf end return orig.nvim_win_get_buf(w) end
api.nvim_get_current_win = function() return vim._cur_float or orig.nvim_get_current_win() end
api.nvim_get_current_buf = function() if vim._cur_float then return floats[vim._cur_float].buf end return orig.nvim_get_current_buf() end
for _, k in ipairs({ "nvim_create_buf", "nvim_buf_get_lines", "nvim_buf_set_lines", "nvim_buf_line_count", "nvim_buf_get_name", "nvim_buf_set_name", "nvim_buf_is_valid", "nvim_buf_is_loaded", "nvim_buf_set_option", "nvim_buf_get_option", "nvim_buf_set_var", "nvim_buf_get_var", "nvim_buf_delete", "nvim_open_win", "nvim_win_set_config", "nvim_win_get_config", "nvim_win_close", "nvim_win_hide", "nvim_win_is_valid", "nvim_win_get_buf", "nvim_get_current_win", "nvim_get_current_buf" }) do rawset(vim.fn, k, api[k]) end

-- vim.keymap.set(…, { buffer = a scratch buffer }): that buffer's own mapping (its float runs it)
local keymap_set = vim.keymap.set
function vim.keymap.set(modes, lhs, rhs, opts)
  local b = opts and opts.buffer
  if b == true or b == 0 then b = vim._cur_float and floats[vim._cur_float].buf or nil end
  if is_vbuf(b) then
    for _, m in ipairs(type(modes) == "string" and { modes } or modes) do
      vbufs[b].maps[m] = vbufs[b].maps[m] or {}
      vbufs[b].maps[m][lhs] = rhs
    end
    return
  end
  return keymap_set(modes, lhs, rhs, opts)
end
api.nvim_buf_set_keymap = (function(old)
  return function(b, mode, lhs, rhs, o)
    if is_vbuf(b) then
      o = o or {}
      vbufs[b].maps[mode] = vbufs[b].maps[mode] or {}
      vbufs[b].maps[mode][lhs] = o.callback or rhs
      return
    end
    if old then return old(b, mode, lhs, rhs, o) end
  end
end)(api.nvim_buf_set_keymap)
rawset(vim.fn, "nvim_buf_set_keymap", api.nvim_buf_set_keymap)

-- ── marks, ticks, offsets ──
-- a mark's place: { row (1-based), col (0-based) }; {0, 0} when not set
local function mark_of(name)
  if name == "." then local r, c = z.buf.cursor() return { r, c } end
  local m = z.buf.marks()[name]
  return m and { m[1], m[2] } or { 0, 0 }
end
api.nvim_buf_get_mark = function(_, name) return mark_of(name) end
api.nvim_buf_set_mark = function(_, name, line, col, _) z.buf.set_mark(name, line, col) return true end
api.nvim_buf_del_mark = function(_, name) z.buf.del_marks(name) return true end
api.nvim_del_mark = function(name) z.buf.del_marks(name) return true end
api.nvim_get_mark = function(name, _) local m = mark_of(name) return { m[1], m[2], m[1] > 0 and cur_buf() or 0, m[1] > 0 and z.buf.path() or "" } end
api.nvim_buf_get_changedtick = function(_) return z.buf.tick() end
api.nvim_buf_get_offset = function(_, line)
  local b = 0
  for _, s in ipairs(api.nvim_buf_get_lines(0, 0, line, false)) do b = b + #s + 1 end
  return b
end
-- getpos / setpos with marks too ('a, '<, '>, …)
set("getpos", function(what)
  if what == "." then local r, c = z.buf.cursor() return { 0, r, c + 1, 0 } end
  local name = what:match("^'(.)$")
  if name then local m = mark_of(name) return { 0, m[1], m[1] > 0 and m[2] + 1 or 0, 0 } end
  return { 0, 0, 0, 0 }
end)
set("setpos", function(what, p)
  if what == "." then z.buf.set_cursor(p[2], math.max(p[3] - 1, 0)) return 0 end
  local name = what:match("^'(.)$")
  if name then z.buf.set_mark(name, p[2], math.max(p[3] - 1, 0)) return 0 end
  return -1
end)

-- ── tab and window variables, tab pages ──
local tvars, wvars = {}, {}
api.nvim_tabpage_get_var = function(t, k) local v = (tvars[t == 0 and 1 or t] or {})[k] if v == nil then error("Key not found: " .. k) end return v end
api.nvim_tabpage_set_var = function(t, k, v) t = t == 0 and 1 or t tvars[t] = tvars[t] or {} tvars[t][k] = v end
api.nvim_tabpage_del_var = function(t, k) t = t == 0 and 1 or t if tvars[t] then tvars[t][k] = nil end end
api.nvim_win_del_var = function(w, k) local ok = pcall(api.nvim_win_set_var, w, k, nil) if not ok and wvars[w] then wvars[w][k] = nil end end
api.nvim_set_current_tabpage = function(t) ex(t .. "tabnext") end
api.nvim_open_tabpage = function(b, enter, _) ex("tabnew") if b and b ~= 0 then ex("buffer " .. b) end if enter == false then ex("tabprevious") end return 1 end

-- ── processes (from /proc) ──
local function proc_stat(pid)
  local f = io.open("/proc/" .. pid .. "/stat")
  if not f then return nil end
  local s = f:read("*l") or ""
  f:close()
  local name, rest = s:match("^%d+ %((.*)%) (.*)$")
  if not name then return nil end
  local ppid = tonumber(rest:match("^%S+ (%d+)"))
  return { name = name, pid = tonumber(pid), ppid = ppid }
end
api.nvim_get_proc = function(pid) return proc_stat(pid) end
api.nvim_get_proc_children = function(pid)
  local out = {}
  local p = io.popen("ls /proc 2>/dev/null")
  for d in (p and p:read("*a") or ""):gmatch("%d+") do
    local s = proc_stat(d)
    if s and s.ppid == pid then out[#out + 1] = s.pid end
  end
  if p then p:close() end
  return out
end

-- ── pasting, commands parsed, options described, colours named ──
api.nvim_paste = function(data, _, _)
  local lines = vim.split(data:gsub("\r\n", "\n"), "\n", { plain = true })
  api.nvim_put(lines, "c", false, true)
  return true
end
-- nvim_parse_cmd: the command's name, bang, args and range (what a plugin most reads of it)
api.nvim_parse_cmd = function(str, _)
  local s = str:gsub("^%s*:*", "")
  local range, rest = s:match("^([%d%.%$%%,;'<>+%-]*)(.*)$")
  local name, bang, args = rest:match("^(%a[%w_]*)(!?)%s*(.*)$")
  if not name then error("E492: Not an editor command: " .. str) end
  local r = {}
  for n in (range or ""):gmatch("%d+") do r[#r + 1] = tonumber(n) end
  return { cmd = name, bang = bang == "!", args = args ~= "" and vim.split(args, "%s+", { trimempty = true }) or {}, range = r, count = -1, reg = "", addr = "line", nargs = "*", nextcmd = "", magic = { file = false, bar = false }, mods = {} }
end
api.nvim_get_option_info = function(name) return api.nvim_get_option_info2(name, {}) end
api.nvim_get_all_options_info = function()
  local out = {}
  for _, n in ipairs(z.option_names and z.option_names() or {}) do out[n] = api.nvim_get_option_info2(n, {}) end
  return out
end
local color_names = { Black = 0x000000, White = 0xffffff, Red = 0xff0000, Green = 0x00ff00, Blue = 0x0000ff, Yellow = 0xffff00, Cyan = 0x00ffff, Magenta = 0xff00ff, Gray = 0xbebebe, Grey = 0xbebebe, DarkGray = 0xa9a9a9, DarkGrey = 0xa9a9a9, LightGray = 0xd3d3d3, LightGrey = 0xd3d3d3, DarkRed = 0x8b0000, DarkGreen = 0x006400, DarkBlue = 0x00008b, DarkYellow = 0xbbbb00, DarkCyan = 0x008b8b, DarkMagenta = 0x8b008b, Brown = 0xa52a2a, Orange = 0xffa500, Purple = 0xa020f0, Pink = 0xffc0cb, LightRed = 0xffbbbb, LightGreen = 0x90ee90, LightBlue = 0xadd8e6, LightYellow = 0xffffe0, LightCyan = 0xe0ffff, LightMagenta = 0xffbbff, SeaGreen = 0x2e8b57, SlateBlue = 0x6a5acd }
api.nvim_get_color_map = function() return vim.deepcopy and vim.deepcopy(color_names) or color_names end

-- ── user commands and autocmds as made (nvim_get_commands, nvim_get_autocmds) ──
local made_cmds, made_au = {}, {}
local create_cmd = api.nvim_create_user_command
api.nvim_create_user_command = function(name, run, o)
  o = o or {}
  made_cmds[name] = { name = name, definition = type(run) == "string" and run or (o.desc or ""), nargs = tostring(o.nargs or 0), bang = o.bang and true or false, range = o.range and tostring(o.range) or nil, bar = o.bar and true or false, callback = type(run) == "function" and run or nil }
  return create_cmd(name, run, o)
end
api.nvim_buf_create_user_command = function(_, name, run, o) return api.nvim_create_user_command(name, run, o) end
local del_cmd = api.nvim_del_user_command
api.nvim_del_user_command = function(name) made_cmds[name] = nil return del_cmd(name) end
api.nvim_get_commands = function(_) return vim.deepcopy and vim.deepcopy(made_cmds) or made_cmds end
api.nvim_buf_get_commands = function(_, o) return api.nvim_get_commands(o) end
local create_au, del_au = api.nvim_create_autocmd, api.nvim_del_autocmd
api.nvim_create_autocmd = function(events, o)
  o = o or {}
  local id = create_au(events, o)
  for _, e in ipairs(type(events) == "string" and { events } or events) do
    local pats = type(o.pattern) == "table" and o.pattern or { o.pattern or "*" }
    for _, p in ipairs(pats) do
      made_au[#made_au + 1] = { id = id, event = e, pattern = p, group = type(o.group) == "number" and o.group or nil, group_name = type(o.group) == "string" and o.group or nil, desc = o.desc, callback = o.callback, command = o.command or "", once = o.once and true or false, buffer = o.buffer }
    end
  end
  return id
end
api.nvim_del_autocmd = function(id)
  for i = #made_au, 1, -1 do if made_au[i].id == id then table.remove(made_au, i) end end
  return del_au(id)
end
-- nvim_get_autocmds({ event =, group =, pattern =, id = }): the matching ones
api.nvim_get_autocmds = function(o)
  o = o or {}
  local evs = type(o.event) == "string" and { o.event } or o.event
  local out = {}
  for _, a in ipairs(made_au) do
    local ok = true
    if evs then ok = false for _, e in ipairs(evs) do if e:lower() == a.event:lower() then ok = true end end end
    if ok and o.group then ok = (a.group == o.group) or (a.group_name == o.group) end
    if ok and o.pattern then ok = a.pattern == o.pattern end
    if ok and o.id then ok = a.id == o.id end
    if ok then out[#out + 1] = vim.deepcopy and vim.deepcopy(a) or a end
  end
  return out
end

-- nvim_buf_attach(buf, _, { on_lines = function("lines", buf, tick, first, last, new_last) }):
-- told after each change (the whole text as the range: zett does not say which lines changed)
api.nvim_buf_attach = function(b, _, cbs)
  cbs = cbs or {}
  b = (b == 0 or b == nil) and cur_buf() or b
  local id
  id = api.nvim_create_autocmd({ "TextChanged", "TextChangedI" }, {
    callback = function()
      if api.nvim_get_current_buf() ~= b then return end
      local n = api.nvim_buf_line_count(0)
      if cbs.on_lines and cbs.on_lines("lines", b, z.buf.tick(), 0, n, n, 0) == true then api.nvim_del_autocmd(id) end
      if cbs.on_changedtick then cbs.on_changedtick("changedtick", b, z.buf.tick()) end
    end,
  })
  if cbs.on_detach then api.nvim_create_autocmd("BufUnload", { once = true, callback = function() cbs.on_detach("detach", b) end }) end
  return true
end
-- the screen lines text takes (no wrapping counted: zett draws long lines on one row by default)
api.nvim_win_text_height = function(_, o)
  o = o or {}
  local n = api.nvim_buf_line_count(0)
  local s, e = o.start_row or 0, o.end_row or (n - 1)
  return { all = math.max(e - s + 1, 0), fill = 0 }
end

for _, k in ipairs({ "nvim_buf_get_mark", "nvim_buf_set_mark", "nvim_buf_del_mark", "nvim_del_mark", "nvim_get_mark", "nvim_buf_get_changedtick", "nvim_buf_get_offset", "nvim_tabpage_get_var", "nvim_tabpage_set_var", "nvim_tabpage_del_var", "nvim_win_del_var", "nvim_set_current_tabpage", "nvim_open_tabpage", "nvim_get_proc", "nvim_get_proc_children", "nvim_paste", "nvim_parse_cmd", "nvim_get_option_info", "nvim_get_all_options_info", "nvim_get_color_map", "nvim_create_user_command", "nvim_buf_create_user_command", "nvim_del_user_command", "nvim_get_commands", "nvim_buf_get_commands", "nvim_create_autocmd", "nvim_del_autocmd", "nvim_get_autocmds", "nvim_buf_attach", "nvim_win_text_height" }) do rawset(vim.fn, k, api[k]) end

-- ── more of vim.fn: lines, lists, text indexes, files, windows and buffers described ──
def("prevnonblank", function(l)
  l = tonumber(l) or vim.fn.line(l)
  for n = math.min(l, api.nvim_buf_line_count(0)), 1, -1 do if (vim.fn.getline(n) or ""):match("%S") then return n end end
  return 0
end)
def("reduce", function(l, f, init)
  local acc, start = init, 1
  if acc == nil then acc, start = l[1], 2 end
  for i = start, #(l or {}) do acc = f(acc, l[i]) end
  return acc
end)
def("slice", function(l, s, e)
  if type(l) == "string" then local cs = vim.fn.str2list(l) local r = vim.fn.slice(cs, s, e) return vim.fn.list2str(r) end
  local n = #l
  s = s < 0 and n + s or s
  e = e == nil and n or (e < 0 and n + e or e)
  local out = {}
  for i = s + 1, math.min(e, n) do out[#out + 1] = l[i] end
  return out
end)
def("trunc", function(x) x = tonumber(x) or 0 return x < 0 and math.ceil(x) or math.floor(x) end)
local seed = os.time()
def("srand", function(s) seed = tonumber(s) or os.time() return { seed, 0, 0, 0 } end)
def("rand", function(_) seed = (seed * 1103515245 + 12345) % 2147483648 return seed end)
def("sha256", function(s)
  local tmp = os.tmpname()
  local f = io.open(tmp, "wb") if f then f:write(s or "") f:close() end
  local p = io.popen("sha256sum " .. vim.fn.shellescape(tmp))
  local out = p and p:read("*l") or ""
  if p then p:close() end
  os.remove(tmp)
  return out:match("^(%x+)") or ""
end)
-- character, byte and UTF-16 indexes of a string
def("strcharlen", function(s) return utf8.len(s or "") or #(s or "") end)
def("strgetchar", function(s, i) local n = 0 for _, c in utf8.codes(s or "") do if n == i then return c end n = n + 1 end return -1 end)
def("byteidx", function(s, n) if n < 0 then return -1 end local b = utf8.offset(s or "", n + 1) return b and b - 1 or -1 end)
def("byteidxcomp", function(s, n) return vim.fn.byteidx(s, n) end)
def("charidx", function(s, b) if b < 0 or b >= #(s or "") then return -1 end return (utf8.len(s:sub(1, b)) or b) end)
def("utf16idx", function(s, b) local n = 0 for p, c in utf8.codes(s or "") do if p > b then break end n = n + (c > 0xFFFF and 2 or 1) end return n - 1 end)
def("strutf16len", function(s) local n = 0 for _, c in utf8.codes(s or "") do n = n + (c > 0xFFFF and 2 or 1) end return n end)
def("charcol", function(e) local c = vim.fn.col(e) local l = vim.fn.getline(".") return (utf8.len(l:sub(1, c - 1)) or c - 1) + 1 end)
def("pathshorten", function(p, len)
  len = len or 1
  local parts = vim.split(p or "", "/", { plain = true })
  for i = 1, #parts - 1 do
    local s = parts[i]
    local dot = s:sub(1, 1) == "." and 1 or 0
    parts[i] = s:sub(1, len + dot)
  end
  return table.concat(parts, "/")
end)
-- files: readdir, type and permissions, copies, finding upward
def("readdir", function(d)
  local out = {}
  local p = io.popen("ls -A " .. vim.fn.shellescape(vim.fn.expand(d)) .. " 2>/dev/null")
  for n in (p and p:read("*a") or ""):gmatch("[^\n]+") do out[#out + 1] = n end
  if p then p:close() end
  return out
end)
local function stat(f, fmt)
  local p = io.popen("stat -c " .. fmt .. " " .. vim.fn.shellescape(vim.fn.expand(f)) .. " 2>/dev/null")
  local s = p and p:read("*l") or ""
  if p then p:close() end
  return s
end
def("getftype", function(f)
  local t = stat(f, "%F")
  return ({ ["regular file"] = "file", ["regular empty file"] = "file", directory = "dir", ["symbolic link"] = "link", fifo = "fifo", socket = "socket", ["character special file"] = "cdev", ["block special file"] = "bdev" })[t] or ""
end)
def("getfperm", function(f) local s = stat(f, "%A") return s ~= "" and s:sub(2) or "" end)
def("setfperm", function(f, perm)
  local n = 0
  for i = 1, 9 do if perm:sub(i, i) ~= "-" then n = n + 2 ^ (9 - i) end end
  return os.execute(string.format("chmod %o %s", math.floor(n), vim.fn.shellescape(vim.fn.expand(f)))) and 1 or 0
end)
def("filecopy", function(a, b)
  if vim.fn.filereadable(b) == 1 then return 0 end
  return os.execute("cp -- " .. vim.fn.shellescape(a) .. " " .. vim.fn.shellescape(b)) and 1 or 0
end)
-- finddir / findfile: in path (a list of folders; ";" upward to the root)
local function find_up(name, path, want_dir)
  path = (path == nil or path == "") and "." or path
  for _, d in ipairs(vim.split(path, ",", { trimempty = true })) do
    local up = d:find(";")
    local dir = vim.fn.fnamemodify(up and d:sub(1, up - 1) ~= "" and d:sub(1, up - 1) or (up and "." or d), ":p"):gsub("/$", "")
    while dir and dir ~= "" do
      local p = dir .. "/" .. name
      local t = vim.fn.getftype(p)
      if (want_dir and t == "dir") or (not want_dir and t ~= "" and t ~= "dir") then return p end
      if not up then break end
      local parent = dir:match("^(.*)/[^/]*$")
      if parent == dir then break end
      dir = parent
    end
  end
  return ""
end
def("finddir", function(name, path) return find_up(name, path, true) end)
def("findfile", function(name, path) return find_up(name, path, false) end)

-- windows and buffers described (getwininfo / getbufinfo), and moved between
def("getwininfo", function(w)
  local out = {}
  for i, x in ipairs(wins()) do
    local h = handle(x.nr)
    if w == nil or w == h then
      out[#out + 1] = { winid = h, winnr = i, bufnr = x.buf, width = x.width, height = x.height, wincol = x.col + 1, winrow = x.row + 1, tabnr = 1, topline = x.top or 1, botline = (x.top or 1) + x.height - 2, textoff = vim.fn._textoff and vim.fn._textoff() or 0, terminal = 0, quickfix = 0, loclist = 0, variables = {} }
    end
  end
  return out
end)
def("getbufinfo", function(b)
  local out = {}
  local cur = cur_buf()
  for i, name in ipairs(z.buffers()) do
    if b == nil or type(b) == "table" or b == i then
      local wins_of = {}
      for _, x in ipairs(wins()) do if x.buf == i then wins_of[#wins_of + 1] = handle(x.nr) end end
      out[#out + 1] = { bufnr = i, name = name, listed = 1, loaded = 1, hidden = #wins_of == 0 and 1 or 0, changed = (i == cur and vim.bo.modified) and 1 or 0, lnum = i == cur and vim.fn.line(".") or 1, linecount = i == cur and api.nvim_buf_line_count(0) or 0, windows = wins_of, variables = {}, lastused = 0 }
    end
  end
  return out
end)
def("win_gotoid", function(w) if api.nvim_win_is_valid(w) then api.nvim_set_current_win(w) return 1 end return 0 end)
def("win_findbuf", function(b) local out = {} for _, x in ipairs(wins()) do if x.buf == b then out[#out + 1] = handle(x.nr) end end return out end)
-- a float (nvim_open_win's handles from 20000) is a "popup", any other window ""
def("win_gettype", function(w) w = w or 0 return (w >= 20000) and "popup" or "" end)
def("win_screenpos", function(w) local x = win_of(w) return x and { x.row + 1, x.col + 1 } or { 0, 0 } end)
def("tabpagebuflist", function(_) local out = {} for _, x in ipairs(wins()) do out[#out + 1] = x.buf end return out end)
def("tabpagewinnr", function(_, a) if a == "$" then return #wins() end return vim.fn.winnr() end)
def("winsaveview", function() local r, c = z.buf.cursor() return { lnum = r, col = c, coladd = 0, curswant = c, topline = r, topfill = 0, leftcol = 0, skipcol = 0 } end)
def("winrestview", function(v) if v and v.lnum then z.buf.set_cursor(v.lnum, v.col or 0) end return 0 end)
def("winlayout", function() local l = {} for _, x in ipairs(wins()) do l[#l + 1] = { "leaf", handle(x.nr) } end if #l == 1 then return l[1] end return { "row", l } end)
-- variables in other places, lines of other buffers, buffers added
def("setbufvar", function(b, name, v) if name:sub(1, 1) == "&" then pcall(api.nvim_buf_set_option, b == "%" and 0 or b, name:sub(2), v) else pcall(api.nvim_buf_set_var, b == "%" and 0 or b, name, v) end end)
def("setwinvar", function(w, name, v) if name:sub(1, 1) == "&" then pcall(function() vim.wo[name:sub(2)] = v end) elseif vim.w then vim.w[name] = v end end)
def("settabvar", function(t, name, v) api.nvim_tabpage_set_var(t, name, v) end)
def("settabwinvar", function(_, w, name, v) vim.fn.setwinvar(w, name, v) end)
def("setbufline", function(b, l, text)
  l = l == "$" and api.nvim_buf_line_count(0) or tonumber(l) or 1
  local t = type(text) == "table" and text or { text }
  api.nvim_buf_set_lines(b == "%" and 0 or b, l - 1, l - 1 + #t, false, t)
  return 0
end)
def("bufadd", function(name) ex("badd " .. name) return vim.fn.bufnr(name) end)
def("bufload", function(_) return 0 end)
def("feedkeys", function(keys, mode) api.nvim_feedkeys(keys, mode or "m", false) return 0 end)
def("getmarklist", function(_)
  local out = {}
  for name, m in pairs(z.buf.marks()) do out[#out + 1] = { mark = "'" .. name, pos = { cur_buf(), m[1], m[2] + 1, 0 }, file = z.buf.path() } end
  table.sort(out, function(a, b) return a.mark < b.mark end)
  return out
end)

-- ── the quickfix list (getqflist / setqflist; setloclist sets the window's list) ──
local function qf_items(file_too)
  local q = z.qflist()
  local out = {}
  for i, it in ipairs(q.items) do
    out[i] = { bufnr = (function() for n, b in ipairs(z.buffers()) do if b == it.file then return n end end return 0 end)(), filename = it.file, lnum = it.lnum, end_lnum = 0, col = it.col, end_col = 0, vcol = 0, nr = it.nr, pattern = "", text = it.text, type = it.type, valid = it.valid and 1 or 0, module = "" }
  end
  return out, q
end
local function getqf(what)
  local items, q = qf_items()
  if type(what) ~= "table" then return items end
  local r = {}
  if what.items then r.items = items end
  if what.title then r.title = q.title end
  if what.idx then r.idx = q.idx end
  if what.size then r.size = #items end
  if what.nr then r.nr = 1 end
  if what.id then r.id = 1 end
  if what.context then r.context = "" end
  if what.winid then r.winid = 0 end
  return r
end
local function setqf(which, list, action, what)
  if type(action) == "table" then what, action = action, " " end
  what = what or {}
  list = list or what.items or {}
  local enc = {}
  for _, e in ipairs(list) do
    local file = e.filename or (e.bufnr and z.buffers()[e.bufnr]) or ""
    enc[#enc + 1] = table.concat({ file, tostring(e.lnum or 0), tostring(e.col or 0), e.type or "", tostring(e.nr or 0), (e.text or ""):gsub("[\30\31\29]", " ") }, "\31")
  end
  z.ui("qfSet", table.concat({ which, (action == nil or action == "") and " " or action, what.title or "setqflist()", table.concat(enc, "\30") }, "\29"))
  return 0
end
set("getqflist", getqf)
set("setqflist", function(list, action, what) return setqf("q", list, action, what) end)
set("setloclist", function(_, list, action, what) return setqf("l", list, action, what) end)

-- ── registers, the : line and its history ──
set("getreg", function(r, _, list)
  local t = z.buf.register((r == nil or r == "") and '"' or r)
  local text = t and t[1] or ""
  if list == 1 or list == true then return vim.split(text:gsub("\n$", ""), "\n", { plain = true }) end
  return text
end)
set("setreg", function(r, v, opts)
  local text = type(v) == "table" and (table.concat(v, "\n") .. "\n") or tostring(v or "")
  if type(opts) == "string" and opts:find("V") and not text:find("\n$") then text = text .. "\n" end
  z.buf.set_register((r == nil or r == "") and '"' or r, text)
  return 0
end)
def("getregtype", function(r) local t = z.buf.register((r == nil or r == "") and '"' or r) return t and t[2] or "" end)
def("getreginfo", function(r)
  r = (r == nil or r == "") and '"' or r
  local t = z.buf.register(r)
  if not t then return {} end
  return { regcontents = vim.split(t[1]:gsub("\n$", ""), "\n", { plain = true }), regtype = t[2], isunnamed = r == '"' }
end)
def("reg_recording", function() return z.buf.recording() end)
def("reg_executing", function() return "" end)
def("getcmdline", function() local c = z.cmdline() return c and c[1] or "" end)
def("getcmdpos", function() local c = z.cmdline() return c and (tonumber(c[2]) + 1) or 0 end)
def("getcmdtype", function() return z.cmdline() and ":" or "" end)
def("getcmdwintype", function() return "" end)
def("setcmdline", function(s, _) z.ui("setCmdline", s) return 0 end)
def("setcmdpos", function(_) return 0 end)
-- history of : lines (the only one zett keeps): index from 1, negative from the end
local function hist_ok(t) return t == nil or t == ":" or t == "cmd" end
def("histnr", function(t) if not hist_ok(t) then return -1 end return #z.history() end)
def("histget", function(t, i)
  if not hist_ok(t) then return "" end
  local h = z.history()
  i = tonumber(i) or -1
  if i < 0 then i = #h + i + 1 end
  return h[i] or ""
end)
def("histadd", function(t, line) if not hist_ok(t) or (line or "") == "" then return 0 end z.ui("histAdd", line) return 1 end)
def("histdel", function(_, _) return 0 end)

-- ── Vim patterns for real (zett.regex: the editor's own engine, as /, :s and :g use) ──
local function icase(pat) return vim.o.ignorecase and not (vim.o.smartcase and pat:find("%u")) end
local function rx(s, pat, start) local ok, r = pcall(z.regex, pat, s, start or 0, icase(pat)) return ok and r or nil end
-- the count-th match from start (match / matchend / matchstr / matchstrpos)
local function nth(s, pat, start, count)
  s = tostring(s or "")
  local at = math.max(start or 0, 0)
  local r
  for _ = 1, math.max(count or 1, 1) do
    r = rx(s, pat, at)
    if not r then return nil end
    at = r[2] > r[1] and r[2] or r[2] + 1
  end
  return r
end
set("match", function(s, pat, start, count) if type(s) == "table" then for i, l in ipairs(s) do if rx(l, pat) then return i - 1 end end return -1 end local r = nth(s, pat, start, count) return r and r[1] or -1 end)
set("matchend", function(s, pat, start, count) local r = nth(s, pat, start, count) return r and r[2] or -1 end)
set("matchstr", function(s, pat, start, count) local r = nth(s, pat, start, count) return r and tostring(s):sub(r[1] + 1, r[2]) or "" end)
set("matchstrpos", function(s, pat, start, count) local r = nth(s, pat, start, count) if not r then return { "", -1, -1 } end return { tostring(s):sub(r[1] + 1, r[2]), r[1], r[2] } end)
set("matchlist", function(s, pat, start, count)
  local r = nth(s, pat, start, count)
  if not r then return {} end
  local out = { tostring(s):sub(r[1] + 1, r[2]) }
  for g = 1, 9 do out[g + 1] = r[g + 2] or "" end
  return out
end)
-- substitute(s, pat, rep, flags): \0 & \1…\9 \n \t \\ in rep (a function rep gets the matchlist)
set("substitute", function(s, pat, rep, flags)
  s = tostring(s or "")
  local all = (flags or ""):find("g") ~= nil
  local out, at = {}, 0
  while at <= #s do
    local r = rx(s, pat, at)
    if not r then break end
    local whole = s:sub(r[1] + 1, r[2])
    local new
    if type(rep) == "function" then
      local m = { whole } for g = 1, 9 do m[g + 1] = r[g + 2] or "" end
      new = tostring(rep(m))
    else
      new = rep:gsub("\\(.)", function(c)
        if c:match("%d") then return c == "0" and whole or (r[tonumber(c) + 2] or "") end
        return ({ n = "\n", t = "\t", r = "\r", ["&"] = "&", ["\\"] = "\\" })[c] or c
      end)
      new = new:gsub("&", whole)
    end
    out[#out + 1] = s:sub(at + 1, r[1])
    out[#out + 1] = new
    at = r[2] > r[1] and r[2] or r[2] + 1
    if r[2] == r[1] and r[1] < #s then out[#out + 1] = s:sub(r[1] + 1, r[1] + 1) end
    if not all then break end
  end
  out[#out + 1] = s:sub(at + 1)
  return table.concat(out)
end)

-- ── matchadd and its family: patterns highlighted in the window (drawn as extmarks are) ──
local matches, match_id = {}, 3
set("matchadd", function(group, pat, prio, id, _)
  if id == nil or id < 1 then match_id = match_id + 1 id = match_id end
  matches[id] = { group = group, pattern = pat, priority = prio or 10, id = id }
  bump()
  return id
end)
set("matchaddpos", function(group, pos, prio, id)
  if id == nil or id < 1 then match_id = match_id + 1 id = match_id end
  matches[id] = { group = group, pos = pos, priority = prio or 10, id = id }
  bump()
  return id
end)
set("matchdelete", function(id) if not matches[id] then return -1 end matches[id] = nil bump() return 0 end)
set("clearmatches", function() matches = {} bump() end)
set("getmatches", function() local out = {} for _, m in pairs(matches) do out[#out + 1] = vim.deepcopy and vim.deepcopy(m) or m end table.sort(out, function(a, b) return a.id < b.id end) return out end)
set("setmatches", function(l) matches = {} for _, m in ipairs(l or {}) do matches[m.id] = m end bump() return 0 end)
set("matcharg", function(n) local m = matches[n] return m and { m.group, m.pattern or "" } or {} end)
-- the matches on rows a..e of the current buffer, as extmark lines for the window
local marks_for = vim._marks_for
function vim._marks_for(b, a, e)
  local base = marks_for(b, a, e)
  if next(matches) == nil or b ~= cur_buf() then return base end
  local out = {}
  local lines = api.nvim_buf_get_lines(0, a, e + 1, false)
  for _, m in pairs(matches) do
    if m.pattern then
      for i, l in ipairs(lines) do
        local at = 0
        while at <= #l do
          local r = rx(l, m.pattern, at)
          if not r then break end
          if r[2] > r[1] then out[#out + 1] = table.concat({ a + i - 1, r[1], a + i - 1, r[2], m.priority, m.group, "", "", "", "", "eol", "", "" }, "\t") end
          at = r[2] > r[1] and r[2] or r[2] + 1
        end
      end
    elseif m.pos then
      for _, p in ipairs(m.pos) do
        local row = type(p) == "table" and p[1] or p
        local col = type(p) == "table" and p[2] or nil
        local len = type(p) == "table" and p[3] or 1
        if row - 1 >= a and row - 1 <= e then
          local s = col and col - 1 or 0
          local en = col and (s + len) or #(lines[row - a] or "")
          out[#out + 1] = table.concat({ row - 1, s, row - 1, en, m.priority, m.group, "", "", "", "", "eol", "", "" }, "\t")
        end
      end
    end
  end
  if #out == 0 then return base end
  return base == "" and table.concat(out, "\30") or (base .. "\30" .. table.concat(out, "\30"))
end

-- ── where things are on screen: the window's first line, its gutter, screen cells ──
local function top_of(w) local x = win_of(w) return x and x.top or 1 end
-- the gutter's width: line numbers (numberwidth, or the digits needed) and a sign column
rawset(fn, "_textoff", function()
  local n = 0
  if vim.wo.number or vim.wo.relativenumber then n = math.max(tonumber(vim.wo.numberwidth) or 4, #tostring(z.buf.line_count()) + 1) end
  local sc = tostring(vim.wo.signcolumn or "auto")
  if sc == "yes" or sc:match("^yes:") then n = n + 2 * (tonumber(sc:match(":(%d)")) or 1) end
  return n
end)
set("line", function(what, w)
  local r, _ = z.buf.cursor()
  if what == "." then return r end
  if what == "$" then return z.buf.line_count() end
  local x = win_of(w)
  if what == "w0" then return top_of(w) end
  if what == "w$" then return math.min(z.buf.line_count(), top_of(w) + (x and x.height or 24) - 2) end
  if type(what) == "string" and what:sub(1, 1) == "'" then local m = z.buf.marks()[what:sub(2)] return m and m[1] or 0 end
  if what == "v" then return r end
  return 0
end)
def("winline", function() local r = z.buf.cursor() return r - top_of(0) + 1 end)
def("wincol", function() local _, c = z.buf.cursor() return c + 1 + fn._textoff() end)
def("screenrow", function() local x = win_of(0) return (x and x.row or 0) + fn.winline() end)
def("screencol", function() local x = win_of(0) return (x and x.col or 0) + fn.wincol() end)
def("screenpos", function(w, lnum, col)
  local x = win_of(w)
  local top = top_of(w)
  if not x or lnum < top or lnum > top + x.height - 2 then return { row = 0, col = 0, endcol = 0, curscol = 0 } end
  local c = x.col + fn._textoff() + col
  return { row = x.row + lnum - top + 1, col = c, endcol = c, curscol = c }
end)
def("getcharpos", function(e) local p = fn.getpos(e) local l = fn.getline(p[2]) or "" p[3] = (utf8.len(l:sub(1, p[3] - 1)) or p[3] - 1) + 1 return p end)
def("setcharpos", function(e, p) local l = fn.getline(p[2]) or "" local b = utf8.offset(l, p[3]) or (#l + 1) return fn.setpos(e, { p[1], p[2], b, p[4] or 0 }) end)
def("getcursorcharpos", function() local r, c = z.buf.cursor() local l = fn.getline(r) or "" return { 0, r, (utf8.len(l:sub(1, c)) or c) + 1, 0, 0 } end)
def("setcursorcharpos", function(l, c) if type(l) == "table" then l, c = l[1], l[2] end local s = fn.getline(l) or "" local b = utf8.offset(s, c) or (#s + 1) z.buf.set_cursor(l, b - 1) return 0 end)

-- ── the jump and change lists ──
def("getjumplist", function()
  local j, at = z.buf.jumps()
  local out = {}
  for i, p in ipairs(j) do out[i] = { bufnr = cur_buf(), lnum = p[1], col = p[2], coladd = 0 } end
  return { out, math.min(at, #out) }
end)
def("getchangelist", function()
  local c = z.buf.changes()
  local out = {}
  for i, p in ipairs(c) do out[i] = { lnum = p[1], col = p[2], coladd = 0 } end
  return { out, #out }
end)

-- ── undotree(): the states in time order; seq_cur the one the text is in ──
def("undotree", function()
  local states = z.buf.undo_states()
  local cur = z.buf.undo_current()
  local entries = {}
  for i = 2, #states do
    local s = states[i]
    entries[#entries + 1] = { seq = s.seq or (i - 1), time = s.time or 0, curhead = (s.seq or (i - 1)) == cur and 1 or nil }
  end
  local last = #states - 1
  return { seq_last = last, seq_cur = cur, synced = 1, save_last = 0, save_cur = 0, time_cur = (states[cur + 1] or {}).time or 0, entries = entries }
end)

-- ── searchpos / searchcount / search with flags (b c n w W e s p), real Vim patterns ──
local function search_pos(pat, flags, stopline)
  flags = flags or ""
  local back, accept, wrap = flags:find("b") ~= nil, flags:find("c") ~= nil, (flags:find("W") == nil) and (flags:find("w") ~= nil or vim.o.wrapscan)
  local want_end = flags:find("e") ~= nil
  local all = lines(0, -1)
  local r, c = z.buf.cursor()
  local n = #all
  local function hits(i)
    local out, at = {}, 0
    local l = all[i] or ""
    while at <= #l do
      local m = rx(l, pat, at)
      if not m then break end
      out[#out + 1] = m
      at = m[2] > m[1] and m[2] or m[2] + 1
    end
    return out
  end
  local order = {}
  if back then
    for i = r, 1, -1 do order[#order + 1] = i end
    if wrap then for i = n, r, -1 do order[#order + 1] = i end end
  else
    for i = r, n do order[#order + 1] = i end
    if wrap then for i = 1, r do order[#order + 1] = i end end
  end
  for k, i in ipairs(order) do
    if stopline and stopline > 0 and ((back and i < stopline) or (not back and i > stopline)) then break end
    local hs = hits(i)
    local first = k == 1
    if back then
      for j = #hs, 1, -1 do
        local s = want_end and math.max(hs[j][2] - 1, hs[j][1]) or hs[j][1]
        if not first or s < c or (accept and s == c) then return i, s end
      end
    else
      for _, h in ipairs(hs) do
        local s = want_end and math.max(h[2] - 1, h[1]) or h[1]
        if not first or s > c or (accept and s == c) or (k == #order and i == r and wrap and s <= c and k > 1) then return i, s end
      end
    end
  end
  return 0, 0
end
set("searchpos", function(pat, flags, stopline)
  local l, c = search_pos(pat, flags, stopline)
  if l > 0 and not (flags or ""):find("n") then z.buf.set_cursor(l, c) end
  return { l, l > 0 and c + 1 or 0 }
end)
set("search", function(pat, flags, stopline)
  local l, c = search_pos(pat, flags, stopline)
  if l > 0 and not (flags or ""):find("n") then z.buf.set_cursor(l, c) end
  return l
end)
def("searchcount", function(o)
  o = o or {}
  local pat = o.pattern or fn.getreg("/")
  if pat == nil or pat == "" then return {} end
  local maxcount = o.maxcount or 99
  local r, c = z.buf.cursor()
  if o.pos then r, c = o.pos[1], o.pos[2] - 1 end
  local total, current = 0, 0
  for i, l in ipairs(lines(0, -1)) do
    local at = 0
    while at <= #l do
      local m = rx(l, pat, at)
      if not m then break end
      total = total + 1
      if i < r or (i == r and m[1] <= c) then current = total end
      at = m[2] > m[1] and m[2] or m[2] + 1
    end
  end
  return { current = math.min(current, maxcount), total = math.min(total, maxcount), exact_match = current > 0 and 1 or 0, incomplete = total > maxcount and 2 or 0, maxcount = maxcount }
end)

-- ── timers: info, pause, stop them all ──
local timer_meta = {}
set("timer_start", function(ms, f, o)
  local t = vim.uv.new_timer()
  local rep = o and o["repeat"] or 0
  vim._timers = vim._timers or {}
  local id = #vim._timers + 1
  local info = { id = id, time = ms, ["repeat"] = rep, remaining = ms, paused = 0, callback = f }
  timer_meta[id] = info
  local n = 0
  t:start(ms, rep ~= 0 and ms or 0, function()
    if info.paused == 1 then return end
    n = n + 1
    f(id)
    if rep > 0 and n >= rep then t:stop() timer_meta[id] = nil end
    if rep == 0 then timer_meta[id] = nil end
  end)
  vim._timers[id] = t
  return id
end)
set("timer_stop", function(id) if vim._timers and vim._timers[id] then vim._timers[id]:stop() end timer_meta[id] = nil end)
def("timer_stopall", function() for id in pairs(timer_meta) do fn.timer_stop(id) end end)
def("timer_pause", function(id, p) if timer_meta[id] then timer_meta[id].paused = (p == true or p == 1) and 1 or 0 end end)
def("timer_info", function(id)
  local out = {}
  for k, v in pairs(timer_meta) do if id == nil or id == k then out[#out + 1] = { id = v.id, time = v.time, ["repeat"] = v["repeat"], remaining = v.remaining, paused = v.paused, callback = v.callback } end end
  return out
end)

-- ── signs (sign_define / sign_place …), drawn as extmarks with sign_text ──
local sign_defs, sign_placed, sign_ns, sign_id = {}, {}, nil, 0
local function sns() sign_ns = sign_ns or api.nvim_create_namespace("zett_signs") return sign_ns end
def("sign_define", function(name, o)
  if type(name) == "table" then for _, d in ipairs(name) do fn.sign_define(d.name, d) end return 0 end
  sign_defs[name] = vim.tbl_extend("force", sign_defs[name] or {}, o or {}, { name = name })
  return 0
end)
def("sign_undefine", function(name) if name == nil then sign_defs = {} else sign_defs[name] = nil end return 0 end)
def("sign_getdefined", function(name) local out = {} for n, d in pairs(sign_defs) do if name == nil or name == n then out[#out + 1] = d end end return out end)
def("sign_place", function(id, group, name, buf, o)
  o = o or {}
  local d = sign_defs[name]
  if not d then return -1 end
  if id == 0 or id == nil then sign_id = sign_id + 1 id = sign_id end
  local b = fn.bufnr(buf)
  local key = (group or "") .. ":" .. id
  local old = sign_placed[key]
  if old then pcall(api.nvim_buf_del_extmark, old.buf, sns(), old.mark) end
  local lnum = o.lnum or (old and old.lnum) or 1
  local mark = api.nvim_buf_set_extmark(b, sns(), lnum - 1, 0, { sign_text = d.text, sign_hl_group = d.texthl, line_hl_group = d.linehl, number_hl_group = d.numhl, priority = o.priority or 10 })
  sign_placed[key] = { id = id, group = group or "", name = name, buf = b, lnum = lnum, priority = o.priority or 10, mark = mark }
  return id
end)
def("sign_placelist", function(l) local out = {} for i, s in ipairs(l) do out[i] = fn.sign_place(s.id or 0, s.group or "", s.name, s.buffer, s) end return out end)
def("sign_unplace", function(group, o)
  o = o or {}
  for k, s in pairs(sign_placed) do
    if (group == "*" or s.group == (group or "")) and (o.id == nil or o.id == s.id) and (o.buffer == nil or fn.bufnr(o.buffer) == s.buf) then
      pcall(api.nvim_buf_del_extmark, s.buf, sns(), s.mark)
      sign_placed[k] = nil
    end
  end
  return 0
end)
def("sign_unplacelist", function(l) local out = {} for i, s in ipairs(l) do out[i] = fn.sign_unplace(s.group or "", s) end return out end)
def("sign_getplaced", function(buf, o)
  o = o or {}
  local by = {}
  for _, s in pairs(sign_placed) do
    if (buf == nil or fn.bufnr(buf) == s.buf) and (o.group == nil or o.group == "*" or o.group == s.group) and (o.lnum == nil or o.lnum == s.lnum) and (o.id == nil or o.id == s.id) then
      by[s.buf] = by[s.buf] or {}
      table.insert(by[s.buf], { id = s.id, group = s.group, name = s.name, lnum = s.lnum, priority = s.priority })
    end
  end
  local out = {}
  for b, signs in pairs(by) do table.sort(signs, function(x, y) return x.lnum < y.lnum end) out[#out + 1] = { bufnr = b, signs = signs } end
  if #out == 0 and buf ~= nil then out[1] = { bufnr = fn.bufnr(buf), signs = {} } end
  return out
end)
def("sign_jump", function(id, group, buf)
  for _, s in pairs(sign_placed) do if s.id == id and s.group == (group or "") and s.buf == fn.bufnr(buf) then z.buf.set_cursor(s.lnum, 0) return s.lnum end end
  return -1
end)

-- ── searchpair / searchpairpos: the matching end of a nested start…end (skip not evaluated) ──
local function pair_pos(s, m, e, flags, _, stopline)
  flags = flags or ""
  local back = flags:find("b") ~= nil
  local all = lines(0, -1)
  local r, c = z.buf.cursor()
  local depth = 0
  local function each_hit(l, pat)
    local out, at = {}, 0
    while at <= #l do local h = rx(l, pat, at) if not h then break end out[#out + 1] = h[1] at = h[2] > h[1] and h[2] or h[2] + 1 end
    return out
  end
  local from, to, step = r, back and 1 or #all, back and -1 or 1
  for i = from, to, step do
    if stopline and stopline > 0 and ((back and i < stopline) or (not back and i > stopline)) then break end
    local l = all[i] or ""
    local evs = {}
    for _, p in ipairs(each_hit(l, s)) do evs[#evs + 1] = { p, "s" } end
    for _, p in ipairs(each_hit(l, e)) do evs[#evs + 1] = { p, "e" } end
    if m and m ~= "" then for _, p in ipairs(each_hit(l, m)) do evs[#evs + 1] = { p, "m" } end end
    table.sort(evs, function(a, b) if back then return a[1] > b[1] end return a[1] < b[1] end)
    for _, ev in ipairs(evs) do
      local past = i ~= r or (back and ev[1] < c) or (not back and ev[1] > c)
      if past then
        local opener, closer = back and "e" or "s", back and "s" or "e"
        if ev[2] == opener then depth = depth + 1
        elseif ev[2] == closer then
          if depth == 0 then return i, ev[1] end
          depth = depth - 1
        elseif ev[2] == "m" and depth == 0 then return i, ev[1] end
      end
    end
  end
  return 0, 0
end
def("searchpairpos", function(s, m, e, flags, skip, stopline)
  local l, c = pair_pos(s, m, e, flags, skip, stopline)
  if l > 0 and not (flags or ""):find("n") then z.buf.set_cursor(l, c) end
  return { l, l > 0 and c + 1 or 0 }
end)
def("searchpair", function(...) return fn.searchpairpos(...)[1] end)

-- ── getregion / getregionpos: the text between two positions (v V or block) ──
local function region(p1, p2, o)
  o = o or {}
  local t = o.type or "v"
  local a, b = p1, p2
  if a[2] > b[2] or (a[2] == b[2] and a[3] > b[3]) then a, b = b, a end
  local out = {}
  for l = a[2], b[2] do
    local s = fn.getline(l) or ""
    local c1, c2
    if t == "V" then c1, c2 = 1, #s
    elseif t:sub(1, 1) == "\22" then c1, c2 = math.min(p1[3], p2[3]), math.max(p1[3], p2[3])
    else c1 = l == a[2] and a[3] or 1 c2 = l == b[2] and b[3] or #s end
    if o.exclusive and l == b[2] and t == "v" then c2 = c2 - 1 end
    out[#out + 1] = { l, c1, c2, s:sub(c1, c2) }
  end
  return out
end
def("getregion", function(p1, p2, o) local out = {} for i, r in ipairs(region(p1, p2, o)) do out[i] = r[4] end return out end)
def("getregionpos", function(p1, p2, o) local out = {} for i, r in ipairs(region(p1, p2, o)) do out[i] = { { 0, r[1], r[2], 0 }, { 0, r[1], r[3], 0 } } end return out end)

-- ── fuzzy matching (matchfuzzy, matchfuzzypos): every char in order, scored like Vim ──
local function fuzzy(text, pat)
  local tl, pl = text:lower(), pat:lower()
  local pos, score, at, prev = {}, 0, 1, -1
  for i = 1, #pl do
    local c = pl:sub(i, i)
    if c ~= " " then
      local f = tl:find(c, at, true)
      if not f then return nil end
      pos[#pos + 1] = f - 1
      score = score + 10
      if f - 1 == prev + 1 then score = score + 15 end
      if f == 1 or text:sub(f - 1, f - 1):match("[%s_%-/%.]") then score = score + 30 end
      score = score - (f - at)
      prev, at = f - 1, f + 1
    end
  end
  return score, pos
end
local function fuzzy_all(list, pat, o)
  o = o or {}
  local res = {}
  for _, it in ipairs(list or {}) do
    local text = type(it) == "string" and it or (o.key and it[o.key]) or (o.text_cb and o.text_cb(it)) or ""
    local sc, pos = fuzzy(text, pat)
    if sc then res[#res + 1] = { it, pos, sc } end
  end
  table.sort(res, function(a, b) return a[3] > b[3] end)
  if o.limit and o.limit > 0 then for i = #res, o.limit + 1, -1 do res[i] = nil end end
  return res
end
def("matchfuzzy", function(list, pat, o) local out = {} for i, r in ipairs(fuzzy_all(list, pat, o)) do out[i] = r[1] end return out end)
def("matchfuzzypos", function(list, pat, o) local a, b, c = {}, {}, {} for i, r in ipairs(fuzzy_all(list, pat, o)) do a[i], b[i], c[i] = r[1], r[2], r[3] end return { a, b, c } end)
def("matchstrlist", function(list, pat, o)
  local out = {}
  for i, s in ipairs(list or {}) do
    local at = 0
    while at <= #s do
      local m = rx(s, pat, at)
      if not m then break end
      local e = { idx = i - 1, byteidx = m[1], text = s:sub(m[1] + 1, m[2]) }
      if o and o.submatches then e.submatches = {} for g = 1, 9 do e.submatches[g] = m[g + 2] or "" end end
      out[#out + 1] = e
      at = m[2] > m[1] and m[2] or m[2] + 1
    end
  end
  return out
end)
def("matchbufline", function(b, pat, l1, l2, o)
  l1 = l1 == "$" and api.nvim_buf_line_count(b) or l1
  l2 = l2 == "$" and api.nvim_buf_line_count(b) or l2
  local ls = api.nvim_buf_get_lines(b == "%" and 0 or b, l1 - 1, l2, false)
  local out = {}
  for _, e in ipairs(fn.matchstrlist(ls, pat, o)) do e.lnum = l1 + e.idx e.idx = nil out[#out + 1] = e end
  return out
end)

-- ── small state: char search, cell widths, digraphs, the tag stack, input save ──
local charsearch = { char = "", forward = 1, ["until"] = 0 }
def("getcharsearch", function() return vim.deepcopy and vim.deepcopy(charsearch) or charsearch end)
def("setcharsearch", function(d) for k, v in pairs(d or {}) do charsearch[k] = v end end)
local cellwidths = {}
def("setcellwidths", function(l) cellwidths = l or {} return 0 end)
def("getcellwidths", function() return cellwidths end)
local digraphs = { ["a:"] = "ä", ["o:"] = "ö", ["u:"] = "ü", ["A:"] = "Ä", ["O:"] = "Ö", ["U:"] = "Ü", ss = "ß", ["e'"] = "é", ["e!"] = "è", ["a'"] = "á", ["a!"] = "à", ["n?"] = "ñ", ["c,"] = "ç", Eu = "€", Pd = "£", Ye = "¥", Co = "©", Rg = "®", ["->"] = "→", ["<-"] = "←", OK = "✓", XX = "✗", ["*X"] = "×", ["-:"] = "÷", ["+-"] = "±", ["!="] = "≠", ["=<"] = "≤", [">="] = "≥", DG = "°", My = "µ", ["p*"] = "π", ["l*"] = "λ", ["a*"] = "α", ["b*"] = "β", ["NS"] = " " }
def("digraph_get", function(k) return digraphs[k] or digraphs[k:reverse()] or "" end)
def("digraph_getlist", function(_) local out = {} for k, v in pairs(digraphs) do out[#out + 1] = { k, v } end table.sort(out, function(a, b) return a[1] < b[1] end) return out end)
def("digraph_set", function(k, v) digraphs[k] = v return true end)
def("digraph_setlist", function(l) for _, p in ipairs(l or {}) do digraphs[p[1]] = p[2] end return true end)
local tagstacks = {}
def("gettagstack", function(w) w = (w == nil or w == 0) and api.nvim_get_current_win() or w local s = tagstacks[w] or { items = {}, curidx = 1 } return { items = s.items, curidx = s.curidx, length = #s.items } end)
def("settagstack", function(w, d, action)
  w = (w == nil or w == 0) and api.nvim_get_current_win() or w
  local s = tagstacks[w] or { items = {}, curidx = 1 }
  if action == "a" then for _, it in ipairs(d.items or {}) do s.items[#s.items + 1] = it end
  elseif d.items then s.items = d.items end
  if d.curidx then s.curidx = d.curidx end
  tagstacks[w] = s
  return 0
end)
def("inputsave", function() return 0 end)
def("inputrestore", function() return 0 end)
def("inputsecret", function(p, d) return fn.input(p, d) end)
def("inputdialog", function(p, d, _) return fn.input(p, d) end)

-- ── windows, tabs, folders, the editor's state ──
def("gettabinfo", function(t)
  local out = {}
  for i, h in ipairs(api.nvim_list_tabpages()) do
    if t == nil or t == i then
      local ws = {} if i == fn.tabpagenr() then for _, x in ipairs(wins()) do ws[#ws + 1] = handle(x.nr) end end
      out[#out + 1] = { tabnr = i, windows = ws, variables = {} , id = h }
    end
  end
  return out
end)
def("haslocaldir", function(_, _) return fn.getcwd(0) ~= fn.getcwd(-1) and 1 or 0 end)
def("win_id2tabwin", function(w) for i, x in ipairs(wins()) do if handle(x.nr) == w then return { fn.tabpagenr(), i } end end return { 0, 0 } end)
def("win_execute", function(w, cmd, _)
  local cur = api.nvim_get_current_win()
  if w ~= cur and api.nvim_win_is_valid(w) then api.nvim_set_current_win(w) end
  local ok, out = pcall(api.nvim_exec2, type(cmd) == "table" and table.concat(cmd, "\n") or cmd, { output = true })
  if w ~= cur then pcall(api.nvim_set_current_win, cur) end
  return ok and out.output or ""
end)
def("winrestcmd", function() local s = {} for i, x in ipairs(wins()) do s[#s + 1] = i .. "resize " .. x.height s[#s + 1] = "vert " .. i .. "resize " .. x.width end return table.concat(s, "|") end)
def("win_splitmove", function(w, target, o)
  o = o or {}
  if not api.nvim_win_is_valid(w) or not api.nvim_win_is_valid(target) then return -1 end
  local b = api.nvim_win_get_buf(w)
  api.nvim_set_current_win(target)
  ex((o.vertical and "vertical " or "") .. (o.rightbelow and "rightbelow " or "aboveleft ") .. "sbuffer " .. b)
  pcall(api.nvim_win_close, w, false)
  return 0
end)
def("win_move_separator", function(w, off) local x = win_of(w) if not x then return false end api.nvim_win_set_width(w, x.width + off) return true end)
def("win_move_statusline", function(w, off) local x = win_of(w) if not x then return false end api.nvim_win_set_height(w, x.height + off) return true end)
def("changenr", function() return z.buf.undo_current() end)
def("undofile", function(name) local d = vim.o.undodir ~= "" and vim.split(vim.o.undodir, ",")[1] or (fn.stdpath("state") .. "/undo") return d .. "/" .. fn.fnamemodify(name, ":p"):gsub("/", "%%") end)
def("virtcol2col", function(_, lnum, vcol)
  local s = fn.getline(lnum) or ""
  local ts, v = vim.bo.tabstop or 8, 0
  for i = 1, #s do
    local ch = s:sub(i, i)
    v = v + (ch == "\t" and (ts - v % ts) or 1)
    if v >= vcol then return i end
  end
  return -1
end)
def("reg_recorded", function() return vim.g._zett_last_recorded or "" end)
def("state", function(what) local s = "" if z.buf.recording() ~= "" then s = s .. "" end return what and s:gsub("[^" .. vim.pesc(what) .. "]", "") or s end)
def("wait", function(ms, cond, interval)
  local ok = vim.wait(ms, type(cond) == "function" and cond or function() return cond ~= 0 and cond ~= false end, interval)
  return ok and 0 or -1
end)
def("interrupt", function() error("Keyboard interrupt") end)
def("islocked", function(_) return 0 end)
def("eventhandler", function() return 0 end)
def("did_filetype", function() return 0 end)
def("getmousepos", function() return { screenrow = 0, screencol = 0, winid = 0, winrow = 0, wincol = 0, line = 0, column = 0, coladd = 0 } end)
def("pum_getpos", function() return {} end)
def("wildmenumode", function() return 0 end)
def("getstacktrace", function() local out = {} for l in debug.traceback("", 2):gmatch("\n%s*([^\n]+)") do out[#out + 1] = { filepath = l:match("^([^:]+)") or "", lnum = tonumber(l:match(":(%d+):")) or 0 } end return out end)
def("getscriptinfo", function(o)
  local out = {}
  for i, f in ipairs(vim.g._zett_sourced or {}) do if not (o and o.name) or f:find(o.name) then out[#out + 1] = { sid = i, name = f, autoload = false, version = 1 } end end
  return out
end)
def("fullcommand", function(name)
  local ok, p = pcall(api.nvim_parse_cmd, name, {})
  return ok and p.cmd or ""
end)
def("expandcmd", function(s) return (s:gsub("%%:?[%w:.~]*", function(m) return fn.expand(m) end):gsub("~/", (os.getenv("HOME") or "~") .. "/")) end)
def("getloclist", function(_, what) return fn.getqflist(what) end)
def("strptime", function(fmt, s)
  local map = { Y = "(%d%d%d%d)", m = "(%d%d?)", d = "(%d%d?)", H = "(%d%d?)", M = "(%d%d?)", S = "(%d%d?)" }
  local keys = {}
  local pat = "^" .. fmt:gsub("([%^%$%(%)%.%[%]%*%+%-%?])", "%%%1"):gsub("%%(%a)", function(k) if map[k] then keys[#keys + 1] = k return map[k] end return ".-" end)
  local caps = { s:match(pat) }
  if #caps == 0 then return 0 end
  local t = { year = 1970, month = 1, day = 1, hour = 0, min = 0, sec = 0 }
  local names = { Y = "year", m = "month", d = "day", H = "hour", M = "min", S = "sec" }
  for i, k in ipairs(keys) do t[names[k]] = tonumber(caps[i]) end
  return os.time(t)
end)
-- soundfold: a rough phonetic key (vowels after the first letter dropped, doubles merged)
def("soundfold", function(w) local s = w:lower() local first = s:sub(1, 1) local rest = s:sub(2):gsub("[aeiouyhw]", "") return (first .. rest):gsub("(.)%1+", "%1") end)
def("charclass", function(s) local c = (s or ""):sub(1, 1) if c == "" or c:match("%s") then return 0 end if c:match("[%w_]") or c:byte() > 127 then return 2 end return 1 end)
def("list2blob", function(l) local t = {} for i, b in ipairs(l or {}) do t[i] = string.char(b) end return table.concat(t) end)
def("blob2list", function(b) local t = {} for i = 1, #(b or "") do t[i] = b:byte(i) end return t end)
def("readblob", function(f, off, size) local h = io.open(fn.expand(f), "rb") if not h then return "" end local d = h:read("*a") h:close() off = off or 0 return d:sub(off + 1, size and (off + size) or nil) end)
-- getcompletion(pat, type): command, option, file, dir, buffer, highlight, filetype, color, event, help (not yet)
def("getcompletion", function(pat, kind, _)
  local items = {}
  local function from(list) for _, v in ipairs(list) do if v:sub(1, #pat) == pat then items[#items + 1] = v end end end
  if kind == "file" or kind == "dir" or kind == "file_in_path" then
    local dir, base = pat:match("^(.*/)([^/]*)$")
    dir = dir or "" base = base or pat
    for _, n in ipairs(fn.readdir(dir == "" and "." or dir)) do
      if n:sub(1, #base) == base then
        local full = dir .. n
        local isdir = fn.isdirectory(full) == 1
        if kind ~= "dir" or isdir then items[#items + 1] = full .. (isdir and "/" or "") end
      end
    end
    table.sort(items)
    return items
  elseif kind == "option" then from(z.option_names and z.option_names() or {})
  elseif kind == "buffer" then for _, b in ipairs(z.buffers()) do if b ~= "" and b:find(pat, 1, true) then items[#items + 1] = b end end
  elseif kind == "highlight" then from(z.hl_names and z.hl_names() or {})
  elseif kind == "command" then local cs = {} for n in pairs(api.nvim_get_commands({})) do cs[#cs + 1] = n end from(cs)
  elseif kind == "filetype" then from({ "c", "cpp", "css", "go", "html", "javascript", "json", "lua", "markdown", "python", "qml", "rust", "sh", "toml", "typescript", "vim", "yaml", "zig" })
  elseif kind == "event" then from({ "BufEnter", "BufLeave", "BufNewFile", "BufReadPost", "BufReadPre", "BufWinEnter", "BufWritePost", "BufWritePre", "CmdlineEnter", "CmdlineLeave", "ColorScheme", "CursorHold", "CursorMoved", "CursorMovedI", "FileType", "FocusGained", "FocusLost", "InsertEnter", "InsertLeave", "LspAttach", "ModeChanged", "TextChanged", "TextChangedI", "TextYankPost", "UIEnter", "VimEnter", "VimLeavePre", "WinEnter", "WinLeave" })
  elseif kind == "color" then from(fn.getcompletion and {} or {})
  end
  table.sort(items)
  return items
end)
def("nvim__id", function(x) return x end)
def("nvim__id_array", function(x) return x end)
def("nvim__id_dict", function(x) return x end)
def("nvim__id_float", function(x) return x end)
api.nvim__id = api.nvim__id or function(x) return x end
api.nvim__id_array = api.nvim__id_array or function(x) return x end
api.nvim__id_dict = api.nvim__id_dict or function(x) return x end
api.nvim__id_float = api.nvim__id_float or function(x) return x end
api.nvim_call_dict_function = api.nvim_call_dict_function or function(d, f, args) local dict = type(d) == "string" and vim.fn.eval and vim.fn.eval(d) or d return dict[f](unpack(args or {})) end
api.nvim_command_output = api.nvim_command_output or function(c) return api.nvim_exec2(c, { output = true }).output end

-- ── zett.plug("owner/repo", { opts = {…} }): a Neovim plugin from GitHub, cloned on first start
-- into ~/.local/share/zett/site/pack/plug/start/<repo>; then require(main).setup(opts) when opts
-- (or setup = true) is given. :PlugUpdate pulls them all, :PlugList lists them. ──
local plug_root = (os.getenv("XDG_DATA_HOME") or ((os.getenv("HOME") or "") .. "/.local/share")) .. "/zett/site/pack/plug/start"
local plugged = {}
function zett.plug(repo, o)
  o = o or {}
  local name = o.name or repo:match("([^/]+)$"):gsub("%.git$", "")
  local dir = plug_root .. "/" .. name
  -- dir = "/a/worktree" (as lazy.nvim): a plugin of my own, linked in, never cloned or pulled
  if o.dir then
    local src = vim.fn.expand(o.dir)
    if vim.fn.isdirectory(src) == 0 then
      vim.notify("plug: no folder " .. src, vim.log.levels.ERROR)
      return
    end
    vim.fn.mkdir(plug_root, "p")
    if vim.fn.isdirectory(dir) == 0 then vim.fn.system({ "ln", "-sfn", src, dir }) end
  end
  if vim.fn.isdirectory(dir) == 0 then
    vim.fn.mkdir(plug_root, "p")
    local url = repo:find("://") and repo or ("https://github.com/" .. repo .. ".git")
    local out = vim.fn.system({ "git", "clone", "--depth", "1", "--quiet", url, dir })
    if vim.v.shell_error ~= 0 or vim.fn.isdirectory(dir) == 0 then
      vim.notify("plug: could not clone " .. repo .. ": " .. tostring(out), vim.log.levels.ERROR)
      return
    end
  end
  local lua_dir = dir .. "/lua"
  local mine = lua_dir .. "/?.lua;" .. lua_dir .. "/?/init.lua;"
  if not package.path:find(mine, 1, true) then package.path = mine .. package.path end
  plugged[#plugged + 1] = { repo = repo, name = name, dir = dir, local_dir = o.dir ~= nil }
  -- after startup (:ZettStore, :lua zett.plug): its plugin/ files now, as at startup they run with the rest
  if vim._zett_entered then
    for _, f in ipairs(vim.fn.glob(dir .. "/plugin/**/*.{vim,lua}", false, true)) do
      local ok, err = pcall(vim.cmd, "source " .. vim.fn.fnameescape(f))
      if not ok then vim.notify("plug: " .. f .. ": " .. tostring(err), vim.log.levels.ERROR) end
    end
  end
  if o.opts ~= nil or o.setup then
    local main = o.main or name:gsub("%.nvim$", ""):gsub("%.lua$", ""):gsub("^nvim%-", "")
    -- a built-in stand-in of the same name (zett's own fidget) steps aside for the real plugin
    if type(package.loaded[main]) == "table" and package.loaded[main].zett_builtin then
      zett._replaced = zett._replaced or {}
      zett._replaced[main] = true
      package.loaded[main] = nil
    end
    local ok, mod = pcall(require, main)
    if not ok then vim.notify("plug: require('" .. main .. "'): " .. tostring(mod), vim.log.levels.ERROR) return end
    if type(mod) == "table" and type(mod.setup) == "function" then
      local ok2, err = pcall(mod.setup, o.opts or {})
      if not ok2 then vim.notify("plug: " .. main .. ".setup: " .. tostring(err), vim.log.levels.ERROR) end
    end
  end
  return dir
end
api.nvim_create_user_command("PlugUpdate", function()
  for _, p in ipairs(plugged) do
    if p.local_dir then print(p.name .. ": a local folder, not pulled") goto next end
    local out = vim.fn.system({ "git", "-C", p.dir, "pull", "--ff-only", "--quiet" })
    print(p.name .. ": " .. (vim.v.shell_error == 0 and "up to date" or out))
    ::next::
  end
end, {})
api.nvim_create_user_command("PlugList", function()
  for _, p in ipairs(plugged) do print(p.name .. "  " .. p.dir) end
end, {})

-- ── LSP progress as Neovim gives it: a client per server (vim.lsp.get_client_by_id) whose
-- `progress` drains what came, and the LspProgress autocommand (fidget.nvim, lualine) ──
local lsp_clients, lsp_ids, lsp_tokens = {}, {}, {}
local function lsp_client(name)
  local id = lsp_ids[name]
  if not id then
    id = #lsp_clients + 1
    lsp_ids[name] = id
    local q = {}
    local prog = setmetatable({ pending = {}, push = function(_, v) q[#q + 1] = v end }, { __call = function() return table.remove(q, 1) end })
    lsp_clients[id] = { id = id, name = name, progress = prog, config = { name = name }, server_capabilities = {}, attached_buffers = {}, initialized = true, is_stopped = function() return false end }
  end
  return lsp_clients[id]
end
vim.lsp.get_client_by_id = function(id) return lsp_clients[id] end
vim.lsp.get_clients = function(f)
  f = f or {}
  local out = {}
  for _, c in ipairs(lsp_clients) do if (f.name == nil or f.name == c.name) and (f.id == nil or f.id == c.id) then out[#out + 1] = c end end
  return out
end
vim.lsp.get_active_clients = vim.lsp.get_clients
vim.lsp.status = function()
  local parts = {}
  for _, c in ipairs(lsp_clients) do
    for p in c.progress do
      local v = p.value or {}
      if v.kind ~= "end" then parts[#parts + 1] = (v.percentage and (v.percentage .. "%: ") or "") .. (v.title or "") .. (v.message and (" " .. v.message) or "") end
    end
  end
  return table.concat(parts, ", ")
end
z.on("lsp_progress", function(e)
  local c = lsp_client(e.server)
  local key = e.server .. "\0" .. e.token
  local kind = e.done == "true" and "end" or (lsp_tokens[key] and "report" or "begin")
  lsp_tokens[key] = kind ~= "end" or nil
  local value = { kind = kind, title = e.title ~= "" and e.title or nil, message = e.message ~= "" and e.message or nil, percentage = tonumber(e.percent) }
  c.progress:push({ token = e.token, value = value })
  local ev = { event = "LspProgress", buf = 0, match = kind, data = { client_id = c.id, params = { token = e.token, value = value } } }
  for _, a in ipairs(made_au) do
    if a.event == "LspProgress" and a.callback then
      local ok, err = pcall(a.callback, ev)
      if not ok then vim.notify("LspProgress: " .. tostring(err), vim.log.levels.ERROR) end
    end
  end
end)

-- ── highlight namespaces: groups set in one (fidget's Normal, FloatBorder) stay out of the
-- editor's colours; a group only a namespace defines (FidgetNoBlend) is made global so the
-- float can still draw it ──
local hl_ns, win_ns, cur_ns = {}, {}, 0
local set_hl_global = api.nvim_set_hl
api.nvim_set_hl = function(ns, name, v)
  if ns == nil or ns == 0 then return set_hl_global(0, name, v) end
  hl_ns[ns] = hl_ns[ns] or {}
  hl_ns[ns][name] = v
  local ok, have = pcall(api.nvim_get_hl, 0, { name = name })
  if not ok or not have or next(have) == nil then set_hl_global(0, name, v) end
end
api.nvim_win_set_hl_ns = function(w, ns) win_ns[w] = ns end
api.nvim_get_hl_ns = function(o) if o and o.winid then return win_ns[o.winid] or -1 end return cur_ns end
api.nvim_set_hl_ns = function(ns) cur_ns = ns end
api.nvim_set_hl_ns_fast = function(ns) cur_ns = ns end
local get_hl_global = api.nvim_get_hl
api.nvim_get_hl = function(ns, o)
  if ns and ns ~= 0 and hl_ns[ns] then
    if o and o.name then return hl_ns[ns][o.name] or {} end
    return hl_ns[ns]
  end
  return get_hl_global(0, o)
end

-- ── processes, waiting, diffs (Neovim's built-ins; gitsigns, conform, lint run on these) ──

-- vim.system(cmd, opts, on_exit): the process runs beside the editor; on_exit({code, signal,
-- stdout, stderr}) on the main loop when it ends; :wait(ms) blocks for it, :kill(sig) ends it
function vim.system(cmd, opts, on_exit)
  if type(opts) == "function" then opts, on_exit = nil, opts end
  opts = opts or {}
  local stdin = opts.stdin
  if type(stdin) == "table" then stdin = table.concat(stdin, "\n") end
  if stdin == true then stdin = "" end
  local env
  if opts.env then
    env = {}
    for k, v in pairs(opts.env) do env[tostring(k)] = tostring(v) end
  end
  local result
  local function fix(r)
    if opts.text then
      r.stdout = (r.stdout or ""):gsub("\r\n", "\n")
      r.stderr = (r.stderr or ""):gsub("\r\n", "\n")
    end
    -- stdout = false: not wanted (Neovim gives nil then)
    if opts.stdout == false then r.stdout = nil end
    if opts.stderr == false then r.stderr = nil end
    return r
  end
  local id, pid = z.job.spawn(cmd, { cwd = opts.cwd, env = env, clear_env = opts.clear_env, stdin = stdin }, function(r)
    result = fix(r)
    if on_exit then on_exit(result) end
  end)
  local obj = { pid = pid }
  function obj:wait(ms)
    if result then return result end
    local r = z.job.wait(id, ms)
    if r then return result or fix(r) end
    z.job.kill(id, 9)
    return { code = 124, signal = 9, stdout = "", stderr = "" }
  end
  function obj:kill(sig)
    local n = ({ sigterm = 15, sigkill = 9, sigint = 2, sighup = 1 })[tostring(sig or ""):lower()] or tonumber(sig) or 15
    z.job.kill(id, n)
  end
  function obj:write(_) end
  function obj:is_closing() return result ~= nil end
  if opts.timeout then
    z.defer(opts.timeout, function() if not result then z.job.kill(id, 15) end end)
  end
  return obj
end

-- vim.wait(ms, cond, interval): runs the loop (jobs ending, timers) until cond() is true or time is up
function vim.wait(ms, cond, interval)
  local stop = z.hrtime() + (ms or 0) * 1e6
  interval = interval or 20
  while true do
    if cond then
      local ok, v = pcall(cond)
      if ok and v then return true end
    end
    if z.hrtime() >= stop then return false, cond and -1 or nil end
    z.job.pump(math.max(1, math.min(interval, math.ceil((stop - z.hrtime()) / 1e6))))
  end
end

-- vim.diff(a, b, opts) / vim.text.diff: lines that changed. result_type "indices" gives
-- { {start_a, count_a, start_b, count_b}, … } as xdiff does (a count of 0 starts at the line
-- before); "unified" (the default) the text of a unified diff with ctxlen lines around each hunk
function vim.diff(a, b, opts)
  opts = opts or {}
  local ops = z.diff(a or "", b or "")
  local hunks, ia, ib, cur = {}, 0, 0, nil
  for _, o in ipairs(ops) do
    local k = o[1]
    if k == " " then
      if cur then hunks[#hunks + 1] = cur; cur = nil end
      ia, ib = ia + 1, ib + 1
    else
      cur = cur or { ia + 1, 0, ib + 1, 0, lines = {} }
      if k == "-" then ia = ia + 1; cur[2] = cur[2] + 1 else ib = ib + 1; cur[4] = cur[4] + 1 end
      cur.lines[#cur.lines + 1] = o
    end
  end
  if cur then hunks[#hunks + 1] = cur end
  for _, h in ipairs(hunks) do
    if h[2] == 0 then h[1] = h[1] - 1 end
    if h[4] == 0 then h[3] = h[3] - 1 end
  end
  if opts.on_hunk then
    for _, h in ipairs(hunks) do
      if opts.on_hunk(h[1], h[2], h[3], h[4]) == -1 then break end
    end
    return nil
  end
  if opts.result_type == "indices" then
    local out = {}
    for i, h in ipairs(hunks) do out[i] = { h[1], h[2], h[3], h[4] } end
    return out
  end
  -- unified: the changed lines with up to ctxlen unchanged ones around them; hunks whose
  -- context would touch are one hunk (as diff -u does)
  local ctx = opts.ctxlen or 3
  local changed = {}
  for i, o in ipairs(ops) do if o[1] ~= " " then changed[#changed + 1] = i end end
  local groups = {}
  for _, i in ipairs(changed) do
    local g = groups[#groups]
    if g and i - g[2] <= 2 * ctx + 1 then g[2] = i else groups[#groups + 1] = { i, i } end
  end
  -- each op's line number on both sides (before it)
  local na, nb, before = 0, 0, {}
  for i, o in ipairs(ops) do
    before[i] = { na, nb }
    if o[1] ~= "+" then na = na + 1 end
    if o[1] ~= "-" then nb = nb + 1 end
  end
  local out = {}
  for _, g in ipairs(groups) do
    local s, e = math.max(1, g[1] - ctx), math.min(#ops, g[2] + ctx)
    local ca, cb = 0, 0
    for i = s, e do
      if ops[i][1] ~= "+" then ca = ca + 1 end
      if ops[i][1] ~= "-" then cb = cb + 1 end
    end
    local sa, sb = before[s][1] + (ca > 0 and 1 or 0), before[s][2] + (cb > 0 and 1 or 0)
    out[#out + 1] = string.format("@@ -%d,%d +%d,%d @@", sa, ca, sb, cb)
    for i = s, e do out[#out + 1] = ops[i][1] .. ops[i][2] end
  end
  return #out > 0 and table.concat(out, "\n") .. "\n" or ""
end
vim.text = vim.text or {}
vim.text.diff = vim.diff

-- the rest of libuv's file calls gitsigns and others use
function vim.uv.fs_realpath(p)
  local out = z.fs.exec({ "realpath", "-e", p })
  out = out and out:gsub("\n$", "")
  return out ~= "" and out or nil
end
function vim.uv.fs_lstat(p) return vim.uv.fs_stat(p) end
function vim.uv.fs_readlink(p)
  local out = z.fs.exec({ "readlink", p })
  out = out and out:gsub("\n$", "")
  return out ~= "" and out or nil
end
-- fs_poll: as fs_event (a change on the path), at the interval asked
function vim.uv.new_fs_poll()
  local ev = vim.uv.new_fs_event()
  local h = {}
  function h:start(path, _, cb) return ev:start(path, {}, function(err) cb(err, {}, {}) end) end
  function h:stop() return ev:stop() end
  function h:close() return ev:stop() end
  function h:is_active() return ev:is_active() end
  return h
end
-- check handles run once per loop turn: here on every timer tick (close enough for plugins)
function vim.uv.new_check()
  local t = vim.uv.new_timer()
  local h = {}
  function h:start(cb) t:start(0, 10, cb) end
  function h:stop() t:stop() end
  function h:close() t:close() end
  return h
end
vim.uv.loop = vim.uv.loop or vim.uv
vim.loop = vim.loop or vim.uv

-- vim.fs.parents(start): the folders above start, nearest first
function vim.fs.parents(start)
  local p = vim.fs.normalize(start)
  return function()
    local up = vim.fs.dirname(p)
    if not up or up == p or up == "." then return nil end
    p = up
    return up
  end
end

-- vim.fs.find(names, { path, upward, stop, type, limit }): paths of the matching files or folders;
-- names a string, a list, or function(name, path) → bool
function vim.fs.find(names, opts)
  opts = opts or {}
  local limit = opts.limit or 1
  local want = type(names) == "function" and names or function(name)
    for _, n in ipairs(type(names) == "table" and names or { names }) do if n == name then return true end end
    return false
  end
  local function fits(p)
    local st = z.fs.stat(p)
    if not st then return false end
    if opts.type == "directory" then return st.dir end
    if opts.type == "file" then return not st.dir end
    return true
  end
  local out = {}
  local start = vim.fs.normalize(opts.path or z.root() or vim.uv.cwd())
  local stop = opts.stop and vim.fs.normalize(opts.stop)
  if opts.upward then
    local dir = start
    while dir do
      for _, e in ipairs(z.fs.list(dir)) do
        if want(e.name, dir) and fits(e.path) then
          out[#out + 1] = e.path
          if #out >= limit then return out end
        end
      end
      if dir == stop or dir == "/" then break end
      local up = vim.fs.dirname(dir)
      if not up or up == dir then break end
      dir = up ~= "" and up or "/"
    end
  else
    local queue = { start }
    while #queue > 0 do
      local dir = table.remove(queue, 1)
      for _, e in ipairs(z.fs.list(dir)) do
        if want(e.name, dir) and fits(e.path) then
          out[#out + 1] = e.path
          if #out >= limit then return out end
        end
        local st = z.fs.stat(e.path)
        if st and st.dir and e.path ~= stop then queue[#queue + 1] = e.path end
      end
    end
  end
  return out
end

-- vim.deep_equal(a, b): the same values all the way down (tables by content)
function vim.deep_equal(a, b)
  if a == b then return true end
  if type(a) ~= "table" or type(b) ~= "table" then return false end
  for k, v in pairs(a) do if not vim.deep_equal(v, b[k]) then return false end end
  for k in pairs(b) do if a[k] == nil then return false end end
  return true
end
-- (no vim.is_thread: there are no Lua worker threads here, and plugins take its presence to mean
-- there are: gitsigns then diffs on a vim.uv.new_work thread)
vim.hl = vim.hl or {}
vim.hl.priorities = vim.hl.priorities or { syntax = 50, treesitter = 100, semantic_tokens = 125, diagnostics = 150, user = 200 }
-- text in another encoding: only to and from UTF-8 itself here (nil otherwise, as Neovim gives on failure)
function vim.iconv(s, from, to)
  local u = function(e) e = (e or ""):lower():gsub("%-", "") return e == "utf8" or e == "" end
  if u(from) and u(to) then return s end
  return nil
end
-- vim.str_utfindex(s, encoding, index) (0.11) or (s, index): the code-point or UTF-16 index of byte index;
-- vim.str_byteindex(s, encoding, index) (0.11) or (s, index, use_utf16): the byte index of a character index
local function units(c, enc)
  if enc == "utf-16" then return c >= 0x10000 and 2 or 1 end
  if enc == "utf-8" then return #utf8.char(c) end
  return 1
end
function vim.str_utfindex(s, enc, idx)
  if type(enc) ~= "string" then enc, idx = "utf-32", enc end
  idx = idx or #s
  local n, pos = 0, 1
  for p, c in utf8.codes(s) do
    if p > idx then break end
    n = n + units(c, enc)
    pos = p
  end
  return n, (enc == "utf-16" and n or nil)
end
function vim.str_byteindex(s, enc, idx, _)
  if type(enc) ~= "string" then enc, idx = (idx and "utf-16" or "utf-32"), enc end
  local n = 0
  for p, c in utf8.codes(s) do
    if n >= idx then return p - 1 end
    n = n + units(c, enc)
  end
  return #s
end

-- ── decoration providers (nvim_set_decoration_provider): run for each window before it is drawn ──
-- on_start, then per window on_win("win", winid, bufnr, toprow, botrow) (0-based, as Neovim) and,
-- unless it returned false, on_line("line", winid, bufnr, row) for each row shown, then on_end.
-- What they set (extmarks: signs, highlights) shows in that frame. gitsigns draws its signs this way.
local decor = {}
local decor_err = {}
api.nvim_set_decoration_provider = function(ns, o)
  decor[ns] = (o and next(o)) and o or nil
  vim._decor_on = next(decor) ~= nil
end
local function run(ns, name, f, ...)
  local ok, r = pcall(f, ...)
  if not ok and not decor_err[ns .. name] then
    decor_err[ns .. name] = true
    vim.notify("decoration provider " .. name .. ": " .. tostring(r), vim.log.levels.ERROR)
  end
  return ok and r
end
-- called by the editor before a frame (only when a provider exists)
function vim._decor_frame()
  for ns, o in pairs(decor) do
    if o.on_start and run(ns, "on_start", o.on_start, "start", 0) == false then goto next end
    for _, w in ipairs(wins()) do
      local top = (w.top or 1) - 1
      local n = api.nvim_buf_line_count(w.buf)
      local bot = math.max(top, math.min(n, top + (w.height or 1)) - 1)
      if o.on_buf then run(ns, "on_buf", o.on_buf, "buf", w.buf, 0) end
      local want = true
      if o.on_win then want = run(ns, "on_win", o.on_win, "win", handle(w.nr), w.buf, top, bot) ~= false end
      if want and o.on_line then
        for r = top, bot do run(ns, "on_line", o.on_line, "line", handle(w.nr), w.buf, r) end
      end
    end
    if o.on_end then run(ns, "on_end", o.on_end, "end", 0) end
    ::next::
  end
end

-- namespace options (nvim__ns_set: the windows a namespace draws in) and a forced redraw:
-- every namespace draws everywhere here and every frame is drawn fresh, so both only answer
api.nvim__ns_set = api.nvim__ns_set or function() end
api.nvim__ns_get = api.nvim__ns_get or function() return { wins = {} } end
api.nvim__redraw = api.nvim__redraw or function() end

-- every vim.uv handle gets what libuv gives them all: is_closing (true once closed), close(cb) calling cb,
-- ref / unref / has_ref, get_type, and getpath on watchers (gitsigns' watcher calls is_closing first)
for kind, name in pairs({ timer = "new_timer", fs_event = "new_fs_event", fs_poll = "new_fs_poll", check = "new_check",
  idle = "new_idle", prepare = "new_prepare", async = "new_async", signal = "new_signal", pipe = "new_pipe",
  tcp = "new_tcp", tty = "new_tty", udp = "new_udp", poll = "new_poll", process = "new_process" }) do
  local make = vim.uv[name]
  if type(make) == "function" then
    vim.uv[name] = function(...)
      local h = make(...)
      if type(h) ~= "table" then return h end
      local closing, refd, close = false, true, h.close
      h.close = function(self, cb)
        if closing then return end
        closing = true
        if close then close(self) end
        if cb then z.defer(0, cb) end
      end
      h.is_closing = h.is_closing or function() return closing end
      h.ref = h.ref or function() refd = true end
      h.unref = h.unref or function() refd = false end
      h.has_ref = h.has_ref or function() return refd end
      h.get_type = h.get_type or function() return kind end
      if kind == "fs_event" or kind == "fs_poll" then h.getpath = h.getpath or function(self) return self.path end end
      return h
    end
  end
end

-- ── the rest of vim.fn / vim.api: jobs and channels, completion, contexts, prompts, screen, syntax, spell ──
do
  local function set(name, f) fn[name] = f rawset(vim.fn, name, f) end
  -- jobs: z.job hands the whole output at exit, so on_stdout/on_stderr get it then, in lines
  local jobs, chans = {}, {}
  local function split_out(s) return vim.split(s or "", "\n", { plain = true }) end
  -- jobs on z.job.stream: output comes while the program runs, in Neovim's shape: on_stdout(id, data, "stdout")
  -- with data the text split at line breaks (its first item ends the last call's unfinished line, "" at the end
  -- when it ended with one), {""} once at the end (EOF); stdout_buffered: all of it in one call at the end
  set("jobstart", function(cmd, o)
    o = o or {}
    if type(cmd) == "string" then cmd = { vim.o.shell or "sh", "-c", cmd } end
    local j = { o = o, buf = { stdout = {}, stderr = {} }, done = nil }
    local function deliver(kind, text)
      local cb = o["on_" .. kind]
      local data = vim.split(text, "\n", { plain = true })
      if o[kind .. "_buffered"] then
        local b = j.buf[kind]
        if #b > 0 then b[#b] = b[#b] .. table.remove(data, 1) end
        for _, l in ipairs(data) do b[#b + 1] = l end
      elseif cb then cb(j.id, data, kind) end
    end
    local env
    if o.env then env = {} for k, v in pairs(o.env) do env[tostring(k)] = tostring(v) end end
    local ok, zid, pid = pcall(z.job.stream, cmd, { cwd = o.cwd, env = env, clear_env = o.clear_env }, function(kind, data)
      if kind == "exit" then
        for _, k in ipairs({ "stdout", "stderr" }) do
          local cb = o["on_" .. k]
          if cb then cb(j.id, o[k .. "_buffered"] and j.buf[k] or { "" }, k) end
        end
        j.done = data
        if o.on_exit then o.on_exit(j.id, data, "exit") end
      else deliver(kind, data) end
    end)
    if not ok then return -1 end
    j.id, j.zid, j.pid = zid + 2, zid, pid
    jobs[j.id] = j chans[j.id] = j
    return j.id
  end)
  set("termopen", function(cmd, o) return fn.jobstart(cmd, o) end)
  set("jobpid", function(id) local j = jobs[id] if not j then error("E900: Invalid channel id") end return j.pid end)
  set("jobstop", function(id) local j = jobs[id] if not j or j.done then return 0 end z.job.kill(j.zid, 15) return 1 end)
  -- jobwait: the loop runs (output and ends arrive) until each has ended or the time is up: its code, -1 time out, -3 no such job
  set("jobwait", function(ids, ms)
    local stop = (ms and ms >= 0) and (z.hrtime() + ms * 1e6) or nil
    while true do
      local left = false
      for _, id in ipairs(ids) do local j = jobs[id] if j and j.done == nil then left = true end end
      if not left or (stop and z.hrtime() >= stop) then break end
      z.job.pump(10)
    end
    local out = {}
    for i, id in ipairs(ids) do local j = jobs[id] out[i] = not j and -3 or j.done or -1 end
    return out
  end)
  set("jobresize", function(id) return jobs[id] and 1 or 0 end)
  -- chansend: text (or a list of lines, joined by line breaks) to the job's stdin
  set("chansend", function(id, data)
    local j = chans[id]
    if not j or not j.zid then return 0 end
    local s = type(data) == "table" and table.concat(data, "\n") or tostring(data)
    return z.job.send(j.zid, s) and #s or 0
  end)
  set("jobsend", function(id, data) return fn.chansend(id, data) end)
  -- chanclose(id, "stdin"): the end of its input; no stream: the job stops
  set("chanclose", function(id, stream)
    local j = chans[id]
    if not j or not j.zid then return 0 end
    if stream == "stdin" then z.job.close_stdin(j.zid) else z.job.kill(j.zid, 15) end
    return 1
  end)
  set("jobclose", function(id, stream) return fn.chanclose(id, stream) end)
  api.nvim_chan_send = function(id, data) fn.chansend(id, data) end
  api.nvim_list_chans = function()
    local r = { { id = 1, stream = "stdio", mode = "rpc" } }
    for id, j in pairs(jobs) do r[#r + 1] = { id = id, stream = "job", mode = "bytes", pty = "", argv = {} } end
    return r
  end
  api.nvim_open_term = function(_, o) local id = #api.nvim_list_chans() + 100 chans[id] = { input = {}, o = o or {} } return id end
  -- rpc, servers, sockets, stdio: zett has no msgpack-rpc peer
  set("rpcstart", function() return 0 end)
  set("rpcstop", function() return 0 end)
  set("rpcnotify", function() return 0 end)
  set("rpcrequest", function() error("zett: no rpc channels") end)
  set("sockconnect", function() return 0 end)
  set("serverstart", function(a) return a or (fn.tempname() .. ".sock") end)
  set("serverstop", function() return 0 end)
  set("stdioopen", function() return 0 end)
  set("api_info", function() local f = {} for k in pairs(api) do f[#f + 1] = { name = k, method = false, since = 0, parameters = {}, return_type = "Object" } end return { version = { major = 0, minor = 11, patch = 0, api_level = 13 }, functions = f, ui_events = {}, types = {}, error_types = {} } end)
  -- other languages: none
  for _, n in ipairs({ "perleval", "py3eval", "pyeval", "pyxeval", "rubyeval", "libcall", "libcallnr" }) do set(n, function() error("zett: " .. n .. " is not supported") end) end

  -- insert completion: complete() opens zett's own popup when it has one, else nothing
  local cinfo = { mode = "", pum_visible = 0, items = {}, selected = -1 }
  set("complete", function(col, items)
    cinfo.items, cinfo.selected, cinfo.mode = {}, -1, "eval"
    for i, it in ipairs(items or {}) do cinfo.items[i] = type(it) == "table" and it or { word = tostring(it) } end
    if z.complete then pcall(z.complete, col, cinfo.items) end
    return ""
  end)
  set("complete_add", function(it) cinfo.items[#cinfo.items + 1] = type(it) == "table" and it or { word = tostring(it) } return 1 end)
  set("complete_check", function() return 0 end)
  set("complete_info", function(what)
    if not what then return cinfo end
    local r = {} for _, k in ipairs(what) do r[k] = cinfo[k] end return r
  end)
  set("preinserted", function() return 0 end)
  set("cmdcomplete_info", function() return { cmdline_orig = "", pum_visible = 0, matches = {}, selected = -1 } end)
  set("wildtrigger", function() return 0 end)
  set("getcmdcomplpat", function() return "" end)
  set("getcmdcompltype", function() return "" end)
  set("getcompletiontype", function() return "" end)
  set("getcmdprompt", function() return "" end)
  set("getcmdscreenpos", function() local c = fn.getcmdpos and fn.getcmdpos() or 1 return c + 1 end)
  api.nvim_select_popupmenu_item = function() end
  api.nvim__complete_set = function() return {} end

  -- contexts: registers, jumps, buffers and globals, pushed and popped
  local ctx = {}
  local function context() return { regs = {}, jumps = {}, bufs = {}, gvars = vim.deepcopy(rawget(vim, "g") and vim.g or {}), funcs = {} } end
  set("ctxpush", function() table.insert(ctx, context()) return 0 end)
  set("ctxpop", function() table.remove(ctx) return 0 end)
  set("ctxget", function(i) return ctx[#ctx - (i or 0)] end)
  set("ctxset", function(c, i) ctx[#ctx - (i or 0)] = c return 0 end)
  set("ctxsize", function() return #ctx end)
  api.nvim_get_context = function() return context() end
  api.nvim_load_context = function(c) if c and c.gvars then for k, v in pairs(c.gvars) do vim.g[k] = v end end return 0 end

  -- prompt buffers: the prompt, a callback for Enter, the text after the prompt
  local prompts = {}
  local function pb(b) b = (b == 0 or b == nil) and api.nvim_get_current_buf() or b prompts[b] = prompts[b] or { prompt = "% " } return prompts[b] end
  set("prompt_setprompt", function(b, s) pb(b).prompt = s return 0 end)
  set("prompt_getprompt", function(b) return pb(b).prompt end)
  set("prompt_setcallback", function(b, f) pb(b).cb = f return 0 end)
  set("prompt_setinterrupt", function(b, f) pb(b).intr = f return 0 end)
  set("prompt_getinput", function(b) local p = pb(b).prompt local l = fn.getline("$") or "" return l:sub(1, #p) == p and l:sub(#p + 1) or l end)
  set("prompt_appendbuf", function(b, l) local n = api.nvim_buf_line_count(b) api.nvim_buf_set_lines(b, n - 1, n - 1, false, type(l) == "table" and l or { l }) return 0 end)

  -- the screen: what a cell shows comes from the buffer under it
  local function cell(r, c)
    local top = fn.line("w0") or 1
    local l = fn.getline(top + r - 1 - (vim.o.showtabline == 2 and 1 or 0)) or ""
    local off = c - 1 - (vim.wo.number and (vim.wo.numberwidth or 4) or 0)
    if off < 0 then return " " end
    local b = utf8.offset(l, off + 1)
    return b and l:sub(b, (utf8.offset(l, off + 2) or (#l + 1)) - 1) or " "
  end
  set("screenstring", function(r, c) local s = cell(r, c) return s == "" and " " or s end)
  set("screenchar", function(r, c) return utf8.codepoint(fn.screenstring(r, c)) end)
  set("screenchars", function(r, c) return { fn.screenchar(r, c) } end)
  set("screenattr", function() return 0 end)
  api.nvim__inspect_cell = function(_, r, c) return { fn.screenstring(r + 1, c + 1), {}, 0 } end
  api.nvim__screenshot = function() end
  api.nvim__redraw = function() vim.cmd("redraw") end
  api.nvim__invalidate_glyph_cache = function() end
  api.nvim_input_mouse = function() end
  api.nvim_ui_send = function() end
  set("getwinpos", function() return { 0, 0 } end)
  set("getwinposx", function() return 0 end)
  set("getwinposy", function() return 0 end)
  set("getfontname", function() return "" end)
  set("windowsversion", function() return "" end)
  set("getcharmod", function() return 0 end)
  set("getcharstr", function(...) local c = fn.getchar(...) return type(c) == "number" and (c == 0 and "" or utf8.char(c)) or c end)

  -- statusline: %f %m %l %c %p %= and literal text; width from the window
  api.nvim_eval_statusline = function(s, o)
    o = o or {}
    local ok, r, c = pcall(z.buf.cursor)
    if not ok then r, c = 1, 0 end
    local okn, n = pcall(api.nvim_buf_line_count, 0)
    n = okn and n or 1
    local items = {
      f = function() return fn.expand("%:~:.") end, F = function() return fn.expand("%:p") end, t = function() return fn.expand("%:t") end,
      m = function() return vim.bo.modified and "[+]" or "" end, M = function() return vim.bo.modified and ",+" or "" end,
      r = function() return vim.bo.readonly and "[RO]" or "" end, y = function() return vim.bo.filetype ~= "" and "[" .. vim.bo.filetype .. "]" or "" end,
      Y = function() return vim.bo.filetype end, l = function() return tostring(r) end, L = function() return tostring(n) end,
      c = function() return tostring(c + 1) end, v = function() return tostring(c + 1) end, V = function() return tostring(c + 1) end,
      p = function() return tostring(math.floor(r * 100 / math.max(1, n))) end, P = function() return r == 1 and "Top" or r == n and "Bot" or math.floor(r * 100 / n) .. "%" end,
      n = function() return tostring(api.nvim_get_current_buf()) end, h = function() return vim.bo.buftype == "help" and "[Help]" or "" end,
      w = function() return "" end, q = function() return "" end, k = function() return "" end, ["%"] = function() return "%" end,
    }
    local left, right, cur = {}, {}, nil
    cur = left
    local i = 1
    while i <= #s do
      local ch = s:sub(i, i)
      if ch == "%" then
        local spec, item = s:match("^%%([-0-9.]*)(.)", i)
        i = i + 1 + #spec + 1
        if item == "=" then cur = right
        elseif item == "#" then local e = s:find("#", i, true) i = (e or i) + 1
        elseif item == "{" then local e = s:find("}", i, true) local ok, v = pcall(vim.api.nvim_eval, s:sub(i, (e or #s) - 1)) cur[#cur + 1] = ok and tostring(v) or "" i = (e or #s) + 1
        elseif items[item] then cur[#cur + 1] = items[item]()
        end
      else cur[#cur + 1] = ch i = i + 1 end
    end
    local l, rr = table.concat(left), table.concat(right)
    local width = o.maxwidth or (vim.o.columns or 80)
    local str = rr ~= "" and (l .. string.rep(" ", math.max(1, width - fn.strdisplaywidth(l) - fn.strdisplaywidth(rr))) .. rr) or l
    local out = { str = str, width = fn.strdisplaywidth(str) }
    if o.highlights then out.highlights = { { start = 0, group = o.use_winbar and "WinBar" or "StatusLine" } } end
    return out
  end

  -- an expression's parse tree (Neovim's nvim_parse_expression), for the shapes plugins ask about
  api.nvim_parse_expression = function(e, _, hl)
    local t = { ast = { type = "Missing", start = { 0, 0 }, len = #e } }
    if e:match("^%s*%-?%d+%s*$") then t.ast = { type = "Integer", start = { 0, 0 }, len = #e, ivalue = tonumber(e) }
    elseif e:match("^%s*'.*'%s*$") then t.ast = { type = "SingleQuotedString", start = { 0, 0 }, len = #e, svalue = e:match("'(.*)'") }
    elseif e:match('^%s*".*"%s*$') then t.ast = { type = "DoubleQuotedString", start = { 0, 0 }, len = #e, svalue = e:match('"(.*)"') }
    elseif e:match("^%s*[%a_][%w_:#]*%s*$") then t.ast = { type = "PlainIdentifier", start = { 0, 0 }, len = #e, ident = vim.trim(e), scope = 0 } end
    if hl then t.highlight = {} end
    return t
  end

  -- folds, diff, syntax
  set("foldclosedend", function(l) local s = fn.foldclosed(l) if s < 0 then return -1 end local e = l while fn.foldclosed(e + 1) == s do e = e + 1 end return e end)
  set("foldtextresult", function(l) return fn.foldclosed(l) < 0 and "" or fn.foldtext() end)
  set("diff_filler", function() return 0 end)
  set("diff_hlID", function() return 0 end)
  set("synIDattr", function(id, what)
    local name = (fn.synIDtrans and rawget(vim.fn, "_hlname") or nil)
    local ok, h = pcall(api.nvim_get_hl, 0, { id = id, link = false })
    if not ok or not h then return "" end
    if what == "name" then return h.name or "" end
    local m = { fg = "fg", bg = "bg", sp = "sp", ["fg#"] = "fg", ["bg#"] = "bg", ["sp#"] = "sp" }
    if m[what] then local v = h[m[what]] return v and string.format("#%06x", v) or "" end
    return h[what] and "1" or ""
  end)
  set("synIDtrans", function(id) return id end)
  set("synstack", function(l, c) local id = fn.synID(l, c, 1) return id > 0 and { id } or {} end)
  set("synconcealed", function() return { 0, "", 0 } end)
  set("searchdecl", function(name, global)
    local pat = "\\<" .. name .. "\\>"
    local p = fn.searchpos(pat, global and "w" or "bW")
    if p[1] == 0 then return 1 end
    if global then local save = fn.getpos(".") vim.cmd("normal! gg") p = fn.searchpos(pat, "W") if p[1] == 0 then fn.setpos(".", save) return 1 end end
    return 0
  end)
  set("cindent", function(l)
    local prev = fn.prevnonblank(l - 1)
    if prev == 0 then return 0 end
    local s = fn.getline(prev) or ""
    local ind = fn.indent(prev) + ((s:match("[{(%[]%s*$") and vim.fn.shiftwidth()) or 0)
    if (fn.getline(l) or ""):match("^%s*[})%]]") then ind = ind - vim.fn.shiftwidth() end
    return math.max(0, ind)
  end)
  set("lispindent", function(l) local p = fn.prevnonblank(l - 1) if p == 0 then return 0 end local s = fn.getline(p) or "" local o = s:find("%([^()]*$") return o and o + 1 or fn.indent(p) end)

  -- spelling through aspell or hunspell when installed
  local speller = (fn.executable("aspell") == 1 and "aspell") or (fn.executable("hunspell") == 1 and "hunspell") or nil
  local function badwords(s)
    if not speller or s == "" then return {} end
    local out = fn.system({ speller, "list" }, s)
    local r = {} for w in out:gmatch("%S+") do r[#r + 1] = w end return r
  end
  set("spellbadword", function(s)
    s = s or fn.expand("<cword>")
    local w = badwords(s)[1]
    return w and { w, "bad" } or { "", "" }
  end)
  set("spellsuggest", function(word, max)
    if not speller then return {} end
    local out = fn.system({ speller, "-a" }, word .. "\n")
    local list = out:match("& [^:]*: ([^\n]*)")
    local r = {}
    if list then for w in list:gmatch("[^, ]+") do if max and #r >= max then break end r[#r + 1] = w end end
    return r
  end)

  -- tags from the tags file(s)
  set("tagfiles", function()
    local r = {}
    for _, t in ipairs(vim.split(vim.o.tags or "./tags,tags", ",", { plain = true })) do
      local p = fn.findfile(t:gsub(";.*", ""), ".;") if p ~= "" and not vim.tbl_contains(r, p) then r[#r + 1] = p end
    end
    return r
  end)
  set("taglist", function(pat)
    local r, re = {}, vim.regex(pat)
    for _, f in ipairs(fn.tagfiles()) do
      for _, l in ipairs(fn.readfile(f)) do
        local name, file, cmd = l:match("^([^\t]+)\t([^\t]+)\t(.-);?\"?\t?")
        if name and not name:match("^!_TAG") and re:match_str(name) then
          r[#r + 1] = { name = name, filename = file, cmd = cmd or "", kind = l:match('\t(%a)\t') or l:match('\t(%a)$') or "", static = 0 }
        end
      end
    end
    return r
  end)

  -- swap files: zett keeps none
  set("swapname", function() return "" end)
  set("swapfilelist", function() return {} end)
  set("swapinfo", function(f) return { error = "Cannot open file" .. (f and (": " .. f) or "") } end)

  -- arguments, menus, maps, the rest
  set("argidx", function() return vim.g._zett_argidx or 0 end)
  set("arglistid", function() return 0 end)
  set("browse", function(_, _, dir, def) return fn.input("File: ", (dir ~= "" and (dir .. "/") or "") .. (def or ""), "file") end)
  set("browsedir", function(_, dir) return fn.input("Directory: ", dir or "", "dir") end)
  set("menu_get", function() return {} end)
  set("menu_info", function() return {} end)
  set("mapset", function(mode, abbr, d)
    if type(mode) == "table" then d, mode = mode, mode.mode end
    if not d or not d.lhs then return 0 end
    local o = { noremap = d.noremap == 1, silent = d.silent == 1, expr = d.expr == 1, nowait = d.nowait == 1, desc = d.desc, callback = d.callback, buffer = d.buffer ~= 0 and d.buffer or nil }
    for m in (mode ~= "" and mode or "nvo"):gmatch(".") do pcall(vim.keymap.set, m == " " and "" or m, d.lhs, d.callback or d.rhs, o) end
    return 0
  end)
  set("submatch", function(n, list) local m = vim.g._zett_submatch or {} local s = m[n + 1] or "" return list and vim.split(s, "\n") or s end)
  set("gettext", function(s) return s end)
  set("debugbreak", function() return 0 end)
  set("interrupt", function() return 0 end)
  set("test_garbagecollect_now", function() collectgarbage() return 0 end)
  set("test_write_list_log", function() return 0 end)
  set("dictwatcheradd", function() return 0 end)
  set("dictwatcherdel", function() return 0 end)
  set("NetUserPass", function() return 0 end)
  set("MatchEnable", function() pcall(vim.cmd, "DoMatchParen") return 0 end)
  set("MatchDisable", function() pcall(vim.cmd, "NoMatchParen") return 0 end)

  -- internal nvim__ calls plugins and tests reach for
  api.nvim__buf_stats = function(b) return { flush_count = 0, true_line_count = api.nvim_buf_line_count(b or 0), current_lnum = (z.buf.cursor()) } end
  api.nvim__buf_debug_extmarks = function() return "" end
  api.nvim__get_lib_dir = function() return fn.stdpath("data") end
  api.nvim__get_runtime = function(pats, all) local r = {} for _, p in ipairs(pats) do for _, f in ipairs(api.nvim_get_runtime_file(p, all)) do r[#r + 1] = f end end return r end
  api.nvim__runtime_inspect = function() return {} end
  api.nvim__stats = function() return { fsync = 0, log_skip = 0, lua_refcount = collectgarbage("count"), redraw = 0, arena_alloc_count = 0 } end
  api.nvim__unpack = function(s) return vim.fn.msgpackparse(s)[1] end

  -- vim.fn.nvim_*: every api function is also a Vim function
  for k, v in pairs(api) do if rawget(vim.fn, k) == nil then fn[k] = v rawset(vim.fn, k, v) end end
end

-- 'tagfunc' for :tag and CTRL-]: the function it names ("v:lua.x.y", a Vim or Lua function) called as Neovim
-- does (pattern, flags, info); its matches as "name\tfile\tcmd\tkind" lines, "NONE" when it gives v:null
function vim._zett_tagfunc(spec, pattern, flags)
  local f
  local lua_path = spec:match("^v:lua%.(.+)$")
  if lua_path then
    f = _G
    for part in lua_path:gmatch("[^%.]+") do f = type(f) == "table" and f[part] or nil end
  elseif type(_G[spec]) == "function" then f = _G[spec]
  else f = function(...) return vim.fn[spec](...) end end
  if type(f) ~= "function" then return "NONE" end
  local ok, r = pcall(f, pattern, flags or "c", { buf_ffname = vim.api.nvim_buf_get_name(0) })
  if not ok or r == nil or r == vim.NIL then return "NONE" end
  local out = {}
  for _, t in ipairs(r) do
    out[#out + 1] = table.concat({ t.name or pattern, t.filename or "", t.cmd or "1", t.kind or "" }, "\t")
  end
  return table.concat(out, "\n")
end

-- 'findfunc' for :find: the files it gives for the name (the first is opened)
function vim._zett_findfunc(spec, name)
  local f = spec:match("^v:lua%.(.+)$")
  local fn_
  if f then
    fn_ = _G
    for part in f:gmatch("[^%.]+") do fn_ = type(fn_) == "table" and fn_[part] or nil end
  elseif type(_G[spec]) == "function" then fn_ = _G[spec]
  else fn_ = function(...) return vim.fn[spec](...) end end
  if type(fn_) ~= "function" then return "" end
  local ok, r = pcall(fn_, name, false)
  if not ok or type(r) ~= "table" then return "" end
  return table.concat(r, "\n")
end

-- 'quickfixtextfunc': called as Neovim does ({quickfix, winid, id, start_idx, end_idx}); its lines, one per item
function vim._zett_qftf(spec, quickfix, n)
  local f = spec:match("^v:lua%.(.+)$")
  local fn_
  if f then
    fn_ = _G
    for part in f:gmatch("[^%.]+") do fn_ = type(fn_) == "table" and fn_[part] or nil end
  elseif type(_G[spec]) == "function" then fn_ = _G[spec]
  else fn_ = function(...) return vim.fn[spec](...) end end
  if type(fn_) ~= "function" then return "" end
  local ok, r = pcall(fn_, { quickfix = quickfix, winid = 0, id = 0, start_idx = 1, end_idx = n })
  if not ok or type(r) ~= "table" then return "" end
  local out = {}
  for i = 1, n do out[i] = tostring(r[i] or ""):gsub("\n", " ") end
  return table.concat(out, "\n")
end

-- msgpack (msgpackdump, msgpackparse): nil, booleans, integers, floats, strings, lists and dicts; the bytes as a
-- readfile()-style list of lines (a NL in the data is a "\0" in the line), as Neovim gives and takes them
do
  local pack, unpack_one
  function pack(v, out)
    local t = type(v)
    if v == nil or v == vim.NIL then out[#out + 1] = "\xc0"
    elseif t == "boolean" then out[#out + 1] = v and "\xc3" or "\xc2"
    elseif t == "number" and math.type(v) == "integer" then
      if v >= 0 and v < 128 then out[#out + 1] = string.char(v)
      elseif v < 0 and v >= -32 then out[#out + 1] = string.char(0x100 + v)
      else out[#out + 1] = "\xd3" .. string.pack(">i8", v) end
    elseif t == "number" then out[#out + 1] = "\xcb" .. string.pack(">d", v)
    elseif t == "string" then
      local n = #v
      out[#out + 1] = (n < 32 and string.char(0xa0 + n) or n < 0x10000 and ("\xda" .. string.pack(">I2", n)) or ("\xdb" .. string.pack(">I4", n))) .. v
    elseif t == "table" then
      if vim.islist(v) then
        out[#out + 1] = #v < 16 and string.char(0x90 + #v) or ("\xdd" .. string.pack(">I4", #v))
        for _, x in ipairs(v) do pack(x, out) end
      else
        local keys = {} for k in pairs(v) do keys[#keys + 1] = k end table.sort(keys, function(a, b) return tostring(a) < tostring(b) end)
        out[#out + 1] = #keys < 16 and string.char(0x80 + #keys) or ("\xdf" .. string.pack(">I4", #keys))
        for _, k in ipairs(keys) do pack(k, out) pack(v[k], out) end
      end
    else error("msgpackdump: cannot pack a " .. t) end
  end
  function unpack_one(s, i)
    local b = s:byte(i)
    if not b then error("msgpackparse: incomplete data") end
    if b < 0x80 then return b, i + 1 end
    if b >= 0xe0 then return b - 0x100, i + 1 end
    if b >= 0xa0 and b < 0xc0 then local n = b - 0xa0 return s:sub(i + 1, i + n), i + 1 + n end
    if b >= 0x90 and b < 0xa0 then local r, j = {}, i + 1 for k = 1, b - 0x90 do r[k], j = unpack_one(s, j) end return r, j end
    if b >= 0x80 and b < 0x90 then local r, j = {}, i + 1 for _ = 1, b - 0x80 do local k k, j = unpack_one(s, j) r[k], j = unpack_one(s, j) end return r, j end
    local fixed = { [0xc0] = function() return vim.NIL, i + 1 end, [0xc2] = function() return false, i + 1 end, [0xc3] = function() return true, i + 1 end }
    if fixed[b] then return fixed[b]() end
    local nums = { [0xcc] = ">I1", [0xcd] = ">I2", [0xce] = ">I4", [0xcf] = ">I8", [0xd0] = ">i1", [0xd1] = ">i2", [0xd2] = ">i4", [0xd3] = ">i8", [0xca] = ">f", [0xcb] = ">d" }
    if nums[b] then local v, j = string.unpack(nums[b], s, i + 1) return v, j end
    local strs = { [0xd9] = ">I1", [0xda] = ">I2", [0xdb] = ">I4", [0xc4] = ">I1", [0xc5] = ">I2", [0xc6] = ">I4" }
    if strs[b] then local n, j = string.unpack(strs[b], s, i + 1) return s:sub(j, j + n - 1), j + n end
    local arrs = { [0xdc] = ">I2", [0xdd] = ">I4" }
    if arrs[b] then local n, j = string.unpack(arrs[b], s, i + 1) local r = {} for k = 1, n do r[k], j = unpack_one(s, j) end return r, j end
    local maps = { [0xde] = ">I2", [0xdf] = ">I4" }
    if maps[b] then local n, j = string.unpack(maps[b], s, i + 1) local r = {} for _ = 1, n do local k k, j = unpack_one(s, j) r[k], j = unpack_one(s, j) end return r, j end
    error(string.format("msgpackparse: type 0x%02x is not read", b))
  end
  local function set(name, f) rawset(vim.fn, name, f) end
  set("msgpackdump", function(list, flags)
    local out = {}
    for _, v in ipairs(list) do pack(v, out) end
    local s = table.concat(out)
    if flags and flags:find("B") then return s end
    return vim.split((s:gsub("\n", "\0")), "\0\n", { plain = true }) -- lines as readfile gives them
  end)
  set("msgpackparse", function(data)
    local s = type(data) == "table" and table.concat(data, "\n"):gsub("%z", "\n") or data
    local r, i = {}, 1
    while i <= #s do r[#r + 1], i = unpack_one(s, i) end
    return r
  end)
end

-- ── WASM plugins: a folder with zett-plugin.json and its .wasm ──
-- { "name": "text-tools", "wasm": "text_tools.wasm", "description": "…",
--   "commands": [ { "name": "SortLines", "call": "sort_lines", "desc": "…" } ] }
-- each command runs on the lines of its range (:'<,'>SortLines), else the whole buffer: they go in as text,
-- what comes back replaces them. zett.wasm_plugins lists what is loaded (the store and :checkhealth show it).
zett.wasm_plugins = zett.wasm_plugins or {}
function zett.wasm_plugin(dir)
  local ok, manifest = pcall(function() return vim.json.decode(table.concat(vim.fn.readfile(dir .. "/zett-plugin.json"), "\n")) end)
  if not ok or type(manifest) ~= "table" then return nil, "no zett-plugin.json in " .. dir end
  local okp, p = pcall(zett.wasm.load, dir .. "/" .. (manifest.wasm or (manifest.name .. ".wasm")))
  if not okp then return nil, p end
  for _, c in ipairs(manifest.commands or {}) do
    vim.api.nvim_create_user_command(c.name, function(a)
      local whole = (a.range or 0) == 0
      local l1, l2 = whole and 1 or a.line1, whole and vim.api.nvim_buf_line_count(0) or a.line2
      local text = table.concat(vim.api.nvim_buf_get_lines(0, l1 - 1, l2, false), "\n")
      local done, out = pcall(p.call, p, c.call, text)
      for _, l in ipairs(p:logs()) do vim.notify(manifest.name .. ": " .. l) end
      if not done then vim.notify(c.name .. ": " .. tostring(out), vim.log.levels.ERROR) return end
      vim.api.nvim_buf_set_lines(0, l1 - 1, l2, false, vim.split(out, "\n", { plain = true }))
    end, { range = true, desc = c.desc or (manifest.name .. ": " .. c.name) })
  end
  zett.wasm_plugins[manifest.name] = { dir = dir, manifest = manifest, plugin = p }
  return p
end
-- at start: every plugin folder under the config's wasm/ and zett's own share/zett/wasm/
for _, root in ipairs({ vim.fn.stdpath("config") .. "/wasm", vim.fn.stdpath("data") .. "/wasm" }) do
  for _, d in ipairs(vim.fn.glob(root .. "/*", false, true)) do
    if vim.fn.filereadable(d .. "/zett-plugin.json") == 1 then
      local _, err = zett.wasm_plugin(d)
      if err then vim.schedule(function() vim.notify("wasm plugin " .. d .. ": " .. err, vim.log.levels.WARN) end) end
    end
  end
end

-- ── the store: extensions known to work in zett (each one tried), installed and removed from :ZettStore ──
-- Neovim plugins go in as zett.plug lines in <config>/plugins/store/init.lua (a file to read and edit by hand);
-- WASM plugins come with zett (<data>/store/<name>) and are copied to <config>/wasm/<name>
zett.store = zett.store or {}
zett.store.catalog = {
  { name = "gitsigns.nvim", kind = "lua", repo = "lewis6991/gitsigns.nvim", desc = "Git changes by the line numbers, hunks to stage, preview and reset, blame" },
  { name = "fidget.nvim", kind = "lua", repo = "j-hui/fidget.nvim", desc = "Language server progress and notifications in a corner" },
  { name = "tinct.nvim", kind = "lua", repo = "SettDEF/tinct.nvim", desc = "Colours from the wallpaper (tinct), as the desktop has them" },
  { name = "text-tools", kind = "wasm", desc = "Sort, dedupe and reverse lines, title case, pretty-print JSON (WASM)" },
}
local function store_file() return vim.fn.stdpath("config") .. "/plugins/store/init.lua" end
local function store_lines() return vim.fn.filereadable(store_file()) == 1 and vim.fn.readfile(store_file()) or {} end
function zett.store.installed(e)
  if e.kind == "wasm" then return zett.wasm_plugins[e.name] ~= nil or vim.fn.isdirectory(vim.fn.stdpath("config") .. "/wasm/" .. e.name) == 1 end
  for _, l in ipairs(store_lines()) do if l:find(e.repo, 1, true) then return true end end
  -- set up by hand (the user's own config) counts too
  return vim.fn.isdirectory(vim.fn.stdpath("data") .. "/site/pack/plug/start/" .. e.name) == 1
end
function zett.store.list()
  local out = {}
  for _, e in ipairs(zett.store.catalog) do
    -- managed: installed by the store (its file or its wasm folder), so the store can take it out again
    local managed = e.kind == "wasm" and vim.fn.isdirectory(vim.fn.stdpath("config") .. "/wasm/" .. e.name) == 1
    if e.kind == "lua" then for _, l in ipairs(store_lines()) do if l:find(e.repo, 1, true) then managed = true end end end
    out[#out + 1] = vim.tbl_extend("force", e, { installed = zett.store.installed(e), managed = managed })
  end
  return out
end

-- ── the marketplace: store.nvim's database of Neovim plugins (7000+, from GitHub), searched here ──
-- downloaded once into the cache folder (zett.store.update() fetches it again), slimmed to what zett shows
zett.store.market_url = "https://github.com/alex-popov-tech/store.nvim.crawler/releases/latest/download/db_minified.json"
local function market_file() return vim.fn.stdpath("cache") .. "/zett/market.json" end
local market
local function load_market()
  if market then return market end
  local f = io.open(market_file(), "rb")
  if not f then return nil end
  local ok, d = pcall(vim.json.decode, f:read("*a"))
  f:close()
  market = ok and type(d) == "table" and d or nil
  return market
end
-- fetch the list (a few MB): true, or false and why
function zett.store.update()
  local raw = market_file() .. ".download"
  vim.fn.mkdir(vim.fn.fnamemodify(raw, ":h"), "p")
  local r = vim.system({ "curl", "-sfL", "--max-time", "60", "-o", raw, zett.store.market_url }):wait()
  if r.code ~= 0 then os.remove(raw) return false, "download failed (curl " .. r.code .. ")" end
  local f = io.open(raw, "rb")
  local ok, d = pcall(vim.json.decode, f and f:read("*a") or "")
  if f then f:close() end
  os.remove(raw)
  if not ok or type(d) ~= "table" or type(d.items) ~= "table" then return false, "the list could not be read" end
  local slim = {}
  for _, e in ipairs(d.items) do
    if e.full_name then
      slim[#slim + 1] = { repo = e.full_name, name = e.name, desc = e.description or "", tags = e.tags or {}, stars = e.stars and e.stars.curr or 0, updated = (e.updated_at or ""):sub(1, 10) }
    end
  end
  table.sort(slim, function(a, b) return a.stars > b.stars end)
  local out = io.open(market_file(), "wb")
  if not out then return false, "cannot write " .. market_file() end
  out:write(vim.json.encode({ fetched = os.time(), items = slim }))
  out:close()
  market = nil
  return true, #slim .. " plugins"
end
-- how many plugins the list has, and when it was fetched (0, nil: not yet)
function zett.store.market_info()
  local m = load_market()
  if not m then return 0, nil end
  return #m.items, m.fetched
end
-- the plugins matching every word of `query` (name, repo, description, tags), most stars first
function zett.store.search(query, limit, installed_only)
  local m = load_market()
  if not m then return {}, "no plugin list yet: zett.store.update() (Settings > Extensions: Update list)" end
  local words = {}
  for w in (query or ""):lower():gmatch("%S+") do words[#words + 1] = w end
  local lines = store_lines()
  local out = {}
  for _, e in ipairs(m.items) do
    local hay = (e.repo .. " " .. e.desc .. " " .. table.concat(e.tags, " ")):lower()
    local fits = true
    for _, w in ipairs(words) do if not hay:find(w, 1, true) then fits = false break end end
    if fits then
      local managed = false
      for _, l in ipairs(lines) do if l:find(e.repo, 1, true) then managed = true break end end
      local installed = managed or vim.fn.isdirectory(vim.fn.stdpath("data") .. "/site/pack/plug/start/" .. e.name) == 1
      if installed or not installed_only then
        out[#out + 1] = { name = e.repo, repo = e.repo, kind = "lua", desc = e.desc, stars = e.stars, tags = e.tags, updated = e.updated, managed = managed, installed = installed }
        if #out >= (limit or 50) then break end
      end
    end
  end
  return out
end
-- by name (the curated list), else a marketplace plugin by its repo ("author/name")
local function find(name)
  for _, e in ipairs(zett.store.catalog) do if e.name == name then return e end end
  local m = load_market()
  for _, e in ipairs(m and m.items or {}) do if e.repo == name then return { name = e.repo, repo = e.repo, kind = "lua" } end end
end
function zett.store.install(name)
  local e = find(name)
  if not e then return false, "no " .. name .. " in the store" end
  if e.kind == "wasm" then
    local from, to = vim.fn.stdpath("data") .. "/store/" .. name, vim.fn.stdpath("config") .. "/wasm/" .. name
    if vim.fn.isdirectory(from) == 0 then return false, "zett came without " .. name .. " (" .. from .. ")" end
    vim.fn.mkdir(to, "p")
    for _, f in ipairs(vim.fn.readdir(from)) do vim.fn.writefile(vim.fn.readblob(from .. "/" .. f), to .. "/" .. f, "b") end
    local _, err = zett.wasm_plugin(to)
    if err then return false, err end
    return true
  end
  local lines = store_lines()
  if #lines == 0 then lines = { "-- Installed from :ZettStore (one zett.plug per plugin; delete a line to remove it)" } end
  local line = ("zett.plug(%q)"):format(e.repo)
  if not vim.tbl_contains(lines, line) then lines[#lines + 1] = line end
  vim.fn.mkdir(vim.fn.fnamemodify(store_file(), ":h"), "p")
  vim.fn.writefile(lines, store_file())
  local ok, err = pcall(zett.plug, e.repo)
  return ok, not ok and tostring(err) or nil
end
function zett.store.remove(name)
  local e = find(name)
  if not e then return false end
  local mine = false
  for _, l in ipairs(store_lines()) do if l:find(e.repo, 1, true) then mine = true end end
  if e.kind == "lua" and not mine then
    return false, "set up in your own config, not by the store: take its zett.plug line out there"
  end
  if e.kind == "wasm" then
    vim.fn.delete(vim.fn.stdpath("config") .. "/wasm/" .. name, "rf")
    zett.wasm_plugins[name] = nil
    return true, "gone after a restart (its commands stay until then)"
  end
  vim.fn.writefile(vim.tbl_filter(function(l) return not l:find(e.repo, 1, true) end, store_lines()), store_file())
  return true, "gone after a restart"
end
vim.api.nvim_create_user_command("ZettStore", function(a)
  -- :ZettStore QUERY: the marketplace (fetched the first time); bare: the curated list
  local items = zett.store.list()
  if a.args ~= "" then
    if zett.store.market_info() == 0 then local ok, err = zett.store.update() if not ok then return vim.notify("ZettStore: " .. err, vim.log.levels.ERROR) end end
    items = vim.tbl_map(function(e) e.desc = ("★%d %s"):format(e.stars, e.desc) return e end, (zett.store.search(a.args, 60)))
  end
  vim.ui.select(items, {
    prompt = "Extensions (✓ installed: choosing one removes it)",
    format_item = function(e) return ("%s %-14s %-4s %s%s"):format(e.installed and "✓" or "·", e.name, e.kind, e.desc, e.installed and not e.managed and "  (in your config)" or "") end,
  }, function(e)
    if not e then return end
    local ok, msg
    if e.installed then ok, msg = zett.store.remove(e.name) else ok, msg = zett.store.install(e.name) end
    vim.notify(("%s %s%s"):format(e.name, ok and (e.installed and "removed" or "installed") or "failed", msg and (": " .. msg) or ""), ok and vim.log.levels.INFO or vim.log.levels.ERROR)
  end)
end, { desc = "Extensions: install and remove (Lua and WASM plugins)", nargs = "?" })

-- vim.base64 (C in Neovim): RFC 4648 with padding
if not rawget(vim, "base64") then
  local abc = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"
  local idx = {} for i = 1, 64 do idx[abc:byte(i)] = i - 1 end
  vim.base64 = {
    encode = function(s)
      vim.validate("str", s, "string")
      return (s:gsub("..?.?", function(c)
        local a, b, cc = c:byte(1, 3)
        local n = a * 65536 + (b or 0) * 256 + (cc or 0)
        local q = { (n >> 18) & 63, (n >> 12) & 63, (n >> 6) & 63, n & 63 }
        local out = abc:sub(q[1] + 1, q[1] + 1) .. abc:sub(q[2] + 1, q[2] + 1)
        return out .. (b and abc:sub(q[3] + 1, q[3] + 1) or "=") .. (cc and abc:sub(q[4] + 1, q[4] + 1) or "=")
      end))
    end,
    decode = function(s)
      vim.validate("str", s, "string")
      if #s % 4 ~= 0 or s:find("[^%w%+/=]") then error("Invalid input") end
      return (s:gsub("....", function(q)
        local v = {} for i = 1, 4 do v[i] = idx[q:byte(i)] end
        local n = (v[1] << 18) | (v[2] << 12) | ((v[3] or 0) << 6) | (v[4] or 0)
        local o = string.char((n >> 16) & 255)
        if v[3] then o = o .. string.char((n >> 8) & 255) end
        if v[4] then o = o .. string.char(n & 255) end
        return o
      end))
    end,
  }
end

-- vim.glob: LSP's file-watch patterns (**/*.rs, src/{a,b}/*.[ch]). Neovim builds an LPeg pattern; zett a
-- regex with the same :match(path) (the end position + 1, or nil)
if not rawget(vim, "glob") then
  local function to_re(g)
    local out, i, depth = { "\\v^" }, 1, 0
    while i <= #g do
      local c, two = g:sub(i, i), g:sub(i, i + 2)
      if two == "**/" then out[#out + 1] = "(.*/)?" i = i + 3
      elseif g:sub(i, i + 1) == "**" then out[#out + 1] = ".*" i = i + 2
      elseif c == "*" then out[#out + 1] = "[^/]*" i = i + 1
      elseif c == "?" then out[#out + 1] = "[^/]" i = i + 1
      elseif c == "{" then depth = depth + 1 out[#out + 1] = "(" i = i + 1
      elseif c == "}" and depth > 0 then depth = depth - 1 out[#out + 1] = ")" i = i + 1
      elseif c == "," and depth > 0 then out[#out + 1] = "|" i = i + 1
      elseif c == "[" then
        local e = g:find("]", i + 1, true)
        if not e then out[#out + 1] = "\\[" i = i + 1
        else
          local inner = g:sub(i + 1, e - 1):gsub("^!", "^")
          out[#out + 1] = "[" .. inner .. "]" i = e + 1
        end
      else
        out[#out + 1] = c:match("[%w/_ ]") and c or ("\\" .. c)
        i = i + 1
      end
    end
    out[#out + 1] = "$"
    return table.concat(out)
  end
  vim.glob = {
    to_lpeg = function(pattern)
      local re = vim.regex(to_re(pattern))
      return setmetatable({ pattern = pattern }, { __index = {
        match = function(_, s) local a, b = re:match_str(s) if a then return b + 1 end return nil end,
      } })
    end,
    _to_regex = to_re,
  }
end

-- vim.health: what a plugin's health.lua reports (:checkhealth gathers them)
if not rawget(vim, "health") then
  local report = {}
  local function add(kind, msg, advice)
    report[#report + 1] = { kind = kind, msg = msg }
    for _, a in ipairs(type(advice) == "table" and advice or advice and { advice } or {}) do report[#report + 1] = { kind = "advice", msg = a } end
  end
  vim.health = {
    start = function(name) report[#report + 1] = { kind = "start", msg = name } end,
    ok = function(m) add("ok", m) end,
    info = function(m) add("info", m) end,
    warn = function(m, a) add("warn", m, a) end,
    error = function(m, a) add("error", m, a) end,
  }
  -- every lua/<plugin>/health.lua (or health/init.lua) on the runtime path: its check(), as rows for :checkhealth
  function vim._zett_health()
    local rows, seen = {}, {}
    local files = vim.list_extend(vim.api.nvim_get_runtime_file("lua/*/health.lua", true), vim.api.nvim_get_runtime_file("lua/*/health/init.lua", true))
    for _, f in ipairs(files) do
      local mod = f:match("lua/([^/]+)/health") .. ".health"
      if not seen[mod] and mod ~= "vim.health" then
        seen[mod] = true
        report = {}
        -- the file found, loaded as it is (require may not reach a folder only on 'runtimepath')
        local ok, m = pcall(require, mod)
        if not ok then ok, m = pcall(dofile, f) end
        if ok and type(m) == "table" and type(m.check) == "function" then
          local done, err = pcall(m.check)
          if not done then add("error", "check() failed: " .. tostring(err)) end
        elseif not ok then add("error", tostring(m)) end
        rows[#rows + 1] = { "── " .. mod:gsub("%.health$", ""), "" }
        for _, r in ipairs(report) do
          rows[#rows + 1] = { ({ start = "  ##", ok = "  ✔ ok", info = "  · info", warn = "  ⚠ warn", error = "  ✖ error", advice = "      →" })[r.kind], r.msg }
        end
      end
    end
    return rows
  end
end

-- vim.regex(pattern): a Vim pattern (magic, \c \v \zs …); match_str(s) → start, end (0-based bytes) or nil;
-- match_line(buf, line, start, end) on a buffer's line (0-based, as Neovim)
function vim.regex(pattern)
  local re = z.vimregex(pattern, vim.o.ignorecase and not (vim.o.smartcase and pattern:find("%u")) or false)
  local o = {}
  function o:match_str(s)
    local a, b = re:find(s, 0)
    if a then return a, b end
    return nil
  end
  function o:match_line(buf, line, s, e)
    local l = vim.api.nvim_buf_get_lines(buf, line, line + 1, false)[1] or ""
    local part = l:sub((s or 0) + 1, e)
    local a, b = re:find(part, 0)
    if a then return a, b end
    return nil
  end
  return o
end

-- nvim_get_runtime_file(name, all): `name` (with * and **) under every 'runtimepath' folder, the config folder
-- and each start package (config and data pack/*/start/*), as Neovim searches; the first only unless `all`
api.nvim_get_runtime_file = function(name, all)
  local dirs, seen, out = {}, {}, {}
  local function add(d) if d and d ~= "" and not seen[d] then seen[d] = true dirs[#dirs + 1] = d end end
  local cfg = type(z.config_dir) == "function" and z.config_dir() or tostring(z.config_dir or vim.fn.stdpath("config"))
  for d in tostring(vim.o.runtimepath or ""):gmatch("[^,]+") do add(vim.fn.expand(d)) end
  add(cfg)
  for _, base in ipairs({ cfg, vim.fn.stdpath("data") .. "/site" }) do
    for _, p in ipairs(vim.fn.glob(base .. "/pack/*/start/*", false, true)) do add(p) end
  end
  for _, d in ipairs(dirs) do
    for _, f in ipairs(vim.fn.glob(d .. "/" .. name, false, true)) do
      out[#out + 1] = f
      if not all then return out end
    end
  end
  return out
end
rawset(vim.fn, "nvim_get_runtime_file", api.nvim_get_runtime_file)

-- libuv's function forms (uv.timer_stop(t) as t:stop()): lualine and others call them so
do
  local uv = vim.uv or vim.loop
  local forms = {
    timer_start = "start", timer_stop = "stop", timer_again = "again", timer_set_repeat = "set_repeat",
    timer_get_repeat = "get_repeat", timer_get_due_in = "get_due_in", close = "close", is_active = "is_active",
    is_closing = "is_closing", ref = "ref", unref = "unref", has_ref = "has_ref",
  }
  for fname, method in pairs(forms) do
    if uv and rawget(uv, fname) == nil then
      uv[fname] = function(h, ...) if h and h[method] then return h[method](h, ...) end end
    end
  end
end
-- startup is over: a plugin added from now on runs its plugin/ files at once (zett.plug)
pcall(vim.api.nvim_create_autocmd, "VimEnter", { callback = function() vim._zett_entered = true end })

-- vim.fs.abspath (Neovim 0.11): a path made absolute against the current folder, ~ expanded
if vim.fs and not vim.fs.abspath then
  function vim.fs.abspath(p)
    vim.validate("path", p, "string")
    if p:sub(1, 1) == "~" then p = (os.getenv("HOME") or "") .. p:sub(2) end
    if p:sub(1, 1) == "/" then return vim.fs.normalize(p) end
    return vim.fs.normalize(vim.fn.getcwd() .. "/" .. p)
  end
end

-- vim.on_key(fn, ns): fn(key, typed) for every key typed (nil fn: that one taken off); the editor calls
-- vim._zett_on_key only while one is set
do
  local watchers, next_ns = {}, 1000
  function vim.on_key(f, ns)
    ns = ns or (function() next_ns = next_ns + 1 return next_ns end)()
    watchers[ns] = f
    z.on_key_watch(next(watchers) ~= nil)
    return ns
  end
  function vim._zett_on_key(key)
    local k = vim.keycode and vim.keycode(key) or key
    for ns, f in pairs(watchers) do
      local ok, r = pcall(f, k, k)
      if not ok then watchers[ns] = nil vim.notify("on_key: " .. tostring(r), vim.log.levels.ERROR) end
    end
    z.on_key_watch(next(watchers) ~= nil)
  end
end

-- getchar / getcharstr: the next key, waited for (the editor reads the terminal); a plain character as its
-- number (getchar) or itself (getcharstr), a special key in Vim's internal form (nvim_replace_termcodes)
do
  local special = { ["<Esc>"] = 27, ["<Enter>"] = 13, ["<Tab>"] = 9, ["<Space>"] = 32 }
  local function next_key(expr)
    if expr == 0 then return nil end -- getchar(0): only when one is waiting; zett has none queued then
    return z.getkey and z.getkey() or nil
  end
  rawset(vim.fn, "getchar", function(expr)
    local k = next_key(expr)
    if not k then return 0 end
    if special[k] then return special[k] end
    local ctrl = k:match("^<C%-(%a)>$")
    if ctrl then return ctrl:lower():byte() - 96 end
    if utf8.len(k) == 1 then return utf8.codepoint(k) end
    return vim.api.nvim_replace_termcodes(k, true, false, true)
  end)
  rawset(vim.fn, "getcharstr", function(expr)
    local c = vim.fn.getchar(expr)
    if type(c) == "number" then return c == 0 and "" or utf8.char(c) end
    return c
  end)
end

-- ── more of libuv (vim.uv): processes with pipes, files, the system ──
do
  local uv = vim.uv or vim.loop
  local function set(name, f) if rawget(uv, name) == nil then uv[name] = f end end
  local function sh(cmd) local p = io.popen(cmd) if not p then return "" end local s = p:read("*a") or "" p:close() return (s:gsub("%s+$", "")) end
  -- an async call: the callback (last argument) gets (err, result) on the next tick; else the result
  local function either(f)
    return function(...)
      local args = { ... }
      local cb = type(args[#args]) == "function" and table.remove(args) or nil
      local ok, r, e2 = pcall(f, unpack(args))
      local err = not ok and tostring(r) or e2
      if cb then vim.schedule(function() cb(err, ok and r or nil) end) return true end
      if err then return nil, err end
      return r
    end
  end

  -- the system
  set("os_uname", function() return { sysname = sh("uname -s"), release = sh("uname -r"), version = sh("uname -v"), machine = sh("uname -m") } end)
  set("os_gethostname", function() local f = io.open("/etc/hostname") local h = f and f:read("*l") if f then f:close() end return h or sh("uname -n") end)
  set("os_getpid", function() return vim.fn.getpid() end)
  set("getpid", function() return vim.fn.getpid() end)
  set("os_tmpdir", function() return os.getenv("TMPDIR") or "/tmp" end)
  set("os_setenv", function(k, v) vim.env[k] = v return true end)
  set("os_unsetenv", function(k) vim.env[k] = nil return true end)
  set("os_get_passwd", function() return { username = os.getenv("USER") or sh("id -un"), uid = tonumber(sh("id -u")), gid = tonumber(sh("id -g")), homedir = os.getenv("HOME"), shell = os.getenv("SHELL") } end)
  set("chdir", function(d) vim.fn.chdir(d) return 0 end)
  set("available_parallelism", function() return tonumber(sh("nproc")) or 1 end)
  set("cpu_info", function() local n = tonumber(sh("nproc")) or 1 local t = {} for i = 1, n do t[i] = { model = "cpu", speed = 0, times = {} } end return t end)
  set("get_total_memory", function() local kb = tonumber(sh("awk '/MemTotal/{print $2}' /proc/meminfo")) return (kb or 0) * 1024 end)
  set("get_free_memory", function() local kb = tonumber(sh("awk '/MemAvailable/{print $2}' /proc/meminfo")) return (kb or 0) * 1024 end)
  set("uptime", function() local f = io.open("/proc/uptime") local s = f and f:read("*n") if f then f:close() end return s or 0 end)
  set("exepath", function() return vim.v.progpath or "zett" end)
  set("random", function(n) local t = {} for i = 1, n do t[i] = string.char(math.random(0, 255)) end return table.concat(t) end)
  set("version", function() return 0x013000 end)
  set("version_string", function() return "1.48.0" end)
  set("guess_handle", function(fd) return fd <= 2 and "tty" or "file" end)
  set("update_time", function() end)
  set("loop_alive", function() return false end)
  set("run", function() return false end)
  set("walk", function() end)
  set("metrics_idle_time", function() return 0 end)
  set("kill", function(pid, sig)
    local n = ({ sigterm = 15, sigkill = 9, sigint = 2, sighup = 1 })[tostring(sig or "sigterm"):lower()] or tonumber(sig) or 15
    os.execute(("kill -%d %d 2>/dev/null"):format(n, pid))
    return 0
  end)

  -- files: fds are zett's own numbers over Lua files
  local files, next_fd = {}, 100
  local modes = { r = "rb", ["r+"] = "r+b", w = "wb", ["w+"] = "w+b", a = "ab", ["a+"] = "a+b" }
  set("fs_open", either(function(path, flags)
    local m = type(flags) == "number" and (flags == 0 and "rb" or "wb") or modes[flags] or flags
    local f, e = io.open(path, m)
    if not f then error(("ENOENT: %s: %s"):format(e or "cannot open", path)) end
    next_fd = next_fd + 1 files[next_fd] = f
    return next_fd
  end))
  set("fs_read", either(function(fd, size, offset)
    local f = files[fd] if not f then error("EBADF: bad file descriptor") end
    if offset and offset >= 0 then f:seek("set", offset) end
    return f:read(size and size > 0 and size or "*a") or ""
  end))
  set("fs_write", either(function(fd, data, offset)
    local f = files[fd] if not f then error("EBADF: bad file descriptor") end
    if offset and offset >= 0 then f:seek("set", offset) end
    data = type(data) == "table" and table.concat(data) or data
    f:write(data) return #data
  end))
  set("fs_close", either(function(fd) local f = files[fd] if f then f:close() files[fd] = nil end return true end))
  set("fs_fstat", either(function(fd) local f = files[fd] if not f then error("EBADF") end local cur = f:seek() local size = f:seek("end") f:seek("set", cur) return { size = size, type = "file", mtime = { sec = 0, nsec = 0 } } end))
  set("fs_mkdir", either(function(p) if vim.fn.mkdir(p) == 0 and vim.fn.isdirectory(p) == 0 then error("EACCES: cannot make " .. p) end return true end))
  set("fs_rmdir", either(function(p) local ok, e = os.remove(p) if not ok then error(e) end return true end))
  set("fs_unlink", either(function(p) local ok, e = os.remove(p) if not ok then error("ENOENT: " .. tostring(e)) end return true end))
  set("fs_rename", either(function(a, b) local ok, e = os.rename(a, b) if not ok then error(e) end return true end))
  set("fs_access", either(function(p, mode) return vim.fn.filereadable(p) == 1 or vim.fn.isdirectory(p) == 1 end))
  set("fs_copyfile", either(function(a, b) local i = io.open(a, "rb") if not i then error("ENOENT: " .. a) end local s = i:read("*a") i:close() local o = io.open(b, "wb") if not o then error("EACCES: " .. b) end o:write(s) o:close() return true end))
  local function entries(dir)
    local out = {}
    for _, n in ipairs(vim.fn.readdir(dir)) do
      local p = dir .. "/" .. n
      out[#out + 1] = { name = n, type = vim.fn.isdirectory(p) == 1 and "directory" or (vim.fn.getftype(p) == "link" and "link" or "file") }
    end
    return out
  end
  set("fs_scandir", either(function(dir)
    if vim.fn.isdirectory(dir) == 0 then error("ENOENT: no such directory: " .. dir) end
    return { list = entries(dir), i = 0 }
  end))
  set("fs_scandir_next", function(h) if not h then return nil end h.i = h.i + 1 local e = h.list[h.i] if e then return e.name, e.type end return nil end)
  set("fs_opendir", either(function(dir, _, n) return { list = entries(dir), i = 0, n = n or 1 } end))
  set("fs_readdir", either(function(h) local out = {} for _ = 1, h.n do h.i = h.i + 1 if not h.list[h.i] then break end out[#out + 1] = h.list[h.i] end return #out > 0 and out or nil end))
  set("fs_closedir", either(function() return true end))

  -- handles that only wait: idle (a callback every tick), async (one when sent)
  set("new_idle", function()
    local h = { on = false }
    function h:start(cb) self.on = true local function tick() if self.on then pcall(cb) vim.defer_fn(tick, 10) end end vim.defer_fn(tick, 0) return 0 end
    function h:stop() self.on = false return 0 end
    function h:close() self.on = false end
    function h:is_closing() return not self.on end
    return h
  end)
  set("new_async", function(cb) local h = {} function h:send(...) local a = { ... } vim.schedule(function() cb(unpack(a)) end) return 0 end function h:close() end function h:is_closing() return false end return h end)
  set("new_work", function(work, done) return { work = work, done = done } end)
  set("queue_work", function(w, ...) local r = { pcall(w.work, ...) } vim.schedule(function() if r[1] and w.done then w.done(select(2, unpack(r))) end end) return true end)

  -- uv.spawn with pipes, on zett's streaming jobs: read_start gets each piece, write goes to stdin
  set("new_pipe", function()
    local p = { readers = {}, closed = false }
    -- what came while nobody read waits in order (the end too, as false); reading hands it out one piece at a
    -- time while a reader is there (telescope reads a piece, stops, starts again)
    function p:_flush()
      while self.reader and self.queue and #self.queue > 0 do
        local d = table.remove(self.queue, 1)
        local r = self.reader
        if d == false then self.eof = true r(nil, nil) else r(nil, d) end
      end
    end
    function p:read_start(cb) self.reader = cb self:_flush() return 0 end
    function p:read_stop() self.reader = nil return 0 end
    function p:write(data, cb) if self.job then z.job.send(self.job, type(data) == "table" and table.concat(data) or data) end if cb then vim.schedule(cb) end return 0 end
    function p:shutdown(cb) if self.job then z.job.close_stdin(self.job) end if cb then vim.schedule(cb) end return 0 end
    function p:close(cb) self.closed = true self.reader = nil if cb then vim.schedule(cb) end end
    function p:is_closing() return self.closed end
    function p:is_active() return not self.closed end
    function p:_deliver(data) self.queue = self.queue or {} if data == nil then table.insert(self.queue, false) else table.insert(self.queue, data) end self:_flush() end
    return p
  end)
  set("spawn", function(cmd, o, on_exit)
    o = o or {}
    local argv = { cmd } for _, a in ipairs(o.args or {}) do argv[#argv + 1] = tostring(a) end
    local stdio = o.stdio or {}
    local env
    if o.env then env = {} for _, kv in ipairs(o.env) do local k, v = tostring(kv):match("^([^=]+)=(.*)$") if k then env[k] = v end end end
    local h = { closed = false }
    local ok, id, pid = pcall(z.job.stream, argv, { cwd = o.cwd, env = env, clear_env = env ~= nil }, function(kind, data)
      if kind == "stdout" and stdio[2] then stdio[2]:_deliver(data)
      elseif kind == "stderr" and stdio[3] then stdio[3]:_deliver(data)
      elseif kind == "exit" then
        for i = 2, 3 do if stdio[i] and stdio[i]._deliver then stdio[i]:_deliver(nil) end end
        h.exited = true
        if on_exit then on_exit(data, 0) end
      end
    end)
    if not ok then return nil, "ENOENT: " .. tostring(id) end
    h.pid, h.job = pid, id
    if stdio[1] then stdio[1].job = id end
    function h:kill(sig) z.job.kill(id, ({ sigterm = 15, sigkill = 9, sigint = 2 })[tostring(sig or "sigterm"):lower()] or tonumber(sig) or 15) return 0 end
    function h:close() self.closed = true end
    function h:is_closing() return self.closed end
    function h:is_active() return not self.exited end
    function h:get_pid() return pid end
    return h, pid
  end)
end

-- uv check / prepare handles (a callback each turn of the loop; plenary's Job waits for its pipes with one),
-- and the function forms of their methods (uv.check_start(h, cb) as h:start(cb))
do
  local uv = vim.uv or vim.loop
  local function looper()
    local h = { on = false, closed = false }
    function h:start(cb) self.on = true local function tick() if self.on and not self.closed then pcall(cb) vim.defer_fn(tick, 5) end end vim.defer_fn(tick, 0) return 0 end
    function h:stop() self.on = false return 0 end
    function h:close(cb) self.on = false self.closed = true if cb then vim.schedule(cb) end end
    function h:is_closing() return self.closed end
    function h:is_active() return self.on end
    return h
  end
  uv.new_check = looper
  uv.new_prepare = looper
  for _, kind in ipairs({ "check", "prepare", "idle" }) do
    uv[kind .. "_start"] = uv[kind .. "_start"] or function(h, cb) return h:start(cb) end
    uv[kind .. "_stop"] = uv[kind .. "_stop"] or function(h) return h:stop() end
  end
  uv.is_active = uv.is_active or function(h) return h and h.is_active and h:is_active() or false end
end

-- the function forms of handle methods (uv.process_kill(h, sig) as h:kill(sig)): telescope and plenary call these
do
  local uv = vim.uv or vim.loop
  local forms = { process_kill = "kill", read_start = "read_start", read_stop = "read_stop", write = "write", shutdown = "shutdown",
    close = "close", process_get_pid = "get_pid", is_readable = "is_active", is_writable = "is_active" }
  for fname, method in pairs(forms) do
    if uv[fname] == nil then
      uv[fname] = function(h, ...) if type(h) == "table" and type(h[method]) == "function" then return h[method](h, ...) end return 0 end
    end
  end
end

-- a float as the current window (nvim_set_current_win, nvim_open_win with enter): keys, fed keys, mode() and the
-- current buffer are the float's until another window is made current or it closes
do
  local api = vim.api
  local set_win, open_win, feed = api.nvim_set_current_win, api.nvim_open_win, api.nvim_feedkeys
  local function focus_float(h)
    vim._float_focus = h
    vim._cur_float = h
    local p = vim._zett_float_panel and vim._zett_float_panel(h)
    if p then pcall(z.panel.focus, p) end
  end
  api.nvim_set_current_win = function(w)
    if vim._zett_is_float and vim._zett_is_float(w) then return focus_float(w) end
    if vim._float_focus then vim._float_focus = nil vim._cur_float = nil pcall(z.panel.focus, nil) end
    return set_win(w)
  end
  api.nvim_open_win = function(b, enter, config)
    local w = open_win(b, enter, config)
    if enter and vim._zett_is_float and vim._zett_is_float(w) then focus_float(w) end
    return w
  end
  -- fed keys while a float has the keyboard go to it, never to the file under it
  api.nvim_feedkeys = function(keys, mode, esc)
    local h = vim._float_focus
    if h and vim._zett_is_float(h) then
      for _, k in ipairs(vim._zett_key_tokens(keys)) do vim._zett_float_key(h, k, tostring(mode or ""):find("n") ~= nil) end
      return
    end
    return feed(keys, mode, esc)
  end
  local mode_fn, get_mode = vim.fn.mode, api.nvim_get_mode
  rawset(vim.fn, "mode", function(...)
    local h = vim._float_focus
    if h and vim._zett_float_mode then return vim._zett_float_mode(h) end
    return mode_fn(...)
  end)
  api.nvim_get_mode = function()
    local h = vim._float_focus
    if h and vim._zett_float_mode then return { mode = vim._zett_float_mode(h), blocking = false } end
    return get_mode()
  end
  for _, k in ipairs({ "nvim_set_current_win", "nvim_open_win", "nvim_feedkeys", "nvim_get_mode" }) do rawset(vim.fn, k, api[k]) end
end

-- text in a buffer that is not the current one (a plugin's scratch buffer, another file): through its lines,
-- never into the current buffer (telescope draws its "> " into the results this way)
do
  local api = vim.api
  local set_text, get_text = api.nvim_buf_set_text, api.nvim_buf_get_text
  local function other(b)
    if b == nil or b == 0 then return vim._cur_float ~= nil end
    return b ~= vim.api.nvim_get_current_buf() or vim._cur_float ~= nil
  end
  api.nvim_buf_get_text = function(b, sr, sc, er, ec, o)
    if not other(b) then return get_text(b, sr, sc, er, ec, o) end
    local l = api.nvim_buf_get_lines(b, sr, er + 1, false)
    if #l == 0 then return {} end
    if #l == 1 then return { l[1]:sub(sc + 1, ec) } end
    l[1] = l[1]:sub(sc + 1)
    l[#l] = l[#l]:sub(1, ec)
    return l
  end
  api.nvim_buf_set_text = function(b, sr, sc, er, ec, repl)
    if not other(b) then return set_text(b, sr, sc, er, ec, repl) end
    local l = api.nvim_buf_get_lines(b, sr, er + 1, false)
    local head, tail = (l[1] or ""):sub(1, sc), (l[#l] or ""):sub(ec + 1)
    local new = {}
    for i, r in ipairs(repl or {}) do new[i] = r end
    if #new == 0 then new = { "" } end
    new[1] = head .. new[1]
    new[#new] = new[#new] .. tail
    api.nvim_buf_set_lines(b, sr, er + 1, false, new)
  end
  rawset(vim.fn, "nvim_buf_set_text", api.nvim_buf_set_text)
  rawset(vim.fn, "nvim_buf_get_text", api.nvim_buf_get_text)
end

-- options set for another file while it was not current (vim.bo[n]) take effect when it is
vim.api.nvim_create_autocmd("BufEnter", {
  callback = function(a)
    local held = vim._zett_held_opts and vim._zett_held_opts[a.buf]
    if not held then return end
    vim._zett_held_opts[a.buf] = nil
    for k, v in pairs(held) do pcall(function() vim.bo[k] = v end) end
  end,
})
