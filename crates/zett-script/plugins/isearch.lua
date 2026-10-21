-- isearch: Emacs's incremental search (C-s / C-r). The search itself is the editor's (the
-- isearch-* commands); this plugin is the keys: C-s / C-r start it, and while it runs the
-- "isearch" keymap is used. A printable key with no binding there types into the query; any
-- other unbound key ends the search and then does what it always does.
-- Settings: zett.opt.isearch = { start = { ["<C-s>"] = "isearch-forward" }, keys = { ... } }

local z = zett
local defaults = {
  -- the keys that start a search, in the Emacs keymap
  start = {
    ["<C-s>"] = "isearch-forward",
    ["<C-r>"] = "isearch-backward",
    ["<C-M-s>"] = "isearch-forward-regexp",
    ["<C-M-r>"] = "isearch-backward-regexp",
    ["<M-s>w"] = "isearch-forward-word",
    ["<M-s>_"] = "isearch-forward-symbol",
    ["<M-s><M-.>"] = "isearch-forward-thing-at-point",
  },
  -- the keys while it runs (Emacs's isearch-mode-map)
  keys = {
    ["<C-s>"] = "isearch-repeat-forward",
    ["<C-r>"] = "isearch-repeat-backward",
    ["<BS>"] = "isearch-delete-char",
    ["<CR>"] = "isearch-exit",
    ["<C-g>"] = "isearch-abort",
    ["<M-r>"] = "isearch-toggle-regexp",
    ["<M-s>w"] = "isearch-toggle-word",
    ["<M-s>_"] = "isearch-toggle-symbol",
  },
}

z.options("isearch", defaults, {
  start = "keys that start a search (Emacs keymap): key = command",
  keys = "keys while a search runs: key = command",
})

-- applied once init.lua has run, so zett.opt.isearch from there counts; zett.opt.isearch = false
-- leaves C-s / C-r to the line search
z.on("start", function()
  local cfg = z.config("isearch", defaults)
  if not cfg then return end
  for key, cmd in pairs(cfg.start) do
    z.map("e", key, cmd)
  end
  for key, cmd in pairs(cfg.keys) do
    z.map("isearch", key, cmd)
  end
end)
-- the line search that C-s opened before: Emacs's occur key
z.map("e", "<M-s>o", "search-buffer", { desc = "Lines matching, in a list" })
