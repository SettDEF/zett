-- Vimscript in zett: each script is translated to Lua (on vim.lua's API) and run. let, unlet,
-- const, if/elseif/else, while, for, break, continue, function (a:, l:, s:, ...), return, call,
-- try/catch/finally, throw, echo/echon/echomsg/echoerr, execute, finish; any other line is an Ex
-- command (vim.cmd). Expressions: numbers, 'strings' and "strings", lists, dicts, lambdas, slices,
-- ?:, || &&, comparisons (==# ==? =~ is isnot …), + - * / % . .., methods (->), funcrefs.
local M = {}
local V = {}
M.V = V
vim._vimscript = M
vim._vimfn = vim._vimfn or {}

-- ── the runtime the translated code calls ──
local function isfloat(x) return math.type(x) == "float" end
function V.t(x)
  if x == nil or x == false or x == vim.NIL then return false end
  if x == true then return true end
  if type(x) == "number" then return x ~= 0 end
  if type(x) == "string" then return (tonumber(x:match("^%s*([-+]?%d+)")) or 0) ~= 0 end
  return true
end
function V.num(x)
  if type(x) == "number" then return x end
  if x == true then return 1 end
  if x == false or x == nil or x == vim.NIL then return 0 end
  if type(x) == "string" then
    local h = x:match("^%s*0[xX](%x+)")
    if h then return tonumber(h, 16) end
    return math.tointeger(tonumber(x:match("^%s*([-+]?%d+)") or "0")) or 0
  end
  error("E745: Using a List as a Number")
end
function V.str(x, inner)
  local t = type(x)
  if t == "string" then return inner and ("'" .. x:gsub("'", "''") .. "'") or x end
  if t == "number" then
    if isfloat(x) then local s = string.format("%g", x) return s:find("[.e]") and s or s .. ".0" end
    return tostring(x)
  end
  if x == true then return "v:true" end
  if x == false then return "v:false" end
  if x == nil or x == vim.NIL then return "v:null" end
  if t == "table" then
    if rawget(x, "__funcref") then return "function('" .. x.__funcref .. "')" end
    local parts = {}
    if vim.islist(x) and (next(x) ~= nil or getmetatable(x) ~= V.dict_mt) then
      for _, v in ipairs(x) do parts[#parts + 1] = V.str(v, true) end
      return "[" .. table.concat(parts, ", ") .. "]"
    end
    local keys = vim.tbl_keys(x)
    table.sort(keys, function(a, b) return tostring(a) < tostring(b) end)
    for _, k in ipairs(keys) do parts[#parts + 1] = "'" .. tostring(k) .. "': " .. V.str(x[k], true) end
    return "{" .. table.concat(parts, ", ") .. "}"
  end
  if t == "function" then return "function('<lambda>')" end
  return tostring(x)
end
V.dict_mt = {}
function V.dict(t) return setmetatable(t, V.dict_mt) end
local function both_num(a, b) return type(a) ~= "string" and type(b) ~= "string" end
local function arith(a, b, f)
  local x, y
  if type(a) == "number" then x = a else x = V.num(a) end
  if type(b) == "number" then y = b else y = V.num(b) end
  return f(x, y)
end
function V.add(a, b)
  if type(a) == "table" and type(b) == "table" then
    local r = {}
    for _, v in ipairs(a) do r[#r + 1] = v end
    for _, v in ipairs(b) do r[#r + 1] = v end
    return r
  end
  return arith(a, b, function(x, y) return x + y end)
end
function V.sub(a, b) return arith(a, b, function(x, y) return x - y end) end
function V.mul(a, b) return arith(a, b, function(x, y) return x * y end) end
function V.div(a, b)
  return arith(a, b, function(x, y)
    if isfloat(x) or isfloat(y) then return x / y end
    if y == 0 then return x == 0 and -0x7fffffffffffffff or (x > 0 and 0x7fffffffffffffff or -0x7fffffffffffffff) end
    local q = x // y
    if (x % y ~= 0) and ((x < 0) ~= (y < 0)) then q = q + 1 end
    return q
  end)
end
function V.mod(a, b)
  return arith(a, b, function(x, y)
    if y == 0 then return 0 end
    local r = math.fmod(x, y)
    return math.tointeger(r) or r
  end)
end
function V.neg(a) return -V.num(a) end
function V.not_(a) return V.t(a) and 0 or 1 end
function V.cat(a, b) return V.str(a) .. V.str(b) end
function V.bool(b) return b and 1 or 0 end
function V.opt(v) if v == true then return 1 elseif v == false then return 0 end return v end
local function deepeq(a, b)
  if type(a) ~= type(b) then return false end
  if type(a) ~= "table" then return a == b end
  for k, v in pairs(a) do if not deepeq(v, b[k]) then return false end end
  for k in pairs(b) do if a[k] == nil then return false end end
  return true
end
function V.cmp(op, case, a, b)
  if op == "is" then return V.bool(type(a) == "table" and a == b or (type(a) ~= "table" and a == b)) end
  if op == "isnot" then return V.not_(V.cmp("is", case, a, b)) end
  if op == "=~" or op == "!~" then
    local s, p = V.str(a), V.str(b)
    if case == "?" or (case == "" and vim.o.ignorecase == true) then s, p = s:lower(), p:lower() end
    local hit = s:find(vim._lua_pattern(p)) ~= nil
    return V.bool(hit == (op == "=~"))
  end
  if type(a) == "table" or type(b) == "table" then
    local eq = deepeq(a, b)
    if op == "==" then return V.bool(eq) end
    if op == "!=" then return V.bool(not eq) end
    error("E692: Invalid operation for List")
  end
  local x, y = a, b
  if type(x) == "string" and type(y) == "string" then
    if case == "?" or (case == "" and vim.o.ignorecase == true) then x, y = x:lower(), y:lower() end
  else
    x, y = (type(x) == "number" and x or V.num(x)), (type(y) == "number" and y or V.num(y))
  end
  if op == "==" then return V.bool(x == y) end
  if op == "!=" then return V.bool(x ~= y) end
  if op == ">" then return V.bool(x > y) end
  if op == ">=" then return V.bool(x >= y) end
  if op == "<" then return V.bool(x < y) end
  if op == "<=" then return V.bool(x <= y) end
  error("unknown comparison " .. op)
end
function V.idx(x, i)
  if type(x) == "string" then
    i = V.num(i)
    if i < 0 then return "" end
    return x:sub(i + 1, i + 1)
  end
  if type(x) ~= "table" then error("E121: Cannot index a " .. type(x)) end
  if vim.islist(x) and next(x) ~= nil and type(i) == "number" then
    if i < 0 then i = #x + i end
    local v = x[i + 1]
    if v == nil then error("E684: List index out of range: " .. i) end
    return v
  end
  if type(i) == "number" and #x > 0 then return x[i + 1] end
  local v = x[tostring(i)]
  if v == nil then v = x[i] end
  if v == nil then error("E716: Key not present in Dictionary: \"" .. tostring(i) .. "\"") end
  return v
end
function V.member(x, k) return V.idx(x, k) end
function V.slice(x, a, b)
  local n = type(x) == "string" and #x or #x
  a = a == nil and 0 or V.num(a)
  b = b == nil and n - 1 or V.num(b)
  if a < 0 then a = math.max(n + a, 0) end
  if b < 0 then b = n + b end
  if type(x) == "string" then return x:sub(a + 1, b + 1) end
  local r = {}
  for k = a + 1, math.min(b + 1, n) do r[#r + 1] = x[k] end
  return r
end
function V.setidx(x, i, v)
  if type(x) ~= "table" then error("E689: Can only index a List or Dictionary") end
  if type(i) == "number" and (vim.islist(x) and next(x) ~= nil) then
    if i < 0 then i = #x + i end
    x[i + 1] = v
  else
    x[tostring(i)] = v
  end
end
function V.delidx(x, i)
  if type(i) == "number" and vim.islist(x) then table.remove(x, (i < 0 and #x + i or i) + 1) else x[tostring(i)] = nil end
end
function V.list(x)
  if type(x) == "table" then
    if vim.islist(x) then return x end
    error("E714: List required")
  end
  if type(x) == "string" then local r = {} for _, c in utf8.codes(x) do r[#r + 1] = utf8.char(c) end return r end
  error("E714: List required")
end
function V.args(names, ...)
  local all = { ... }
  local A = { firstline = 1, lastline = 1 }
  for i, n in ipairs(names) do A[n] = all[i] end
  local rest = {}
  for i = #names + 1, #all do rest[#rest + 1] = all[i] A[tostring(i - #names)] = all[i] end
  A["0"] = #rest
  A["000"] = rest
  return A
end
local function fname(S, name)
  name = name:gsub("^g:", "")
  if name:match("^s:") or name:match("^<SID>") then return S, "fn:" .. name:gsub("^s:", ""):gsub("^<SID>", "") end
  return vim._vimfn, name
end
function V.def(S, name, f, bang)
  local t, k = fname(S, name)
  if t[k] and not bang then error("E122: Function " .. name .. " already exists, add ! to replace it") end
  t[k] = f
  -- a script's own function is also <SNR>N_Name, so its mappings and commands find it later
  if t == S and S.__sid then vim._vimfn["<SNR>" .. S.__sid .. "_" .. k:sub(4)] = f end
end
-- an Ex line from a script: its <SID> and s:Name( become <SNR>N_Name (Vim's way to reach them)
function V.sid(S, line)
  if not S.__sid then return line end
  local snr = "<SNR>" .. S.__sid .. "_"
  line = line:gsub("<SID>", snr)
  return (line:gsub("%f[%w_:]s:([%a_][%w_]*)%s*%(", snr .. "%1("))
end
function V.undef(S, name) local t, k = fname(S, name) t[k] = nil end
function V.fn(S, name)
  local t, k = fname(S, name)
  local f = t[k]
  if f then return f end
  if type(vim.g[name]) == "function" or type(vim.g[name]) == "table" then return vim.g[name] end
  return nil
end
function V.call(S, name, ...)
  local f = V.fn(S, name)
  if f then return f(...) end
  local b = rawget(vim.fn, name)
  if b then return b(...) end
  error("E117: Unknown function: " .. name)
end
function V.callv(f, ...)
  if type(f) == "string" then return vim.fn[f](...) end
  if f == nil then error("E117: Unknown function") end
  return f(...)
end
function V.funcref(S, name, bound)
  if type(name) ~= "string" then return name end
  local fr = { __funcref = name }
  return setmetatable(fr, {
    __call = function(_, ...)
      local a = { table.unpack(bound or {}) }
      for _, v in ipairs({ ... }) do a[#a + 1] = v end
      local user = V.fn(S, name)
      if user then return user(table.unpack(a)) end
      return vim.fn[name](table.unpack(a))
    end,
  })
end
-- map() and filter() with a string ('v:val * 2') or a funcref/lambda (key, val)
function V.mapf(S, how, x, f)
  local run
  if type(f) == "string" then
    local code = M.expr_lua(f, { top = true })
    local chunk = assert(load("local V, S = ... return function() return " .. code .. " end", "=map()", "t"))(V, S)
    run = function(k, v) vim.v.key, vim.v.val = k, v return chunk() end
  else
    run = function(k, v) return f(k, v) end
  end
  if vim.islist(x) then
    local out = {}
    for i, v in ipairs(x) do
      local r = run(i - 1, v)
      if how == "map" then out[#out + 1] = r elseif V.t(r) then out[#out + 1] = v end
    end
    for i = #x, 1, -1 do x[i] = nil end
    for i, v in ipairs(out) do x[i] = v end
  else
    for k, v in pairs(x) do
      local r = run(k, v)
      if how == "map" then x[k] = r elseif not V.t(r) then x[k] = nil end
    end
  end
  return x
end
function V.echo(sep, ...)
  local parts = {}
  for i = 1, select("#", ...) do parts[#parts + 1] = V.str((select(i, ...))) end
  zett.notify(table.concat(parts, sep))
end
function V.echoerr(...) local p = {} for i = 1, select("#", ...) do p[#p + 1] = V.str((select(i, ...))) end zett.notify("E: " .. table.concat(p, " ")) end
function V.join(sep, ...) local p = {} for i = 1, select("#", ...) do p[#p + 1] = V.str((select(i, ...))) end return table.concat(p, sep) end
function V.throw(e) return { __vimexc = V.str(e) } end
function V.exc(err)
  if type(err) == "table" and err.__vimexc then return err.__vimexc end
  local s = tostring(err)
  s = s:gsub("^[^:]*:%d+: ", "")
  if not s:match("^E%d+") and not s:match("^Vim") then s = "Vim:" .. s end
  return s
end
function V.catch(exc, pat)
  if pat == nil or pat == "" then return true end
  return exc:find(vim._lua_pattern(pat)) ~= nil
end
V.RET = {}
function V.ret(v) return { [V.RET] = true, v = v } end
function V.isret(x) return type(x) == "table" and x[V.RET] == true end
function V.discard() end
function V.reg() return "" end
-- :set inside a script: at once (a later &ts sees it), and for the editor as typed
function V.set(scope, args)
  for a in args:gmatch("%S+") do
    local name, op, val = a:match("^([%w_]+)([+%-^]?=)(.*)$")
    if not name then name, op, val = a:match("^([%w_]+)(:)(.*)$") end
    local tbl = scope == "local" and vim.opt_local or vim.opt
    if name then
      if op == "+=" then tbl[name]:append(val)
      elseif op == "-=" then tbl[name]:remove(val)
      elseif op == "^=" then tbl[name]:prepend(val)
      else
        local i = zett.option_info and zett.option_info(name)
        if i and i.kind == "int" then val = tonumber(val) or 0 end
        tbl[name] = val
      end
    else
      local bang = a:match("^([%w_]+)!$")
      local inv = a:match("^inv([%w_]+)$")
      local no = a:match("^no([%w_]+)$")
      local q = a:match("^([%w_]+)%?$") or a:match("^([%w_]+)&$")
      local o = scope == "local" and vim.bo or vim.o
      if bang or (inv and zett.option_info(inv)) then local n = bang or inv o[n] = not V.t(V.opt(o[n]))
      elseif q then zett.ui("ex", "set " .. a)
      elseif no and zett.option_info(no) and zett.option_info(no).kind == "bool" then o[no] = false
      elseif zett.option_info(a) and zett.option_info(a).kind == "bool" then o[a] = true
      else zett.ui("ex", (scope == "local" and "setlocal " or "set ") .. a) end
    end
  end
end

-- ── lexing ──
local function lex(s)
  local toks, i, n = {}, 1, #s
  local function push(k, v, sp) toks[#toks + 1] = { k = k, v = v, sp = sp } end
  while i <= n do
    local c = s:sub(i, i)
    local sp = false
    while c:match("%s") do i = i + 1 c = s:sub(i, i) sp = true end
    if i > n then break end
    local two, three = s:sub(i, i + 1), s:sub(i, i + 2)
    if c:match("%d") then
      local num = s:match("^0[xX]%x+", i) or s:match("^0[bB][01]+", i) or s:match("^%d+%.%d+[eE][-+]?%d+", i) or s:match("^%d+%.%d+", i) or s:match("^%d+", i)
      local v
      if num:match("^0[xX]") then v = tonumber(num:sub(3), 16)
      elseif num:match("^0[bB]") then v = tonumber(num:sub(3), 2)
      else v = math.tointeger(tonumber(num)) or tonumber(num) end
      push("num", v, sp)
      i = i + #num
    elseif c == "'" then
      local out, j = {}, i + 1
      while j <= n do
        local d = s:sub(j, j)
        if d == "'" then
          if s:sub(j + 1, j + 1) == "'" then out[#out + 1] = "'" j = j + 2 else break end
        else out[#out + 1] = d j = j + 1 end
      end
      push("str", table.concat(out), sp)
      i = j + 1
    elseif c == '"' then
      local out, j = {}, i + 1
      local esc = { n = "\n", t = "\t", r = "\r", e = "\27", b = "\b", f = "\f", ["\\"] = "\\", ['"'] = '"' }
      while j <= n do
        local d = s:sub(j, j)
        if d == '"' then break end
        if d == "\\" then
          local e = s:sub(j + 1, j + 1)
          if esc[e] then out[#out + 1] = esc[e] j = j + 2
          elseif e == "x" or e == "X" then local h = s:match("^%x%x?", j + 2) or "" out[#out + 1] = string.char(tonumber(h, 16) or 0) j = j + 2 + #h
          elseif e == "u" or e == "U" then local h = s:match("^%x%x?%x?%x?%x?%x?%x?%x?", j + 2) or "" out[#out + 1] = utf8.char(tonumber(h, 16) or 0) j = j + 2 + #h
          elseif e == "<" then local k = s:match("^<[^>]+>", j + 1) or "<" out[#out + 1] = k j = j + 1 + #k
          else out[#out + 1] = e j = j + 2 end
        else out[#out + 1] = d j = j + 1 end
      end
      push("str", table.concat(out), sp)
      i = j + 1
    elseif c == "&" and s:sub(i + 1, i + 1):match("[%a]") and s:sub(i + 1, i + 1) ~= "&" then
      local scope, name = s:match("^&([lg]):([%w_]+)", i)
      if scope then push("opt", { scope = scope, name = name }, sp) i = i + 3 + #name
      else name = s:match("^&([%w_]+)", i) push("opt", { scope = "", name = name }, sp) i = i + 1 + #name end
    elseif c == "$" and s:sub(i + 1, i + 1):match("[%w_]") then
      local name = s:match("^%$([%w_]+)", i)
      push("env", name, sp)
      i = i + 1 + #name
    elseif c == "@" and i < n then
      push("reg", s:sub(i + 1, i + 1), sp)
      i = i + 2
    elseif c:match("[%a_]") or (c == "<" and (s:sub(i, i + 4) == "<SID>" or s:sub(i, i + 4) == "<SNR>")) then
      local id
      if c == "<" then id = s:sub(i, i + 4) .. (s:match("^[%w_#]*", i + 5) or "") else
        local sc = s:match("^([gbwtslav]):", i)
        if sc and s:sub(i + 2, i + 2):match("[%w_]") then id = sc .. ":" .. s:match("^[%w_#]*", i + 2)
        -- g: alone is the dictionary of global variables (get(g:, 'x', 0))
        elseif sc and (s:sub(i + 2, i + 2) == "" or s:sub(i + 2, i + 2):match("[,%)%]%[]")) and (s:sub(i - 1, i - 1) == "" or not s:sub(i - 1, i - 1):match("[%w_]")) then id = sc .. ":"
        else id = s:match("^[%w_#]+", i) end
      end
      -- curly-brace names: g:terminal_color_{i} (a part of the name computed); not a lambda
      while s:sub(i + #id, i + #id) == "{" and s:sub(i + #id - 1, i + #id - 1):match("[%w_]") do
        local j, depth = i + #id + 1, 1
        while j <= n and depth > 0 do
          local d = s:sub(j, j)
          if d == "{" then depth = depth + 1 elseif d == "}" then depth = depth - 1 end
          j = j + 1
        end
        if depth > 0 or s:sub(i + #id + 1, j - 2):find("->", 1, true) then break end
        id = s:sub(i, j - 1) .. (s:match("^[%w_#]*", j) or "")
      end
      push("id", id, sp)
      i = i + #id
    elseif three == "==#" or three == "==?" or three == "!=#" or three == "!=?" or three == ">=#" or three == ">=?" or three == "<=#" or three == "<=?" or three == "=~#" or three == "=~?" or three == "!~#" or three == "!~?" or three == "..=" then
      push("op", three, sp) i = i + 3
    elseif two == "==" or two == "!=" or two == ">=" or two == "<=" or two == "=~" or two == "!~" or two == "||" or two == "&&" or two == "->" or two == ".." or two == "+=" or two == "-=" or two == "*=" or two == "/=" or two == "%=" or two == ".=" or two == "#{" then
      push("op", two, sp) i = i + 2
    elseif c == ">" or c == "<" then
      local cs = s:sub(i + 1, i + 1)
      if cs == "#" or cs == "?" then push("op", c .. cs, sp) i = i + 2 else push("op", c, sp) i = i + 1 end
    else
      push("op", c, sp)
      i = i + 1
    end
  end
  push("eof", nil, true)
  return toks
end

-- ── expressions → Lua source ──
local P = {}
P.__index = P
local function parser(s, ctx) return setmetatable({ t = lex(s), i = 1, ctx = ctx or {}, src = s }, P) end
function P:peek(o) return self.t[self.i + (o or 0)] end
function P:next() local t = self.t[self.i] self.i = self.i + 1 return t end
function P:is(v) local t = self.t[self.i] return t.k == "op" and t.v == v end
function P:accept(v) if self:is(v) then self.i = self.i + 1 return true end return false end
function P:expect(v) if not self:accept(v) then error("E15: Invalid expression: expected " .. v .. " in \"" .. self.src .. "\"", 0) end end
local function q(s) return string.format("%q", s) end

-- a variable's Lua place: g:x vim.g["x"], l:x / a bare one in a function L["x"], a:x A["x"]…
function P:var(id)
  -- a curly-brace name: the table of its scope, keyed by the name put together
  if id:find("{", 1, true) then
    local sc, rest = id:match("^(%a):(.*)$")
    if not sc then rest = id end
    local parts, pos = {}, 1
    while true do
      local a = rest:find("{", pos, true)
      if not a then
        if pos <= #rest then parts[#parts + 1] = q(rest:sub(pos)) end
        break
      end
      if a > pos then parts[#parts + 1] = q(rest:sub(pos, a - 1)) end
      local depth, j = 1, a + 1
      while j <= #rest and depth > 0 do
        local d = rest:sub(j, j)
        if d == "{" then depth = depth + 1 elseif d == "}" then depth = depth - 1 end
        j = j + 1
      end
      parts[#parts + 1] = "V.str(" .. parser(rest:sub(a + 1, j - 2), self.ctx):expr() .. ")"
      pos = j
    end
    local base = sc and (({ g = "vim.g", l = "L", a = "A", s = "S", v = "vim.v" })[sc] or ("vim." .. sc)) or (self.ctx.fn and "L" or "vim.g")
    return base .. "[" .. table.concat(parts, " .. ") .. "]"
  end
  local params = self.ctx.params
  if params and params[id] then return "P_" .. id end
  local sc, name = id:match("^(%a):(.*)$")
  if sc and name == "" then
    return ({ g = "vim.g", l = "L", a = "A", s = "S", v = "vim.v" })[sc] or ("vim." .. sc)
  end
  if not sc then
    if id == "true" or id == "false" or id == "null" then return nil end
    if self.ctx.fn then return "L[" .. q(id) .. "]" end
    return "vim.g[" .. q(id) .. "]"
  end
  if sc == "g" then return "vim.g[" .. q(name) .. "]" end
  if sc == "l" then return "L[" .. q(name) .. "]" end
  if sc == "a" then return "A[" .. q(name) .. "]" end
  if sc == "s" then return "S[" .. q(name) .. "]" end
  if sc == "v" then
    if name == "true" then return "true" elseif name == "false" then return "false" elseif name == "null" then return "vim.NIL" end
    return "vim.v[" .. q(name) .. "]"
  end
  return "vim." .. sc .. "[" .. q(name) .. "]"
end

local BIN = {
  { ops = { ["||"] = true } },
  { ops = { ["&&"] = true } },
  { cmp = true },
  { ops = { ["+"] = "V.add", ["-"] = "V.sub", ["."] = "V.cat", [".."] = "V.cat" } },
  { ops = { ["*"] = "V.mul", ["/"] = "V.div", ["%"] = "V.mod" } },
}
local CMP = { ["=="] = 1, ["!="] = 1, [">"] = 1, [">="] = 1, ["<"] = 1, ["<="] = 1, ["=~"] = 1, ["!~"] = 1 }

function P:expr()
  local c = self:binary(1)
  if self:accept("?") then
    local a = self:expr()
    self:expect(":")
    local b = self:expr()
    return "(function() if V.t(" .. c .. ") then return " .. a .. " end return " .. b .. " end)()"
  end
  return c
end
function P:binary(level)
  if level > #BIN then return self:unary() end
  local lhs = self:binary(level + 1)
  local spec = BIN[level]
  while true do
    local t = self:peek()
    if spec.cmp then
      local op, case
      if t.k == "op" then
        local base, cs = t.v:match("^([=!<>~]+)([#?]?)$")
        if base and CMP[base] then op, case = base, cs end
      elseif t.k == "id" and (t.v == "is" or t.v == "isnot" or t.v == "is#" or t.v == "isnot#" or t.v == "is?" or t.v == "isnot?") then
        op, case = t.v:match("^(%a+)([#?]?)$")
      end
      if not op then return lhs end
      self:next()
      local rhs = self:binary(level + 1)
      lhs = "V.cmp(" .. q(op) .. ", " .. q(case) .. ", " .. lhs .. ", " .. rhs .. ")"
    else
      if t.k ~= "op" or not spec.ops[t.v] then return lhs end
      -- "a.b" without spaces after a variable is a dict member, done in postfix already
      self:next()
      local rhs = self:binary(level + 1)
      local f = spec.ops[t.v]
      if t.v == "||" then lhs = "V.bool(V.t(" .. lhs .. ") or V.t(" .. rhs .. "))"
      elseif t.v == "&&" then lhs = "V.bool(V.t(" .. lhs .. ") and V.t(" .. rhs .. "))"
      else lhs = f .. "(" .. lhs .. ", " .. rhs .. ")" end
    end
  end
end
function P:unary()
  if self:accept("!") then return "V.not_(" .. self:unary() .. ")" end
  if self:accept("-") then return "V.neg(" .. self:unary() .. ")" end
  if self:accept("+") then return "V.num(" .. self:unary() .. ")" end
  return self:postfix(self:primary())
end
function P:args()
  local a = {}
  if self:accept(")") then return a end
  repeat a[#a + 1] = self:expr() until not self:accept(",")
  self:expect(")")
  return a
end
-- a call by name: Vim's own functions (lower case) are vim.fn's, a user's (Capital, s:, g:, #) V.call
function P:call(name, args, first)
  if first then table.insert(args, 1, first) end
  local all = table.concat(args, ", ")
  local params = self.ctx.params
  if params and params[name] then return "V.callv(P_" .. name .. (all ~= "" and ", " .. all or "") .. ")" end
  if name == "function" or name == "funcref" then
    return "V.funcref(S, " .. (args[1] or "nil") .. (args[2] and (", " .. args[2]) or "") .. ")"
  end
  if name == "map" or name == "filter" then return "V.mapf(S, " .. q(name) .. ", " .. all .. ")" end
  if name == "exists" then return "V.exists(S, L, " .. all .. ")" end
  if name == "call" then return "V.callv(" .. (args[1]:match("^\"") and ("V.funcref(S, " .. args[1] .. ")") or args[1]) .. ", table.unpack(" .. (args[2] or "{}") .. "))" end
  if name == "eval" then return "M.eval(" .. all .. ")" end
  if name == "execute" then return "V.exec(" .. all .. ")" end
  local local_var = name:match("^l:") or (self.ctx.fn and self.ctx.locals and self.ctx.locals[name])
  if local_var then return "V.callv(" .. self:var(name) .. (all ~= "" and ", " .. all or "") .. ")" end
  if name:match("^[a-z][%w_]*$") then return "vim.fn[" .. q(name) .. "](" .. all .. ")" end
  return "V.call(S, " .. q(name) .. (all ~= "" and ", " .. all or "") .. ")"
end
function P:primary()
  local t = self:next()
  if t.k == "num" then return tostring(t.v) end
  if t.k == "str" then return q(t.v) end
  -- &t_Co: the colours there are (true colour reports 256, as Neovim does)
  if t.k == "opt" and t.v.name == "t_Co" then return "256" end
  if t.k == "opt" then
    local tbl = t.v.scope == "l" and "vim.bo" or "vim.o"
    return "V.opt(" .. tbl .. "[" .. q(t.v.name) .. "])"
  end
  if t.k == "env" then return "(vim.env[" .. q(t.v) .. "] or \"\")" end
  if t.k == "reg" then return "V.reg(" .. q(t.v) .. ")" end
  if t.k == "op" and t.v == "(" then
    local e = self:expr()
    self:expect(")")
    return "(" .. e .. ")"
  end
  if t.k == "op" and t.v == "[" then
    local items = {}
    if not self:accept("]") then
      repeat
        if self:is("]") then break end
        items[#items + 1] = self:expr()
      until not self:accept(",")
      self:expect("]")
    end
    return "{" .. table.concat(items, ", ") .. "}"
  end
  if t.k == "op" and (t.v == "{" or t.v == "#{") then
    -- a lambda {a, b -> expr} or a dict {'k': v} (#{k: v}: keys as words)
    if t.v == "{" then
      local j, depth = self.i, 0
      local lambda = false
      while self.t[j].k ~= "eof" do
        local x = self.t[j]
        if x.k == "op" and (x.v == "{" or x.v == "#{" or x.v == "[" or x.v == "(") then depth = depth + 1 end
        if x.k == "op" and (x.v == "}" or x.v == "]" or x.v == ")") then
          if depth == 0 then break end
          depth = depth - 1
        end
        if depth == 0 and x.k == "op" and x.v == "->" then lambda = true break end
        if depth == 0 and x.k == "op" and x.v == ":" then break end
        j = j + 1
      end
      if lambda then
        local names = {}
        while not self:is("->") do
          local p = self:next()
          if p.k == "id" then names[#names + 1] = p.v end
          self:accept(",")
        end
        self:expect("->")
        local saved = self.ctx.params
        local params = setmetatable({}, { __index = saved })
        for _, n in ipairs(names) do params[n] = true end
        self.ctx.params = params
        local body = self:expr()
        self.ctx.params = saved
        self:expect("}")
        local plist = table.concat(vim.tbl_map(function(n) return "P_" .. n end, names), ", ")
        return "function(" .. plist .. ") return " .. body .. " end"
      end
    end
    local parts = {}
    local literal = t.v == "#{"
    if not self:accept("}") then
      repeat
        if self:is("}") then break end
        local k
        if literal then
          local kt = self:next()
          k = q(tostring(kt.v))
        else
          k = "V.str(" .. self:expr() .. ")"
        end
        self:expect(":")
        parts[#parts + 1] = "[" .. k .. "] = " .. self:expr()
      until not self:accept(",")
      self:expect("}")
    end
    return "V.dict({" .. table.concat(parts, ", ") .. "})"
  end
  if t.k == "id" then
    if self:is("(") and not self:peek().sp then
      self:next()
      return self:call(t.v, self:args())
    end
    if t.v == "v:true" then return "true" end
    if t.v == "v:false" then return "false" end
    local v = self:var(t.v)
    if not v then error("E121: Undefined variable: " .. t.v, 0) end
    return v
  end
  error("E15: Invalid expression: \"" .. self.src .. "\"", 0)
end
function P:postfix(e)
  while true do
    local t = self:peek()
    if t.k == "op" and t.v == "[" and not t.sp then
      self:next()
      local a, b
      if self:accept(":") then
        if not self:is("]") then b = self:expr() end
        self:expect("]")
        e = "V.slice(" .. e .. ", nil, " .. (b or "nil") .. ")"
      else
        a = self:expr()
        if self:accept(":") then
          if not self:is("]") then b = self:expr() end
          self:expect("]")
          e = "V.slice(" .. e .. ", " .. a .. ", " .. (b or "nil") .. ")"
        else
          self:expect("]")
          e = "V.idx(" .. e .. ", " .. a .. ")"
        end
      end
    elseif t.k == "op" and t.v == "." and not t.sp and self:peek(1).k == "id" and not self:peek(1).sp then
      self:next()
      local k = self:next().v
      e = "V.member(" .. e .. ", " .. q(k) .. ")"
    elseif t.k == "op" and t.v == "->" then
      self:next()
      local f = self:next()
      if f.k == "op" and f.v == "{" then
        -- ->{lambda}(args)
        self.i = self.i - 1
        local lam = self:primary()
        self:expect("(")
        local a = self:args()
        table.insert(a, 1, e)
        e = "(" .. lam .. ")(" .. table.concat(a, ", ") .. ")"
      else
        self:expect("(")
        e = self:call(f.v, self:args(), e)
      end
    elseif t.k == "op" and t.v == "(" and not t.sp then
      self:next()
      local a = self:args()
      e = "V.callv(" .. e .. (#a > 0 and (", " .. table.concat(a, ", ")) or "") .. ")"
    else
      return e
    end
  end
end

function M.expr_lua(s, ctx)
  local p = parser(s, ctx)
  local e = p:expr()
  if p:peek().k ~= "eof" then error("E15: Invalid expression: \"" .. s .. "\"", 0) end
  return e
end
-- several expressions in a row (echo a b, execute a b)
local function exprs(s, ctx)
  local p = parser(s, ctx)
  local list = {}
  while p:peek().k ~= "eof" do list[#list + 1] = p:expr() end
  return list
end

function V.exists(S, L, e)
  if type(e) ~= "string" then return 0 end
  local kind = e:sub(1, 1)
  if kind == "*" then return V.bool(V.fn(S, e:sub(2)) ~= nil or rawget(vim.fn, e:sub(2)) ~= nil) end
  local sc, name = e:match("^(%a):([%w_#]+)$")
  if sc == "s" then return V.bool(S[name] ~= nil) end
  if sc == "l" then return V.bool(L and L[name] ~= nil) end
  if not sc and e:match("^[%a_][%w_#]*$") then return V.bool((L and L[e] ~= nil) or vim.g[e] ~= nil) end
  return vim.fn.exists(e)
end
function V.exec(...) local s = V.join(" ", ...) vim.cmd(s) return "" end

-- ── statements ──
local KW = {
  { "let", "let" }, { "unl", "unlet" }, { "cons", "const" }, { "if", "if" }, { "elsei", "elseif" }, { "el", "else" }, { "en", "endif" },
  { "wh", "while" }, { "endw", "endwhile" }, { "for", "for" }, { "endfo", "endfor" }, { "brea", "break" }, { "con", "continue" },
  { "fu", "function" }, { "endf", "endfunction" }, { "retu", "return" }, { "cal", "call" }, { "try", "try" }, { "cat", "catch" },
  { "fina", "finally" }, { "endt", "endtry" }, { "th", "throw" }, { "ec", "echo" }, { "echon", "echon" }, { "echom", "echomsg" },
  { "echoe", "echoerr" }, { "echoh", "echohl" }, { "exe", "execute" }, { "fini", "finish" }, { "lockv", "lockvar" },
  { "unlo", "unlockvar" }, { "delf", "delfunction" }, { "defe", "defer" }, { "se", "set" }, { "setl", "setlocal" }, { "setg", "setglobal" },
}
-- the command a line starts with, by Vim's shortest-form rules (only the language's own commands)
local function keyword(line)
  local word = line:match("^%s*([%a]+)")
  if not word then return nil end
  for _, k in ipairs(KW) do
    local short, full = k[1], k[2]
    if #word >= #short and full:sub(1, #word) == word then
      -- :echo vs :echon/:echom…, :fu vs :function, :en vs :endif: the longest match wins
      local best = full
      for _, k2 in ipairs(KW) do if k2[2] == word then best = k2[2] end end
      return best, line:match("^%s*[%a]+!?(.*)$"), line:match("^%s*[%a]+(!)") ~= nil
    end
  end
  return nil
end
M.keyword = keyword
function M.handles(line)
  local k = keyword(line)
  return k ~= nil and k ~= "set" and k ~= "setlocal" and k ~= "setglobal"
end

-- split a line at | (outside strings), the way Vim does for these commands
local function bars(line)
  local out, cur, i, quote = {}, {}, 1, nil
  while i <= #line do
    local c = line:sub(i, i)
    if quote then
      cur[#cur + 1] = c
      if c == "\\" and quote == '"' then cur[#cur + 1] = line:sub(i + 1, i + 1) i = i + 1
      elseif c == quote then quote = nil end
    elseif c == "'" or c == '"' then
      -- a " after a command word starts a comment: only within an expression is it a string
      quote = c
      cur[#cur + 1] = c
    elseif c == "|" and line:sub(i + 1, i + 1) ~= "|" and line:sub(i - 1, i - 1) ~= "|" then
      out[#out + 1] = table.concat(cur)
      cur = {}
    else
      cur[#cur + 1] = c
    end
    i = i + 1
  end
  out[#out + 1] = table.concat(cur)
  return out
end

-- the lines of a script: continuation lines joined, comments and blank lines left out
local function script_lines(src)
  local raw = {}
  for l in (src .. "\n"):gmatch("([^\n]*)\n") do raw[#raw + 1] = l end
  local out = {}
  for _, l in ipairs(raw) do
    local cont = l:match("^%s*\\(.*)$")
    if cont and #out > 0 then out[#out] = out[#out] .. cont
    elseif l:match("^%s*\"") or l:match("^%s*$") then -- comment / blank
    else out[#out + 1] = l end
  end
  local final = {}
  for _, l in ipairs(out) do
    if keyword(l) and keyword(l) ~= "set" then
      for _, part in ipairs(bars(l)) do if not part:match("^%s*$") then final[#final + 1] = part end end
    else
      final[#final + 1] = l
    end
  end
  return final
end

-- an assignment's left side: its Lua place, or a setter for indexes and members
local function target(s, ctx)
  local p = parser(s, ctx)
  local t = p:next()
  local place
  if t.k == "opt" then
    place = { kind = "opt", lua = (t.v.scope == "l" and "vim.bo" or "vim.o") .. "[" .. q(t.v.name) .. "]" }
  elseif t.k == "env" then
    place = { kind = "var", lua = "vim.env[" .. q(t.v) .. "]" }
  elseif t.k == "reg" then
    place = { kind = "reg" }
  elseif t.k == "id" then
    place = { kind = "var", lua = p:var(t.v), name = t.v }
  else
    error("E461: Illegal variable name: " .. s, 0)
  end
  -- x[i] / x.k / x[a][b]: the last index is set on what comes before
  while true do
    local n = p:peek()
    if n.k == "op" and n.v == "[" then
      p:next()
      local idx = p:expr()
      p:expect("]")
      place = { kind = "idx", obj = place.kind == "idx" and ("V.idx(" .. place.obj .. ", " .. place.idx .. ")") or place.lua, idx = idx }
    elseif n.k == "op" and n.v == "." and not n.sp and p:peek(1).k == "id" then
      p:next()
      local k = p:next().v
      place = { kind = "idx", obj = place.kind == "idx" and ("V.idx(" .. place.obj .. ", " .. place.idx .. ")") or place.lua, idx = q(k) }
    else
      break
    end
  end
  return place, p
end
local function read(place)
  if place.kind == "idx" then return "V.idx(" .. place.obj .. ", " .. place.idx .. ")" end
  if place.kind == "opt" then return "V.opt(" .. place.lua .. ")" end
  if place.kind == "reg" then return "\"\"" end
  return place.lua
end
local function write(place, value)
  if place.kind == "idx" then return "V.setidx(" .. place.obj .. ", " .. place.idx .. ", " .. value .. ")" end
  if place.kind == "reg" then return "V.discard(" .. value .. ")" end
  return place.lua .. " = " .. value
end
local COMPOUND = { ["+="] = "V.add", ["-="] = "V.sub", ["*="] = "V.mul", ["/="] = "V.div", ["%="] = "V.mod", [".="] = "V.cat", ["..="] = "V.cat" }

function M.translate(src)
  local out = { "local V, S, M = ...", "local A, L = {}, {}" }
  local stack = {}
  local n = 0
  local ctx = { fn = false }
  local function emit(s) out[#out + 1] = s end
  local function uid() n = n + 1 return n end
  local function in_try_in_fn()
    for i = #stack, 1, -1 do
      if stack[i].kind == "function" then return false end
      if stack[i].kind == "try" then return true end
    end
    return false
  end
  local function loop_top()
    for i = #stack, 1, -1 do
      if stack[i].kind == "while" or stack[i].kind == "for" then return stack[i] end
      if stack[i].kind == "function" then return nil end
    end
  end
  for _, line in ipairs(script_lines(src)) do
    local kw, rest, bang = keyword(line)
    rest = rest and rest:gsub("^%s+", "") or ""
    if kw == "let" or kw == "const" then
      local lhs, op, rhs
      if rest:match("^%[") then
        -- let [a, b] = list
        local names, after = rest:match("^%[(.-)%]%s*(.*)$")
        local value = after:match("^=%s*(.*)$")
        local tmp = "__l" .. uid()
        emit("do local " .. tmp .. " = V.list(" .. M.expr_lua(value, ctx) .. ")")
        local k = 0
        for name in names:gmatch("[^,%s]+") do
          k = k + 1
          if name:sub(1, 1) == ";" then break end
          local place = target(name, ctx)
          if ctx.fn and place.name and not place.name:match(":") then ctx.locals[place.name] = true end
          emit(write(place, tmp .. "[" .. k .. "]"))
        end
        emit("end")
      else
        -- the place, the operator, the value
        local place, p = target(rest, ctx)
        local t = p:next()
        op = t.v
        rhs = rest
        local consumed = 0
        for i = 1, p.i - 1 do consumed = consumed + 1 end
        -- the value is the text after the operator
        local at = rest:find(vim.pesc(op), 1)
        local after = rest
        -- find the operator's position by re-lexing up to it
        local depth, pos = 0, nil
        local i = 1
        while i <= #rest do
          local c = rest:sub(i, i)
          if c == "[" or c == "(" or c == "{" then depth = depth + 1 elseif c == "]" or c == ")" or c == "}" then depth = depth - 1 end
          if depth == 0 and rest:sub(i, i + #op - 1) == op and (op ~= "=" or not rest:sub(i - 1, i - 1):match("[=!<>~+%-*/%%.]")) then pos = i break end
          i = i + 1
        end
        if not pos then error("E15: Invalid expression: let " .. rest, 0) end
        after = rest:sub(pos + #op)
        local value = M.expr_lua(after, ctx)
        if ctx.fn and place.name and not place.name:match(":") then ctx.locals[place.name] = true end
        if COMPOUND[op] then emit(write(place, COMPOUND[op] .. "(" .. read(place) .. ", " .. value .. ")"))
        elseif op == "=" then emit(write(place, value))
        else error("E15: Invalid expression: let " .. rest, 0) end
      end
    elseif kw == "unlet" then
      for name in rest:gmatch("%S+") do
        local place = target(name, ctx)
        if place.kind == "idx" then emit("V.delidx(" .. place.obj .. ", " .. place.idx .. ")") else emit(write(place, "nil")) end
      end
    elseif kw == "if" then
      emit("if V.t(" .. M.expr_lua(rest, ctx) .. ") then")
      stack[#stack + 1] = { kind = "if" }
    elseif kw == "elseif" then
      emit("elseif V.t(" .. M.expr_lua(rest, ctx) .. ") then")
    elseif kw == "else" then
      emit("else")
    elseif kw == "endif" then
      table.remove(stack)
      emit("end")
    elseif kw == "while" then
      local id = uid()
      emit("while V.t(" .. M.expr_lua(rest, ctx) .. ") do")
      stack[#stack + 1] = { kind = "while", id = id }
    elseif kw == "endwhile" or kw == "endfor" then
      local top = table.remove(stack)
      emit("::cont" .. top.id .. ":: end")
    elseif kw == "for" then
      local vars, list = rest:match("^(.-)%s+in%s+(.*)$")
      local id = uid()
      emit("for _, __v" .. id .. " in ipairs({ table.unpack(V.list(" .. M.expr_lua(list, ctx) .. ")) }) do")
      if vars:match("^%[") then
        local k = 0
        for name in vars:gmatch("[%w_:]+") do
          k = k + 1
          local place = target(name, ctx)
          if ctx.fn and not name:match(":") then ctx.locals[name] = true end
          emit(write(place, "__v" .. id .. "[" .. k .. "]"))
        end
      else
        local place = target(vars, ctx)
        if ctx.fn and not vars:match(":") then ctx.locals[vars] = true end
        emit(write(place, "__v" .. id))
      end
      stack[#stack + 1] = { kind = "for", id = id }
    elseif kw == "break" then
      emit("break")
    elseif kw == "continue" then
      local top = loop_top()
      if not top then error("E586: :continue without :while or :for", 0) end
      emit("goto cont" .. top.id)
    elseif kw == "function" then
      local name, params, flags = rest:match("^([%w_:#<>.]+)%s*%((.-)%)%s*(.*)$")
      if not name then
        -- :function alone lists them; :function Name shows one: what zett can show
        emit("V.echo(' ', table.concat(vim.tbl_keys(vim._vimfn), ' '))")
      else
        local names = {}
        local varargs = false
        for pname in params:gmatch("[^,%s]+") do
          if pname == "..." then varargs = true else names[#names + 1] = pname:gsub("=.*$", "") end
        end
        local list = "{" .. table.concat(vim.tbl_map(q, names), ", ") .. "}"
        emit("V.def(S, " .. q(name) .. ", function(...)")
        emit("local A = V.args(" .. list .. ", ...)")
        emit("local L = {}")
        if flags:find("dict") then emit("L.self = A.self") end
        -- the body runs inside pcall, so what :defer asked runs at the end, error or not
        emit("local __defer = {}")
        emit("local __r = table.pack(pcall(function()")
        stack[#stack + 1] = { kind = "function", saved = ctx }
        ctx = { fn = true, locals = {}, params = nil }
        stack[#stack].bang = bang
      end
    elseif kw == "endfunction" then
      local top = table.remove(stack)
      ctx = top.saved
      emit("return 0")
      emit("end)) for __i = #__defer, 1, -1 do pcall(__defer[__i]) end if not __r[1] then error(__r[2], 0) end return __r[2] end, true)")
    elseif kw == "return" then
      local value = rest ~= "" and M.expr_lua(rest, ctx) or "0"
      if in_try_in_fn() then emit("do return V.ret(" .. value .. ") end") else emit("do return " .. value .. " end") end
    elseif kw == "call" then
      emit("V.discard(" .. M.expr_lua(rest, ctx) .. ")")
    elseif kw == "try" then
      local id = uid()
      emit("do local __ok" .. id .. ", __err" .. id .. " = pcall(function()")
      stack[#stack + 1] = { kind = "try", id = id, phase = "body", catches = 0 }
    elseif kw == "catch" then
      local top = stack[#stack]
      local pat = rest:match("^/(.*)/$") or (rest ~= "" and rest or nil)
      if top.phase == "body" then
        emit("end)")
        emit("local __h" .. top.id .. " = __ok" .. top.id)
        emit("if not __ok" .. top.id .. " and not V.isret(__err" .. top.id .. ") then local __exc = V.exc(__err" .. top.id .. ") vim.v.exception = __exc")
        emit("if V.catch(__exc, " .. (pat and q(pat) or "nil") .. ") then __h" .. top.id .. " = true")
      else
        emit("elseif V.catch(__exc, " .. (pat and q(pat) or "nil") .. ") then __h" .. top.id .. " = true")
      end
      top.phase = "catch"
      top.catches = top.catches + 1
    elseif kw == "finally" then
      local top = stack[#stack]
      if top.phase == "body" then
        emit("end)")
        emit("local __h" .. top.id .. " = __ok" .. top.id)
      else
        emit("end end")
      end
      top.phase = "finally"
    elseif kw == "endtry" then
      local top = table.remove(stack)
      if top.phase == "body" then
        emit("end)")
        emit("local __h" .. top.id .. " = __ok" .. top.id)
      elseif top.phase == "catch" then
        emit("end end")
      end
      emit("if not __h" .. top.id .. " then error(__err" .. top.id .. ", 0) end")
      if ctx.fn then
        if in_try_in_fn() then emit("if V.isret(__err" .. top.id .. ") then return __err" .. top.id .. " end")
        else emit("if V.isret(__err" .. top.id .. ") then return __err" .. top.id .. ".v end") end
      end
      emit("end")
    elseif kw == "throw" then
      emit("error(V.throw(" .. M.expr_lua(rest, ctx) .. "), 0)")
    elseif kw == "echo" or kw == "echon" or kw == "echomsg" then
      local list = exprs(rest, ctx)
      emit("V.echo(" .. q(kw == "echon" and "" or " ") .. (#list > 0 and (", " .. table.concat(list, ", ")) or "") .. ")")
    elseif kw == "echoerr" then
      emit("V.echoerr(" .. table.concat(exprs(rest, ctx), ", ") .. ")")
    elseif kw == "echohl" or kw == "lockvar" or kw == "unlockvar" then
      -- colours of :echo and locked variables: nothing to do
    elseif kw == "execute" then
      emit("V.exec(" .. table.concat(exprs(rest, ctx), ", ") .. ")")
    elseif kw == "finish" then
      emit("do return end")
    elseif kw == "defer" then
      if not ctx.fn then error("E1287: :defer can only be used in a function", 0) end
      emit("__defer[#__defer + 1] = function() V.discard(" .. M.expr_lua(rest, ctx) .. ") end")
    elseif kw == "delfunction" then
      emit("V.undef(S, " .. q(rest) .. ")")
    elseif kw == "set" or kw == "setlocal" or kw == "setglobal" then
      if rest == "" or rest:match("^all") then emit("vim.cmd(" .. q(line) .. ")")
      else emit("V.set(" .. q(kw == "setlocal" and "local" or "global") .. ", " .. q(rest) .. ")") end
    else
      emit("vim.cmd(V.sid(S, " .. q((line:gsub("^%s*:*", ""))) .. "))")
    end
  end
  if #stack > 0 then error("E171: Missing :end" .. (stack[#stack].kind == "if" and "if" or stack[#stack].kind), 0) end
  return table.concat(out, "\n")
end

-- the script-local variables (s:) of each file, kept between runs of it
M.scripts = M.scripts or {}
function M.run(src, name)
  local ok, code = pcall(M.translate, src)
  if not ok then zett.notify(tostring(code)) return false, code end
  local chunk, err = load(code, "=" .. (name or "vimscript"), "t")
  if not chunk then zett.notify("E: " .. err) return false, err end
  local S = M.scripts[name or ""]
  if not S then
    M.sids = (M.sids or 0) + 1
    S = { __sid = name and M.sids or nil }
    M.scripts[name or ""] = S
  end
  local ran, e = pcall(chunk, V, S, M)
  if not ran then
    local msg = V.exc(e)
    zett.notify(msg)
    return false, msg
  end
  return true
end
function M.source(path)
  local f = io.open(path, "r")
  if not f then zett.notify("E484: Can't open file " .. path) return false end
  local src = f:read("a")
  f:close()
  return M.run(src, path)
end
function M.eval(expr)
  local code = M.expr_lua(expr, { top = true })
  local chunk = assert(load("local V, S, M = ... local A, L = {}, {} return " .. code, "=eval", "t"))
  return chunk(V, M.scripts[""] or {}, M)
end
-- what :echo shows for an expression typed on the : line
function M.echo(expr)
  local ok, e = pcall(function()
    local list = exprs(expr, { top = true })
    local code = "local V, S, M = ... local A, L = {}, {} return V.join(' ', " .. table.concat(list, ", ") .. ")"
    return assert(load(code, "=echo", "t"))(V, M.scripts[""] or {}, M)
  end)
  return ok, e
end
M.lua_pattern = vim._lua_pattern
-- the text :echo shows, or an error (the : line shows that instead)
function M.echo_text(expr)
  local ok, e = M.echo(expr)
  if not ok then error(V.exc(e), 0) end
  return e
end
-- foldmethod=expr: each line's level from foldexpr (v:lnum the line), as "level[:start],…" for
-- the editor; Vim's answers: N, "=" (as the line before), "aN" "sN" (more, less), ">N" (a fold
-- starts), "<N" (one ends here), "-1" (as the line before). v:lua.f() calls Lua's f.
-- a completion function (completefunc, omnifunc, thesaurusfunc: a function's name or v:lua.f)
-- called as Vim calls it: findstart 1 gives the start column (bytes, -2 / -3 cancel); 0 the
-- matches, one "word\tabbr\tmenu\tkind" line each (separated by \30)
function M.complete_call(name, findstart, base)
  local f
  local lua = name:match("^v:lua%.(.+)$")
  if lua then
    local chunk = load("return " .. lua, "=completefunc", "t")
    local ok, v = pcall(chunk or function() end)
    f = ok and v or nil
  else
    f = function(a, b) return vim.fn[name](a, b) end
  end
  if type(f) ~= "function" then return "" end
  local ok, r = pcall(f, findstart, base)
  if not ok or r == nil then return "" end
  if findstart == 1 then return tostring(math.tointeger(tonumber(r) or -3) or -3) end
  if type(r) == "table" and r.words then r = r.words end
  local out = {}
  for _, it in ipairs(type(r) == "table" and r or {}) do
    if type(it) == "table" then
      out[#out + 1] = table.concat({ tostring(it.word or ""), tostring(it.abbr or ""), tostring(it.menu or ""), tostring(it.kind or "") }, "\t")
    else
      out[#out + 1] = tostring(it) .. "\t\t\t"
    end
  end
  return table.concat(out, "\30")
end

-- an expression in a format (%{expr}, statuscolumn's %!expr): Vimscript, as Neovim reads it;
-- Lua when it is not Vimscript (zett's own formats took Lua before)
function M.eval_str(expr)
  local lua = expr:match("^%s*v:lua%.(.-)%s*$")
  local ok, code
  if lua then ok, code = true, lua else ok, code = pcall(M.expr_lua, expr, { top = true }) end
  local f
  if ok then
    local chunk = load("local V, S, M = ... local A, L = {}, {} return function() return " .. code .. " end", "=expr", "t")
    f = chunk and chunk(V, M.scripts[""] or {}, M)
  end
  f = f or load("return " .. expr, "=expr", "t")
  if not f then return "" end
  local ok2, r = pcall(f)
  if not ok2 or r == nil then return "" end
  if type(r) == "number" then r = math.tointeger(r) or r end
  if type(r) == "boolean" then r = r and 1 or 0 end
  return tostring(r)
end

-- foldtext for the closed fold at lines s..e (level lv): its text, or "" when it gives none
function M.fold_text(expr, s, e, lv)
  local code = expr:match("^v:lua%.(.*)$") or M.expr_lua(expr, { top = true })
  local ok, f = pcall(function()
    return assert(load("local V, S, M = ... local A, L = {}, {} return function() return " .. code .. " end", "=foldtext", "t"))(V, M.scripts[""] or {}, M)
  end)
  if not ok then return "" end
  vim.v.foldstart, vim.v.foldend, vim.v.foldlevel, vim.v.folddashes = s, e, lv, string.rep("-", lv)
  local ok2, r = pcall(f)
  if not ok2 or r == nil then return "" end
  -- a list of [text, group] chunks (Neovim): their texts
  if type(r) == "table" then
    local t = {}
    for _, c in ipairs(r) do t[#t + 1] = type(c) == "table" and tostring(c[1] or "") or tostring(c) end
    return table.concat(t)
  end
  return tostring(r)
end

function M.fold_levels(expr, n)
  local code = expr:match("^v:lua%.(.*)$") or M.expr_lua(expr, { top = true })
  local f = assert(load("local V, S, M = ... local A, L = {}, {} return function() return " .. code .. " end", "=foldexpr", "t"))(V, M.scripts[""] or {}, M)
  local out, prev, after = {}, 0, nil
  for l = 1, n do
    vim.v.lnum = l
    local ok, r = pcall(f)
    local s = ok and (type(r) == "number" and tostring(math.tointeger(r) or math.floor(r)) or tostring(r)) or "0"
    local base = after or prev
    after = nil
    local lv, start
    if s == "=" or s == "-1" then lv = base
    elseif s:match("^a%d+$") then lv = base + tonumber(s:sub(2))
    elseif s:match("^s%d+$") then lv = math.max(base - tonumber(s:sub(2)), 0)
    elseif s:match("^>%d+$") then lv = tonumber(s:sub(2)) start = lv
    elseif s:match("^<%d+$") then lv = tonumber(s:sub(2)) after = math.max(lv - 1, 0)
    else lv = math.max(math.tointeger(tonumber(s) or 0) or 0, 0) end
    prev = lv
    out[#out + 1] = start and (lv .. ":" .. start) or tostring(lv)
  end
  return table.concat(out, ",")
end
