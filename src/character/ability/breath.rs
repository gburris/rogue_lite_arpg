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
        Attack
        on(on_breath_took_effect)
    }
}

/// Fires the breath's projectile templates from the holder along its aim.
fn on_breath_took_effect(
    took_effect: On<ActionTookEffect>,
    mut commands: Commands,
    breathers: Query<(&Transform, &Vision)>,
) {
    let Ok((transform, vision)) = breathers.get(took_effect.holder) else {
        return;
    };
    commands.trigger(FireProjectiles {
        source: took_effect.source,
        holder: took_effect.holder,
        position: transform.translation.xy(),
        aim_direction: vision.aim_direction,
    });
}
