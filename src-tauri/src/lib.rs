mod db;

use db::{DayEntry, Db, HabitData, HabitRecord, LogData};
use std::sync::Mutex;
use tauri::Manager;

pub struct DbState(pub Mutex<Db>);

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

// ─── App setup ────────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let db = Db::new(&data_dir.join("enhabitz.db"))
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
            app.manage(DbState(Mutex::new(db)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_habits,
            add_habit,
            update_habit,
            delete_habit,
            save_log,
            load_log_history,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
