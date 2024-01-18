use serde_derive::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timer {
    start_time: SystemTime,
    paused_duration: Option<Duration>,
}

impl Default for Timer {
    fn default() -> Self {
        Timer {
            start_time: SystemTime::now(),
            paused_duration: None,
        }
    }
}

impl Timer {
    pub fn new() -> Timer {
        Timer::default()
    }

    pub fn restart(&mut self) {
        self.start_time = SystemTime::now();
        self.paused_duration = None;
    }

    pub fn pause(&mut self) {
        if self.paused_duration.is_none() {
            self.paused_duration = Some(self.elapsed_duration());
        }
    }

    pub fn resume(&mut self) {
        if let Some(paused_duration) = self.paused_duration.take() {
            self.start_time = SystemTime::now() - paused_duration;
        }
    }

    pub fn elapsed(&self) -> u64 {
        self.elapsed_duration().as_millis() as u64
    }

    fn elapsed_duration(&self) -> Duration {
        match self.paused_duration {
            Some(paused_duration) => paused_duration,
            None => self
                .start_time
                .elapsed()
                .expect("SystemTime before UNIX EPOCH!"),
        }
    }
}
