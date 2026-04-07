# Xplorer Product Spec

> A modern, keyboard-first file manager for Windows built with Rust + egui.
> Inspired by File Pilot, Marta, Files App, and One Commander.

---

## Vision

**Xplorer = Files App's clean UX + Marta's keyboard-first power + File Pilot's preview intelligence.**

Not a Finder clone. Not a Directory Opus control surface. A file manager that:
- Looks modern and approachable out of the box
- Rewards keyboard users with speed
- Shows useful information without requiring clicks
- Grows with the user (simple defaults → discoverable power)

---

## Design Principles

1. **Keyboard-first, mouse-friendly** — Every action has a shortcut. Shortcuts are visible in UI (menus, tooltips, command palette). But mouse workflows are never second-class.
2. **Information density over whitespace** — Compact 13-14px base. Dense rows. Show more files, not more padding. But never feel cramped.
3. **Progressive disclosure** — Single pane by default. Dual pane on demand. Command palette for hidden power. Settings for customization. Don't overwhelm beginners.
4. **Fast feedback** — Directory loads feel instant. Operations show progress. Errors are clear. No silent failures.
5. **Predictable behavior** — Right-click acts on what you clicked. Ctrl+C copies what's selected. No surprises.

---

## Architecture: Current State vs Target

### What Works Today ✅
- Multi-tab via egui_dock with split panes
- Back/Forward/Up navigation with per-tab history
- Breadcrumb bar → click-to-edit address bar (Ctrl+L)
- Table view with 4 sortable columns (Name, Size, Type, Modified)
- Multi-selection (click, Ctrl+click, Shift+click, Ctrl+A)
- Keyboard navigation (↑↓, Enter, Backspace, F2)
- Inline rename and new file/folder creation
- Copy/Cut/Paste via keyboard shortcuts
- Delete (trash) and permanent delete with confirmation
- Sidebar: Quick Access + Drives (with capacity bars) + Favorites
- Filter input (Ctrl+F) — local name filter
- Context menus for files and empty area
- Background file operations via worker thread
- Filesystem watcher with debounced auto-refresh
- Toast notifications for operation results
- Phosphor icons with per-type color coding

### What's Broken/Incomplete ⚠️
- Context menu acts on selection, not right-clicked item (mismatch)
- No scroll-to-selected on keyboard navigation
- Cut items not visually dimmed
- Drives list not refreshed after startup
- Watcher only watches focused tab
- Font loading hardcoded to Windows path
- New item path separator hardcoded `\\`
- Status bar shows stale counts during loading
- "Show in Explorer" name is confusing (we ARE the explorer)

### What's Missing Entirely ❌
- Preview panel
- Command palette
- Grid/icon view mode
- Hidden files toggle
- Search (global or local beyond filter)
- Drag and drop
- Session persistence (tabs/paths across restarts)
- Settings/preferences UI
- Keyboard shortcut help overlay
- Operation progress (copy/move progress bar)
- Batch rename
- "Open With" functionality
- Properties/info dialog
- Undo for file operations
- Column view / Miller columns

---

## Feature Tiers

### Tier 1: Foundation (Must Have)
These fix broken things and add the most impactful missing features.

#### 1.1 Fix Context Menu Selection Behavior
Right-clicking an unselected item should select it first, then show the context menu. This is how every file manager works. Current behavior is confusing.

#### 1.2 Hidden Files Toggle
- `Ctrl+H` toggles `show_hidden` per tab
- Status bar or top bar shows indicator when hidden files are visible
- Filter in `ensure_filtered()` — exclude entries starting with `.` when hidden=false
- Default: hidden files OFF

#### 1.3 Scroll-to-Selected
When keyboard ↓/↑ moves selection, auto-scroll the table to keep the selected row visible. Essential for keyboard navigation in long directories.

#### 1.4 Cut Items Visual Feedback
Items in the cut clipboard should render at 50% opacity in the file list. This is universal across all file managers.

#### 1.5 Preview Panel (Right Side)
- Toggle: `Ctrl+P` or a toolbar button
- 380px wide panel on the right side (like File Pilot)
- Shows for single-selected file:
  - **Header**: filename, type badge, size, modified date
  - **Body by type**:
    - Images: thumbnail preview
    - Text/code: first ~50 lines with syntax highlighting (use `syntect` crate)
    - Audio/video: metadata card (duration, codec, resolution)
    - Archives: entry listing
    - Other: hex dump or "No preview available"
  - **Actions**: Open, Copy Path, Rename, Delete
- Shows for directory: item count, total size (async), recent files
- Shows for multi-selection: count, total size

#### 1.6 Command Palette
- Trigger: `Ctrl+K` (like File Pilot) or `Ctrl+Shift+P` (like VS Code)
- Centered overlay with search input
- Contents:
  - **Actions**: all available commands with their shortcuts (Go Back, Go Forward, New Tab, Split Right, Toggle Sidebar, Toggle Hidden, etc.)
  - **Recent locations**: shown when input is empty
  - **File search**: after 2+ chars, search current directory tree
