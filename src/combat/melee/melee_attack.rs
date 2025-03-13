use avian2d::prelude::*;
use bevy::{prelude::*, utils::HashSet};

use crate::{
    ai::state::ActionState,
    combat::{
        damage::{AttemptDamageEvent, Damage},
        melee::{MeleeSwingType, MeleeWeapon},
    },
};

use super::MELEE_WEAPON_ROTATION;

#[derive(Component)]
#[require(CollidingEntities, Sensor)]
pub struct ActiveMeleeAttack {
    /// Comes from the direction the entity holding the weapon is aiming
    pub initial_angle: f32,
    /// Comes from "attack_speed" defined on MeleeWeapon
    duration: Timer,
    entities_damaged: HashSet<Entity>,
}

impl ActiveMeleeAttack {
    pub fn new(initial_angle: f32, speed: f32) -> Self {
        Self {
            initial_angle,
            duration: Timer::from_seconds(speed, TimerMode::Once),
            entities_damaged: HashSet::new(),
        }
    }
}

pub fn start_melee_attack(
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
    mut query: Query<(Entity, &Parent, &ActiveMeleeAttack)>,
    mut action_state_query: Query<&mut ActionState>,
) {
    for (entity, parent, attack) in query.iter_mut() {
        if attack.duration.just_finished() {
            if let Ok(mut action_state) = action_state_query.get_mut(parent.get()) {
                // This handles the edge case of dying mid-swing
                if *action_state != ActionState::Defeated {
                    *action_state = ActionState::Movement;
                }
                commands.entity(entity).remove::<ActiveMeleeAttack>();
            }
        }
    }
}

pub fn process_melee_attacks(
    time: Res<Time>,
    mut attack_query: Query<(&MeleeWeapon, &mut Transform, &mut ActiveMeleeAttack)>,
) {
    for (melee_weapon, mut transform, mut active_attack) in attack_query.iter_mut() {
        active_attack.duration.tick(time.delta());
        let attack_progress = active_attack.duration.fraction();

        match melee_weapon.attack_type {
            MeleeSwingType::Stab { reach } => {
                // Total distance of stab * time of swing gets new position each tick
                let distance = reach * attack_progress;

                let forward = Vec2::new(
                    active_attack.initial_angle.cos(),
                    active_attack.initial_angle.sin(),
                );

                let new_stab_position = forward * (melee_weapon.hold_distance + distance);

                transform.translation = new_stab_position.extend(0.0);
                transform.rotation =
                    Quat::from_rotation_z(active_attack.initial_angle - MELEE_WEAPON_ROTATION);
            }
            MeleeSwingType::Slash { arc_distance } => {
                let start_angle = active_attack.initial_angle - (arc_distance / 2.0);

                let current_angle = start_angle + (arc_distance * attack_progress);

                let new_axe_position = Vec2::new(current_angle.cos(), current_angle.sin())
                    * melee_weapon.hold_distance;

                transform.translation = new_axe_position.extend(0.0);
                transform.rotation = Quat::from_rotation_z(current_angle - MELEE_WEAPON_ROTATION);
            }
        }
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
                commands.trigger_targets(
                    AttemptDamageEvent {
                        damage: Damage::Range(melee_weapon.damage),
                        damage_source: Some(weapon_entity),
                    },
                    colliding_entity,
                );
                active_melee_attack
                    .entities_damaged
                    .insert(colliding_entity);
            }
        }
    }
}
