use crate::{
    combat::{
        mana::{ManaCost, ManaDrainRate},
        Mana,
    },
    configuration::ZLayer,
    items::{
        equipment::{EquipmentOf, EquipmentTransform},
        ItemOf, Shield,
    },
    prelude::*,
};
use avian2d::prelude::Collider;
use bevy::prelude::*;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};

use super::{components::ProjectileReflection, ActiveShield};

pub fn update_active_shields(
    mut commands: Commands,
    time: Res<Time>,
    mut active_shield_query: Query<
        (Entity, &ManaDrainRate, &ItemOf, &mut Sprite),
        (With<ActiveShield>, With<EquipmentOf>),
    >,
    mut holder_query: Query<(&Vision, &FacingDirection, Option<&mut Mana>)>,
) {
    for (shield_entity, mana_drain_rate, item_of, mut shield_sprite) in
        active_shield_query.iter_mut()
    {
        let (vision, facing_direction, mana) = holder_query
            .get_mut(item_of.0)
            .expect("Shield holder missing necessary components");

        let block_angle = vision.aim_direction.y.atan2(vision.aim_direction.x) + FRAC_PI_2;

        let normalized_angle = if block_angle < -PI {
            block_angle + 2.0 * PI
        } else if block_angle > PI {
            block_angle - 2.0 * PI
        } else {
            block_angle
        };

        let atlas_index = if normalized_angle > -FRAC_PI_4 && normalized_angle < FRAC_PI_4 {
            0
        } else if (-3.0 * FRAC_PI_4..=-FRAC_PI_4).contains(&normalized_angle) {
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

        if let Some(mut mana) = mana {
            let drain_amount = ManaCost(mana_drain_rate.0 * time.delta_secs());
            if !mana.attempt_use_mana(&drain_amount) {
                deactivate_shield(
                    &mut commands,
                    shield_entity,
                    *facing_direction,
                    &mut shield_sprite,
                );
            }
        }
    }
}

pub fn deactivate_shield(
    commands: &mut Commands,
    shield_entity: Entity,
    facing_direction: FacingDirection,
    shield_sprite: &mut Sprite,
) {
    commands
        .entity(shield_entity)
        .remove::<(ActiveShield, Collider)>()
        .insert(EquipmentTransform::get(facing_direction).offhand);

    if let Some(atlas) = &mut shield_sprite.texture_atlas {
        atlas.index = 0;
    }
}

pub fn activate_shield(
    trigger: Trigger<OnAdd, ActiveShield>,
    mut commands: Commands,
    shield_query: Query<&Shield>,
) {
    if let Ok(activated_shield) = shield_query.get(trigger.target()) {
        commands
            .entity(trigger.target())
            .insert(activated_shield.hitbox.clone())
            .insert(ProjectileReflection::collision_layers());
    } else {
        warn!("Active Shield added to something that isn't a shield");
    }
}
