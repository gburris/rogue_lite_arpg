//! [`Cooldowns`], when each motion and action may start again, and the keys that index it.

use super::action::{Ability, Action};
use super::motion::Motion;

/// A motion or action with its own entry in [`Cooldowns`].
pub(super) trait CooldownKey: Copy {
    fn cooldown_index(self) -> usize;
}

/// Motions come first in [`Cooldowns`].
impl CooldownKey for Motion {
    fn cooldown_index(self) -> usize {
        self.index()
    }
}

/// Actions follow the motions in [`Cooldowns`].
impl CooldownKey for Action {
    fn cooldown_index(self) -> usize {
        Motion::COUNT + self.index()
    }
}

impl CooldownKey for Ability {
    fn cooldown_index(self) -> usize {
        Action::Ability(self).cooldown_index()
    }
}

/// The virtual time at which each motion and action is off cooldown.
#[derive(Clone, Copy, Debug)]
pub(super) struct Cooldowns([f32; Motion::COUNT + Action::COUNT]);

impl Default for Cooldowns {
    fn default() -> Self {
        Self([0.0; Motion::COUNT + Action::COUNT])
    }
}

impl Cooldowns {
    pub(super) fn is_ready(&self, key: impl CooldownKey, now: f32) -> bool {
        now >= self.0[key.cooldown_index()]
    }

    pub(super) fn start(&mut self, key: impl CooldownKey, ready_at: f32) {
        self.0[key.cooldown_index()] = ready_at;
    }
}
