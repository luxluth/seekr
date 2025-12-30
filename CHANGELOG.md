## v0.3.1

### Bug Fixes

- **Indexer**: Fixed a critical issue where the file search stopped working after reopening the application window due to the indexer thread being terminated prematurely.
- **Indexer**: Optimized resource management and performance by reusing a persistent `IndexReader` instead of recreating it for every search request.
- **Search**: Added comprehensive error logging for background search tasks to improve maintainability and debugging.

## v0.3.0

### Features

- **Native Indexer**: Replaced GNOME Tracker Miner dependency with a native [Tantivy](https://github.com/quickwit-oss/tantivy) based indexer. This significantly improves search performance and simplifies installation by removing external dependencies.
- **CI/CD**: Introduced GitHub Actions workflows for continuous integration and automated releases.

### Improvements & Bug Fixes

- **Indexer**: Added proper MIME type handling for directories (`inode/directory`), ensuring folders are correctly indexed and searchable.
- **Plugin System**:
  - Optimized plugin execution: prevented redundant global searches when a specific plugin is explicitly triggered (e.g., via prefix).
  - Robustness: Added fallback logic for the image search plugin when `XDG_CONFIG_HOME` is not set.
- **Documentation**:
  - Created `BUILDING.md` with comprehensive build and setup instructions.
  - Updated `README.md` and `CONTRIBUTING.md` to reflect the latest project state and architectural changes.

## v0.2.0

### Features

- **Lua Plugin System**: Introduced a powerful Lua-based plugin architecture allowing users to extend `seekr` with custom search providers and UI widgets.
- **Enhanced UI for Plugins**:
  - Added a high-performance `Image Grid` widget for plugins (used in image search).
  - Expanded the Plugin API with `onEnter` event handlers and `seekr:close()` for better interaction control.
- **Developer Experience**:
  - Added automatic generation of Lua stubs for better IDE support during plugin development.
  - Comprehensive documentation for the `seekr` Lua API.
- **UI Layout**: Documentation of the UI structure via a new `UI_LAYOUT.md` (ASCII art representation).

## v0.1.3

### Bug Fixes

- **Local Search**: Improved error handling and reliability in the local search module.
- **Configuration**: Removed `localsearch` from default features to streamline the default build.

## v0.1.2

- Sorting result for better matches
- Macros hint
- App style simplifications
- New css styles changes and attributes :
  - `completionLabel` - the macro hint label on the right side of the text entry
  - `completionBox` - the box surrounding the `completionLabel`.
  - `file`, `fileIcon`, `fileName`, `fileDetails` - equivalent style attributes
    for the file search result
- The completion box and the input box are overlayed. If the input box
  background isn't transparent, the completion box will not be visible
- Math result can now be copied
- Rust edition 2024
- Fixing the `freedesktop-desktop-entry` crate version to `v0.7.5` because the
  newer version gives descriptions as name.
- Searching ways to add lua plugins in the future

