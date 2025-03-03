use crate::{items::Shield, labels::layer::ZLayer};
use bevy::prelude::*;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_3, FRAC_PI_4, PI};

pub fn start_shield_block(
    commands: &mut Commands,
    shield_entity: Entity, // Shield Entity
    shield: &Shield,       // Shield item
    block_angle: f32,      // Angle from player to mouse direction
    shield_sprite: &mut Sprite,
    shield_transform: &Transform,
) {
    warn!("blocking started");
    warn!("Trying to use an angle {:?}", block_angle);

    // Normalize the angle to be between -PI and PI
    let normalized_angle = if block_angle < -PI {
        block_angle + 2.0 * PI
    } else if block_angle > PI {
        block_angle - 2.0 * PI
    } else {
        block_angle
    };

    // Determine atlas index based on angle quadrants using PI constants
    let atlas_index = if normalized_angle > -FRAC_PI_4 && normalized_angle < FRAC_PI_4 {
        // UP direction
        0
    } else if normalized_angle >= -3.0 * FRAC_PI_4 && normalized_angle <= -FRAC_PI_4 {
        // RIGHT direction
        2
    } else if (normalized_angle <= -3.0 * FRAC_PI_4) || (normalized_angle >= 3.0 * FRAC_PI_4) {
        // DOWN direction
        3
    } else {
        // LEFT direction
        1
    };

    // Calculate position offset based on the actual angle
    // We want a consistent 25.0 unit distance from center
    let offset_distance = 40.0;
    let position_offset = Vec3::new(
        offset_distance * normalized_angle.sin(),  // X coordinate
        -offset_distance * normalized_angle.cos(), // Y coordinate (negative because Y is inverted)
        if atlas_index == 0 {
            ZLayer::WeaponAboveSprite.z()
        } else {
            ZLayer::WeaponBehindSprite.z()
        },
    );

    // Update the sprite texture atlas index
    if let Some(atlas) = &mut shield_sprite.texture_atlas {
        atlas.index = atlas_index;
    }

    // Set the transform
    commands.entity(shield_entity).insert(Transform::from_xyz(
        position_offset.x,
        position_offset.y,
        position_offset.z,
    ));

    // Add the ActiveShieldBlock component with transform information
    commands.entity(shield_entity).insert(ActiveShieldBlock {
        direction: normalized_angle,
        position_offset,
        // Add any other data needed for collision detection
    });
}

// Define the ActiveShieldBlock component (if not already defined elsewhere)
#[derive(Component)]
pub struct ActiveShieldBlock {
    pub direction: f32,
    pub position_offset: Vec3,
    // Add other fields as needed for collision handling
}
