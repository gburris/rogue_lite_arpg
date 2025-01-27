use super::equipment::EquipmentSlots;
use crate::{labels::layer::ZLayer, player::movement::MovementDirection};
use bevy::prelude::*;

#[derive(Component)]
pub struct EquipmentTransform;

#[derive(Clone, Copy)]
pub struct DirectionTransforms {
    pub mainhand: Transform,
    pub head: Transform,
}

impl From<MovementDirection> for DirectionTransforms {
    fn from(direction: MovementDirection) -> Self {
        match direction {
            MovementDirection::Up => DirectionTransforms {
                mainhand: Transform::from_xyz(0.0, -8.0, ZLayer::WeaponAboveSprite.z())
                    .with_rotation(Quat::from_rotation_z(30.0f32.to_radians()))
                    .with_scale(Vec3::new(0.17, 0.17, 0.17)),
                head: Transform::from_xyz(0.0, 5.0, ZLayer::WeaponAboveSprite.z())
                    .with_scale(Vec3::new(0.15, 0.15, 0.15)),
            },
            MovementDirection::Down => DirectionTransforms {
                mainhand: Transform::from_xyz(0.0, 8.0, ZLayer::WeaponBehindSprite.z())
                    .with_rotation(Quat::from_rotation_z(-30.0f32.to_radians()))
                    .with_scale(Vec3::new(0.17, 0.17, 0.17)),
                head: Transform::from_xyz(0.0, 0.0, ZLayer::WeaponAboveSprite.z())
                    .with_scale(Vec3::new(0.15, 0.15, 0.15)),
            },
            MovementDirection::Left => DirectionTransforms {
                mainhand: Transform::from_xyz(-8.0, -15.0, ZLayer::WeaponBehindSprite.z())
                    .with_rotation(Quat::from_rotation_z(90.0f32.to_radians()))
                    .with_scale(Vec3::new(0.17, 0.17, 0.17)),
                head: Transform::from_xyz(-5.0, 0.0, ZLayer::WeaponAboveSprite.z())
                    .with_scale(Vec3::new(0.15, 0.15, 0.15)),
            },
            MovementDirection::Right => DirectionTransforms {
                mainhand: Transform::from_xyz(8.0, -15.0, ZLayer::WeaponAboveSprite.z())
                    .with_rotation(Quat::from_rotation_z(-90.0f32.to_radians()))
                    .with_scale(Vec3::new(0.17, 0.17, 0.17)),
                head: Transform::from_xyz(5.0, 0.0, ZLayer::WeaponAboveSprite.z())
                    .with_scale(Vec3::new(0.15, 0.15, 0.15)),
            },
            MovementDirection::None => {
                // Use default/down position or maintain current
                MovementDirection::Down.into()
            }
        }
    }
}

pub fn update_equipment_transforms(
    all_worn_equipment_in_game: Query<
        (&EquipmentSlots, &MovementDirection),
        Changed<MovementDirection>,
    >,
    mut transforms: Query<&mut Transform>,
) {
    for (equipment_slots, direction) in &all_worn_equipment_in_game {
        if *direction == MovementDirection::None {
            return;
            //Don't update it in this case.
        }
        let direction_transforms = DirectionTransforms::from(*direction);

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
