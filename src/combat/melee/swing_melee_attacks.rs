use crate::{
    animation::MovementDirection, configuration::GameCollisionLayer,
    items::equipment::equipment_transform::DirectionTransforms, player::systems::ActionState,
};

use super::components::{ActiveMeleeAttack, MeleeSwingType, MeleeWeapon};
use avian2d::prelude::CollisionLayers;
use bevy::prelude::*;

#[derive(PartialEq)]
pub enum MeleeSourceType {
    Player,
    Enemy,
    NPC,
    Environment,
}

pub fn start_melee_attack(
    source: MeleeSourceType,
    commands: &mut Commands,
    weapon_entity: Entity,
    melee_weapon: &mut MeleeWeapon,
    attack_angle: f32,
) {
    melee_weapon.attack_duration.reset();
    if source == MeleeSourceType::Player {
        commands.entity(weapon_entity).insert(ActiveMeleeAttack {
            initial_angle: attack_angle,
        });
    } else if source == MeleeSourceType::Enemy {
        commands.entity(weapon_entity).insert((
            ActiveMeleeAttack {
                initial_angle: attack_angle,
            },
            enemy_melee_collision_layers(),
        ));
    }
}

fn enemy_melee_collision_layers() -> CollisionLayers {
    CollisionLayers::new(GameCollisionLayer::Grounded, [GameCollisionLayer::Player])
}

pub fn end_melee_attacks(
    mut commands: Commands,
    mut query: Query<(Entity, &Parent, &MeleeWeapon, &mut Transform), With<ActiveMeleeAttack>>,
    mut action_state_query: Query<&mut ActionState>,
) {
    for (entity, parent, melee_weapon, mut transform) in query.iter_mut() {
        if melee_weapon.attack_duration.just_finished() {
            if let Ok(mut action_state) = action_state_query.get_mut(parent.get()) {
                commands.entity(entity).remove::<ActiveMeleeAttack>();
                *action_state = ActionState::None;
                *transform = DirectionTransforms::get(MovementDirection::Down).mainhand;
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
