# faitout

A minimal, local-first note-taking app written in Rust with the Iced GUI toolkit. Notes support Markdown preview, tags, color labels, search, and multi-window viewing. Appearance (theme, font, size) is configurable.

## Features

- Notebook list with search-by-title and per-note color labels
- Create/edit notes with live Markdown preview or split editor+preview
- Open a note in a separate window
- Persist notes to [notes.json](notes.json)
- Persist appearance settings to [settings.json](settings.json)

## Quick start

Prerequisites:
- Rust and Cargo (stable recommended)
- Windows, macOS, or Linux. On Windows, MSVC toolchain is recommended.

Run in debug:
```sh
cargo run
```

Build release:
```sh
cargo build --release
```

The app stores data next to the executable/workspace:
- Notes: [notes.json](notes.json)
- Settings: [settings.json](settings.json)

Build with embedded icon (Windows):
```sh
cargo build --release --features embed-icon
```

Note: At runtime, the app also attempts to load an icon from `assets/icon.ico` (or falls back to `assets/icon.png`) via [`crate::load_app_icon`](src/main.rs).

## Data format

Notes file (excerpt):
```json
{
  "entries": [
    {
      "title": "My title",
      "body": "Markdown content...",
      "tags": ["tag1", "tag2"],
      "color": "Violet"
    }
  ]
}
```

Settings file (excerpt):
```json
{
  "selected_theme": "SolarizedDark",
  "selected_font": "Sans",
  "font_size": 25
}
```