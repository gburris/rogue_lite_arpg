use bevy::prelude::*;

use crate::prelude::*;

/// Names an [`ActionSource`] of a character: the item equipped in a slot, or one of its abilities
/// by kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ActionKey {
    Equipped(EquipmentSlot),
    Ability(Ability),
}

impl ActionKey {
    /// Every key, equipment first.
    pub fn all() -> impl Iterator<Item = Self> {
        let equipment = EquipmentSlot::ALL.into_iter().map(Self::Equipped);
        equipment.chain(Ability::ALL.into_iter().map(Self::Ability))
    }
}

/// The farthest target distance at which an AI holder uses this action source.
#[derive(Component, Clone, Copy, Debug, Deref)]
pub struct ActionReach(pub f32);

/// Marks an action source whose use is an attack, like a swing, a cast or a bite.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Attack;

/// A kind of item the holder must carry to use this action source, such as a heal's potion.
#[derive(Component, Clone, Copy, Debug, Deref)]
pub struct RequiredItem(pub ItemKind);

/// How a use ends once it takes effect.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub enum ActionUseStyle {
    /// Ends the moment it takes effect, like a heal or a tome's spell.
    #[default]
    Instant,
    /// Plays out for `seconds` after it takes effect, then ends, like a sword swing or a staff
    /// cast.
    Timed { seconds: f32 },
    /// Lasts until it is stopped, like a shield block.
    Sustained,
}

impl ActionUseStyle {
    /// Whether a use in this style ends without being stopped.
    pub fn ends_on_its_own(self) -> bool {
        match self {
            Self::Instant | Self::Timed { .. } => true,
            Self::Sustained => false,
        }
    }
}

/// How long each use winds up before it takes effect, a window for a hit to interrupt it. Without
/// it, a use takes effect on the frame it starts.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct ActionWindup {
    pub seconds: f32,
}

/// Lights the holder in this color while a use winds up.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct ActionGlow(pub Color);
