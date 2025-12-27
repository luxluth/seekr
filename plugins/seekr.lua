---@meta

--- @class SeekrGlobal
--- @field config_dir string
seekr = {}

--- @param key string
--- @return string
function seekr:env(key) end

--- @param plugin_name string
--- @param message string
function seekr:log(plugin_name, message) end

--- @param plugin_name string
function seekr:clear_results(plugin_name) end

--- @param plugin_name string
--- @param images string[]
--- @param subtitle string|nil
function seekr:show_image_grid(plugin_name, images, subtitle) end

--- @param plugin_name string
--- @param title string
--- @param body string
function seekr:show_info_box(plugin_name, title, body) end

--- @param plugin_name string
--- @param command string
function seekr:show_console(plugin_name, command) end

--- @param pattern string
--- @return string[]
function seekr:glob(pattern) end

--- @param cmd string
function seekr:exec(cmd) end

--- @param cmd string
--- @return string
function seekr:read(cmd) end

--- @param json string
--- @return table|nil
function seekr:json_to_lua(json) end

function seekr:close() end
