use bevy::{platform::collections::HashMap, prelude::*};

use std::sync::LazyLock;

use crate::{
    items::{
        Items,
        equipment::{Equipped, Mainhand, Offhand},
    },
    prelude::*,
};

// Simple transform tuple for concise definitions
pub struct EquipmentTransformDef {
    pos: (f32, f32),
    rotation: f32, // in radians
    z_layer: ZLayer,
}

impl From<(f32, f32)> for EquipmentTransformDef {
    fn from(pos: (f32, f32)) -> Self {
        EquipmentTransformDef {
            pos,
            rotation: 0.0,
            z_layer: ZLayer::AboveSprite,
        }
    }
}

impl From<((f32, f32), f32)> for EquipmentTransformDef {
    fn from((pos, rotation): ((f32, f32), f32)) -> Self {
        EquipmentTransformDef {
            pos,
            rotation: rotation.to_radians(),
            z_layer: ZLayer::AboveSprite,
        }
    }
}

impl From<((f32, f32), f32, ZLayer)> for EquipmentTransformDef {
    fn from((pos, rotation, z_layer): ((f32, f32), f32, ZLayer)) -> Self {
        EquipmentTransformDef {
            pos,
            rotation: rotation.to_radians(),
            z_layer,
        }
    }
}

// Convert EquipmentTransformDef to actual Transform
impl From<EquipmentTransformDef> for Transform {
    fn from(def: EquipmentTransformDef) -> Self {
        Transform::from_xyz(def.pos.0, def.pos.1, def.z_layer.z())
            .with_rotation(Quat::from_rotation_z(def.rotation))
    }
}
#[macro_export]
macro_rules! equipment_transforms {
    ([$(($dir:ident, $def:expr)),* $(,)?]) => {
        LazyLock::new(|| {
            HashMap::from([
                $(
                    (FacingDirection::$dir, Transform::from(EquipmentTransformDef::from($def)))
                ),*
            ])
        })
    };
}

use ZLayer::BehindSprite;
pub static DEFAULT_EQUIPMENT_TRANSFORM_MAP: LazyLock<HashMap<FacingDirection, Transform>> =
    equipment_transforms!([
        (Up, ((0.0, -8.0), 30.0)),
        (Down, ((0.0, 8.0), -30.0, BehindSprite)),
        (Left, ((1.0, -15.0), 90.0)),
        (Right, ((8.0, -15.0), -90.0, BehindSprite)),
    ]);

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
        && let Some(new_transform) = equippable.transforms.get(&direction)
    {
        *transform = *new_transform;
    }
}
