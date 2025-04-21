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

--- @param pattern string
--- @return string[]
function seekr:glob(pattern) end
