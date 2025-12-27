---@meta

--- @class SeekrGlobal
--- @field config_dir string The directory where seekr configuration and plugins are stored.
seekr = {}

--- @class Plugin
--- @field name string The name of the plugin.
--- @field api_version integer The version of the API the plugin uses.
--- @field title string|nil The title displayed in the UI for the plugin.
--- @field description string|nil A short description of the plugin.
--- @field triggers string[]|nil List of triggers that activate the plugin (e.g., "any", "command=/test", "contains=foo").
--- @field onInput (fun(input: string))|nil Callback triggered when the search term matches a trigger and changes.
--- @field onEnter (fun(input: string))|nil Callback triggered when the user presses Enter and the term matches a trigger.
--- @field onActivate (fun(payload: string))|nil Callback triggered when a result from this plugin is activated in the UI.
--- @field onStartup (fun())|nil Callback triggered when seekr starts and the plugin is loaded.
--- @field onExit (fun())|nil Callback triggered when seekr is closing.

--- Returns the value of an environment variable.
--- @param key string The name of the environment variable.
--- @return string The value of the environment variable or an empty string if not found.
function seekr:env(key) end

--- Logs a message to the seekr debug logs.
--- @param plugin_name string The name of the plugin logging the message.
--- @param message string The message to log.
function seekr:log(plugin_name, message) end

--- Clears the current results for the specified plugin in the UI.
--- @param plugin_name string The name of the plugin.
function seekr:clear_results(plugin_name) end

--- Displays a grid of images in the seekr UI.
--- @param plugin_name string The name of the plugin.
--- @param images string[] A list of paths or URLs to the images to display.
--- @param subtitle string|nil An optional subtitle to display for the grid.
function seekr:show_image_grid(plugin_name, images, subtitle) end

--- Displays an info box with a title and body text in the UI.
--- @param plugin_name string The name of the plugin.
--- @param title string The title of the info box.
--- @param body string The body text.
function seekr:show_info_box(plugin_name, title, body) end

--- Displays a console view that runs a command and shows its output.
--- @param plugin_name string The name of the plugin.
--- @param command string The shell command to execute in the console.
function seekr:show_console(plugin_name, command) end

--- Performs a glob search for files and directories.
--- @param pattern string The glob pattern (e.g., "/home/user/*.png").
--- @return string[] A list of matching file paths.
function seekr:glob(pattern) end

--- Executes a shell command asynchronously without capturing its output.
--- @param cmd string The shell command to execute.
function seekr:exec(cmd) end

--- Executes a shell command and returns its standard output.
--- @param cmd string The shell command to execute.
--- @return string The stdout of the command.
function seekr:read(cmd) end

--- Parses a JSON string into a Lua table.
--- @param json string The JSON string to parse.
--- @return table|nil The resulting Lua table, or nil if parsing failed.
function seekr:json_to_lua(json) end

--- Closes the seekr window.
function seekr:close() end

