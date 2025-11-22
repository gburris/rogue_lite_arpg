use bevy::prelude::*;

use std::{collections::HashMap, sync::LazyLock};

use crate::{
    items::{
        Items,
        equipment::{EquipmentType, Equipped, Mainhand, Offhand},
        melee::ActiveMeleeAttack,
    },
    prelude::*,
};

#[derive(Clone, Copy)]
pub struct EquipmentTransform {
    pub mainhand: Transform,
    pub offhand: Transform,
}

// Simple transform tuple for concise definitions
struct EquipmentTransformDef {
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

macro_rules! add_equipment_transforms {
    ($map:ident, $equip_type:expr, [$(($dir:ident, $def:expr)),* $(,)?]) => {
        $map.insert(
            $equip_type,
            HashMap::from([
                $(
                    (FacingDirection::$dir, Transform::from(EquipmentTransformDef::from($def)))
                ),*
            ])
        );
    };
}

static NEW_EQUIPMENT_TRANSFORM_MAP: LazyLock<
    HashMap<EquipmentType, HashMap<FacingDirection, Transform>>,
> = LazyLock::new(|| {
    use ZLayer::BehindSprite;

    let mut equipment_map = HashMap::new();

    add_equipment_transforms!(
        equipment_map,
        EquipmentType::Sword,
        [
            (Up, ((0.0, -8.0), 30.0)),
            (Down, ((0.0, 8.0), -30.0, BehindSprite)),
            (Left, ((-8.0, -15.0), 90.0, BehindSprite)),
            (Right, ((8.0, -15.0), -90.0)),
        ]
    );

    add_equipment_transforms!(
        equipment_map,
        EquipmentType::Staff,
        [
            (Up, ((10.0, 2.0), 0.0, BehindSprite)),
            (Down, ((-10.0, 0.0), 0.0)),
            (Left, ((-4.0, -7.0), 50.0, BehindSprite)),
            (Right, ((4.0, -8.0), -50.0)),
        ]
    );

    add_equipment_transforms!(
        equipment_map,
        EquipmentType::Axe,
        [
            (Up, ((0.0, -8.0), 30.0)),
            (Down, ((0.0, 8.0), -30.0, BehindSprite)),
            (Left, ((-8.0, -15.0), 90.0, BehindSprite)),
            (Right, ((8.0, -15.0), -90.0)),
        ]
    );

    add_equipment_transforms!(
        equipment_map,
        EquipmentType::Spellbook,
        [
            (Up, ((0.0, -8.0), 30.0)),
            (Down, ((0.0, 8.0), -30.0, BehindSprite)),
            (Left, ((1.0, -15.0), 90.0)),
            (Right, ((8.0, -15.0), -90.0, BehindSprite)),
        ]
    );

    add_equipment_transforms!(
        equipment_map,
        EquipmentType::Shield,
        [
            (Up, ((0.0, -8.0), 30.0)),
            (Down, ((0.0, 8.0), -30.0, BehindSprite)),
            (Left, ((1.0, -15.0), 90.0)),
            (Right, ((8.0, -15.0), -90.0, BehindSprite)),
        ]
    );

    equipment_map
});

pub(super) fn update_equipment_transforms(
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
    mut transforms: Query<
        (&Equippable, &mut Transform),
        (With<Equipped>, Without<ActiveMeleeAttack>),
    >,
) {
    for (mainhand, offhand, direction) in &all_worn_equipment {
        update_single_equipment(mainhand.map(|Mainhand(e)| *e), direction, &mut transforms);
        update_single_equipment(offhand.map(|Offhand(e)| *e), direction, &mut transforms);
    }
}

fn update_single_equipment(
    equipment_entity: Option<Entity>,
    direction: &FacingDirection,
    transforms: &mut Query<
        (&Equippable, &mut Transform),
        (With<Equipped>, Without<ActiveMeleeAttack>),
    >,
) {
    if let Some(entity) = equipment_entity
        && let Ok((equippable, mut transform)) = transforms.get_mut(entity)
    {
        let new_transform = NEW_EQUIPMENT_TRANSFORM_MAP
            .get(&equippable.equip_type)
            .and_then(|dir_map| dir_map.get(direction));

        if let Some(new_transform) = new_transform {
            *transform = *new_transform;
        }
    }
}
