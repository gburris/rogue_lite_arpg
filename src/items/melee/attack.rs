use avian2d::prelude::*;
use bevy::prelude::*;

use super::MeleeWeapon;
use crate::prelude::*;

#[derive(Component)]
pub struct ActiveMeleeAttack {
    /// Comes from the direction the entity holding the weapon is aiming.
    pub(super) initial_angle: f32,
    /// Comes from "`attack_time`" defined on `MeleeWeapon`.
    pub(super) duration: Timer,
}

impl ActiveMeleeAttack {
    /// A swing along `initial_angle` lasting `seconds`.
    pub fn new(initial_angle: f32, seconds: f32) -> Self {
        Self {
            initial_angle,
            duration: Timer::from_seconds(seconds, TimerMode::Once),
        }
    }
}

/// Spawns the transient swing hitbox removed by `on_stop_melee`, dealing damage scaled by the
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
    // Hitbox and layers only exist for the swing; the stop observer removes them.
    commands.entity(took_effect.source).insert((
        ActiveMeleeAttack::new(angle, melee_weapon.attack_time),
        HitBox::new(took_effect.holder, damage, Vec2::from_angle(angle)),
        melee_weapon.hitbox.clone(),
        side.hitbox_layers(),
    ));
    Ok(())
}

/// Ends each swing whose timer has finished.
pub(super) fn end_melee_attacks(
    mut commands: Commands,
    swings: Query<(Entity, &ActiveMeleeAttack)>,
) {
    for (entity, attack) in &swings {
        if attack.duration.just_finished() {
            commands.trigger(StopAction { source: entity });
        }
    }
}

/// Removes the transient swing state.
pub(super) fn on_stop_melee(stop: On<StopAction>, mut commands: Commands) {
    commands.entity(stop.source).remove::<(
        ActiveMeleeAttack,
        HitBox,
        Collider,
        CollisionLayers,
        CollidingEntities,
    )>();
}
