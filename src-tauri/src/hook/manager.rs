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

pub async fn start_hook() {
    let mut last_title = String::new();
    let mut current_record: Option<WindowRecord> = None;
    let mut idle_detector = IdleDetector::new();

    println!("[HOOK] Window logger started (500ms poll, 5min idle threshold)");

    loop {
        let idle_status = idle_detector.check();

        match idle_status {
            IdleStatus::BecameIdle => {
                // User just went idle — flush current record and insert IDLE entry
                if let Some(mut record) = current_record.take() {
                    flush_record(&mut record);
                }
                println!("[HOOK] IDLE detected — pausing time accumulation");
                last_title.clear();
            }
            IdleStatus::StillIdle => {
                // Do nothing while idle
            }
            IdleStatus::ResumedActivity | IdleStatus::Active => {
                if idle_status == IdleStatus::ResumedActivity {
                    println!("[HOOK] Activity resumed");
                }

                if let Some((title, app_name)) = get_active_window() {
                    if title != last_title {
                        let now = now_secs();

                        // Flush previous record
                        if let Some(mut record) = current_record.take() {
                            flush_record(&mut record);
                        }

                        // Start new record
                        current_record = Some(WindowRecord {
                            timestamp: now,
                            app_name: app_name.clone(),
                            window_title: title.clone(),
                            start_time: now,
                            duration: 0,
                        });

                        last_title = title;
                    }
                }
            }
        }

        sleep(Duration::from_millis(500)).await;
    }
}
