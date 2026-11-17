use bevy::prelude::*;

use super::MeleeWeapon;
use crate::prelude::*;

/// Present on a melee weapon while it swings, for as long as its [`ActionUseStyle::Timed`] use.
#[derive(Component, Clone, Copy, Debug)]
pub(super) struct ActiveMeleeAttack {
    /// Comes from the direction the entity holding the weapon is aiming.
    pub(super) initial_angle: f32,
}

/// Spawns the swing's hitbox on the weapon for the duration of the use. Its damage is scaled by the
/// holder's stats at the time of the swing.
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
    let weapon = took_effect.source;
    commands.entity(weapon).insert(ActiveMeleeAttack {
        initial_angle: angle,
    });
    commands.spawn((
        HitBox::new(took_effect.holder, damage, Vec2::from_angle(angle)).with_source_item(weapon),
        melee_weapon.hitbox.clone(),
        side.hitbox_layers(),
        Transform::default(),
        ChildOf(weapon),
        SpawnedForUse { source: weapon },
    ));
    Ok(())
}

/// Stops the swing.
pub(super) fn on_melee_finished(finished: On<ActionFinished>, mut commands: Commands) {
    commands
        .entity(finished.source)
        .try_remove::<ActiveMeleeAttack>();
}
