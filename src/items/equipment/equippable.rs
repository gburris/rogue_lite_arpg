use bevy::prelude::*;

use super::equipment_transform::DirectionalPoses;
use crate::prelude::*;

/// An item that can be equipped in a slot and used from it.
#[derive(Component, Clone, Debug)]
#[require(ActionSource)]
pub struct Equippable {
    pub slot: EquipmentSlot,
    pub poses: &'static DirectionalPoses,
}

impl Equippable {
    pub fn new(slot: EquipmentSlot, poses: &'static DirectionalPoses) -> Self {
        Self { slot, poses }
    }
}

#[derive(Component, Clone, Debug)]
pub struct Equipped;
