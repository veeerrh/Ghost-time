mod hook;
mod db;
mod classifier;
mod commands;
mod state;

use tauri::Manager;
use state::AppState;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      // Initialize Database
      let db_conn = db::open_db(app.handle())?;
      println!("[DB] Vault successfully opened and decrypted with OS Keychain");

      // Create a second connection for the hook (runs in background)
      let hook_conn = db::open_db(app.handle())?;
      
      // Load rules for classifier
      let rules = vec![]; // TODO: Load from DB in future step

      // Manage AppState for IPC commands
      app.manage(AppState {
          db: Mutex::new(db_conn),
          summary_cache: Mutex::new(std::collections::HashMap::new()),
      });

      tauri::async_runtime::spawn(async move {
        hook::manager::start_hook(hook_conn, rules).await;
      });

      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
        commands::get_daily_summary,
        commands::approve_entry,
        commands::upsert_matter,
        commands::export_timesheet,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
