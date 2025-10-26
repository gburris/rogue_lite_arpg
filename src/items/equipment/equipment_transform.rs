use bevy::prelude::*;

use std::{collections::HashMap, sync::LazyLock};

use crate::{
    items::{
        Items,
        equipment::{Equipped, Mainhand, Offhand},
        melee::ActiveMeleeAttack,
    },
    prelude::*,
};

// Constants for transform values
const MAINHAND_SCALE: Vec3 = Vec3::new(1.0, 1.0, 1.0);
const OFFHAND_SCALE: Vec3 = Vec3::new(1.0, 1.0, 1.0);
#[derive(Clone, Copy)]
pub struct EquipmentTransform {
    pub mainhand: Transform,
    pub offhand: Transform,
}

//You wish this wasn't like this but it is
//See std lib example here https://crates.io/crates/lazy_static
static EQUIPMENT_TRANSFORM_MAP: LazyLock<HashMap<FacingDirection, EquipmentTransform>> =
    LazyLock::new(|| {
        let mut m = HashMap::new();

        // Up direction
        m.insert(
            FacingDirection::Up,
            EquipmentTransform {
                mainhand: Transform::from_xyz(0.0, -8.0, ZLayer::AboveSprite.z())
                    .with_rotation(Quat::from_rotation_z(30.0f32.to_radians()))
                    .with_scale(MAINHAND_SCALE),
                offhand: Transform::from_xyz(0.0, -8.0, ZLayer::AboveSprite.z())
                    .with_rotation(Quat::from_rotation_z(30.0f32.to_radians()))
                    .with_scale(OFFHAND_SCALE),
            },
        );

        // Down direction
        m.insert(
            FacingDirection::Down,
            EquipmentTransform {
                mainhand: Transform::from_xyz(0.0, 8.0, ZLayer::BehindSprite.z())
                    .with_rotation(Quat::from_rotation_z(-30.0f32.to_radians()))
                    .with_scale(MAINHAND_SCALE),
                offhand: Transform::from_xyz(0.0, 8.0, ZLayer::BehindSprite.z())
                    .with_rotation(Quat::from_rotation_z(-30.0f32.to_radians()))
                    .with_scale(OFFHAND_SCALE),
            },
        );

        // Left direction
        m.insert(
            FacingDirection::Left,
            EquipmentTransform {
                mainhand: Transform::from_xyz(-8.0, -15.0, ZLayer::BehindSprite.z())
                    .with_rotation(Quat::from_rotation_z(90.0f32.to_radians()))
                    .with_scale(MAINHAND_SCALE),
                offhand: Transform::from_xyz(1.0, -15.0, ZLayer::AboveSprite.z())
                    .with_rotation(Quat::from_rotation_z(90.0f32.to_radians()))
                    .with_scale(OFFHAND_SCALE),
            },
        );

        // Right direction
        m.insert(
            FacingDirection::Right,
            EquipmentTransform {
                mainhand: Transform::from_xyz(8.0, -15.0, ZLayer::AboveSprite.z())
                    .with_rotation(Quat::from_rotation_z(-90.0f32.to_radians()))
                    .with_scale(MAINHAND_SCALE),
                offhand: Transform::from_xyz(8.0, -15.0, ZLayer::BehindSprite.z())
                    .with_rotation(Quat::from_rotation_z(-90.0f32.to_radians()))
                    .with_scale(OFFHAND_SCALE),
            },
        );

        m
    });

impl EquipmentTransform {
    pub fn get(direction: FacingDirection) -> Result<Self> {
        EQUIPMENT_TRANSFORM_MAP
            .get(&direction)
            .copied()
            .ok_or(BevyError::from("Direction not found"))
    }
}

pub fn update_equipment_transforms(
    all_worn_equipment: Query<
        (Option<&Mainhand>, Option<&Offhand>, &FacingDirection),
        (
            Or<(
                // Update when holder changes direction
                Changed<FacingDirection>,
                // Update when holder stops attacking, stops casting, etc...
                Changed<AttackState>,
                // Update when new item is equipped
                Changed<Mainhand>,
                Changed<Offhand>,
            )>,
            With<Items>,
        ),
    >,
    mut transforms: Query<&mut Transform, (With<Equipped>, Without<ActiveMeleeAttack>)>,
) -> Result {
    for (mainhand, offhand, direction) in &all_worn_equipment {
        let direction_transforms = EquipmentTransform::get(*direction)?;

        // Update mainhand equipment
        if let Some(Mainhand(mainhand)) = mainhand
            && let Ok(mut transform) = transforms.get_mut(*mainhand)
        {
            *transform = direction_transforms.mainhand;
        }

        // Update offhand equipment
        if let Some(Offhand(offhand)) = offhand
            && let Ok(mut transform) = transforms.get_mut(*offhand)
        {
            *transform = direction_transforms.offhand;
        }
    }
    Ok(())
}
