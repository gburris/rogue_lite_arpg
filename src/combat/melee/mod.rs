use avian2d::prelude::*;
use bevy::{platform::collections::HashSet, prelude::*};

use crate::{
    combat::damage::{AttemptDamage, Damage, DamageSource},
    configuration::GameCollisionLayer,
    items::equipment::UseEquipment,
    prelude::{ActionState, Vision},
};

mod swing;

pub use swing::{MeleeSwingType, process_melee_attacks};

/// Our pixel weapons all face upwards currently, so we must rotate them 90 degrees for attacks to
/// occur in the direction we expect. This value will need to be updated if our assets change
pub const MELEE_WEAPON_ROTATION: f32 = std::f32::consts::FRAC_PI_2;

//Repesent a melee weapon
#[derive(Component, Clone, Debug)]
pub struct MeleeWeapon {
    // Time it takes (seconds) to complete the attack, smaller = faster
    pub attack_time: f32,
    pub damage: (f32, f32),
    pub hitbox: Collider,
    pub attack_type: swing::MeleeSwingType,
    pub hold_distance: f32,
}

impl MeleeWeapon {
    /// Gets collision layers for melee weapon based on source of damage
    /// It can either target allies or enemies
    pub fn collision_layers(damage_source: DamageSource) -> CollisionLayers {
        CollisionLayers::new(GameCollisionLayer::HitBox, LayerMask::from(damage_source))
    }
}

pub fn handle_melee_collisions(
    mut commands: Commands,
    mut melee_query: Query<(
        Entity,
        &MeleeWeapon,
        &mut ActiveMeleeAttack,
        &CollidingEntities,
    )>,
) {
    for (weapon_entity, melee_weapon, mut active_melee_attack, colliding_entities) in
        melee_query.iter_mut()
    {
        for &colliding_entity in colliding_entities.iter() {
            // We only hit a given entity once per attack
            if !active_melee_attack
                .entities_damaged
                .contains(&colliding_entity)
            {
                commands.trigger(AttemptDamage {
                    entity: colliding_entity,
                    ignore_invulnerable: false,
                    damage: Damage::Range(melee_weapon.damage),
                    damage_source: Some(weapon_entity),
                    direction: Some(Vec2::from_angle(active_melee_attack.initial_angle)),
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
    /// Comes from the direction the entity holding the weapon is aiming
    initial_angle: f32,
    /// Comes from "attack_speed" defined on MeleeWeapon
    duration: Timer,
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

pub fn on_weapon_melee(
    melee_weapon_used: On<UseEquipment>,
    mut commands: Commands,
    mut weapon_query: Query<(Entity, &mut MeleeWeapon)>,
    mut action_state_query: Query<&mut ActionState>,
    holder_query: Query<&Vision>,
) {
    let Ok((weapon_entity, mut melee_weapon)) = weapon_query.get_mut(melee_weapon_used.entity)
    else {
        warn!("Tried to melee attack with invalid weapon");
        return;
    };

    let Ok(vision) = holder_query.get(melee_weapon_used.holder) else {
        warn!("Holder missing required components");
        return;
    };

    let attack_angle = vision.aim_direction.to_angle();

    start_melee_attack(
        &mut commands,
        weapon_entity,
        &mut melee_weapon,
        attack_angle,
    );

    if let Ok(mut action_state) = action_state_query.get_mut(melee_weapon_used.holder) {
        *action_state = ActionState::Attacking;
    }
}

fn start_melee_attack(
    commands: &mut Commands,
    weapon_entity: Entity,
    melee_weapon: &mut MeleeWeapon,
    attack_angle: f32,
) {
    commands
        .entity(weapon_entity)
        .insert(ActiveMeleeAttack::new(
            attack_angle,
            melee_weapon.attack_time,
        ));
}

pub fn end_melee_attacks(
    mut commands: Commands,
    mut query: Query<(Entity, &ChildOf, &ActiveMeleeAttack)>,
    mut action_state_query: Query<&mut ActionState>,
) {
    for (entity, child_of, attack) in query.iter_mut() {
        if attack.duration.just_finished() {
            if let Ok(mut action_state) = action_state_query.get_mut(child_of.parent()) {
                *action_state = ActionState::Movement;
            }
            commands.entity(entity).remove::<ActiveMeleeAttack>();
        }
    }
}
