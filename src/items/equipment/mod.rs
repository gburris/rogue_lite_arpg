use core::fmt;
use std::sync::LazyLock;

use bevy::{platform::collections::HashMap, prelude::*};

mod equip;
mod equipment_transform;
mod use_equipped;

use crate::prelude::*;

pub mod prelude {
    pub use super::equip::*;
    pub use super::equipment_transform::*;
    pub use super::use_equipped::*;
    pub use super::{
        EquipmentSlot, Equippable, Equipped, Mainhand, MainhandOf, Offhand, OffhandOf,
    };
}

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(equip::plugin);

    app.add_systems(
        Update,
        (
            // Always run this system InGame and InMenu so weapon transforms update as inventory is interacted with
            equipment_transform::update_equipment_transforms.in_set(MainSystems::Shared),
            use_equipped::tick_equippable_cooldown.in_set(InGameSystems::Simulation),
        ),
    );
}

/// An item that can be equipped and used from a slot. `reach` is the farthest
/// target distance an AI holder uses it at; `None` means AI never aims it.
#[derive(Component, Clone, Debug)]
pub struct Equippable {
    pub slot: EquipmentSlot,
    pub cooldown: Timer, // swing a sword, shoot a weapon, etc...
    pub transforms: &'static HashMap<FacingDirection, Transform>,
    pub reach: Option<f32>,
}

impl Equippable {
    pub fn new(
        slot: EquipmentSlot,
        duration: f32,
        transforms: &'static LazyLock<HashMap<FacingDirection, Transform>>,
    ) -> Self {
        Equippable {
            slot,
            transforms,
            cooldown: Timer::from_seconds(duration, TimerMode::Once),
            reach: None,
        }
    }

    /// Lets an AI holder use this item on targets up to `reach` away.
    pub fn with_reach(self, reach: f32) -> Self {
        Self {
            reach: Some(reach),
            ..self
        }
    }
}

/// Goes on the equipment marking where it should be equipped
///
/// Note: We pass this by value a lot, don't add data to it without consideration for passing this by reference
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum EquipmentSlot {
    Mainhand,
    Offhand,
}

impl EquipmentSlot {
    pub const ALL: [Self; 2] = [Self::Mainhand, Self::Offhand];

    /// Item currently equipped in this slot, if any.
    pub fn get_equipped(
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

    /// Whether the equipment in this slot is used instantly (sword swing) rather
    /// than held until released (shield block).
    pub fn is_instant_use(self) -> bool {
        matches!(self, Self::Mainhand)
    }
}

impl fmt::Display for EquipmentSlot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let variant_name = match *self {
            EquipmentSlot::Mainhand => "Main Hand",
            EquipmentSlot::Offhand => "Off Hand",
        };
        write!(f, "{variant_name}")
    }
}

#[derive(Component, Clone, Debug)]
pub struct Equipped;

#[derive(Component, Clone)]
#[relationship(relationship_target = Mainhand)]
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
pub struct OffhandOf(pub Entity);

#[derive(Component, Clone, Debug)]
#[relationship_target(relationship = OffhandOf)]
pub struct Offhand(Entity);

impl Offhand {
    pub fn get(&self) -> Entity {
        self.0
    }
}
