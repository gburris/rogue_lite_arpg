use super::equipment_slots::EquipmentSlots;
use crate::animation::FacingDirection;
use crate::combat::components::ActionState;
use crate::labels::layer::ZLayer;
use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::OnceLock;

// Constants for transform values
const MAINHAND_SCALE: Vec3 = Vec3::new(1.0, 1.0, 1.0);
const HEAD_SCALE: Vec3 = Vec3::new(0.15, 0.15, 1.0);

#[derive(Clone, Copy)]
pub struct EquipmentTransform {
    pub mainhand: Transform,
    pub head: Transform,
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
                head: Transform::from_xyz(0.0, 5.0, ZLayer::WeaponAboveSprite.z())
                    .with_scale(HEAD_SCALE),
            },
        );

        // Down direction
        m.insert(
            FacingDirection::Down,
            EquipmentTransform {
                mainhand: Transform::from_xyz(0.0, 8.0, ZLayer::WeaponBehindSprite.z())
                    .with_rotation(Quat::from_rotation_z(-30.0f32.to_radians()))
                    .with_scale(MAINHAND_SCALE),
                head: Transform::from_xyz(0.0, 0.0, ZLayer::WeaponAboveSprite.z())
                    .with_scale(HEAD_SCALE),
            },
        );

        // Left direction
        m.insert(
            FacingDirection::Left,
            EquipmentTransform {
                mainhand: Transform::from_xyz(-8.0, -15.0, ZLayer::WeaponBehindSprite.z())
                    .with_rotation(Quat::from_rotation_z(90.0f32.to_radians()))
                    .with_scale(MAINHAND_SCALE),
                head: Transform::from_xyz(-5.0, 0.0, ZLayer::WeaponAboveSprite.z())
                    .with_scale(HEAD_SCALE),
            },
        );

        // Right direction
        m.insert(
            FacingDirection::Right,
            EquipmentTransform {
                mainhand: Transform::from_xyz(8.0, -15.0, ZLayer::WeaponAboveSprite.z())
                    .with_rotation(Quat::from_rotation_z(-90.0f32.to_radians()))
                    .with_scale(MAINHAND_SCALE),
                head: Transform::from_xyz(5.0, 0.0, ZLayer::WeaponAboveSprite.z())
                    .with_scale(HEAD_SCALE),
            },
        );

        m
    })
}

fn defeated_transform() -> &'static EquipmentTransform {
    static TRANSFORM: OnceLock<EquipmentTransform> = OnceLock::new();
    TRANSFORM.get_or_init(|| EquipmentTransform {
        mainhand: Transform::from_xyz(25.0, -55.0, ZLayer::WeaponBehindSprite.z())
            .with_rotation(Quat::from_rotation_z(90.0f32.to_radians()))
            .with_scale(MAINHAND_SCALE),
        head: Transform::from_xyz(0.0, -5.0, ZLayer::WeaponBehindSprite.z()).with_scale(HEAD_SCALE),
    })
}

impl EquipmentTransform {
    pub fn get(direction: FacingDirection) -> Self {
        direction_transforms().get(&direction).copied().unwrap()
    }
    pub fn get_defeated() -> Self {
        *defeated_transform()
    }
}

pub fn update_equipment_transforms(
    all_worn_equipment_in_game: Query<
        (&EquipmentSlots, &ActionState, &FacingDirection),
        Or<(Changed<FacingDirection>, Changed<ActionState>)>,
    >,
    mut transforms: Query<&mut Transform>,
) {
    for (equipment_slots, action_state, direction) in &all_worn_equipment_in_game {
        if *action_state == ActionState::Attacking {
            return;
        }

        let direction_transforms = if *action_state == ActionState::Defeated {
            EquipmentTransform::get_defeated()
        } else {
            EquipmentTransform::get(*direction)
        };

        // Update mainhand equipment
        if let Some(entity) = equipment_slots.mainhand {
            if let Ok(mut transform) = transforms.get_mut(entity) {
                *transform = direction_transforms.mainhand;
            }
        }

        // Update head equipment
        if let Some(entity) = equipment_slots.head {
            if let Ok(mut transform) = transforms.get_mut(entity) {
                *transform = direction_transforms.head;
            }
        }
    }
}
