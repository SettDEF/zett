-- Trail: :Trail toggles the panel, :TrailClear takes the marks off; every save is kept for it.
if vim.g.loaded_trail then return end
vim.g.loaded_trail = true

vim.api.nvim_create_user_command("Trail", function() require("trail").toggle() end, { desc = "The file's changes in a side panel" })
vim.api.nvim_create_user_command("TrailClear", function() require("trail").clear() end, { desc = "Take Trail's marks off the file" })

vim.api.nvim_create_autocmd("BufWritePost", {
  group = vim.api.nvim_create_augroup("trail", { clear = true }),
  callback = function(a)
    local path = vim.fn.fnamemodify(a.file, ":p")
    if path == "" or path:find("^%a+://") then return end
    require("trail").snapshot(path)
    -- the panel open: its list follows the save
    local t = require("trail")
    if package.loaded.trail then pcall(t.refresh) end
  end,
})

-- SPC g h (beside git status SPC g g and blame SPC g b); vim.g.trail_no_mappings = true leaves it out
if not vim.g.trail_no_mappings and vim.fn.maparg("<leader>gh", "n") == "" then
  vim.keymap.set("n", "<leader>gh", function() require("trail").toggle() end, { desc = "Trail: the file's history" })
end
