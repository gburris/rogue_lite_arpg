use crate::{
    animation::FacingDirection,
    combat::{
        attributes::{Mana, ManaDrainRate},
        components::AimPosition,
    },
    enemy::Enemy,
    items::{
        equipment::{EquipmentTransform, Equipped},
        Shield,
    },
    labels::layer::ZLayer,
};
use avian2d::prelude::{Collider, CollisionLayers};
use bevy::prelude::*;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};

use super::{components::ProjectileReflection, ActiveShield};

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

        // Drain mana
        if let Ok(mut mana) = mana_query.get_mut(holder_entity) {
            //This is wrong, drain amount should = drain rate when delta time is exactly 1 frame long
            let drain_amount = mana_drain_rate.0 * time.delta_secs();
            if mana.current_mana < drain_amount {
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
    commands
        .entity(shield_entity)
        .remove::<ActiveShield>()
        .remove::<Collider>()
        .insert(EquipmentTransform::get(facing_direction).offhand);

    if let Some(sprite) = shield_sprite {
        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = 0;
        }
    }
}
pub fn activate_shield(
    trigger: Trigger<OnAdd, ActiveShield>,
    mut commands: Commands,
    shield_query: Query<&Shield>,
) {
    if let Ok(activated_shield) = shield_query.get(trigger.entity()) {
        commands
            .entity(trigger.entity())
            .insert(activated_shield.hitbox.clone())
            .insert(ProjectileReflection::collision_layers());
    } else {
        warn!("Active Shield added to something that isn't a shield");
    }
}
