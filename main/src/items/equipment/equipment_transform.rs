use super::EquipmentSlot;
use crate::{
    ai::state::{ActionState, FacingDirection},
    items::inventory::Inventory,
    labels::layer::ZLayer,
};
use bevy::prelude::*;
use std::{collections::HashMap, sync::OnceLock};

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
fn direction_transforms() -> &'static HashMap<FacingDirection, EquipmentTransform> {
    static TRANSFORMS: OnceLock<HashMap<FacingDirection, EquipmentTransform>> = OnceLock::new();
    TRANSFORMS.get_or_init(|| {
        let mut m = HashMap::new();

        // Up direction
        m.insert(
            FacingDirection::Up,
            EquipmentTransform {
                mainhand: Transform::from_xyz(0.0, -8.0, ZLayer::WeaponAboveSprite.z())
                    .with_rotation(Quat::from_rotation_z(30.0f32.to_radians()))
                    .with_scale(MAINHAND_SCALE),
                offhand: Transform::from_xyz(0.0, -8.0, ZLayer::WeaponAboveSprite.z())
                    .with_rotation(Quat::from_rotation_z(30.0f32.to_radians()))
                    .with_scale(OFFHAND_SCALE),
            },
        );

        // Down direction
        m.insert(
            FacingDirection::Down,
            EquipmentTransform {
                mainhand: Transform::from_xyz(0.0, 8.0, ZLayer::WeaponBehindSprite.z())
                    .with_rotation(Quat::from_rotation_z(-30.0f32.to_radians()))
                    .with_scale(MAINHAND_SCALE),
                offhand: Transform::from_xyz(0.0, 8.0, ZLayer::WeaponBehindSprite.z())
                    .with_rotation(Quat::from_rotation_z(-30.0f32.to_radians()))
                    .with_scale(OFFHAND_SCALE),
            },
        );

        // Left direction
        m.insert(
            FacingDirection::Left,
            EquipmentTransform {
                mainhand: Transform::from_xyz(-8.0, -15.0, ZLayer::WeaponBehindSprite.z())
                    .with_rotation(Quat::from_rotation_z(90.0f32.to_radians()))
                    .with_scale(MAINHAND_SCALE),
                offhand: Transform::from_xyz(1.0, -15.0, ZLayer::WeaponAboveSprite.z())
                    .with_rotation(Quat::from_rotation_z(90.0f32.to_radians()))
                    .with_scale(OFFHAND_SCALE),
            },
        );

        // Right direction
        m.insert(
            FacingDirection::Right,
            EquipmentTransform {
                mainhand: Transform::from_xyz(8.0, -15.0, ZLayer::WeaponAboveSprite.z())
                    .with_rotation(Quat::from_rotation_z(-90.0f32.to_radians()))
                    .with_scale(MAINHAND_SCALE),
                offhand: Transform::from_xyz(8.0, -15.0, ZLayer::WeaponBehindSprite.z())
                    .with_rotation(Quat::from_rotation_z(-90.0f32.to_radians()))
                    .with_scale(OFFHAND_SCALE),
            },
        );

        m
    })
}

impl EquipmentTransform {
    pub fn get(direction: FacingDirection) -> Self {
        direction_transforms().get(&direction).copied().unwrap()
    }
}

pub fn update_equipment_transforms(
    all_worn_equipment: Query<
        (&Inventory, &ActionState, &FacingDirection),
        Or<(Changed<FacingDirection>, Changed<ActionState>)>,
    >,
    mut transforms: Query<&mut Transform>,
) {
    for (inventory, action_state, direction) in &all_worn_equipment {
        if *action_state == ActionState::Attacking {
            return;
        }

        let direction_transforms = EquipmentTransform::get(*direction);

        // Update mainhand equipment
        if let Some(entity) = inventory.get_equipped(EquipmentSlot::Mainhand) {
            if let Ok(mut transform) = transforms.get_mut(entity) {
                *transform = direction_transforms.mainhand;
            }
        }

        // Update offhand equipment
        if let Some(entity) = inventory.get_equipped(EquipmentSlot::Offhand) {
            if let Ok(mut transform) = transforms.get_mut(entity) {
                *transform = direction_transforms.offhand;
            }
        }
    }
}
