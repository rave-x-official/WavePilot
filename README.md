# WavePilot

An open-source, offline-first music production toolkit for organizing, analyzing, and managing your projects. No telemetry, no user accounts, no cloud dependencies.

## Status

Early development. v1 focuses on **Project Manager** and **Backup Cleaner** with an **Audio Analysis** module for loudness measurement.

## Features

### Project Manager
- Import and organize music projects from any folder
- Grid or list view with sort (name, date, BPM, artist)
- Search by name, artist, keywords, tags
- Star favorites and filter by favorites
- Tag support with autocomplete

### Backup Cleaner
- Register directories to scan for backup files
- Detects `.bak`, `.backup`, `.old`, `.rpp-bak` and name-marked backups
- Groups backups by parent project, keeps N latest
- Preview what will be deleted before committing
- Scan history log

### Audio Analysis
- Analyze WAV files (16/24/32-bit, mono to 5.1 surround)
- Integrated LUFS, Short-Term LUFS, Momentary LUFS per ITU-R BS.1770-4
- Peak dB and RMS dB
- Results cached to avoid re-scanning unchanged files
- Dashboard UI with color-coded loudness ranges

### Planned
- True Peak, Dynamic Range, Crest Factor, Stereo Width, Phase Correlation
- BPM detection, musical key detection
- Lyrics workspace
- Release checklist

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Frontend | React 19, TypeScript, Vite, TailwindCSS |
| Backend | Rust, Tauri 2 |
| Database | SQLite via rusqlite (WAL mode) |
| Audio | hound (WAV), custom ITU-R BS.1770-4 LUFS |

## Getting Started

### Prerequisites

- Rust 1.96+ — `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- Node.js 26+ — https://nodejs.org
- Tauri system deps (Linux): `webkit2gtk-4.1`, `libgtk-3-dev`, `libayatana-appindicator3-dev`

### Install & Run

```bash
git clone https://github.com/YOUR_USERNAME/WavePilot
cd WavePilot
npm install

# Terminal 1 — frontend dev server
npm run dev

# Terminal 2 — native desktop window
cargo run
```

On Linux Wayland, prefix with `WAYLAND_DISPLAY=` if you get a SIGBUS crash:

```bash
WAYLAND_DISPLAY= cargo run
```

### Run Tests

```bash
cd src-tauri && cargo test
```

42 tests covering project CRUD, backup scanning/cleanup, LUFS analysis, filters, caching.

## Project Structure

```
src/                          # Frontend (React/TypeScript)
├── pages/                    # Projects, BackupCleaner, Analysis, etc.
├── components/ui/            # Reusable: Button, Card, Input, Badge, Modal, Sidebar
├── hooks/                    # useSettings
└── types/                    # TypeScript interfaces

src-tauri/src/                # Backend (Rust)
├── commands/                 # Tauri command handlers
│   ├── projects.rs
│   ├── backup.rs
│   ├── analysis.rs
│   └── settings.rs
├── services/                 # Business logic
│   ├── project_service.rs    # CRUD, search, sort, favorites
│   ├── backup_service.rs     # Scan, preview, cleanup, history
│   ├── analysis_service.rs   # LUFS, Peak, RMS, caching
│   └── settings_service.rs
├── db/                       # Database
│   ├── migrations.rs         # v1–v3 schema
│   ├── schema.rs             # Row structs
│   └── mod.rs                # Database, lock()
├── models/                   # Request/response types
│   ├── project.rs
│   ├── backup.rs
│   ├── analysis.rs
│   └── settings.rs
└── utils.rs                  # new_id, now_timestamp, resolve_canonical_path, collect_ok
```

## Building

```bash
# Production build
cd src-tauri && cargo build --release

# Package (.deb)
cargo tauri build --bundles deb

# Package (.AppImage) — may need workarounds on bleeding-edge distros
cargo tauri build --bundles appimage
```

The release binary is at `target/release/wavepilot` (~17 MB).

## Architecture Decisions

- **Mutex<Connection>** for thread-safe SQLite access — simple, sufficient for single-writer desktop
- **Normalized tags** via `project_tags` table (UNIQUE per project+tag, COLLATE NOCASE) not JSON blobs
- **Dynamic SET builder** for project updates — replaces 11 single-field UPDATE functions
- **collect_ok()** logs row-level SQL errors instead of silently dropping them
- **file_hash** caching avoids re-analyzing unchanged audio files

## License

MIT
