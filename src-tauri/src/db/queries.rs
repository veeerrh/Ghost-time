pub const CREATE_MATTERS_TABLE: &str = include_str!("schema.sql");

pub const INSERT_WINDOW_LOG: &str = "
    INSERT INTO window_log (timestamp, app_name, window_title, duration_ms, is_idle, matter_id)
    VALUES (?1, ?2, ?3, ?4, ?5, ?6)
";

pub const GET_ALL_MATTERS: &str = "
    SELECT id, code, keywords FROM matters WHERE is_active = 1
";

pub const GET_DAILY_SUMMARY: &str = "
    SELECT
        wl.id, wl.timestamp, wl.app_name, wl.window_title,
        wl.duration_ms, wl.is_idle, wl.is_approved,
        wl.matter_id, m.code AS matter_code, m.client_name
    FROM window_log wl
    LEFT JOIN matters m ON wl.matter_id = m.id
    WHERE wl.timestamp >= ?1 AND wl.timestamp < ?2
    ORDER BY wl.timestamp ASC
";

pub const APPROVE_ENTRY: &str = "
    UPDATE window_log SET is_approved = 1, duration_ms = COALESCE(?2, duration_ms)
    WHERE id = ?1
";

pub const UPSERT_MATTER_INSERT: &str = "
    INSERT INTO matters (code, client_name, keywords, rate_cents)
    VALUES (?1, ?2, ?3, ?4)
";

pub const UPSERT_MATTER_UPDATE: &str = "
    UPDATE matters SET code = ?2, client_name = ?3, keywords = ?4, rate_cents = ?5
    WHERE id = ?1
";

pub const EXPORT_TIMESHEET: &str = "
    SELECT
        wl.timestamp, wl.app_name, wl.window_title,
        wl.duration_ms, wl.is_idle,
        m.code AS matter_code, m.client_name, m.rate_cents
    FROM window_log wl
    LEFT JOIN matters m ON wl.matter_id = m.id
    WHERE wl.is_approved = 1
      AND wl.timestamp >= ?1 AND wl.timestamp < ?2
    ORDER BY wl.timestamp ASC
";
