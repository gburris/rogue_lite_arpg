use avian2d::prelude::*;
use bevy::{prelude::*, utils::HashSet};

use crate::{
    ai::state::{ActionState, FacingDirection},
    combat::{
        damage::{AttemptDamageEvent, Damage},
        melee::{MeleeSwingType, MeleeWeapon},
    },
    enemy::Enemy,
    items::equipment::EquipmentTransform,
    player::Player,
};

#[derive(Component)]
#[require(CollidingEntities, Sensor)]
pub struct ActiveMeleeAttack {
    pub initial_angle: f32,
    pub entities_damaged: HashSet<Entity>,
}

pub fn start_melee_attack(
    commands: &mut Commands,
    weapon_entity: Entity,
    melee_weapon: &mut MeleeWeapon,
    attack_angle: f32,
) {
    melee_weapon.attack_duration.reset();
    commands.entity(weapon_entity).insert(ActiveMeleeAttack {
        initial_angle: attack_angle,
        entities_damaged: HashSet::new(),
    });
}

pub fn end_melee_attacks(
    mut commands: Commands,
    mut query: Query<(Entity, &Parent, &MeleeWeapon, &mut Transform), With<ActiveMeleeAttack>>,
    mut action_state_query: Query<&mut ActionState>,
) {
    for (entity, parent, melee_weapon, mut transform) in query.iter_mut() {
        if melee_weapon.attack_duration.just_finished() {
            if let Ok(mut action_state) = action_state_query.get_mut(parent.get()) {
                // This handles the edge case of dying mid-swing
                if *action_state != ActionState::Defeated {
                    *action_state = ActionState::Movement;
                    *transform = EquipmentTransform::get(FacingDirection::Down).mainhand;
                }
                commands.entity(entity).remove::<ActiveMeleeAttack>();
            }
        }
    }
}

pub fn process_melee_attacks(
    time: Res<Time>,
    mut attack_query: Query<(&mut MeleeWeapon, &mut Transform, &ActiveMeleeAttack)>,
) {
    for (mut melee_weapon, mut transform, active_attack) in attack_query.iter_mut() {
        melee_weapon.attack_duration.tick(time.delta());

        match melee_weapon.attack_type {
            MeleeSwingType::Stab { speed, .. } => {
                let progress = melee_weapon.attack_duration.elapsed_secs()
                    / melee_weapon.attack_duration.duration().as_secs_f32();

                let distance = 2.0 * speed * (std::f32::consts::PI * progress).sin();

                let attack_offset = 25.0;

                let forward = Vec2::new(
                    (active_attack.initial_angle + std::f32::consts::FRAC_PI_2).cos(),
                    (active_attack.initial_angle + std::f32::consts::FRAC_PI_2).sin(),
                );

                let stab_start_position =
                    Vec3::new(forward.x * attack_offset, forward.y * attack_offset, 0.0);

                transform.translation = stab_start_position
                    + Vec3::new(forward.x * distance, forward.y * distance, 0.0);

                transform.rotation = Quat::from_rotation_z(active_attack.initial_angle);
            }
            MeleeSwingType::Slash { radius, .. } => {
                let progress = melee_weapon.attack_duration.elapsed_secs()
                    / melee_weapon.attack_duration.duration().as_secs_f32();

                let adjusted_angle = active_attack.initial_angle + std::f32::consts::FRAC_PI_2; // Rotate by -90°

                let start_angle = adjusted_angle - 60f32.to_radians();
                let end_angle = adjusted_angle + 60f32.to_radians();

                let current_angle = start_angle + (end_angle - start_angle) * progress;

                let axe_head_position = Vec3::new(
                    current_angle.cos() * radius,
                    current_angle.sin() * radius,
                    0.0,
                );

                let blade_angle = current_angle - std::f32::consts::FRAC_PI_2;

                transform.translation = axe_head_position;
                transform.rotation = Quat::from_rotation_z(blade_angle);
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
    enemy_query: Query<&Enemy>,
    player: Single<Entity, With<Player>>,
) {
    let player_entity = player.into_inner();

    for (weapon_entity, melee_weapon, mut active_melee_attack, colliding_entities) in
        melee_query.iter_mut()
    {
        for &colliding_entity in colliding_entities.iter() {
            if (enemy_query.contains(colliding_entity) || colliding_entity == player_entity)
                && (!active_melee_attack
                    .entities_damaged
                    .contains(&colliding_entity))
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
