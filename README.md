# WavePilot

A desktop toolkit for music producers who want to keep their projects organized without relying on cloud services or subscriptions. Everything runs locally, your data stays on your machine.

Built with Tauri 2 (Rust + React), available for Windows and Linux.

## What's in the box

### Project Manager
Import any folder as a project. Give it a name, artist, BPM, key, tags — whatever helps you find it later. Grid or list view, sort by anything, filter by favorites. Duplicate detection so you don't import the same folder twice.

### Backup Cleaner
Point it at your backup directories and it'll find `.bak`, `.backup`, `.old`, `.rpp-bak` files and other clutter that accumulates over time. Groups backups by parent project, lets you preview what gets deleted before you commit, and keeps a history of cleanups.

### Audio Analysis
Drop in a WAV file (16/24/32-bit, mono through surround) and get a loudness reading: Integrated, Short-Term, and Momentary LUFS per ITU-R BS.1770-4, plus peak and RMS levels. Results are cached by file hash so you don't re-analyze unchanged files. Color-coded dashboard makes it easy to see where you're at.

### Lyrics Workspace
Write and organize lyrics per project. Split by sections (verse, chorus, bridge, etc.), tag by language, search across all your lyrics. Split-panel editor with the list on the left and a clean writing area on the right.

### Release Checklist
Track what's left before a release. Each project gets its own checklist with sensible defaults (mix done, master exported, cover ready, uploaded, etc.). Add custom items, check things off, see your progress at a glance across all projects.

### Advanced Search
Filter by name, artist, musical key, BPM range, tags, keywords, favorites — or any combination. Sort results by date, name, BPM, or artist. Tag chips are auto-discovered from your library so you can click to filter.

## Getting started

### Prerequisites

- **Rust** 1.70+ — [rustup.rs](https://rustup.rs)
- **Node.js** 18+ — [nodejs.org](https://nodejs.org)
- **Linux only**: `webkit2gtk-4.1`, `libgtk-3-dev`, `libayatana-appindicator3-dev`

### Running in dev mode

```bash
git clone https://github.com/rave-x-official/WavePilot
cd WavePilot
npm install
npm run dev &              # frontend dev server in background
WAYLAND_DISPLAY= cargo run  # native window (prefix needed on Wayland)
```

If you're on X11, just `cargo run` works fine.

### Running tests

```bash
cd src-tauri && cargo test
```

55 tests covering project CRUD, backup scanning/cleanup, audio analysis, lyrics, release checklists, and filters.

## Building for production

```bash
# Linux .deb
cargo tauri build --bundles deb

# Windows NSIS installer
cargo tauri build --bundles nsis

# Full release binary
cd src-tauri && cargo build --release
```

The release binary is ~17 MB. The .deb package is ~6 MB.

## Tech stack

| Layer | What |
|-------|------|
| Frontend | React 19, TypeScript, Vite, TailwindCSS |
| Backend | Rust, Tauri 2 |
| Database | SQLite (rusqlite, WAL mode) |
| Audio | hound for WAV reading, custom ITU-R BS.1770-4 implementation |

## Project layout

```
src/                          Frontend
├── pages/                    Each feature is its own page
├── components/ui/            Reusable UI components
└── types/                    Shared TypeScript interfaces

src-tauri/src/                Backend
├── commands/                 Tauri command handlers (IPC boundary)
├── services/                 Business logic + tests
├── db/                       SQLite migrations + schema
├── models/                   Request/response types
└── utils.rs                  Shared helpers
```

## License

GPL v3 — see [LICENSE](LICENSE)
