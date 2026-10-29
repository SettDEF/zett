-- Install missing language servers: nothing runs until the user picks Install.
local M = {}

local INFO, WARN, ERR = vim.log.levels.INFO, vim.log.levels.WARN, vim.log.levels.ERROR

-- one entry per server: its filetypes, its executable and its install argv
local servers = {
  { fts = { "lua" }, exe = "lua-language-server", argv = { "sudo", "pacman", "-S", "--needed", "lua-language-server" } },
  { fts = { "qml" }, exe = "qmlls", argv = { "sudo", "pacman", "-S", "--needed", "qt6-declarative" } },
  { fts = { "python" }, exe = "pyright-langserver", argv = { "npm", "install", "-g", "pyright" } },
  { fts = { "sh", "bash", "zsh" }, exe = "bash-language-server", argv = { "npm", "install", "-g", "bash-language-server" } },
  { fts = { "go" }, exe = "gopls", argv = { "go", "install", "golang.org/x/tools/gopls@latest" } },
  { fts = { "rust" }, exe = "rust-analyzer", argv = { "rustup", "component", "add", "rust-analyzer" } },
  { fts = { "c", "cpp" }, exe = "clangd", argv = { "sudo", "pacman", "-S", "--needed", "clang" } },
  { fts = { "javascript", "typescript" }, exe = "typescript-language-server", argv = { "npm", "install", "-g", "typescript-language-server", "typescript" } },
  { fts = { "json" }, exe = "vscode-json-language-server", argv = { "npm", "install", "-g", "vscode-langservers-extracted" } },
}

local by_ft = {}
for _, s in ipairs(servers) do
  for _, ft in ipairs(s.fts) do by_ft[ft] = s end
end

local hinted = {}

local function cmd(s) return table.concat(s.argv, " ") end

-- the last lines of a failed command's output
local function tail(text)
  local lines = vim.split(vim.trim(text or ""), "\n", { plain = true })
  return table.concat(vim.list_slice(lines, math.max(1, #lines - 2)), "\n")
end

function M.install(ft)
  if not ft or ft == "" then ft = vim.bo.filetype end
  local s = by_ft[ft]
  if not s then return vim.notify("LspInstall: no known language server for " .. (ft == "" and "this buffer" or ft), WARN) end
  if vim.fn.executable(s.exe) == 1 then return vim.notify(s.exe .. " is already installed", INFO) end
  local line = cmd(s)
  vim.ui.select({ "Install", "Cancel" }, { prompt = ("%s is missing. Install runs: %s"):format(s.exe, line) }, function(choice)
    if choice ~= "Install" then return end
    -- sudo cannot ask for a password inside the editor: hand the command over instead
    if s.argv[1] == "sudo" then
      pcall(vim.fn.setreg, "+", line)
      return vim.notify("run in a terminal: " .. line .. " (copied to the + register)", WARN)
    end
    local ok, err = pcall(vim.system, s.argv, { text = true }, function(r)
      vim.schedule(function()
        if r.code == 0 then
          vim.notify(s.exe .. " installed", INFO)
        else
          vim.notify(("%s install failed (exit %d):\n%s"):format(s.exe, r.code, tail(r.stderr ~= "" and r.stderr or r.stdout)), ERR)
        end
      end)
    end)
    if not ok then vim.notify(("could not start %s: %s"):format(s.argv[1], err), ERR) end
  end)
end

function M.status()
  local out = {}
  for _, s in ipairs(servers) do
    out[#out + 1] = vim.fn.executable(s.exe) == 1 and ("✓ " .. s.exe) or ("· %s  (install: %s)"):format(s.exe, cmd(s))
  end
  vim.notify(table.concat(out, "\n"), INFO)
end

-- once per filetype per session, and never with vim.g.lsp_install_no_hint
function M.hint(ft)
  if vim.g.lsp_install_no_hint or hinted[ft] then return end
  local s = by_ft[ft]
  if not s or vim.fn.executable(s.exe) == 1 then return end
  hinted[ft] = true
  vim.notify(s.exe .. " is not installed: :LspInstall sets it up", INFO)
end

return M
