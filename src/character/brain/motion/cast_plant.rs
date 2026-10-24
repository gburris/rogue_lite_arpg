//! Brief plant while a caster's wind-up reads.

use bevy::prelude::*;

/// Brief plant while a caster's equipment is in use. Opt-in at spawn; absent
/// means the character keeps moving through its cast.
///
/// The plant is measured from the start of the attack, so a cast shorter than
/// `hold` leaves a readable recovery beat instead of sliding straight into the
/// next move.
#[derive(Component, Clone, Copy, Debug)]
pub struct CastPlant {
    pub hold: f32,
    hold_until: f32,
    was_attacking: bool,
}

impl CastPlant {
    /// Plants for `hold` seconds starting when the caster begins an attack.
    pub fn new(hold: f32) -> Self {
        Self {
            hold,
            hold_until: 0.0,
            was_attacking: false,
        }
    }

    /// Returns whether the caster should stay planted this frame.
    pub(super) fn update(&mut self, attacking: bool, now: f32) -> bool {
        if attacking && !self.was_attacking {
            self.hold_until = now + self.hold;
        }
        self.was_attacking = attacking;
        attacking || now < self.hold_until
    }
}
