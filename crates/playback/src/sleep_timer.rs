use std::time::{Duration, Instant};

pub struct SleepTimer {
    target_time: Option<Instant>,
    fade_duration: Duration,
}

impl SleepTimer {
    pub fn new() -> Self {
        Self {
            target_time: None,
            fade_duration: Duration::from_secs(20),
        }
    }

    pub fn set_minutes(&mut self, minutes: u32) {
        if minutes == 0 {
            self.target_time = None;
        } else {
            self.target_time = Some(Instant::now() + Duration::from_secs(minutes as u64 * 60));
        }
    }

    pub fn cancel(&mut self) {
        self.target_time = None;
    }

    pub fn is_active(&self) -> bool {
        self.target_time.is_some()
    }

    pub fn remaining_seconds(&self) -> Option<u64> {
        self.target_time.and_then(|t| {
            let now = Instant::now();
            if now >= t {
                Some(0)
            } else {
                Some((t - now).as_secs())
            }
        })
    }

    pub fn check_tick(&self) -> (bool, f32) {
        if let Some(target) = self.target_time {
            let now = Instant::now();
            if now >= target {
                return (true, 0.0);
            }

            let rem = target - now;
            if rem < self.fade_duration {
                let ratio = rem.as_secs_f32() / self.fade_duration.as_secs_f32();
                return (false, ratio.clamp(0.0, 1.0));
            }
        }
        (false, 1.0)
    }
}

impl Default for SleepTimer {
    fn default() -> Self {
        Self::new()
    }
}
