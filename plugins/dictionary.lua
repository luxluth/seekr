--- @meta '../seekr.lua'
--- @diagnostic disable: undefined-global

--- @class Plugin
local Dictionary = {
	name = "dictionary",
	title = "Dictionary",
	description = "Define words",
	api_version = 1,
	triggers = { "command=/def", "command=/define" },
}

--- @param str string
--- @return string
local function escape_markup(str)
	return str:gsub("&", "&amp;"):gsub("<", "&lt;"):gsub(">", "&gt;")
end

--- @param text string
function Dictionary.onEnter(text)
	seekr:clear_results(Dictionary.name)

	-- Strip command
	local query = nil
	if text:sub(1, 7) == "/define" then
		query = text:sub(9)
	elseif text:sub(1, 4) == "/def" then
		query = text:sub(6)
	end

	if not query or query == "" then
		return
	end

	local safe_query = query:gsub("'", "'\\''")

	-- Try local dictd first
	local has_dict = seekr:read("which dict")
	if has_dict ~= "" then
		local command = "dict -d english '" .. safe_query .. "'"
		seekr:log(Dictionary.name, command)
		local output = seekr:read(command)
		-- dict returns "No definitions found for ..." on stderr usually, or stdout?
		-- output capture only captures stdout in seekr:read impl currently?
		-- If it captures stdout, and dict prints not found to stderr, output might be empty?
		-- Or it prints to stdout.
		if output ~= "" and not output:find("No definitions found") then
			seekr:show_info_box(Dictionary.name, query, escape_markup(output))
			return
		end
	end

	-- Fallback to online API
	local json = seekr:read("curl -s 'https://api.dictionaryapi.dev/api/v2/entries/en/" .. safe_query .. "'")

	if not json or json == "" then
		return
	end

	local data = seekr:json_to_lua(json)

	if data and #data > 0 then
		-- Navigate the structure: [ { "meanings": [ { "definitions": [ { "definition": "..." } ] } ] } ]
		local first_entry = data[1]
		if first_entry and first_entry.meanings and #first_entry.meanings > 0 then
			local first_meaning = first_entry.meanings[1]
			if first_meaning and first_meaning.definitions and #first_meaning.definitions > 0 then
				local def = first_meaning.definitions[1].definition
				if def then
					seekr:show_info_box(Dictionary.name, query, "<b>Definition:</b>\n" .. def)
					return
				end
			end
		end
	end

	if json:find("No Definitions Found") then
		seekr:show_info_box(Dictionary.name, query, "No definition found.")
	end
end

function Dictionary.onActivate(payload)
	-- This might be unused if InfoBox is not clickable, but keeping logic in case
	seekr:exec("xdg-open 'https://en.wiktionary.org/wiki/" .. payload .. "'")
end

function Dictionary.onExit() end
function Dictionary.onStartup() end

return Dictionary
