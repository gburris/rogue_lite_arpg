use crate::player::systems::CurrentActionState;

use super::components::{ActiveMeleeAttack, MeleeSwingType};
use avian2d::prelude::Collider;
use bevy::prelude::*;

pub fn process_melee_attacks(
    time: Res<Time>,
    mut commands: Commands,
    mut attack_query: Query<(Entity, &Parent, &mut Transform, &mut ActiveMeleeAttack)>,
    mut action_state_query: Query<&mut CurrentActionState>,
) {
    for (entity, parent, mut transform, mut active_attack) in attack_query.iter_mut() {
        active_attack.timer.tick(time.delta());

        match active_attack.attack_type {
            MeleeSwingType::Stab { speed, .. } => {
                let progress = active_attack.timer.elapsed_secs()
                    / active_attack.timer.duration().as_secs_f32();

                let distance = 2.0 * speed * (std::f32::consts::PI * progress).sin();

                let attack_offset = 25.0; // Define the stab-specific offset

                // Forward direction based on stab angle
                let forward = Vec2::new(
                    (active_attack.initial_angle + std::f32::consts::FRAC_PI_2).cos(),
                    (active_attack.initial_angle + std::f32::consts::FRAC_PI_2).sin(),
                );

                // Adjust starting position at the beginning of the stab
                let stab_start_position = active_attack.starting_transform.translation
                    + Vec3::new(forward.x * attack_offset, forward.y * attack_offset, 0.0);

                // Apply position update
                transform.translation = stab_start_position
                    + Vec3::new(forward.x * distance, forward.y * distance, 0.0);

                transform.rotation = Quat::from_rotation_z(active_attack.initial_angle);
            }
            MeleeSwingType::Slash {
                radius, duration, ..
            } => {
                let progress = active_attack.timer.elapsed_secs() / duration;

                // Slash moves from 30° right to 30° left (total 60° arc)
                let start_angle = active_attack.initial_angle - 30f32.to_radians();
                let end_angle = active_attack.initial_angle + 30f32.to_radians();

                let current_angle = start_angle + (end_angle - start_angle) * progress;

                // Position: Move along the arc
                let forward = Vec2::new(current_angle.cos(), current_angle.sin());
                let offset_position = active_attack.starting_transform.translation
                    + Vec3::new(forward.x * radius, forward.y * radius, 0.0);

                // Rotation: Always face the player (handle inward)
                let to_player = (active_attack.starting_transform.translation - offset_position)
                    .truncate()
                    .normalize();
                let angle_to_player = to_player.y.atan2(to_player.x) + std::f32::consts::PI; // Flip 180° to point handle inward

                // Apply transform updates
                transform.translation = offset_position;
                transform.rotation = Quat::from_rotation_z(angle_to_player);
            }
        }
        if let Ok(mut action_state) = action_state_query.get_mut(parent.get()) {
            // Check if attack is finished
            if active_attack.timer.just_finished() {
                commands
                    .entity(entity)
                    .remove::<ActiveMeleeAttack>()
                    .remove::<Collider>();
                *action_state = CurrentActionState::None;
                // Reset transform to starting position
                *transform = active_attack.starting_transform;
            }
        }
    }
}
