local ImageSearch = {
	name = "image_search",
	title = "Images",
	description = "Quick lookup of your images",
	api_version = 1,
}

--- @param text string
function ImageSearch.onInput(text)
	print(text)
end

function ImageSearch.onExit() end
function ImageSearch.onStartup()
	print("ImageSearch started")
end

return ImageSearch
