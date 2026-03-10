-- Run as the DB initializer on first launch
 
CREATE TABLE IF NOT EXISTS matters (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    code        TEXT NOT NULL UNIQUE,   -- e.g. 'SMITH-990'
    client_name TEXT NOT NULL,
    keywords    TEXT NOT NULL,          -- JSON array: ['Smith', 'Smith v Doe']
    rate_cents  INTEGER DEFAULT 0,      -- hourly rate in cents (avoids float errors)
    is_active   INTEGER DEFAULT 1
);
 
CREATE TABLE IF NOT EXISTS window_log (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp    INTEGER NOT NULL,     -- Unix timestamp in milliseconds
    app_name     TEXT NOT NULL,
    window_title TEXT NOT NULL,
    duration_ms  INTEGER NOT NULL,
    is_idle      INTEGER DEFAULT 0,    -- 1 if this is an IDLE entry
    matter_id    INTEGER REFERENCES matters(id),  -- NULL = unclassified
    is_approved  INTEGER DEFAULT 0     -- 1 = lawyer approved for billing
);
 
CREATE INDEX IF NOT EXISTS idx_window_log_timestamp ON window_log(timestamp);
CREATE INDEX IF NOT EXISTS idx_window_log_matter    ON window_log(matter_id);
CREATE INDEX IF NOT EXISTS idx_window_log_composite ON window_log(timestamp, matter_id);
