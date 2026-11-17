use core::fmt;

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

/// Goes on the equipment marking where it should be equipped
///
/// Note: We pass this by value a lot, don't add data to it without consideration for passing this
/// by reference.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum EquipmentSlot {
    #[default]
    Mainhand,
    Offhand,
}

impl EquipmentSlot {
    pub const ALL: [Self; 2] = [Self::Mainhand, Self::Offhand];

    /// Item currently equipped in this slot, if any.
    pub fn equipped(
        self,
        mainhand: Option<&Mainhand>,
        offhand: Option<&Offhand>,
    ) -> Option<Entity> {
        match self {
            Self::Mainhand => mainhand.map(Mainhand::get),
            Self::Offhand => offhand.map(Offhand::get),
        }
    }

    /// Insert the relationship that places `item` in this slot on `holder`.
    pub fn place_in_slot(self, item: &mut EntityCommands, holder: Entity) {
        match self {
            Self::Mainhand => item.insert(MainhandOf(holder)),
            Self::Offhand => item.insert(OffhandOf(holder)),
        };
    }
}

impl fmt::Display for EquipmentSlot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let variant_name = match *self {
            Self::Mainhand => "Main Hand",
            Self::Offhand => "Off Hand",
        };
        write!(f, "{variant_name}")
    }
}

#[derive(Component, Clone)]
#[relationship(relationship_target = Mainhand)]
#[require(Equipped)]
pub struct MainhandOf(pub Entity);

#[derive(Component, Clone, Debug)]
#[relationship_target(relationship = MainhandOf)]
pub struct Mainhand(Entity);

impl Mainhand {
    pub fn get(&self) -> Entity {
        self.0
    }
}

#[derive(Component, Clone)]
#[relationship(relationship_target = Offhand)]
#[require(Equipped)]
pub struct OffhandOf(pub Entity);

#[derive(Component, Clone, Debug)]
#[relationship_target(relationship = OffhandOf)]
pub struct Offhand(Entity);

impl Offhand {
    pub fn get(&self) -> Entity {
        self.0
    }
}
