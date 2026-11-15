use bevy::prelude::*;

use super::equipment_transform::DirectionalPoses;
use crate::prelude::*;

/// An item that can be equipped and used from a slot. `reach` is the farthest
/// target distance an AI holder uses it at; `None` means AI never aims it.
#[derive(Component, Clone, Debug)]
pub struct Equippable {
    pub slot: EquipmentSlot,
    pub cooldown: Timer, // swing a sword, shoot a weapon, etc...
    pub poses: &'static DirectionalPoses,
    pub reach: Option<f32>,
    pub use_style: EquipmentUseStyle,
}

impl Equippable {
    pub fn new(slot: EquipmentSlot, duration: f32, poses: &'static DirectionalPoses) -> Self {
        Self {
            slot,
            poses,
            cooldown: Timer::from_seconds(duration, TimerMode::Once),
            reach: None,
            use_style: EquipmentUseStyle::Instant,
        }
    }

    /// Makes each use of this item end the way `use_style` says.
    pub fn with_use_style(self, use_style: EquipmentUseStyle) -> Self {
        Self { use_style, ..self }
    }

    /// Lets an AI holder use this item on targets up to `reach` away.
    pub fn with_reach(self, reach: f32) -> Self {
        Self {
            reach: Some(reach),
            ..self
        }
    }
}

/// How a use of an item ends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EquipmentUseStyle {
    /// The use ends on its own, like a sword swing or a staff cast.
    Instant,
    /// The use lasts until its holder releases it, like a shield block.
    Held,
}

impl EquipmentUseStyle {
    /// Whether a use in this style ends without being released.
    pub fn ends_on_its_own(self) -> bool {
        match self {
            Self::Instant => true,
            Self::Held => false,
        }
    }
}

#[derive(Component, Clone, Debug)]
pub struct Equipped;
