use rusqlite::Connection;
use std::sync::Mutex;
use std::collections::HashMap;
use std::time::Instant;

pub struct CachedSummary {
    pub entries: Vec<crate::commands::TimelineEntry>,
    pub timestamp: Instant,
}

pub struct AppState {
    pub db: Mutex<Connection>,
    pub summary_cache: Mutex<HashMap<String, CachedSummary>>,
}