- Fuzzy matching on action labels
- Arrow keys to navigate, Enter to execute, Escape to close
- Each result row: icon + label + shortcut/path on right

#### 1.7 Session Persistence
- Save on exit: open tabs (paths), active tab, split layout, sidebar visibility
- Restore on startup
- Config file: `~/.config/xplorer/session.json` or Windows AppData equivalent

#### 1.8 Operation Progress
- When copy/move/delete operations are running, show a progress panel
- Floating panel at bottom-right (above toasts) or status bar integration
- Shows: operation type, file count, current file, progress bar
- Cancel button

### Tier 2: Power Features (Should Have)

#### 2.1 Unified Top Bar (Omnibar-style)
Evolve the current breadcrumb + filter into a single unified control:
- **Default state**: breadcrumb segments (clickable)
- **Click breadcrumb background or Ctrl+L**: edit mode (type path, with autocomplete)
- **Ctrl+F**: filter mode (filters current listing)
- **Ctrl+K**: opens command palette (separate overlay)
- Visual mode indicators so user knows which mode they're in

#### 2.2 Grid/Icon View Mode
- Toggle button in top bar (or Ctrl+1 = Details, Ctrl+2 = Grid)
- Grid shows: icon/thumbnail + filename
- Useful for image-heavy directories
- Per-tab setting (not global)

#### 2.3 Keyboard Shortcut Overlay
- `?` or `Ctrl+/` opens a modal showing all available shortcuts
- Grouped by category: Navigation, Selection, File Operations, View, Tabs
- Like File Pilot's shortcuts overlay

#### 2.4 Drag and Drop
- Drag files between panes → move (default) or copy (Ctrl held)
- Drag to sidebar favorites → add bookmark
- Drag from OS Explorer into Xplorer → copy
- Visual feedback: drop target highlighting, ghost preview

#### 2.5 Search
Two distinct systems (like File Pilot):
1. **Quick filter** (existing, Ctrl+F): filters current listing by name
2. **Deep search** (Ctrl+Shift+F): modal overlay, searches recursively
   - Modes: filename, content grep, regex
   - Scope: current folder or all indexed locations
   - Results as a navigable list

#### 2.6 Batch Rename
- Select multiple files → Ctrl+Shift+R or context menu "Batch Rename..."
- Dialog with:
  - Find/Replace (regex supported)
  - Sequential numbering
  - Case transformation
  - Extension change
  - Live preview of changes

#### 2.7 "Open With" Support
- Context menu → "Open With..." → show system-registered applications
- On Windows: query registry for associated programs
- Allow typing app name/path

#### 2.8 Properties / Info Dialog
- Context menu → "Properties" or `Alt+Enter`
- Shows: full path, type, size (recursive for dirs), created/modified/accessed
- Windows: attributes (hidden, readonly, system)
- Permissions display

### Tier 3: Differentiators (Nice to Have)

#### 3.1 Workspaces
- Save current tab layout + paths as a named workspace
- Quick-switch between workspaces
- Like Sigma's workspace concept

#### 3.2 Columns/Miller Columns View
- Third view mode: columns that show nested directory hierarchy
- Good for deep navigation
- Like One Commander / Finder column view

#### 3.3 Stash/Shelf
- Temporary collection area for files from different locations
- Drag files to stash, then paste them all to a destination
- Like QSpace's Stash Shelf

#### 3.4 Git Integration
- Show git status badges on files (modified, untracked, ignored)
- Status bar shows branch name when in a git repo
- Like File Pilot's git status feature

#### 3.5 Archive Browsing
- Navigate into ZIP/7z/tar files as if they were folders
- Extract here, extract to folder
- Like Marta's archive-as-folder

#### 3.6 Undo System
- Undo last file operation (move, rename, copy, delete)
- Ctrl+Z with undo stack
- Status bar shows "N operations undoable"

---

## Layout Specification

```
┌─────────────────────────────────────────────────────────────┐
│  Window Title: "{dir} — Xplorer"                            │
├────────┬──────────────────────────────────┬─────────────────┤
│        │  [←] [→] [↑] [breadcrumb/path]  │  [filter] [⚙]  │
│        │  ─ ─ ─ Top Bar ─ ─ ─ ─ ─ ─ ─ ─ │                 │
│        ├──────────────────────────────────┤                 │
│  Side  │                                  │    Preview      │
│  bar   │       File List                  │    Panel        │
│        │       (table / grid)             │    (380px)      │
│ 220px  │                                  │                 │
│        │                                  │                 │
│        │                                  │                 │
│        ├──────────────────────────────────┤                 │
│        │  Status Bar: items | selected |  │                 │
│        │  path | free space               │                 │
├────────┴──────────────────────────────────┴─────────────────┤
│  [Operations Progress]                              Toasts  │
└─────────────────────────────────────────────────────────────┘
```

