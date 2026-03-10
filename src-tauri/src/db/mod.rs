use rusqlite::Connection;
use keyring::Entry;
use std::path::PathBuf;
use rand::RngCore;
use std::fs;

mod queries;

pub fn open_db(app_handle: &tauri::AppHandle) -> rusqlite::Result<Connection> {
    let key = get_or_create_key();
    let path = db_path(app_handle);
    open_db_with_key(&path, &key)
}

pub fn insert_window_log(
    conn: &Connection,
    timestamp: i64,
    app_name: &str,
    window_title: &str,
    duration_ms: i64,
    is_idle: bool,
    matter_id: Option<i64>,
) -> rusqlite::Result<()> {
    conn.execute(
        queries::INSERT_WINDOW_LOG,
        rusqlite::params![
            timestamp,
            app_name,
            window_title,
            duration_ms,
            if is_idle { 1 } else { 0 },
            matter_id
        ],
    )?;
    Ok(())
}

/// Opens (or creates) an encrypted SQLCipher database at the given path.
/// This is the core function used by both production and tests.
pub fn open_db_with_key(path: &PathBuf, key: &str) -> rusqlite::Result<Connection> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|_e| rusqlite::Error::InvalidPath(path.to_path_buf()))?;
    }

    let conn = Connection::open(path)?;

    // SQLCipher requires the key to be set immediately after opening
    conn.execute_batch(&format!(
        "PRAGMA key='{}'; PRAGMA cipher_page_size=4096;",
        key
    ))?;

    run_migrations(&conn)?;

    Ok(conn)
}

fn get_or_create_key() -> String {
    let entry = Entry::new("ghost-time", "db-key").unwrap();
    match entry.get_password() {
        Ok(key) => key,
        Err(_) => {
            let k = generate_key_256bit();
            entry.set_password(&k).expect("Failed to store key in OS keychain");
            k
        }
    }
}

pub fn generate_key_256bit() -> String {
    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    hex::encode(key)
}

fn db_path(app_handle: &tauri::AppHandle) -> PathBuf {
    use tauri::Manager;
    app_handle.path().app_data_dir().unwrap().join("ghost-time.db")
}

fn run_migrations(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(queries::CREATE_MATTERS_TABLE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_db_path(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("ghost-time-tests");
        fs::create_dir_all(&dir).unwrap();
        dir.join(format!("{}.db", name))
    }

    fn cleanup(path: &PathBuf) {
        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_db_file_is_encrypted() {
        let path = test_db_path("encrypted_check");
        cleanup(&path);
        let key = generate_key_256bit();

        // Create and populate the DB
        {
            let conn = open_db_with_key(&path, &key).unwrap();
            conn.execute(
                "INSERT INTO matters (code, client_name, keywords) VALUES (?1, ?2, ?3)",
                rusqlite::params!["TEST-001", "Test Client", "[\"test\"]"],
            ).unwrap();
        }

        // Read raw bytes — should NOT start with "SQLite format 3"
        let bytes = fs::read(&path).unwrap();
        let header = String::from_utf8_lossy(&bytes[..16]);
        assert!(
            !header.contains("SQLite format 3"),
            "DB file should be encrypted! Found plain SQLite header."
        );
        println!("✅ DB file is encrypted — no SQLite header visible");
        println!("   First 16 bytes (hex): {:02x?}", &bytes[..16]);

        cleanup(&path);
    }

    #[test]
    fn test_open_with_correct_key() {
        let path = test_db_path("correct_key");
        cleanup(&path);
        let key = generate_key_256bit();

        let conn = open_db_with_key(&path, &key).unwrap();
        let result: i64 = conn.query_row("SELECT 1", [], |row| row.get(0)).unwrap();
        assert_eq!(result, 1);
        println!("✅ Correct key opens DB: SELECT 1 returns Ok(1)");

        cleanup(&path);
    }

    #[test]
    fn test_open_with_wrong_key() {
        let path = test_db_path("wrong_key");
        cleanup(&path);
        let correct_key = generate_key_256bit();
        let wrong_key = generate_key_256bit();

        // Create DB with the correct key and write some data
        {
            let conn = open_db_with_key(&path, &correct_key).unwrap();
            conn.execute(
                "INSERT INTO matters (code, client_name, keywords) VALUES (?1, ?2, ?3)",
                rusqlite::params!["TEST-WK", "Wrong Key Test", "[\"test\"]"],
            ).unwrap();
        }

        // Try to open with the wrong key — PRAGMA key succeeds, but reading fails
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch(&format!(
            "PRAGMA key='{}'; PRAGMA cipher_page_size=4096;",
            wrong_key
        )).unwrap();

        // This is the standard SQLCipher test — reading sqlite_master requires decryption
        let result = conn.query_row(
            "SELECT count(*) FROM sqlite_master",
            [],
            |row| row.get::<_, i64>(0),
        );
        assert!(result.is_err(), "Wrong key should fail to read encrypted data");
        println!("✅ Wrong key is rejected: {:?}", result.err().unwrap());

        cleanup(&path);
    }

    #[test]
    fn test_insert_window_log() {
        let path = test_db_path("insert_log");
        cleanup(&path);
        let key = generate_key_256bit();

        let conn = open_db_with_key(&path, &key).unwrap();

        // Insert a window log entry
        conn.execute(
            "INSERT INTO window_log (timestamp, app_name, window_title, duration_ms, is_idle)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                1710000000000i64,  // Unix ms
                "Code.exe",
                "main.rs - Ghost-time",
                5000,              // 5 seconds
                0                  // not idle
            ],
        ).unwrap();

        // Verify the row exists
        let (app, title, duration): (String, String, i64) = conn.query_row(
            "SELECT app_name, window_title, duration_ms FROM window_log WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        ).unwrap();

        assert_eq!(app, "Code.exe");
        assert_eq!(title, "main.rs - Ghost-time");
        assert_eq!(duration, 5000);
        println!("✅ Window log insert works: {} | {} | {}ms", app, title, duration);

        cleanup(&path);
    }
}
