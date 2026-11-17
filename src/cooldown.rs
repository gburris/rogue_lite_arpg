use rand::RngExt;

/// How long something waits after each use before it can be used again, measured on the virtual
/// clock.
#[derive(Clone, Copy, Debug, Default)]
pub struct Cooldown {
    seconds: f32,
    jitter: f32,
    ready_at: f32,
}

impl Cooldown {
    /// Waits `seconds` after each use.
    pub fn new(seconds: f32) -> Self {
        Self {
            seconds,
            ..Self::default()
        }
    }

    /// Varies each wait by up to `jitter` seconds either way, so things that start being used
    /// together drift out of step.
    pub fn with_jitter(self, jitter: f32) -> Self {
        Self { jitter, ..self }
    }

    /// Seconds left at virtual time `now` before the next use can start.
    pub fn remaining_seconds(&self, now: f32) -> f32 {
        (self.ready_at - now).max(0.0)
    }

    /// Whether the wait has run out at virtual time `now`.
    pub fn is_ready(&self, now: f32) -> bool {
        now >= self.ready_at
    }

    /// Starts a wait at virtual time `now`.
    pub fn start(&mut self, now: f32) {
        let offset = if self.jitter > 0.0 {
            rand::rng().random_range(-self.jitter..=self.jitter)
        } else {
            0.0
        };
        self.ready_at = now + (self.seconds + offset).max(0.0);
    }

    /// Keeps it from being ready for at least `seconds` after `now`, on top of any wait already
    /// running.
    pub fn lock_out(&mut self, now: f32, seconds: f32) {
        self.ready_at = self.ready_at.max(now + seconds);
    }
}
