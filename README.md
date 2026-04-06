# Xplorer

A fast, GPU-rendered file manager built entirely in Rust with [egui](https://github.com/emilk/egui). Inspired by [File Pilot](https://filepilot.tech).

## Features

- **Panel splitting** — split tabs side-by-side via middle-click, context menu, or keyboard
- **Multi-tab browsing** — Ctrl+T / Ctrl+W, drag-and-drop tab reordering (egui_dock)
- **Keyboard-driven** — Tab to switch panels, Ctrl+Tab to cycle tabs, full shortcut coverage
- **File operations** — copy, cut, paste, rename, delete, trash, create file/folder
- **Breadcrumb navigation** — clickable path segments, editable address bar (Ctrl+L)
- **Column sorting** — sort by name, size, type, or modified date
- **Filter** — Ctrl+F to filter files in the current directory
- **Context menus** — right-click on files or empty area for actions
- **Bookmarks & drives** — sidebar with quick access, favorites, and mounted drives
- **Directory watcher** — auto-refresh on filesystem changes
- **Dark theme** — File Pilot-inspired palette with Phosphor icons

## Architecture

```
crates/
├── xplorer-core/    # Pure Rust library — file ops, system info, bookmarks
└── xplorer-egui/    # Desktop GUI — eframe + egui + egui_dock
```

| Layer | Technology |
|---|---|
| GUI framework | egui 0.31 (eframe) |
| Panel management | egui_dock 0.16 |
| Icons | egui-phosphor |
| Notifications | egui-notify |
| File watching | notify 7 |
| System clipboard | arboard |
| Async runtime | Tokio |

## Getting Started

### Prerequisites

- Rust (latest stable via [rustup](https://rustup.rs))

### Run

```bash
cargo run -p xplorer-egui
```

### Build

```bash
cargo build -p xplorer-egui --release
```

### Test

```bash
cargo test -p xplorer-core
```

## Keyboard Shortcuts

| Shortcut | Action |
|---|---|
| Ctrl+T | New tab |
| Ctrl+W | Close tab |
| Tab | Switch focus between panels |
| Ctrl+Tab / Ctrl+Shift+Tab | Cycle tabs within panel |
| Alt+Left / Alt+Right | Back / Forward |
| Alt+Up / Backspace | Go to parent directory |
| Ctrl+L | Edit address bar |
| Ctrl+F | Focus filter input |
| Ctrl+B | Toggle sidebar |
| Ctrl+C / Ctrl+X / Ctrl+V | Copy / Cut / Paste |
| Delete | Move to trash |
| Shift+Delete | Permanent delete |
| F2 | Rename |
| F5 | Refresh |
| Ctrl+A | Select all |
| Enter | Open selected |
| Middle-click folder | Split right with that folder |

## License

AGPL-3.0. See [LICENSE](LICENSE) for details.
