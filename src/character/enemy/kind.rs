use super::{bat, mage, warrior};
use crate::character::brain::BrainDefinition;

/// Recipes name these directly, e.g. `(what: Enemy(FireMage), ...)` in `map_recipes.ron`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, serde::Deserialize)]
pub enum EnemyKind {
    Warrior,
    IceMage,
    FireMage,
    /// Flying enemy: circles its target and breathes fire without a weapon.
    Bat,
}

impl EnemyKind {
    /// Every enemy kind, used for random spawns and exhaustive iteration.
    pub const ALL: &[Self] = &[Self::Warrior, Self::IceMage, Self::FireMage, Self::Bat];

    pub fn brain(self) -> &'static BrainDefinition {
        match self {
            Self::Warrior => &warrior::WARRIOR,
            Self::IceMage => &mage::ICE_MAGE,
            Self::FireMage => &mage::FIRE_MAGE,
            Self::Bat => &bat::BAT,
        }
    }
}
