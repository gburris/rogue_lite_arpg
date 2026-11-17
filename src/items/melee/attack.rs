use avian2d::prelude::*;
use bevy::platform::collections::HashSet;
use bevy::prelude::*;

use super::MeleeWeapon;
use crate::prelude::*;

pub(super) fn damage_melee_hits(
    mut commands: Commands,
    mut melee_query: Query<(Entity, &mut ActiveMeleeAttack, &CollidingEntities, &ItemOf)>,
) {
    for (weapon_entity, mut active_melee_attack, colliding_entities, item_of) in &mut melee_query {
        for &colliding_entity in colliding_entities.iter() {
            // We only hit a given entity once per attack
            if !active_melee_attack
                .entities_damaged
                .contains(&colliding_entity)
            {
                commands.trigger(AttemptDamage {
                    entity: colliding_entity,
                    damage: active_melee_attack.damage,
                    source_item: Some(weapon_entity),
                    attacker: Some(item_of.holder),
                    direction: Some(Vec2::from_angle(active_melee_attack.initial_angle)),
                    ..default()
                });
                active_melee_attack
                    .entities_damaged
                    .insert(colliding_entity);
            }
        }
    }
}

#[derive(Component)]
#[require(CollidingEntities, Sensor)]
pub struct ActiveMeleeAttack {
    /// Comes from the direction the entity holding the weapon is aiming.
    pub(super) initial_angle: f32,
    /// Comes from "`attack_time`" defined on `MeleeWeapon`.
    pub(super) duration: Timer,
    damage: Damage,
    entities_damaged: HashSet<Entity>,
}

impl ActiveMeleeAttack {
    /// A swing along `initial_angle` lasting `seconds` that deals `damage` to each target once.
    pub fn new(initial_angle: f32, seconds: f32, damage: Damage) -> Self {
        Self {
            initial_angle,
            duration: Timer::from_seconds(seconds, TimerMode::Once),
            damage,
            entities_damaged: HashSet::default(),
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
    let damage = melee_weapon.damage.scaled(stat_multipliers.damage);
    // Hitbox and layers only exist for the swing; the stop observer removes them.
    commands.entity(took_effect.source).insert((
        ActiveMeleeAttack::new(
            vision.aim_direction.to_angle(),
            melee_weapon.attack_time,
            damage,
        ),
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
        Collider,
        CollisionLayers,
        CollidingEntities,
    )>();
}
