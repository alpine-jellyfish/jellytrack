use std::sync::OnceLock;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{TrayIcon, TrayIconBuilder},
    AppHandle, Emitter, Manager, PhysicalPosition, State, Wry,
};
use tauri_plugin_sql::{Migration, MigrationKind};

struct TrayMenu {
    toggle: MenuItem<Wry>,
}

const INIT_SQL: &str = r#"
CREATE TABLE projects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    color TEXT,
    archived_at TEXT
);
CREATE TABLE tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    archived_at TEXT,
    UNIQUE(project_id, name)
);
CREATE TABLE entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    start_time TEXT NOT NULL,
    end_time TEXT,
    project_id INTEGER REFERENCES projects(id) ON DELETE SET NULL,
    task_id INTEGER REFERENCES tasks(id) ON DELETE SET NULL,
    note TEXT
);
CREATE INDEX idx_entries_start ON entries(start_time DESC);
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT
);
INSERT INTO projects (name) VALUES ('General');
INSERT INTO tasks (project_id, name) VALUES (1, 'Work');
INSERT INTO settings (key, value) VALUES ('default_project_id', '1');
INSERT INTO settings (key, value) VALUES ('default_task_id', '1');
"#;

const RENAME_SETTINGS_SQL: &str = "ALTER TABLE settings RENAME TO preferences;";

const ICON_SIZE: u32 = 32;

fn generate_play_bytes() -> Vec<u8> {
    let w = ICON_SIZE;
    let h = ICON_SIZE;
    let mut bytes = vec![0u8; (w * h * 4) as usize];
    let x0 = 10.0_f32;
    let x1 = 24.0_f32;
    let y_mid = h as f32 / 2.0;
    let base_half = 9.0_f32;
    for y in 0..h {
        for x in 0..w {
            let fx = x as f32 + 0.5;
            let fy = y as f32 + 0.5;
            if fx >= x0 && fx <= x1 {
                let t = (fx - x0) / (x1 - x0);
                let half = base_half * (1.0 - t);
                if (fy - y_mid).abs() <= half {
                    let i = ((y * w + x) * 4) as usize;
                    bytes[i] = 0;
                    bytes[i + 1] = 0;
                    bytes[i + 2] = 0;
                    bytes[i + 3] = 255;
                }
            }
        }
    }
    bytes
}

fn generate_pause_bytes() -> Vec<u8> {
    let w = ICON_SIZE;
    let h = ICON_SIZE;
    let mut bytes = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let in_v = y >= 8 && y < 24;
            let in_left = x >= 10 && x < 15;
            let in_right = x >= 18 && x < 23;
            if in_v && (in_left || in_right) {
                let i = ((y * w + x) * 4) as usize;
                bytes[i] = 0;
                bytes[i + 1] = 0;
                bytes[i + 2] = 0;
                bytes[i + 3] = 255;
            }
        }
    }
    bytes
}

fn play_icon_bytes() -> &'static [u8] {
    static BYTES: OnceLock<Vec<u8>> = OnceLock::new();
    BYTES.get_or_init(generate_play_bytes)
}

fn pause_icon_bytes() -> &'static [u8] {
    static BYTES: OnceLock<Vec<u8>> = OnceLock::new();
    BYTES.get_or_init(generate_pause_bytes)
}

fn play_image() -> Image<'static> {
    Image::new(play_icon_bytes(), ICON_SIZE, ICON_SIZE)
}

fn pause_image() -> Image<'static> {
    Image::new(pause_icon_bytes(), ICON_SIZE, ICON_SIZE)
}

