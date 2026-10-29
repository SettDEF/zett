-- LspInstall: offers to install a missing language server; LspInstallStatus lists them; a hint on a filetype's first open.
if vim.g.loaded_lsp_install then return end
vim.g.loaded_lsp_install = true

vim.api.nvim_create_user_command("LspInstall", function(a) require("lsp-install").install(a.args) end, { nargs = "?", desc = "Install the language server for a filetype (default: this buffer's)" })
vim.api.nvim_create_user_command("LspInstallStatus", function() require("lsp-install").status() end, { desc = "Which language servers are installed" })

vim.api.nvim_create_autocmd("FileType", {
  group = vim.api.nvim_create_augroup("lsp-install", { clear = true }),
  callback = function(a) require("lsp-install").hint(a.match) end,
})
