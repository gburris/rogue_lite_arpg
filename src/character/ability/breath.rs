use bevy::prelude::*;

use super::Ability;
use crate::prelude::*;

/// An innate projectile attack fired from its ability entity's [`Projectiles`] templates along the
/// holder's aim.
#[derive(Component, Clone, Copy, Debug, Default)]
#[require(Ability::Breath)]
pub struct Breath;

/// A breath ability that fires its [`Projectiles`] templates at targets within `reach`. The reach
/// only tells a brain when to breathe; a projectile may fly past it.
pub fn breath(reach: f32) -> impl Scene {
    bsn! {
        Breath
        ~{ActionReach(reach)}
        on(on_breath_took_effect)
    }
}

/// Fires each projectile template along the holder's aim.
fn on_breath_took_effect(
    took_effect: On<ActionTookEffect>,
    mut commands: Commands,
    breaths: Query<&Projectiles, With<Breath>>,
    breathers: Query<(&Transform, &CombatSide, &Vision)>,
) {
    let Ok(templates) = breaths.get(took_effect.entity) else {
        return;
    };
    let holder = took_effect.holder;
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
