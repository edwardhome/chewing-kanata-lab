-- Chewing × Kanata Context Bridge for Neovim
-- 監聽 ModeChanged autocmd 並透過本機 IPC 回報編輯器模式給輸入法前端

local M = {}

local config = {
  enabled = true,
  socket_path = (vim.fn.has("win32") == 1) and "\\\\.\\pipe\\chewing_context_pipe" or "/tmp/chewing_context.sock",
  expires_after_ms = 1000,
  report_filetype = true,
}

local session_id = string.format("%08x-%04x", math.random(0, 0xFFFFFFFF), math.random(0, 0xFFFF))
local sequence_num = 0

local function normalize_vim_mode(mode)
  if mode == "n" or mode == "no" or mode == "nov" or mode == "noV" then
    return "normal"
  elseif mode == "v" or mode == "V" or mode == "\22" then
    return "visual"
  elseif mode == "i" or mode == "ic" or mode == "ix" then
    return "insert"
  elseif mode == "c" then
    local cmd_type = vim.fn.getcmdtype()
    if cmd_type == "/" or cmd_type == "?" then
      return "search"
    else
      return "command_line"
    end
  elseif mode == "t" then
    return "terminal"
  else
    return "unknown"
  end
end

local function send_context_event(raw_mode)
  if not config.enabled then return end

  sequence_num = sequence_num + 1
  local normalized = normalize_vim_mode(raw_mode)
  local ft = config.report_filetype and vim.bo.filetype or ""

  local payload = {
    schema_version = 1,
    session_id = session_id,
    sequence = sequence_num,
    timestamp_ms = vim.loop.now(),
    source = "neovim",
    editor_mode = normalized,
    filetype = ft,
    focused = (vim.fn.has("gui_running") == 1 or true),
    expires_after_ms = config.expires_after_ms
  }

  -- 透過 libuv 非同步傳送，不阻塞 Neovim 主線程
  local json_str = vim.fn.json_encode(payload)
  local client = vim.loop.new_pipe(false)
  client:connect(config.socket_path, function(err)
    if not err then
      client:write(json_str .. "\n", function()
        client:shutdown()
        client:close()
      end)
    else
      client:close()
    end
  end)
end

function M.setup(opts)
  config = vim.tbl_deep_extend("force", config, opts or {})
  if not config.enabled then return end

  local augroup = vim.api.nvim_create_augroup("ChewingContextBridge", { clear = true })

  vim.api.nvim_create_autocmd("ModeChanged", {
    group = augroup,
    callback = function()
      local mode = vim.api.nvim_get_mode().mode
      send_context_event(mode)
    end,
  })

  vim.api.nvim_create_autocmd("FocusGained", {
    group = augroup,
    callback = function()
      local mode = vim.api.nvim_get_mode().mode
      send_context_event(mode)
    end,
  })
end

return M
