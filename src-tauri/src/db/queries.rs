pub const CREATE_MATTERS_TABLE: &str = include_str!("schema.sql");

pub const INSERT_WINDOW_LOG: &str = "
    INSERT INTO window_log (timestamp, app_name, window_title, duration_ms, is_idle, matter_id)
    VALUES (?1, ?2, ?3, ?4, ?5, ?6)
";

pub const GET_ALL_MATTERS: &str = "
    SELECT id, code, keywords FROM matters WHERE is_active = 1
";
