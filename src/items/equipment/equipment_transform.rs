use super::equipment_slots::EquipmentSlots;
use crate::animation::MovementDirection;
use crate::combat::components::ActionState;
use crate::labels::layer::ZLayer;
use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::OnceLock;

// Constants for transform values
const MAINHAND_SCALE: Vec3 = Vec3::new(1.0, 1.0, 1.0);
const HEAD_SCALE: Vec3 = Vec3::new(0.15, 0.15, 1.0);

#[derive(Clone, Copy)]
pub struct DirectionTransforms {
    pub mainhand: Transform,
    pub head: Transform,
}

//You wish this wasn't like this but it is
//See std lib example here https://crates.io/crates/lazy_static
fn direction_transforms() -> &'static HashMap<MovementDirection, DirectionTransforms> {
    static TRANSFORMS: OnceLock<HashMap<MovementDirection, DirectionTransforms>> = OnceLock::new();
    TRANSFORMS.get_or_init(|| {
        let mut m = HashMap::new();

        // Up direction
        m.insert(
            MovementDirection::Up,
            DirectionTransforms {
                mainhand: Transform::from_xyz(0.0, -8.0, ZLayer::WeaponAboveSprite.z())
                    .with_rotation(Quat::from_rotation_z(30.0f32.to_radians()))
                    .with_scale(MAINHAND_SCALE),
                head: Transform::from_xyz(0.0, 5.0, ZLayer::WeaponAboveSprite.z())
                    .with_scale(HEAD_SCALE),
            },
        );

        // Down direction
        m.insert(
            MovementDirection::Down,
            DirectionTransforms {
                mainhand: Transform::from_xyz(0.0, 8.0, ZLayer::WeaponBehindSprite.z())
                    .with_rotation(Quat::from_rotation_z(-30.0f32.to_radians()))
                    .with_scale(MAINHAND_SCALE),
                head: Transform::from_xyz(0.0, 0.0, ZLayer::WeaponAboveSprite.z())
                    .with_scale(HEAD_SCALE),
            },
        );

        // Left direction
        m.insert(
            MovementDirection::Left,
            DirectionTransforms {
                mainhand: Transform::from_xyz(-8.0, -15.0, ZLayer::WeaponBehindSprite.z())
                    .with_rotation(Quat::from_rotation_z(90.0f32.to_radians()))
                    .with_scale(MAINHAND_SCALE),
                head: Transform::from_xyz(-5.0, 0.0, ZLayer::WeaponAboveSprite.z())
                    .with_scale(HEAD_SCALE),
            },
        );

        // Right direction
        m.insert(
            MovementDirection::Right,
            DirectionTransforms {
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

impl DirectionTransforms {
    pub fn get(direction: MovementDirection) -> Self {
        // We default to down equipment position if entity is not moving
        let direction = if direction == MovementDirection::None {
            MovementDirection::Down
        } else {
            direction
        };

        direction_transforms().get(&direction).copied().unwrap()
    }
}

pub fn update_equipment_transforms(
    all_worn_equipment_in_game: Query<
        (&EquipmentSlots, &ActionState, &MovementDirection),
        Changed<MovementDirection>,
    >,
    mut transforms: Query<&mut Transform>,
) {
    for (equipment_slots, action_state, direction) in &all_worn_equipment_in_game {
        if *direction == MovementDirection::None || *action_state == ActionState::Attacking {
            return;
        }

        let direction_transforms = DirectionTransforms::get(*direction);

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
