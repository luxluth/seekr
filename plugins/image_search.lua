--- @meta '../seekr.lua'
--- @diagnostic disable: undefined-global

local ImageSearch = {
	name = "image_search",
	title = "Images",
	description = "Quick lookup of your images",
	api_version = 1,
	index = {},
}

local extensions = { ".png", ".jpeg", ".jpg", ".jxl", ".tif", ".gif" }
--- @type string[]
local index = {}

--- @param path string
--- @return boolean
local function is_image_path(path)
	local ok = false

	for _, ext in pairs(extensions) do
		if string.sub(path, -#ext) == ext then
			ok = true
			break
		end
	end

	return ok
end

--- @param str string
local function expand_env_vars(str)
	return str:gsub("%$(%w+)", function(var)
		return seekr:env(var)
	end)
end

--- @param text string
function ImageSearch.onInput(text)
	seekr:log(ImageSearch.name, "recieved '" .. text .. "'")
end

function ImageSearch.onExit() end

function ImageSearch.onStartup()
	local config_dir = seekr:env("XDG_CONFIG_HOME")
	if config_dir:len() == 0 then
		seekr:log(ImageSearch.name, "Unable to find the config dir")
	else
		seekr:log(ImageSearch.name, "user-dirs -> " .. config_dir .. "/user-dirs.dirs")

		local file = io.open(config_dir .. "/user-dirs.dirs", "r")
		local pattern = "XDG_PICTURES_DIR="

		if file then
			for line in file:lines("l") do
				if string.sub(line, 1, #pattern) == pattern then
					local input, _ = string.sub(line, #pattern + 1):gsub('"', "")
					local pictures_dir, _ = expand_env_vars(input)
					for _, path in pairs(seekr:glob(pictures_dir .. "/**/*")) do
						if is_image_path(path) then
							table.insert(index, path)
						end
					end
					ImageSearch.index = index
					seekr:log(ImageSearch.name, "Collected " .. tostring(#ImageSearch.index) .. " image(s)")
				end
			end

			file:close()
		end
	end
end

return ImageSearch
