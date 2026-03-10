mod hook;
mod db;
mod classifier;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      // Initialize Database
      let db_conn = db::open_db(app.handle())?;
      println!("[DB] Vault successfully opened and decrypted with OS Keychain");
      
      // Load rules for classifier
      let rules = vec![]; // TODO: Load from DB in future step

      tauri::async_runtime::spawn(async move {
        hook::manager::start_hook(db_conn, rules).await;
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
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