### Dual Pane Mode (Ctrl+\)
```
┌────────┬───────────────────┬───────────────────┬────────────┐
│        │    Left Pane       │    Right Pane      │  Preview   │
│  Side  │    (active)        │    (inactive,      │  (optional)│
│  bar   │                    │     dimmed border)  │            │
│        │    File List       │    File List       │            │
│        │                    │                    │            │
├────────┴───────────────────┴───────────────────┴────────────┤
│  Status Bar                                                  │
└──────────────────────────────────────────────────────────────┘
```

---

## Keyboard Shortcuts (Complete Map)

### Navigation
| Shortcut | Action |
|----------|--------|
| `Alt+←` | Go back |
| `Alt+→` | Go forward |
| `Alt+↑` | Go to parent |
| `Enter` | Open file / Enter directory |
| `Backspace` | Go to parent |
| `Ctrl+L` | Edit address bar |
| `Tab` | Switch active pane (dual pane mode) |

### File Operations
| Shortcut | Action |
|----------|--------|
| `Ctrl+C` | Copy |
| `Ctrl+X` | Cut |
| `Ctrl+V` | Paste |
| `F2` | Rename |
| `Del` | Move to Trash |
| `Shift+Del` | Delete permanently |
| `Ctrl+Shift+N` | New Folder |
| `Ctrl+N` | New File |

### Selection
| Shortcut | Action |
|----------|--------|
| `↑` / `↓` | Move selection |
| `Ctrl+A` | Select all |
| `Ctrl+Click` | Toggle selection |
| `Shift+Click` | Range selection |
| `Escape` | Clear selection / Close overlay |

### View & Panels
| Shortcut | Action |
|----------|--------|
| `Ctrl+B` | Toggle sidebar |
| `Ctrl+P` | Toggle preview panel |
| `Ctrl+H` | Toggle hidden files |
| `Ctrl+1` | Details view |
| `Ctrl+2` | Grid view |
| `Ctrl+\` | Toggle dual pane |
| `F5` | Refresh |

### Tabs
| Shortcut | Action |
|----------|--------|
| `Ctrl+T` | New tab |
| `Ctrl+W` | Close tab |
| `Ctrl+Tab` | Next tab |
| `Ctrl+Shift+Tab` | Previous tab |

### Search & Command
| Shortcut | Action |
|----------|--------|
| `Ctrl+F` | Filter current listing |
| `Ctrl+K` | Command palette |
| `Ctrl+Shift+F` | Deep search |
| `?` | Shortcut help overlay |

---

## Visual Design

### Color System
Continue with the Tokyo Night-inspired dark theme, extended:
- **Background**: deep blue-gray (#1a1b26)
- **Surface**: slightly lighter (#24283b)
- **Active pane border**: accent blue (#7aa2f7)
- **Inactive pane border**: muted gray (#565f89)
- **Text primary**: light (#c0caf5)
- **Text secondary/muted**: (#565f89)
- **Accent**: blue (#7aa2f7)
- **Warning**: amber (#e0af68)
- **Error/Destructive**: red (#f7768e)
- **Success**: green (#9ece6a)

### Typography
- **UI font**: Segoe UI (Windows) / system default, 13px
- **Monospace**: JetBrains Mono or Cascadia Code for paths, code preview, sizes
- **Icon font**: Phosphor icons (already in use)

### Density
- Row height: ~24px (compact, like File Pilot's 13px base)
- Sidebar width: 220px default, resizable 160-400px
- Preview panel: 380px, collapsible
- Status bar: 28px
- Top bar: 36px
- Minimal padding everywhere. Every pixel earns its place.

---

## Implementation Order

### Phase 1: Fix & Foundation (1-2 weeks)
1. Fix context menu selection behavior
2. Hidden files toggle (Ctrl+H)
3. Scroll-to-selected on keyboard nav
4. Cut items visual feedback
5. Session persistence (save/restore tabs)
6. Fix: watcher for all tabs, not just focused
7. Fix: status bar during loading

### Phase 2: Core Power Features (2-3 weeks)
1. Preview panel
2. Command palette
3. Operation progress panel
4. Keyboard shortcut overlay
5. Grid view mode

### Phase 3: Advanced Features (2-3 weeks)
1. Deep search (recursive filename + content)
2. Unified top bar / Omnibar evolution
3. Drag and drop
4. Batch rename
5. Properties dialog

### Phase 4: Differentiators (ongoing)
1. Git integration
2. Workspaces
3. Archive browsing
4. Undo system
5. Columns view

---

## Anti-Patterns to Avoid

1. **Don't build a settings page before the features it configures** — Ship features first, settings later.
2. **Don't add UI elements without keyboard shortcuts** — Every button must have a key equivalent.
3. **Don't make the UI modal unnecessarily** — Prefer inline interactions (inline rename, inline filter) over dialog boxes.
4. **Don't optimize for screenshots over real use** — Dense, information-rich layouts beat pretty-but-empty ones.
5. **Don't forget Windows** — We're on Windows. Test with Windows paths, Windows Terminal, Windows icons. Not macOS.
6. **Don't add features nobody asked for** — Each feature must solve a real workflow problem.
