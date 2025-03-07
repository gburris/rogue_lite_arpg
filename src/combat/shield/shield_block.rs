use crate::{
    animation::FacingDirection,
    combat::{
        attributes::{Mana, ManaDrainRate},
        components::AimPosition,
    },
    items::{
        equipment::{EquipmentTransform, Equipped},
        ActiveShield, Shield,
    },
    labels::layer::ZLayer,
};
use bevy::prelude::*;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};

pub fn start_shield_block(
    commands: &mut Commands,
    shield_entity: Entity, // Shield Entity
    block_angle: f32,      // Angle from player to mouse direction
    shield_sprite: &mut Sprite,
) {
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
        0
    } else if normalized_angle >= -3.0 * FRAC_PI_4 && normalized_angle <= -FRAC_PI_4 {
        2
    } else if (normalized_angle <= -3.0 * FRAC_PI_4) || (normalized_angle >= 3.0 * FRAC_PI_4) {
        3
    } else {
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
}

pub fn update_active_shields(
    mut commands: Commands,
    time: Res<Time>,
    active_shields: Query<(Entity, &Shield, &ManaDrainRate, &ActiveShield)>,
    mut sprites: Query<&mut Sprite>,
    transforms: Query<&Transform>,
    equipped: Query<&Equipped>,
    holder_query: Query<(&Transform, &AimPosition, &FacingDirection)>,
    mut mana_query: Query<&mut Mana>,
) {
    for (shield_entity, shield, mana_drain_rate, shield_state) in active_shields.iter() {
        // Get the holder of this shield
        let Ok(equipped_info) = equipped.get(shield_entity) else {
            continue;
        };
        let holder_entity = equipped_info.get_equipped_to();

        // Update shield position
        let Ok((holder_transform, aim_pos, facing_direction)) = holder_query.get(holder_entity)
        else {
            continue;
        };
        let holder_pos = holder_transform.translation.truncate();
        let aim_direction: Vec2 = (aim_pos.position - holder_pos).normalize();
        let block_angle = aim_direction.y.atan2(aim_direction.x) + FRAC_PI_2;

        // Get shield components for updating
        let Ok(shield_transform) = transforms.get(shield_entity) else {
            continue;
        };
        let Ok(mut shield_sprite) = sprites.get_mut(shield_entity) else {
            continue;
        };

        // Update shield position
        start_shield_block(
            &mut commands,
            shield_entity,
            block_angle,
            &mut shield_sprite,
        );

        // Drain mana
        if let Ok(mut mana) = mana_query.get_mut(holder_entity) {
            //This is wrong, drain amount should = drain rate when delta time is exactly 1 frame long
            let drain_amount = mana_drain_rate.0 * time.delta_secs();
            warn!("draining {:?}", drain_amount);
            if mana.current_mana < drain_amount {
                // Get the holder's facing direction
                deactivate_shield(
                    &mut commands,
                    shield_entity,
                    *facing_direction,
                    Some(&mut shield_sprite),
                );
            } else {
                mana.current_mana -= drain_amount;
            }
        }
    }
}

pub fn deactivate_shield(
    commands: &mut Commands,
    shield_entity: Entity,
    facing_direction: FacingDirection,
    shield_sprite: Option<&mut Sprite>,
) {
    // Remove the ActiveShield component
    let mut entity_commands = commands.entity(shield_entity);
    entity_commands.remove::<ActiveShield>();

    // Reset the transform to default equipment position
    entity_commands.insert(EquipmentTransform::get(facing_direction).offhand);

    // Reset the sprite atlas index if sprite is provided
    if let Some(sprite) = shield_sprite {
        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = 0;
        }
    }
}
