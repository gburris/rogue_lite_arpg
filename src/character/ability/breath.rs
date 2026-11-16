use bevy::prelude::*;

use super::{Ability, AbilityCooldown, AbilityOf, AbilityTookEffect};
use crate::prelude::*;

/// An innate projectile attack fired from its ability entity's [`Projectiles`] templates along the
/// holder's aim.
#[derive(Component, Clone, Copy, Debug)]
#[require(Ability::Breath, AbilityCooldown)]
pub struct Breath {
    reach: f32,
}

impl Breath {
    /// Breathes at targets within `reach`. The reach only tells a brain when to breathe; a
    /// projectile may fly past it.
    pub fn new(reach: f32) -> Self {
        Self { reach }
    }

    pub(super) fn reach(self) -> f32 {
        self.reach
    }
}

/// A breath ability that fires its [`Projectiles`] templates at targets within `reach`.
pub fn breath(reach: f32) -> impl Scene {
    bsn! {
        ~Breath::new(reach)
        on(on_breath_took_effect)
    }
}

/// Fires each projectile template along the holder's aim.
fn on_breath_took_effect(
    took_effect: On<AbilityTookEffect>,
    mut commands: Commands,
    breaths: Query<(&AbilityOf, &Projectiles), With<Breath>>,
    breathers: Query<(&Transform, &CombatSide, &Vision)>,
) {
    let Ok((ability_of, templates)) = breaths.get(took_effect.entity) else {
        return;
    };
    let holder = ability_of.holder;
    let Ok((transform, side, vision)) = breathers.get(holder) else {
        return;
    };
    let Some(aim_direction) = vision.aim_direction.try_normalize() else {
        return;
    };
    for template in templates.iter() {
        commands.trigger(FireProjectile {
            item: template,
            side: *side,
            holder: Some(holder),
            position: transform.translation.xy(),
            aim_direction,
        });
    }
}
