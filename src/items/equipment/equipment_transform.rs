use bevy::prelude::*;

use super::{Equipped, Mainhand, Offhand};
use crate::items::Items;
use crate::prelude::*;

/// Where an equipped item sits relative to its holder for one facing direction.
#[derive(Clone, Copy, Debug)]
pub struct EquipmentPose {
    offset: Vec2,
    rotation_degrees: f32,
    z_layer: ZLayer,
}

impl EquipmentPose {
    /// A pose drawn above the holder's sprite.
    pub const fn above(x: f32, y: f32, rotation_degrees: f32) -> Self {
        Self {
            offset: Vec2::new(x, y),
            rotation_degrees,
            z_layer: ZLayer::AboveSprite,
        }
    }

    /// A pose drawn behind the holder's sprite.
    pub const fn behind(x: f32, y: f32, rotation_degrees: f32) -> Self {
        Self {
            z_layer: ZLayer::BehindSprite,
            ..Self::above(x, y, rotation_degrees)
        }
    }

    fn transform(self) -> Transform {
        Transform::from_xyz(self.offset.x, self.offset.y, self.z_layer.z())
            .with_rotation(Quat::from_rotation_z(self.rotation_degrees.to_radians()))
    }
}

/// An item's [`EquipmentPose`] for each way its holder can face.
#[derive(Debug)]
pub struct DirectionalPoses {
    pub up: EquipmentPose,
    pub down: EquipmentPose,
    pub left: EquipmentPose,
    pub right: EquipmentPose,
}

impl DirectionalPoses {
    /// The transform that places an item on a holder facing `direction`.
    pub fn transform(&self, direction: FacingDirection) -> Transform {
        match direction {
            FacingDirection::Up => self.up,
            FacingDirection::Down => self.down,
            FacingDirection::Left => self.left,
            FacingDirection::Right => self.right,
        }
        .transform()
    }
}

pub static DEFAULT_EQUIPMENT_POSES: DirectionalPoses = DirectionalPoses {
    up: EquipmentPose::above(0.0, -8.0, 30.0),
    down: EquipmentPose::behind(0.0, 8.0, -30.0),
    left: EquipmentPose::above(1.0, -15.0, 90.0),
    right: EquipmentPose::behind(8.0, -15.0, -90.0),
};

pub(super) fn update_equipment_transforms(
    all_worn_equipment: Query<
        (
            Option<&Mainhand>,
            Option<&Offhand>,
            &FacingDirection,
            &AttackState,
        ),
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
    mut transforms: Query<(&Equippable, &mut Transform), With<Equipped>>,
) {
    for (mainhand, offhand, direction, attack_state) in &all_worn_equipment {
        if !attack_state.is_attacking {
            update_single_equipment(mainhand.map(|Mainhand(e)| *e), *direction, &mut transforms);
            update_single_equipment(offhand.map(|Offhand(e)| *e), *direction, &mut transforms);
        }
    }
}

fn update_single_equipment(
    equipment_entity: Option<Entity>,
    direction: FacingDirection,
    transforms: &mut Query<(&Equippable, &mut Transform), With<Equipped>>,
) {
    if let Some(entity) = equipment_entity
        && let Ok((equippable, mut transform)) = transforms.get_mut(entity)
    {
        *transform = equippable.poses.transform(direction);
    }
}
