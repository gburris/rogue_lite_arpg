use crate::{
    combat::damage::components::CollisionDamage,
    items::equipment::equipment_transform::DirectionTransforms,
    player::{movement::MovementDirection, systems::CurrentActionState},
};

use super::components::{ActiveMeleeAttack, MeleeSwingType, MeleeWeapon};
use avian2d::prelude::Collider;
use bevy::prelude::*;

pub fn start_melee_attack(
    commands: &mut Commands,
    weapon_entity: Entity,
    melee_weapon: &MeleeWeapon,
    attack_angle: f32,
    equipped_item_transform: Transform,
) {
    commands
        .entity(weapon_entity)
        .insert(ActiveMeleeAttack {
            timer: Timer::from_seconds(melee_weapon.swing_duration, TimerMode::Once),
            initial_angle: attack_angle,
            attack_type: melee_weapon.melee_attack.swing_type.clone(),
        })
        .insert(Collider::rectangle(
            melee_weapon.melee_attack.hitbox.width,
            melee_weapon.melee_attack.hitbox.length,
        ))
        .insert(CollisionDamage {
            damage: melee_weapon.melee_attack.damage.damage,
        });
}

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
                //active_attack.initial_angle -= std::f32::consts::FRAC_PI_2;
                let progress = active_attack.timer.elapsed_secs()
                    / active_attack.timer.duration().as_secs_f32();

                let distance = 2.0 * speed * (std::f32::consts::PI * progress).sin();

                let attack_offset = 25.0; // Define the stab-specific offset

                // Forward direction based on stab angle
                let forward = Vec2::new(
                    (active_attack.initial_angle + std::f32::consts::FRAC_PI_2).cos(),
                    (active_attack.initial_angle + std::f32::consts::FRAC_PI_2).sin(),
                );

                let stab_start_position =
                    Vec3::new(forward.x * attack_offset, forward.y * attack_offset, 0.0);

                // Apply position update
                transform.translation = stab_start_position
                    + Vec3::new(forward.x * distance, forward.y * distance, 0.0);

                transform.rotation = Quat::from_rotation_z(active_attack.initial_angle);
            }
            MeleeSwingType::Slash { radius, .. } => {
                let progress = active_attack.timer.elapsed_secs()
                    / active_attack.timer.duration().as_secs_f32();

                let adjusted_angle = active_attack.initial_angle + std::f32::consts::FRAC_PI_2; // Rotate by -90°

                let start_angle = adjusted_angle - 60f32.to_radians();
                let end_angle = adjusted_angle + 60f32.to_radians();

                // Compute current swing angle based on progress
                let current_angle = start_angle + (end_angle - start_angle) * progress;

                // Compute the axe head's position along the arc
                let axe_head_position = Vec3::new(
                    current_angle.cos() * radius,
                    current_angle.sin() * radius,
                    0.0,
                );

                // **New Fix**: Make sure the axe rotates properly
                // The axe head should be perpendicular to the swing motion
                let blade_angle = current_angle - std::f32::consts::FRAC_PI_2; // Flip the angle

                // Apply position and rotation updates
                transform.translation = axe_head_position;
                transform.rotation = Quat::from_rotation_z(blade_angle);
            }
        }
        if let Ok(mut action_state) = action_state_query.get_mut(parent.get()) {
            // Check if attack is finished
            if active_attack.timer.just_finished() {
                commands
                    .entity(entity)
                    .remove::<ActiveMeleeAttack>()
                    .remove::<Collider>()
                    .remove::<CollisionDamage>();
                *action_state = CurrentActionState::None;
                // Reset transform to starting position
                //TODO: Call some system here to set the position of the sword to the current player direction
                //For now, we can default to "on the back" since it's the least visually jarring
                *transform = DirectionTransforms::get(MovementDirection::Down).mainhand;
            }
        }
    }
}
