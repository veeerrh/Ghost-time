use serde::{Deserialize, Serialize};

use crate::db::queries;
use crate::state::AppState;

// ─── Data Types ────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct TimelineEntry {
    pub id: i64,
    pub timestamp: i64,
    pub app_name: String,
    pub window_title: String,
    pub duration_ms: i64,
    pub is_idle: bool,
    pub is_approved: bool,
    pub matter_id: Option<i64>,
    pub matter_code: Option<String>,
    pub client_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MatterInput {
    pub id: Option<i64>,        // None = insert, Some = update
    pub code: String,
    pub client_name: String,
    pub keywords: Vec<String>,
    pub rate_cents: i64,
}

// ─── IPC Commands ──────────────────────────────────────────

#[tauri::command]
pub async fn get_daily_summary(
    date: String,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<TimelineEntry>, String> {
    // Check Cache
    {
        let cache = state.summary_cache.lock().map_err(|e| e.to_string())?;
        if let Some(cached) = cache.get(&date) {
            if cached.timestamp.elapsed().as_secs() < 30 {
                return Ok(cached.entries.clone());
            }
        }
    }

    let db = state.db.lock().map_err(|e| e.to_string())?;

    // Parse date string (YYYY-MM-DD) into start/end timestamps (ms)
    let start = chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid date: {}", e))?;
    let start_ms = start
        .and_hms_opt(0, 0, 0).unwrap()
        .and_utc()
        .timestamp_millis();
    let end_ms = start_ms + 86_400_000; // +24h

    let mut stmt = db
        .prepare(queries::GET_DAILY_SUMMARY)
        .map_err(|e| e.to_string())?;

    let entries = stmt
        .query_map(rusqlite::params![start_ms, end_ms], |row| {
            Ok(TimelineEntry {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                app_name: row.get(2)?,
                window_title: row.get(3)?,
                duration_ms: row.get(4)?,
                is_idle: row.get::<_, i32>(5)? == 1,
                is_approved: row.get::<_, i32>(6)? == 1,
                matter_id: row.get(7)?,
                matter_code: row.get(8)?,
                client_name: row.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // Update Cache
    {
        let mut cache = state.summary_cache.lock().map_err(|e| e.to_string())?;
        cache.insert(date, crate::state::CachedSummary {
            entries: entries.clone(),
            timestamp: std::time::Instant::now(),
        });
    }

    Ok(entries)
}

#[tauri::command]
pub async fn approve_entry(
    entry_id: i64,
    duration_override_ms: Option<i64>,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute(
        queries::APPROVE_ENTRY,
        rusqlite::params![entry_id, duration_override_ms],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn upsert_matter(
    matter: MatterInput,
    state: tauri::State<'_, AppState>,
) -> Result<i64, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let keywords_json = serde_json::to_string(&matter.keywords)
        .map_err(|e| e.to_string())?;

    match matter.id {
        Some(id) => {
            db.execute(
                queries::UPSERT_MATTER_UPDATE,
                rusqlite::params![id, matter.code, matter.client_name, keywords_json, matter.rate_cents],
            )
            .map_err(|e| e.to_string())?;
            Ok(id)
        }
        None => {
            db.execute(
                queries::UPSERT_MATTER_INSERT,
                rusqlite::params![matter.code, matter.client_name, keywords_json, matter.rate_cents],
            )
            .map_err(|e| e.to_string())?;
            Ok(db.last_insert_rowid())
        }
    }
}

#[tauri::command]
pub async fn export_timesheet(
    date_range: (String, String),
    format: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let start = chrono::NaiveDate::parse_from_str(&date_range.0, "%Y-%m-%d")
        .map_err(|e| format!("Invalid start date: {}", e))?;
    let end = chrono::NaiveDate::parse_from_str(&date_range.1, "%Y-%m-%d")
        .map_err(|e| format!("Invalid end date: {}", e))?;

    let start_ms = start.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp_millis();
    let end_ms = end.and_hms_opt(23, 59, 59).unwrap().and_utc().timestamp_millis() + 999;

    let mut stmt = db
        .prepare(queries::EXPORT_TIMESHEET)
        .map_err(|e| e.to_string())?;

    let rows: Vec<ExportRow> = stmt
        .query_map(rusqlite::params![start_ms, end_ms], |row| {
            Ok(ExportRow {
                timestamp: row.get(0)?,
                app_name: row.get(1)?,
                window_title: row.get(2)?,
                duration_ms: row.get(3)?,
                is_idle: row.get::<_, i32>(4)? == 1,
                matter_code: row.get::<_, Option<String>>(5)?.unwrap_or_default(),
                client_name: row.get::<_, Option<String>>(6)?.unwrap_or_default(),
                rate_cents: row.get::<_, Option<i64>>(7)?.unwrap_or(0),
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let export_dir = std::env::temp_dir().join("ghost-time-exports");
    std::fs::create_dir_all(&export_dir).map_err(|e| e.to_string())?;

    match format.as_str() {
        "csv" => export_csv(&export_dir, &date_range, &rows),
        "pdf" => export_pdf(&export_dir, &date_range, &rows),
        _ => Err(format!("Unsupported format: {}. Use 'csv' or 'pdf'.", format)),
    }
}

// ─── Export Helpers ────────────────────────────────────────

fn export_pdf(
    dir: &std::path::Path,
    date_range: &(String, String),
    rows: &[ExportRow],
) -> Result<String, String> {
    use printpdf::*;
    let filename = format!("ghost-time_{}_{}.pdf", date_range.0, date_range.1);
    let path = dir.join(&filename);

    let (doc, page1, layer1) = PdfDocument::new("Ghost-Time Timesheet", Mm(210.0), Mm(297.0), "Layer 1");
    let current_layer = doc.get_page(page1).get_layer(layer1);

    // Title
    let font = doc.add_builtin_font(BuiltinFont::HelveticaBold).map_err(|e| e.to_string())?;
    current_layer.use_text("Ghost-Time Timesheet", 18.0, Mm(20.0), Mm(270.0), &font);
    
    // Date Range
    let font_reg = doc.add_builtin_font(BuiltinFont::Helvetica).map_err(|e| e.to_string())?;
    current_layer.use_text(format!("Range: {} to {}", date_range.0, date_range.1), 12.0, Mm(20.0), Mm(260.0), &font_reg);

    // Table Header
    let mut y = 245.0;
    current_layer.use_text("Date", 10.0, Mm(20.0), Mm(y), &font);
    current_layer.use_text("Matter", 10.0, Mm(50.0), Mm(y), &font);
    current_layer.use_text("App", 10.0, Mm(80.0), Mm(y), &font);
    current_layer.use_text("Duration", 10.0, Mm(130.0), Mm(y), &font);
    current_layer.use_text("Amount", 10.0, Mm(160.0), Mm(y), &font);

    y -= 10.0;

    for row in rows.iter().take(20) { // Limit to 20 for now to avoid page overflow logic
        let dt = chrono::DateTime::from_timestamp_millis(row.timestamp).unwrap_or_default();
        let duration_min = row.duration_ms as f64 / 60000.0;
        let amount = (duration_min / 60.0) * (row.rate_cents as f64 / 100.0);

        current_layer.use_text(dt.format("%Y-%m-%d").to_string(), 9.0, Mm(20.0), Mm(y), &font_reg);
        current_layer.use_text(&row.matter_code, 9.0, Mm(50.0), Mm(y), &font_reg);
        
        // Truncate app name
        let app = if row.app_name.len() > 15 { format!("{}...", &row.app_name[..12]) } else { row.app_name.clone() };
        current_layer.use_text(app, 9.0, Mm(80.0), Mm(y), &font_reg);
        
        current_layer.use_text(format!("{:.1}m", duration_min), 9.0, Mm(130.0), Mm(y), &font_reg);
        current_layer.use_text(format!("${:.2}", amount), 9.0, Mm(160.0), Mm(y), &font_reg);
        
        y -= 7.0;
    }

    let file = std::fs::File::create(&path).map_err(|e| e.to_string())?;
    let mut buf = std::io::BufWriter::new(file);
    doc.save(&mut buf).map_err(|e| e.to_string())?;

    Ok(path.to_string_lossy().to_string())
}

#[derive(Debug)]
struct ExportRow {
    timestamp: i64,
    app_name: String,
    window_title: String,
    duration_ms: i64,
    is_idle: bool,
    matter_code: String,
    client_name: String,
    rate_cents: i64,
}

fn export_csv(
    dir: &std::path::Path,
    date_range: &(String, String),
    rows: &[ExportRow],
) -> Result<String, String> {
    let filename = format!("ghost-time_{}_{}.csv", date_range.0, date_range.1);
    let path = dir.join(&filename);

    // Write UTF-8 BOM first so Excel on Windows detects encoding correctly
    use std::io::Write;
    let file = std::fs::File::create(&path).map_err(|e| e.to_string())?;
    let mut buf = std::io::BufWriter::new(file);
    buf.write_all(b"\xEF\xBB\xBF").map_err(|e| e.to_string())?;

    let mut wtr = csv::Writer::from_writer(buf);

    wtr.write_record([
        "Date", "Time", "App", "Window Title", "Duration (min)",
        "Idle", "Matter", "Client", "Rate ($/hr)", "Amount ($)",
    ])
    .map_err(|e| e.to_string())?;

    for row in rows {
        let dt = chrono::DateTime::from_timestamp_millis(row.timestamp)
            .unwrap_or_default();
        let duration_min = row.duration_ms as f64 / 60_000.0;
        let rate_hr = row.rate_cents as f64 / 100.0;
        let amount = duration_min / 60.0 * rate_hr;

        // Sanitize title: replace non-ASCII chars to avoid garbled output
        let safe_title: String = row.window_title.chars()
            .map(|c| if c.is_ascii() { c } else { '?' })
            .collect();

        wtr.write_record(&[
            dt.format("%Y-%m-%d").to_string(),
            dt.format("%H:%M:%S").to_string(),
            row.app_name.clone(),
            safe_title,
            format!("{:.2}", duration_min),
            if row.is_idle { "Yes" } else { "No" }.to_string(),
            row.matter_code.clone(),
            row.client_name.clone(),
            format!("{:.2}", rate_hr),
            format!("{:.2}", amount),
        ])
        .map_err(|e| e.to_string())?;
    }

    wtr.flush().map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}
