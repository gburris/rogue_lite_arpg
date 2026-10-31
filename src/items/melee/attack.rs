use avian2d::prelude::*;
use bevy::platform::collections::HashSet;
use bevy::prelude::*;

use super::MeleeWeapon;
use crate::prelude::*;

pub(super) fn damage_melee_hits(
    mut commands: Commands,
    mut melee_query: Query<(
        Entity,
        &MeleeWeapon,
        &mut ActiveMeleeAttack,
        &CollidingEntities,
        &ItemOf,
    )>,
) {
    for (weapon_entity, melee_weapon, mut active_melee_attack, colliding_entities, item_of) in
        &mut melee_query
    {
        for &colliding_entity in colliding_entities.iter() {
            // We only hit a given entity once per attack
            if !active_melee_attack
                .entities_damaged
                .contains(&colliding_entity)
            {
                commands.trigger(AttemptDamage {
                    entity: colliding_entity,
                    damage: melee_weapon.damage,
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
    /// Comes from "`attack_speed`" defined on `MeleeWeapon`.
    pub(super) duration: Timer,
    entities_damaged: HashSet<Entity>,
}

impl ActiveMeleeAttack {
    pub fn new(initial_angle: f32, speed: f32) -> Self {
        Self {
            initial_angle,
            duration: Timer::from_seconds(speed, TimerMode::Once),
            entities_damaged: HashSet::default(),
        }
    }
}

/// Start observer: spawns the transient swing hitbox removed by `on_stop_melee`.
pub(super) fn on_start_melee(
    melee: On<StartUsingEquipment>,
    mut commands: Commands,
    weapon_query: Query<&MeleeWeapon>,
    holder_query: Query<(&Vision, &CombatSide)>,
) {
    let Ok(melee_weapon) = weapon_query.get(melee.item) else {
        warn!("Tried to melee attack with invalid weapon");
        return;
    };

    let Ok((vision, side)) = holder_query.get(melee.holder) else {
        warn!("Holder missing vision");
        return;
    };

    let attack_angle = vision.aim_direction.to_angle();

    // Hitbox and layers only exist for the swing; the stop observer removes them.
    commands.entity(melee.item).insert((
        EquipmentInUse,
        ActiveMeleeAttack::new(attack_angle, melee_weapon.attack_time),
        melee_weapon.hitbox.clone(),
        MeleeWeapon::collision_layers(*side),
    ));
}

/// End polling: queues `StopUsingEquipment` once each swing timer finishes.
pub(super) fn end_melee_attacks(
    mut commands: Commands,
    swings: Query<(Entity, &ItemOf, &ActiveMeleeAttack)>,
) {
    for (item, item_of, attack) in &swings {
        if attack.duration.just_finished() {
            commands.trigger(StopUsingEquipment {
                item,
                holder: item_of.holder,
            });
        }
    }
}

/// Stop observer: removes the transient swing state queued by `end_melee_attacks`.
pub(super) fn on_stop_melee(stop: On<StopUsingEquipment>, mut commands: Commands) {
    commands
        .entity(stop.item)
        .remove::<(EquipmentInUse, ActiveMeleeAttack, Collider, CollisionLayers)>();
}
