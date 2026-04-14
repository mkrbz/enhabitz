mod db;

use db::{DayEntry, Db, HabitData, HabitRecord, LogData};
use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WebviewWindow,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

pub struct DbState(pub Mutex<Db>);

const DEFAULT_SHORTCUT: &str = "CommandOrControl+Shift+H";
const SHORTCUT_FILENAME: &str = "shortcut.txt";

// ─── Commands ─────────────────────────────────────────────────────────────────

#[tauri::command]
fn load_habits(state: tauri::State<'_, DbState>) -> Result<Vec<HabitRecord>, String> {
    state
        .0
        .lock()
        .unwrap()
        .load_habits()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn add_habit(state: tauri::State<'_, DbState>, data: HabitData) -> Result<i64, String> {
    state
        .0
        .lock()
        .unwrap()
        .add_habit(data)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn update_habit(state: tauri::State<'_, DbState>, id: i64, data: HabitData) -> Result<(), String> {
    state
        .0
        .lock()
        .unwrap()
        .update_habit(id, data)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_habit(state: tauri::State<'_, DbState>, id: i64) -> Result<(), String> {
    state
        .0
        .lock()
        .unwrap()
        .delete_habit(id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn save_log(state: tauri::State<'_, DbState>, log: LogData) -> Result<(), String> {
    state
        .0
        .lock()
        .unwrap()
        .save_log(log)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn load_log_history(state: tauri::State<'_, DbState>) -> Result<Vec<DayEntry>, String> {
    state
        .0
        .lock()
        .unwrap()
        .load_log_history()
        .map_err(|e| e.to_string())
}

// ─── Shortcut commands ────────────────────────────────────────────────────────

#[tauri::command]
fn get_shortcut(app: tauri::AppHandle) -> String {
    let path = shortcut_path(&app);
    std::fs::read_to_string(&path)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| DEFAULT_SHORTCUT.to_string())
}

#[tauri::command]
fn set_shortcut(app: tauri::AppHandle, shortcut: String) -> Result<(), String> {
    let shortcut = shortcut.trim().to_string();
    if shortcut.is_empty() {
        return Err("Shortcut cannot be empty".into());
    }

    // Unregister all current shortcuts
    app.global_shortcut()
        .unregister_all()
        .map_err(|e| e.to_string())?;

    // Register the new one
    register_shortcut(&app, &shortcut)?;

    // Persist
    let path = shortcut_path(&app);
    std::fs::write(&path, &shortcut).map_err(|e| e.to_string())?;

    Ok(())
}

// ─── Shortcut helpers ─────────────────────────────────────────────────────────

fn shortcut_path(app: &tauri::AppHandle) -> std::path::PathBuf {
    app.path()
        .app_data_dir()
        .expect("no app data dir")
        .join(SHORTCUT_FILENAME)
}

fn register_shortcut(app: &tauri::AppHandle, shortcut: &str) -> Result<(), String> {
    let handle = app.clone();
    app.global_shortcut()
        .on_shortcut(shortcut, move |_app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                toggle_widget(&handle);
            }
        })
        .map_err(|e| e.to_string())
}

// ─── macOS dock visibility ────────────────────────────────────────────────────

/// Show or hide the app in the macOS dock and cmd+tab switcher.
/// Regular (0) = visible; Accessory (1) = hidden.
#[cfg(target_os = "macos")]
#[allow(unexpected_cfgs)]
fn set_dock_visible(visible: bool) {
    use objc::{class, msg_send, runtime::Object, sel, sel_impl};
    let policy: i64 = if visible { 0 } else { 1 };
    unsafe {
        let ns_app: *mut Object = msg_send![class!(NSApplication), sharedApplication];
        let _: () = msg_send![ns_app, setActivationPolicy: policy];
        if visible {
            // Bring the app to front after restoring Regular policy.
            let _: () = msg_send![ns_app, activateIgnoringOtherApps: true];
        }
    }
}

// ─── Widget helpers ───────────────────────────────────────────────────────────

fn toggle_widget(app: &tauri::AppHandle) {
    let Some(widget) = app.get_webview_window("widget") else {
        return;
    };
    if widget.is_visible().unwrap_or(false) {
        let _ = widget.hide();
    } else {
        position_widget(&widget);
        let _ = widget.show();
        let _ = widget.set_focus();
    }
}

fn position_widget(window: &WebviewWindow) {
    let Ok(Some(monitor)) = window.primary_monitor() else {
        return;
    };
    let scale = monitor.scale_factor();
    let screen = monitor.size();
    let win = window
        .outer_size()
        .unwrap_or(tauri::PhysicalSize::new(380, 540));
    // Bottom-right corner with a small margin (48px bottom for taskbar)
    let x = screen.width as f64 / scale - win.width as f64 / scale - 16.0;
    let y = screen.height as f64 / scale - win.height as f64 / scale - 48.0;
    let _ = window.set_position(tauri::LogicalPosition::new(x, y));
}

// ─── App setup ────────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            // ── Database ──────────────────────────────────────────────────────
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let db = Db::new(&data_dir.join("enhabitz.db"))
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
            app.manage(DbState(Mutex::new(db)));

            // ── Global shortcut ───────────────────────────────────────────────
            let saved = std::fs::read_to_string(data_dir.join(SHORTCUT_FILENAME))
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| DEFAULT_SHORTCUT.to_string());
            if let Err(e) = register_shortcut(app.handle(), &saved) {
                eprintln!("Failed to register shortcut '{}': {}", saved, e);
            }

            // ── Main window: hide instead of close ────────────────────────────
            let main = app.get_webview_window("main").unwrap();
            let main_clone = main.clone();
            main.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = main_clone.hide();
                    #[cfg(target_os = "macos")]
                    set_dock_visible(false);
                }
            });

            // ── System tray ───────────────────────────────────────────────────
            let widget = MenuItem::with_id(app, "widget", "Toggle Widget", true, None::<&str>)?;
            let show = MenuItem::with_id(app, "show", "Show Enhabitz", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&widget, &show, &quit])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "widget" => toggle_widget(app),
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            #[cfg(target_os = "macos")]
                            set_dock_visible(true);
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        toggle_widget(tray.app_handle());
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_habits,
            add_habit,
            update_habit,
            delete_habit,
            save_log,
            load_log_history,
            get_shortcut,
            set_shortcut,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
