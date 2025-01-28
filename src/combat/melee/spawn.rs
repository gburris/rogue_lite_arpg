use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    combat::melee::components::MeleeSwingMarker,
    combat::{melee::components::MeleeAttack, projectile::components::*},
    despawn::components::LiveDuration,
};

use super::components::MeleeSwingPropertiesBundle;

pub fn spawn_melee_attack(
    commands: &mut Commands,
    caster: Entity,
    caster_transform: &Transform,
    caster_aim_position: Vec2,
    melee_bundle: &MeleeSwingPropertiesBundle,
) {
    //TODO: Hide the thing on the guys back
    // Calculate the direction the caster is aiming
    let aim_direction = (caster_aim_position - caster_transform.translation.truncate()).normalize();

    // Offset the hitbox by 50 units in the direction of the aim
    let offset = aim_direction * 50.0;
    let mut transform = Transform {
        translation: caster_transform.translation + Vec3::new(offset.x, offset.y, 0.0),
        ..default()
    };

    // Calculate the angle between the caster's forward direction and the aim direction
    let caster_direction = caster_transform.local_x().truncate();
    let angle = caster_direction.angle_to(aim_direction);

    // Rotate the hitbox to align with the aim direction
    transform.rotate_z(angle + 270.0);
    transform = transform.with_scale(Vec3::new(0.2, 0.2, 0.2));
    warn!("Spawning melee swing with transform {:?}", transform);
    // Spawn the melee attack with the adjusted transform
    //How should we melee attack this direction?
    // Take the players mainahand equipment
    // - Toggle on the collider
    // - Begin updating its transform 
    let melee_attack_entity_id = commands
        .spawn((
            MeleeAttack {
                caster_entity: caster,
            },
            MeleeSwingMarker,
            LiveDuration::new(melee_bundle.swing_type.get_total_duration()),
            melee_bundle.clone(),
            Collider::rectangle(melee_bundle.hitbox.width, melee_bundle.hitbox.length),
            transform,
        ))
        .id();
    commands.entity(caster).add_child(melee_attack_entity_id);
}
