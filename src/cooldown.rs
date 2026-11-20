use bevy::prelude::*;

use crate::interval::Spread;

/// A wait after each use of an action or motion source before the next use can start, measured in
/// virtual time. The source's owner decides when a use starts the wait.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Cooldown {
    seconds: f32,
    jitter: Spread,
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

    /// Varies each wait by up to `jitter` seconds either way, so cooldowns started together do not
    /// stay in sync.
    pub fn with_jitter(self, jitter: f32) -> Self {
        Self {
            jitter: Spread::new(jitter),
            ..self
        }
    }

    /// Seconds left at virtual time `now` before the next use can start.
    pub fn remaining_seconds(&self, now: f32) -> f32 {
        (self.ready_at - now).max(0.0)
    }

    /// Whether the wait is over at virtual time `now`.
    pub fn is_ready(&self, now: f32) -> bool {
        now >= self.ready_at
    }

    /// Starts a wait at virtual time `now`.
    pub fn start(&mut self, now: f32) {
        let offset = self.jitter.pick_random_value();
        self.ready_at = now + (self.seconds + offset).max(0.0);
    }

    /// Delays readiness until at least `seconds` after `now`, keeping any longer wait.
    pub fn lock_out(&mut self, now: f32, seconds: f32) {
        self.ready_at = self.ready_at.max(now + seconds);
    }
}
