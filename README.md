# jellytrack - Time tracking made for workers, not managers.

A personal time-tracking menu bar app for macOS. Click the tray icon, pick a project and task, hit Start. Stop when you're done. Notes per session; manual entries when you forgot to hit Start.

[![Fuel jellytrack — support the developer](./support-badge.svg)](https://buy.stripe.com/9B67sLeaM6zi5B22yw2wU00)

## Philosophy

**Collect data. Report separately.**

Most time trackers try to be a tracker *and* a dashboard *and* a billing tool at once — which means guessing what you want to see and burying the raw truth under charts. jellytrack does one job: record what you worked on and when. The SQLite file is right there, in a schema you can read.

Want a report? Write it. `sqlite3`, DuckDB, pandas, a Jupyter notebook, your accountant's spreadsheet — pick your tool. Nothing in the app fights you over how to slice your own data. Built for people who'd rather `GROUP BY` than click through yet another "insights" tab.

- See our fully functional, quarto demo-reports for inspiration: [jellytrack-reports](https://codefloe.com/alpine-jellyfish/jellytrack-reports)


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

## Install (macOS)

Download the latest DMG from codefloe, open it, and drag **jellytrack.app** to `/Applications`.

### "jellytrack is damaged and can't be opened" — this is safe and expected

jellytrack is **not signed with an Apple Developer ID** (that program costs $99/year for me as a dev, just to be able to deliver a verified DMG). macOS therefore quarantines the download and Gatekeeper reports it as "damaged". Yet, you have two good options to use the app.

1) Trust me: 

After copying the app to `/Applications`, remove the quarantine flag once:

```sh
xattr -dr com.apple.quarantine /Applications/jellytrack.app
```

Then open it normally. You only need to do this once per install (and again after each update).


2) Build it yourself

Clone the app from codefloe, make sure you got the stack to build the app and build on your own. Take a look as close as you want – it's all open source. 


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

### Change the database location

Open the **Projects** tab → **Database** section:

- **Create new…** picks a folder + filename and creates a fresh jellytrack DB there.
- **Open existing…** points jellytrack at another `.db` file — useful for keeping work and personal databases separate, or for opening a DB synced via Dropbox/iCloud.
- **Reset to default** returns to `~/Library/Application Support/com.jellyfish.jellytrack/jellytrack.db`.

Any change requires a restart of jellytrack; the UI prompts with a Relaunch button. Your choice is persisted in `db-location.json` next to the default DB — deleting that file also reverts to the default.

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
