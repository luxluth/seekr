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
