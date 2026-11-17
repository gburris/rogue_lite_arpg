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
    /// Equips into `slot`, resting in `poses` while not in use.
    pub fn new(slot: EquipmentSlot, poses: &'static DirectionalPoses) -> Self {
        Self { slot, poses }
    }
}

/// Present on an item while it sits in one of its holder's equipment slots. Required by the slot
/// relationships, and removed with them on unequip.
#[derive(Component, Clone, Debug, Default)]
pub struct Equipped;
