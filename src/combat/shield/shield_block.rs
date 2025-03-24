use crate::{
    ai::state::{AimPosition, FacingDirection},
    combat::{mana::ManaDrainRate, Mana},
    configuration::ZLayer,
    items::{
        equipment::{EquipmentTransform, Equipped},
        Shield,
    },
};
use avian2d::prelude::Collider;
use bevy::prelude::*;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};

use super::{components::ProjectileReflection, ActiveShield};

pub fn update_active_shields(
    mut commands: Commands,
    time: Res<Time>,
    active_shields: Query<(Entity, &ManaDrainRate, &ActiveShield)>,
    mut sprites: Query<&mut Sprite>,
    equipped: Query<&Equipped>,
    holder_query: Query<(&Transform, &AimPosition, &FacingDirection)>,
    mut mana_query: Query<&mut Mana>,
) {
    for (shield_entity, mana_drain_rate, _) in active_shields.iter() {
        let Ok(equipped_info) = equipped.get(shield_entity) else {
            continue;
        };
        let holder_entity = equipped_info.get_equipped_to();

        let Ok((holder_transform, aim_pos, facing_direction)) = holder_query.get(holder_entity)
        else {
            continue;
        };
        let holder_pos = holder_transform.translation.truncate();
        let aim_direction: Vec2 = (aim_pos.position - holder_pos).normalize();
        let block_angle = aim_direction.y.atan2(aim_direction.x) + FRAC_PI_2;

        let Ok(mut shield_sprite) = sprites.get_mut(shield_entity) else {
            continue;
        };

        let normalized_angle = if block_angle < -PI {
            block_angle + 2.0 * PI
        } else if block_angle > PI {
            block_angle - 2.0 * PI
        } else {
            block_angle
        };

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
            offset_distance * normalized_angle.sin(),
            -offset_distance * normalized_angle.cos(),
            if atlas_index == 0 {
                ZLayer::AboveSprite.z()
            } else {
                ZLayer::BehindSprite.z()
            },
        );

        if let Some(atlas) = &mut shield_sprite.texture_atlas {
            atlas.index = atlas_index;
        }

        commands.entity(shield_entity).insert(Transform::from_xyz(
            position_offset.x,
            position_offset.y,
            position_offset.z,
        ));

        if let Ok(mut mana) = mana_query.get_mut(holder_entity) {
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
