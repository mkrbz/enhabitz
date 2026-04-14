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

/// Tracks which shortcuts are using the Hyprland backend.
/// None = plugin backend active, Some(shortcut) = Hyprland backend active.
pub struct HyprlandBackend {
    pub widget: Mutex<Option<String>>,
    pub main: Mutex<Option<String>>,
}

const DEFAULT_WIDGET_SHORTCUT: &str = "CommandOrControl+Shift+H";
const DEFAULT_MAIN_SHORTCUT: &str = "CommandOrControl+Shift+E";
const WIDGET_SHORTCUT_FILE: &str = "widget_shortcut.txt";
const MAIN_SHORTCUT_FILE: &str = "main_shortcut.txt";

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
fn get_shortcut(app: tauri::AppHandle, target: String) -> String {
    read_shortcut(&app, &target)
}

#[tauri::command]
fn set_shortcut(app: tauri::AppHandle, target: String, shortcut: String) -> Result<(), String> {
    let shortcut = shortcut.trim().to_string();
    if shortcut.is_empty() {
        return Err("Shortcut cannot be empty".into());
    }

    let hypr = app.state::<HyprlandBackend>();

    let slot = match target.as_str() {
        "widget" => &hypr.widget,
        "main" => &hypr.main,
        other => return Err(format!("Unknown target: {}", other)),
    };

    let mut guard = slot.lock().unwrap();

    #[cfg(target_os = "linux")]
    if let Some(old) = guard.as_ref() {
        hyprland_unbind(old);
        let flag = hyprland_flag(&target);
        hyprland_bind(&shortcut, flag)
            .then_some(())
            .ok_or_else(|| "hyprctl bind failed".to_string())?;
        *guard = Some(shortcut.clone());
        write_shortcut(&app, &target, &shortcut)?;
        return Ok(());
    }

    // Plugin backend
    app.global_shortcut()
        .unregister_all()
        .map_err(|e| e.to_string())?;
    // Re-register the OTHER shortcut so it stays active after unregister_all
    let other_target = if target == "widget" { "main" } else { "widget" };
    let other = read_shortcut(&app, other_target);
    register_plugin_shortcut(&app, &other, other_target)?;
    register_plugin_shortcut(&app, &shortcut, &target)?;

    drop(guard);
    write_shortcut(&app, &target, &shortcut)?;
    Ok(())
}

// ─── Shortcut helpers ─────────────────────────────────────────────────────────

fn shortcut_file(target: &str) -> &'static str {
    match target {
        "main" => MAIN_SHORTCUT_FILE,
        _ => WIDGET_SHORTCUT_FILE,
    }
}

fn default_shortcut(target: &str) -> &'static str {
    match target {
        "main" => DEFAULT_MAIN_SHORTCUT,
        _ => DEFAULT_WIDGET_SHORTCUT,
    }
}

fn read_shortcut(app: &tauri::AppHandle, target: &str) -> String {
    let path = app
        .path()
        .app_data_dir()
        .expect("no app data dir")
        .join(shortcut_file(target));
    std::fs::read_to_string(path)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| default_shortcut(target).to_string())
}

fn write_shortcut(app: &tauri::AppHandle, target: &str, shortcut: &str) -> Result<(), String> {
    let path = app
        .path()
        .app_data_dir()
        .expect("no app data dir")
        .join(shortcut_file(target));
    std::fs::write(path, shortcut).map_err(|e| e.to_string())
}

fn register_plugin_shortcut(
    app: &tauri::AppHandle,
    shortcut: &str,
    target: &str,
) -> Result<(), String> {
    let handle = app.clone();
    let target = target.to_string();
    app.global_shortcut()
        .on_shortcut(shortcut, move |_app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                match target.as_str() {
                    "main" => toggle_main(&handle),
                    _ => toggle_widget(&handle),
                }
            }
        })
        .map_err(|e| e.to_string())
}

// ─── Hyprland shortcut backend (Linux only) ───────────────────────────────────

#[cfg(target_os = "linux")]
const HYPRLAND_WIDGET_FLAG: &str = "/tmp/enhabitz-widget";
#[cfg(target_os = "linux")]
const HYPRLAND_MAIN_FLAG: &str = "/tmp/enhabitz-main";

#[cfg(target_os = "linux")]
fn hyprland_flag(target: &str) -> &'static str {
    if target == "main" {
        HYPRLAND_MAIN_FLAG
    } else {
        HYPRLAND_WIDGET_FLAG
    }
}

#[cfg(target_os = "linux")]
fn is_hyprland() -> bool {
    std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok()
}

