use std::time::{Duration, Instant};
use device_query::{DeviceQuery, DeviceState};

pub struct IdleDetector {
    device_state: DeviceState,
    last_pos: (i32, i32),
    last_keys: Vec<device_query::Keycode>,
    last_activity: Instant,
    idle_threshold: Duration,
    is_idle: bool,
}

impl IdleDetector {
    pub fn new() -> Self {
        Self::with_threshold(Duration::from_secs(300)) // 5 minutes default
    }

    pub fn with_threshold(threshold: Duration) -> Self {
        let device_state = DeviceState::new();
        let mouse = device_state.get_mouse();
        let keys = device_state.get_keys();
        Self {
            device_state,
            last_pos: mouse.coords,
            last_keys: keys,
            last_activity: Instant::now(),
            idle_threshold: threshold,
            is_idle: false,
        }
    }

    /// Returns true if user has been idle for longer than the threshold.
    /// Also detects transitions: idle -> active and active -> idle.
    pub fn check(&mut self) -> IdleStatus {
        let mouse = self.device_state.get_mouse();
        let keys = self.device_state.get_keys();

        let mouse_moved = mouse.coords != self.last_pos;
        let keys_changed = keys != self.last_keys;

        if mouse_moved || keys_changed {
            self.last_pos = mouse.coords;
            self.last_keys = keys;
            self.last_activity = Instant::now();

            if self.is_idle {
                self.is_idle = false;
                return IdleStatus::ResumedActivity;
            }
            return IdleStatus::Active;
        }

        if !self.is_idle && self.last_activity.elapsed() > self.idle_threshold {
            self.is_idle = true;
            return IdleStatus::BecameIdle;
        }

        if self.is_idle {
            IdleStatus::StillIdle
        } else {
            IdleStatus::Active
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum IdleStatus {
    /// User is actively using the computer
    Active,
    /// User just went idle (crossed the threshold)
    BecameIdle,
    /// User is still idle
    StillIdle,
    /// User just came back from idle
    ResumedActivity,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_idle_detection_with_short_threshold() {
        // Use a 2-second threshold for testing instead of 5 minutes
        let mut detector = IdleDetector::with_threshold(Duration::from_secs(2));

        // First check should be Active (we just created it)
        let status = detector.check();
        println!("  Initial status: {:?}", status);
        assert!(
            status == IdleStatus::Active || status == IdleStatus::ResumedActivity,
            "Initial status should be Active"
        );

        // Wait beyond the threshold without any input
        println!("  Waiting 3 seconds for idle threshold...");
        thread::sleep(Duration::from_secs(3));

        let status = detector.check();
        println!("  Status after 3s wait: {:?}", status);
        assert_eq!(status, IdleStatus::BecameIdle, "Should detect idle after threshold");
        println!("✅ Idle detection triggers correctly after threshold");

        // Subsequent check should be StillIdle
        let status = detector.check();
        println!("  Status on next check: {:?}", status);
        assert_eq!(status, IdleStatus::StillIdle, "Should remain StillIdle");
        println!("✅ StillIdle state maintained correctly");
    }

    #[test]
    fn test_idle_detector_initializes() {
        let detector = IdleDetector::new();
        println!("✅ IdleDetector created with 5-min threshold");
        assert!(!detector.is_idle); // Should start as active
    }
}
