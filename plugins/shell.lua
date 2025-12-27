--- @meta '../seekr.lua'

--- @class Plugin
local Shell = {
	name = "shell",
	title = "Shell",
	description = "Run shell commands",
	api_version = 1,
	triggers = { "command=/sh", "command=/exec", "command=$" },
}

--- @param text string
function Shell.onEnter(text)
	seekr:clear_results(Shell.name)

	-- Detect trigger and extract command
	local cmd = nil
	if text:sub(1, 4) == "/sh " then
		cmd = text:sub(5)
	elseif text:sub(1, 6) == "/exec " then
		cmd = text:sub(7)
	elseif text:sub(1, 1) == "$" then
		cmd = text:sub(2)
	end

	if not cmd or cmd == "" then
		return
	end

	-- We just want to execute it in the console widget
	seekr:show_console(Shell.name, cmd)
end

function Shell.onActivate(payload) end

function Shell.onStartup() end
function Shell.onExit() end

return Shell