/// Convert a Tauri accelerator string to Hyprland (mods, key) pair.
/// e.g. "CommandOrControl+Shift+H" → ("CTRL SHIFT", "h")
#[cfg(target_os = "linux")]
fn tauri_to_hyprland(shortcut: &str) -> (String, String) {
    let mut mods: Vec<&str> = Vec::new();
    let mut key = String::from("h");
    for part in shortcut.split('+') {
        match part.trim() {
            "CommandOrControl" | "Control" | "Ctrl" => mods.push("CTRL"),
            "Shift" => mods.push("SHIFT"),
            "Alt" => mods.push("ALT"),
            "Super" | "Meta" | "Command" => mods.push("SUPER"),
            k => key = k.to_lowercase(),
        }
    }
    (mods.join(" "), key)
}

#[cfg(target_os = "linux")]
fn hyprland_bind(shortcut: &str, flag: &str) -> bool {
    let (mods, key) = tauri_to_hyprland(shortcut);
    let bind_arg = format!("{},{},exec,/usr/bin/touch {}", mods, key, flag);
    eprintln!("[enhabitz] hyprctl keyword bind {}", bind_arg);
    let out = std::process::Command::new("hyprctl")
        .arg("keyword")
        .arg("bind")
        .arg(&bind_arg)
        .output();
    match out {
        Ok(o) => {
            if !o.status.success() {
                eprintln!(
                    "[enhabitz] hyprctl stderr: {}",
                    String::from_utf8_lossy(&o.stderr)
                );
            }
            o.status.success()
        }
        Err(e) => {
            eprintln!("[enhabitz] hyprctl exec error: {}", e);
            false
        }
    }
}

#[cfg(target_os = "linux")]
fn hyprland_unbind(shortcut: &str) {
    let (mods, key) = tauri_to_hyprland(shortcut);
    let _ = std::process::Command::new("hyprctl")
        .args(["keyword", "unbind", &format!("{},{}", mods, key)])
        .status();
}

/// Spawn a background thread watching both Hyprland flag files.
#[cfg(target_os = "linux")]
fn start_hyprland_watcher(app: tauri::AppHandle) {
    let _ = std::fs::remove_file(HYPRLAND_WIDGET_FLAG);
    let _ = std::fs::remove_file(HYPRLAND_MAIN_FLAG);
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_millis(150));
        if std::fs::remove_file(HYPRLAND_WIDGET_FLAG).is_ok() {
            toggle_widget(&app);
        }
        if std::fs::remove_file(HYPRLAND_MAIN_FLAG).is_ok() {
            toggle_main(&app);
        }
    });
}

// ─── macOS dock visibility ────────────────────────────────────────────────────

#[cfg(target_os = "macos")]
#[allow(unexpected_cfgs)]
fn set_dock_visible(visible: bool) {
    use objc::{class, msg_send, runtime::Object, sel, sel_impl};
    let policy: i64 = if visible { 0 } else { 1 };
    unsafe {
        let ns_app: *mut Object = msg_send![class!(NSApplication), sharedApplication];
        let _: () = msg_send![ns_app, setActivationPolicy: policy];
        if visible {
            let _: () = msg_send![ns_app, activateIgnoringOtherApps: true];
        }
    }
}

// ─── Window helpers ───────────────────────────────────────────────────────────

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

fn toggle_main(app: &tauri::AppHandle) {
    let Some(main) = app.get_webview_window("main") else {
        return;
    };
    if main.is_visible().unwrap_or(false) {
        let _ = main.hide();
        #[cfg(target_os = "macos")]
        set_dock_visible(false);
    } else {
        #[cfg(target_os = "macos")]
        set_dock_visible(true);
        let _ = main.show();
        let _ = main.set_focus();
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

            // ── Global shortcuts ──────────────────────────────────────────────
            let widget_sc = read_shortcut(app.handle(), "widget");
            let main_sc = read_shortcut(app.handle(), "main");

            #[allow(unused_mut)]
            let mut widget_hypr = false;
            #[allow(unused_mut)]
            let mut main_hypr = false;

            #[cfg(target_os = "linux")]
            if is_hyprland() {
                widget_hypr = hyprland_bind(&widget_sc, HYPRLAND_WIDGET_FLAG);
                main_hypr = hyprland_bind(&main_sc, HYPRLAND_MAIN_FLAG);
                if widget_hypr || main_hypr {
                    start_hyprland_watcher(app.handle().clone());
                }
            }

            if !widget_hypr {
                if let Err(e) = register_plugin_shortcut(app.handle(), &widget_sc, "widget") {
                    eprintln!("[enhabitz] widget shortcut failed: {}", e);
                }
            }
            if !main_hypr {
                if let Err(e) = register_plugin_shortcut(app.handle(), &main_sc, "main") {
                    eprintln!("[enhabitz] main shortcut failed: {}", e);
                }
            }

            app.manage(HyprlandBackend {
                widget: Mutex::new(if widget_hypr { Some(widget_sc) } else { None }),
                main: Mutex::new(if main_hypr { Some(main_sc) } else { None }),
            });

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
