use avian2d::prelude::*;
use bevy::prelude::*;

use super::MeleeWeapon;
use crate::prelude::*;

/// Present on a melee weapon while it swings, for as long as its [`ActionUseStyle::Timed`] use.
#[derive(Component, Clone, Copy, Debug)]
pub(super) struct ActiveMeleeAttack {
    /// Comes from the direction the entity holding the weapon is aiming.
    pub(super) initial_angle: f32,
}

/// Spawns the transient swing hitbox removed by `on_melee_finished`, dealing damage scaled by the
/// holder's stats at the moment of the swing.
pub(super) fn on_melee_took_effect(
    took_effect: On<ActionTookEffect>,
    mut commands: Commands,
    weapon_query: Query<&MeleeWeapon>,
    holder_query: Query<(&Vision, &CombatSide, &StatMultipliers)>,
) -> Result {
    let melee_weapon = weapon_query.get(took_effect.source)?;
    let Ok((vision, side, stat_multipliers)) = holder_query.get(took_effect.holder) else {
        return Ok(());
    };
    let angle = vision.aim_direction.to_angle();
    let damage = melee_weapon.damage.scaled(stat_multipliers.damage);
    // Hitbox and layers only exist for the swing; the finish observer removes them.
    commands.entity(took_effect.source).insert((
        ActiveMeleeAttack {
            initial_angle: angle,
        },
        HitBox::new(took_effect.holder, damage, Vec2::from_angle(angle)),
        melee_weapon.hitbox.clone(),
        side.hitbox_layers(),
    ));
    Ok(())
}

/// Removes the transient swing state.
pub(super) fn on_melee_finished(finished: On<ActionFinished>, mut commands: Commands) {
    commands.entity(finished.source).try_remove::<(
        ActiveMeleeAttack,
        HitBox,
        Collider,
        CollisionLayers,
        CollidingEntities,
    )>();
}