fn position_below_tray(win: &tauri::WebviewWindow, tray: &TrayIcon) {
    let Ok(Some(rect)) = tray.rect() else { return };
    let scale = win.scale_factor().unwrap_or(1.0);

    let (icon_x, icon_y, icon_w, icon_h): (f64, f64, f64, f64) = match (rect.position, rect.size) {
        (tauri::Position::Physical(p), tauri::Size::Physical(s)) => {
            (p.x as f64, p.y as f64, s.width as f64, s.height as f64)
        }
        (tauri::Position::Logical(p), tauri::Size::Logical(s)) => (
            p.x * scale,
            p.y * scale,
            s.width * scale,
            s.height * scale,
        ),
        _ => return,
    };

    let win_size = match win.outer_size() {
        Ok(s) => s,
        Err(_) => return,
    };
    let win_w = win_size.width as f64;

    let icon_center_x = icon_x + icon_w / 2.0;
    let mut x = (icon_center_x - win_w / 2.0).round();
    let y = (icon_y + icon_h + 6.0 * scale).round();

    if let Ok(Some(monitor)) = win.current_monitor() {
        let m_pos = monitor.position();
        let m_size = monitor.size();
        let min_x = m_pos.x as f64 + 4.0;
        let max_x = m_pos.x as f64 + m_size.width as f64 - win_w - 4.0;
        x = x.clamp(min_x, max_x);
    }

    let _ = win.set_position(PhysicalPosition::new(x as i32, y as i32));
}

fn show_window(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        if let Some(tray) = app.tray_by_id("jellytrack-tray") {
            position_below_tray(&win, &tray);
        }
        let _ = win.show();
        let _ = win.set_focus();
    }
}

fn open_tab(app: &AppHandle, tab: &str) {
    show_window(app);
    let _ = app.emit("tray-open-tab", tab.to_string());
}

#[tauri::command]
fn set_tray_running(app: AppHandle, running: bool, state: State<'_, TrayMenu>) {
    if let Some(tray) = app.tray_by_id("jellytrack-tray") {
        let icon = if running { pause_image() } else { play_image() };
        let _ = tray.set_icon(Some(icon));
        let _ = tray.set_icon_as_template(true);
    }
    let _ = state.toggle.set_text(if running { "Stop" } else { "Start" });
}

#[tauri::command]
fn show_main_window(app: AppHandle) {
    show_window(&app);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let migrations = vec![
        Migration {
            version: 1,
            description: "create_initial_tables",
            sql: INIT_SQL,
            kind: MigrationKind::Up,
        },
        Migration {
            version: 2,
            description: "rename_settings_to_preferences",
            sql: RENAME_SETTINGS_SQL,
            kind: MigrationKind::Up,
        },
    ];

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![set_tray_running, show_main_window])
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:jellytrack.db", migrations)
                .build(),
        )
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let toggle_i = MenuItem::with_id(app, "toggle", "Start", true, None::<&str>)?;
            let tracker_i =
                MenuItem::with_id(app, "open_tracker", "Open Tracker", true, None::<&str>)?;
            let entries_i =
                MenuItem::with_id(app, "open_entries", "Open Entries", true, None::<&str>)?;
            let projects_i =
                MenuItem::with_id(app, "open_projects", "Open Projects", true, None::<&str>)?;
            let sep1 = PredefinedMenuItem::separator(app)?;
            let sep2 = PredefinedMenuItem::separator(app)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(
                app,
                &[
                    &toggle_i,
                    &sep1,
                    &tracker_i,
                    &entries_i,
                    &projects_i,
                    &sep2,
                    &quit_i,
                ],
            )?;

            app.manage(TrayMenu {
                toggle: toggle_i.clone(),
            });

            let tray = TrayIconBuilder::with_id("jellytrack-tray")
                .icon(play_image())
                .icon_as_template(true)
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "toggle" => {
                        let _ = app.emit("tray-toggle-running", ());
                    }
                    "quit" => app.exit(0),
                    "open_tracker" => open_tab(app, "tracker"),
                    "open_entries" => open_tab(app, "entries"),
                    "open_projects" => open_tab(app, "projects"),
                    _ => {}
                })
                .build(app)?;
            let _ = tray.set_icon_as_template(true);

            if let Some(win) = app.get_webview_window("main") {
                let _ = win.hide();
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
