# jellytrack

A personal time-tracking menu bar app for macOS. Click the tray icon, pick a project and task, hit Start. Stop when you're done. Notes per session; manual entries when you forgot to hit Start.

[![Fuel jellytrack — support the developer](./support-badge.svg)](https://buy.stripe.com/9B67sLeaM6zi5B22yw2wU00)

## What it does

- **Menu bar app** (no dock icon). The tray icon shows play (idle) or pause (running).
- **Projects and tasks** with soft-archive; a default project/task can be set for one-click starts.
- **Grace-window resume**: pressing Start within 60s of the last Stop *and* on the same project/task un-stops the last entry instead of creating a new one. Kills accidental tiny sessions. Grace window is configurable — see [Preferences](#preferences).
- **Notes** on the live entry and on every past entry, autosaved on blur.
- **Manual entries** with date/start/end/project/task/notes on the Entries tab.
- **Entries view** grouped by day with per-day totals.

## Stack

- [Tauri 2](https://tauri.app) (Rust) — shell, tray, window
- [SvelteKit](https://kit.svelte.dev) + Svelte 5 runes — UI
- [tauri-plugin-sql](https://docs.rs/tauri-plugin-sql) with SQLite — storage
- [Bun](https://bun.sh) — package manager and dev server runner
- macOS only (ARM64/Intel) — uses `NSStatusItem` semantics for the tray

## Prerequisites

- macOS
- [Bun](https://bun.sh) ≥ 1.1
- Rust toolchain (`rustup` + stable)
- Xcode Command Line Tools (`xcode-select --install`)

## Develop

```sh
bun install
bun run tauri dev
```

Rust changes trigger a rebuild + relaunch automatically. Svelte changes hot-reload.

Type-check:

```sh
bun run check     # svelte-check + tsc
cd src-tauri && cargo check
```

## Production build

```sh
bun run tauri build
```

Produces:

- `src-tauri/target/release/bundle/macos/jellytrack.app` — bundled app
- `src-tauri/target/release/bundle/dmg/jellytrack_<version>_<arch>.dmg` — installer

To build for a specific architecture:

```sh
bun run tauri build --target aarch64-apple-darwin   # Apple Silicon
bun run tauri build --target x86_64-apple-darwin    # Intel
```

## Database

SQLite file lives at:

```
~/Library/Application Support/com.jellyfish.jellytrack/jellytrack.db
```

Schema is managed by tauri-plugin-sql migrations declared in `src-tauri/src/lib.rs`.

### Query manually

```sh
sqlite3 ~/Library/Application\ Support/com.jellyfish.jellytrack/jellytrack.db
```

Useful:

```sql
.tables
.schema entries
SELECT * FROM preferences;
SELECT id, start_time, end_time, project_id, task_id, note FROM entries ORDER BY start_time DESC LIMIT 20;
```

### Preferences

The `preferences` table is a key/value store. Currently used keys:

| key                    | default | meaning                                                                                  |
| ---------------------- | ------- | ---------------------------------------------------------------------------------------- |
| `default_project_id`   | `1`     | Project pre-selected on the Tracker tab when idle.                                       |
| `default_task_id`      | `1`     | Task pre-selected on the Tracker tab when idle.                                          |
| `resume_grace_seconds` | `60`    | If Start fires within this many seconds of Stop on the *same* project/task, resume it.   |

Set a value:

```sh
sqlite3 ~/Library/Application\ Support/com.jellyfish.jellytrack/jellytrack.db \
  "INSERT OR REPLACE INTO preferences (key, value) VALUES ('resume_grace_seconds', '120');"
```

Restart the app for changes to be picked up cleanly (though most reads happen fresh on each action).

### Change the linked database

The path is derived by Tauri from the app identifier (`com.jellyfish.jellytrack`) and isn't configurable at runtime today. Two options:

1. **Symlink** (recommended) — move the DB anywhere, then symlink it back:

   ```sh
   cd ~/Library/Application\ Support/com.jellyfish.jellytrack
   mv jellytrack.db /path/to/wherever/jellytrack.db
   ln -s /path/to/wherever/jellytrack.db jellytrack.db
   ```

   Useful for syncing via iCloud/Dropbox or keeping the DB with other tracked data.

2. **Change the identifier or path in code** — edit `identifier` in `src-tauri/tauri.conf.json` and/or the `sqlite:` URI in `src-tauri/src/lib.rs`, then rebuild. Note this creates a *new* app-data directory; migrate the file across yourself.

A GUI setting for the DB path is a possible future addition.

## Layout

```
jellytrack/
├── src/                 # Svelte 5 frontend
│   ├── lib/             # components (Tracker, Entries, Projects, Select, db.ts, format.ts)
│   └── routes/          # +page.svelte, +layout.ts
├── src-tauri/           # Rust backend (tray, window, DB migrations, commands)
│   ├── src/
│   ├── icons/
│   ├── Cargo.toml
│   └── tauri.conf.json
├── package.json
└── README.md
```
