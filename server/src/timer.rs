use serde_derive::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timer {
    start_time: SystemTime,
}

impl Default for Timer {
    fn default() -> Self {
        Timer {
            start_time: SystemTime::now(),
        }
    }
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            start_time: SystemTime::now(),
        }
    }
    pub fn restart(&mut self) {
        self.start_time = SystemTime::now();
    }
    pub fn elapsed(&self) -> u64 {
        let now = SystemTime::now();
        if let Ok(elapsed) = now.duration_since(self.start_time) {
            elapsed.as_millis() as u64
        } else {
            0 // Handle the case where time goes backwards (unlikely)
        }
    }
}
