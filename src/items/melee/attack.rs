use avian2d::prelude::*;
use bevy::prelude::*;

use super::MeleeWeapon;
use crate::prelude::*;

#[derive(Component)]
pub struct ActiveMeleeAttack {
    /// Comes from the direction the entity holding the weapon is aiming.
    pub(super) initial_angle: f32,
    /// Comes from the weapon's [`ActionUseStyle::Timed`] seconds.
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

/// Spawns the transient swing hitbox removed by `on_melee_finished`, dealing damage scaled by the
/// holder's stats at the moment of the swing.
pub(super) fn on_melee_took_effect(
    took_effect: On<ActionTookEffect>,
    mut commands: Commands,
    weapon_query: Query<(&MeleeWeapon, &ActionUseStyle)>,
    holder_query: Query<(&Vision, &CombatSide, &StatMultipliers)>,
) -> Result {
    let (melee_weapon, style) = weapon_query.get(took_effect.source)?;
    let seconds = style
        .timed_seconds()
        .ok_or("a melee weapon's use must be timed")?;
    let Ok((vision, side, stat_multipliers)) = holder_query.get(took_effect.holder) else {
        return Ok(());
    };
    let angle = vision.aim_direction.to_angle();
    let damage = melee_weapon.damage.scaled(stat_multipliers.damage);
    // Hitbox and layers only exist for the swing; the finish observer removes them.
    commands.entity(took_effect.source).insert((
        ActiveMeleeAttack::new(angle, seconds),
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
