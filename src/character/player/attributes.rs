use bevy::prelude::*;

/// The player's upgradeable attributes.
#[derive(Component, Clone, Copy)]
pub struct PlayerAttributes {
    pub agility: u32,   // Movement speed, roll range
    pub strength: u32,  // Melee swing damage
    pub dexterity: u32, // Critical Strike Chance
    pub intellect: u32, // Spell damage
    pub luck: u32,      // Drop rate
}

impl Default for PlayerAttributes {
    fn default() -> Self {
        Self {
            agility: 1,
            strength: 1,
            dexterity: 1,
            intellect: 1,
            luck: 99,
        }
    }
}

/// One of the [`PlayerAttributes`] fields, for shop UI and attribute changes.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Attribute {
    #[default]
    Agility,
    Strength,
    Dexterity,
    Intellect,
    Luck,
}

impl Attribute {
    /// Every attribute, in the order menus list them.
    pub const ALL: [Self; 5] = [
        Self::Agility,
        Self::Strength,
        Self::Dexterity,
        Self::Intellect,
        Self::Luck,
    ];

    pub fn description(self) -> &'static str {
        match self {
            Self::Agility => "Movement speed, roll range",
            Self::Strength => "Melee swing damage",
            Self::Dexterity => "Critical Strike Chance",
            Self::Intellect => "Spell damage",
            Self::Luck => "Drop rate",
        }
    }

    pub fn value(self, attributes: &PlayerAttributes) -> u32 {
        match self {
            Self::Agility => attributes.agility,
            Self::Strength => attributes.strength,
            Self::Dexterity => attributes.dexterity,
            Self::Intellect => attributes.intellect,
            Self::Luck => attributes.luck,
        }
    }
}
