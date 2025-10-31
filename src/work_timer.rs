use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
pub struct WorkTimer {
    last_tick: Option<Instant>,
    work_duration_remaining: Duration
}

impl WorkTimer {
    pub fn new(work_duration: Duration) -> Self {
        Self {
            last_tick: Some(Instant::now()),
            work_duration_remaining: work_duration
        }
    }

    pub fn duration_remaning(&self) -> Duration {
        self.work_duration_remaining
    }

    pub fn is_paused(&self) -> bool {
        self.last_tick.is_none()
    }

    pub fn toggle_pause(&mut self) {
        self.last_tick = match self.last_tick {
            Some(tick) => {
                self.work_duration_remaining =
                    self.work_duration_remaining.saturating_sub(tick.elapsed());
                None
            }
            None => Some(Instant::now())
        }
    }

    pub fn on_tick(&mut self) {
        if let Some(tick) = self.last_tick {
            let now = Instant::now();
            self.work_duration_remaining = self
                .work_duration_remaining
                .saturating_sub(now.duration_since(tick));
            self.last_tick = Some(now);
        }
    }
}
