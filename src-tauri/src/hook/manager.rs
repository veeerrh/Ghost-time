use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;
use crate::hook::windows::get_active_window;
use crate::hook::idle::{IdleDetector, IdleStatus};

#[derive(Debug, Clone)]
pub struct WindowRecord {
    pub timestamp: u64,
    pub app_name: String,
    pub window_title: String,
    pub start_time: u64,
    pub duration: u64,
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn flush_record(record: &mut WindowRecord) {
    record.duration = now_secs() - record.start_time;
    println!(
        "[HOOK] Window: {} | App: {} | Duration: {}s",
        record.window_title, record.app_name, record.duration
    );
    // TODO: Record to DB
}

pub async fn start_hook(conn: Connection, rules: Vec<Rule>) {
    let mut current: Option<WindowRecord> = None;
    let mut idle_detector = IdleDetector::new();
    let mut interval = time::interval(Duration::from_millis(500));

    println!("[HOOK] Window logger started (500ms poll, 5min idle threshold)");

    loop {
        interval.tick().await;

        match idle_detector.check() {
            IdleStatus::BecameIdle => {
                if let Some(mut rec) = current.take() {
                    rec.duration = rec.start_time.elapsed().as_millis() as u64;
                    persist_record(&conn, &rules, rec, true);
                }
            }
            IdleStatus::StillIdle => {}
            IdleStatus::ResumedActivity | IdleStatus::Active => {
                if let Some((app_name, window_title)) = get_active_window() {
                    let should_switch = current.as_ref().map_or(true, |c| {
                        c.app_name != app_name || c.window_title != window_title
                    });

                    if should_switch {
                        if let Some(mut old_rec) = current.take() {
                            old_rec.duration = old_rec.start_time.elapsed().as_millis() as u64;
                            persist_record(&conn, &rules, old_rec, false);
                        }
                        current = Some(WindowRecord {
                            timestamp: SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_millis() as u64,
                            app_name: app_name,
                            window_title: window_title,
                            start_time: Instant::now(),
                            duration: 0,
                        });
                    }
                }
            }
        }
    }
}

fn persist_record(conn: &Connection, rules: &[Rule], rec: WindowRecord, is_idle: bool) {
    let matter_id = classifier::classify(&rec.window_title, rules);
    let _ = db::insert_window_log(
        conn,
        rec.timestamp as i64,
        &rec.app_name,
        &rec.window_title,
        rec.duration as i64,
        is_idle,
        matter_id,
    );
    
    let tag = matter_id.map_or("Unclassified".to_string(), |id| format!("Matter #{}", id));
    println!(
        "[{}] {} | {} | {}ms | {}",
        if is_idle { "IDLE" } else { "LOG" },
        rec.app_name,
        rec.window_title,
        rec.duration,
        tag
    );
}
