# CarbonEditor

CarbonEditor is an egui text editor and a sister library to [CarbonUI](../carbonui). It is a standalone crate: CarbonUI is not a dependency. v1 edits a buffer, wraps long lines, and keeps a project of open files. Text is set in [Lyth Mono](https://github.com/why-trv/LythMono). Dark and light themes restyle the editor and the stock egui widgets around it.

## Run the gallery

```bash
cargo run -p carboneditor-gallery
```

The same binary builds for the browser. From this directory, with [Trunk](https://trunkrs.dev/) installed:

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
trunk serve
```

`trunk serve` from `crates/carboneditor-gallery` is the same app.

Trunk serves the gallery at <http://127.0.0.1:8735>.

## Use it

```rust
carboneditor::Theme::dark().install(&ctx);

let mut project = carboneditor::Project::from_dir("path/to/project")?;
project.open_file("src/main.rs")?;

let wrap = true;
if let Some(document) = project.active_mut() {
    let path = document.path().to_owned();
    carboneditor::Editor::new(document.buffer_mut())
        .path(&path)
        .wrap(wrap)
        .show(ui);
}
```

`Theme::light()` is the paper theme. `Editor::wrap(false)` leaves long lines on one row and scrolls horizontally. `Project` is the project context: the indexed text files, the buffers open from them, and the directory they belong to. `Project::memory` is the same API without a filesystem.

The gallery is one editor on a sample buffer. Word wrap and the light/dark toggle sit in the top bar. The same screen is what Trunk serves.

## Layout

- `crates/carboneditor` — the library. Depends only on egui.
- `crates/carboneditor-gallery` — the shell, native and wasm.

## Font

Lyth Mono Regular v0.10.0 is vendored from the [v0.10.0 release](https://github.com/why-trv/LythMono/releases/tag/v0.10.0). The face is unmodified. It is licensed under the SIL Open Font License; see `crates/carboneditor/fonts/OFL.txt`.
# carboneditor
