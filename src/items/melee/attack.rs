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
    holders: Query<&StatMultipliers>,
) {
    for (weapon_entity, melee_weapon, mut active_melee_attack, colliding_entities, item_of) in
        &mut melee_query
    {
        let damage_multiplier = holders
            .get(item_of.holder)
            .map_or(1.0, |multipliers| multipliers.damage);
        for &colliding_entity in colliding_entities.iter() {
            // We only hit a given entity once per attack
            if !active_melee_attack
                .entities_damaged
                .contains(&colliding_entity)
            {
                commands.trigger(AttemptDamage {
                    entity: colliding_entity,
                    damage: melee_weapon.damage.scaled(damage_multiplier),
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

/// Color of the glow on a melee weapon while it winds up.
const WIND_UP_GLOW_COLOR: Color = Color::srgb(1.0, 0.2, 0.1);

/// Seconds a holder draws its melee weapon back before each swing, so the target can see the blow
/// coming and step out of its way. The swing keeps the direction aimed when the wind-up began.
#[derive(Component, Clone, Copy, Debug, Deref)]
pub struct MeleeWindUp(pub f32);

#[derive(Component)]
#[require(CollidingEntities, Sensor)]
pub struct ActiveMeleeAttack {
    /// Comes from the direction the entity holding the weapon is aiming.
    pub(super) initial_angle: f32,
    /// Runs before the swing for a holder with a [`MeleeWindUp`].
    pub(super) wind_up: Option<Timer>,
    /// Comes from "`attack_speed`" defined on `MeleeWeapon`.
    pub(super) duration: Timer,
    entities_damaged: HashSet<Entity>,
}

impl ActiveMeleeAttack {
    pub fn new(initial_angle: f32, wind_up: Option<MeleeWindUp>, speed: f32) -> Self {
        Self {
            initial_angle,
            wind_up: wind_up.map(|wind_up| Timer::from_seconds(*wind_up, TimerMode::Once)),
            duration: Timer::from_seconds(speed, TimerMode::Once),
            entities_damaged: HashSet::default(),
        }
    }
}

/// Start observer: begins the swing, with the transient hitbox removed by `on_stop_melee`. A
/// holder with a [`MeleeWindUp`] gets the hitbox once its wind-up ends; see
/// `process_melee_attacks`.
pub(super) fn on_start_melee(
    melee: On<StartUsingEquipment>,
    mut commands: Commands,
    weapon_query: Query<&MeleeWeapon>,
    holder_query: Query<(&Vision, &CombatSide, Option<&MeleeWindUp>)>,
) {
    let Ok(melee_weapon) = weapon_query.get(melee.item) else {
        warn!("Tried to melee attack with invalid weapon");
        return;
    };

    let Ok((vision, side, wind_up)) = holder_query.get(melee.holder) else {
        warn!("Holder missing vision");
        return;
    };

    let attack_angle = vision.aim_direction.to_angle();

    // Hitbox and layers only exist for the swing; the stop observer removes them.
    let mut weapon = commands.entity(melee.item);
    weapon.insert((
        EquipmentInUse,
        ActiveMeleeAttack::new(attack_angle, wind_up.copied(), melee_weapon.attack_time),
        MeleeWeapon::collision_layers(*side),
    ));
    if wind_up.is_none() {
        weapon.insert(melee_weapon.hitbox.clone());
        return;
    }

    // A red glow on the weapon while it winds up, which shows in the dark better than its tint.
    let weapon_entity = melee.item;
    commands.spawn_scene(bsn! {
        @point_light(WIND_UP_GLOW_COLOR, 3.0, 40.0, 3.0)
        ChildOf(weapon_entity)
    });
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

/// Stop observer: removes the transient swing state queued by `end_melee_attacks`, and any
/// wind-up tint and glow left by a swing stopped early.
pub(super) fn on_stop_melee(
    stop: On<StopUsingEquipment>,
    mut commands: Commands,
    mut sprites: Query<&mut Sprite>,
) {
    if let Ok(mut sprite) = sprites.get_mut(stop.item) {
        sprite.color = Color::WHITE;
    }
    commands
        .entity(stop.item)
        .remove::<(EquipmentInUse, ActiveMeleeAttack, Collider, CollisionLayers)>()
        .despawn_children();
}
