use bevy::prelude::*;

#[derive(Component, Clone, Copy)]
pub struct PlayerStats {
    pub agility: u32,   //Movement speed, roll range
    pub strength: u32,  //Melee swing damage
    pub dexterity: u32, //Critical Stike Change
    pub intellect: u32, //Spell damage
    pub luck: u32,      //Drop rate
}

impl Default for PlayerStats {
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DisplayableStatType {
    Agility,
    Strength,
    Dexterity,
    Intellect,
    Luck,
}

impl DisplayableStatType {
    pub fn description(self) -> &'static str {
        match self {
            Self::Agility => "Movement speed, roll range",
            Self::Strength => "Melee swing damage",
            Self::Dexterity => "Critical Strike Chance",
            Self::Intellect => "Spell damage",
            Self::Luck => "Drop rate",
        }
    }

    pub fn value(self, stats: &PlayerStats) -> u32 {
        match self {
            Self::Agility => stats.agility,
            Self::Strength => stats.strength,
            Self::Dexterity => stats.dexterity,
            Self::Intellect => stats.intellect,
            Self::Luck => stats.luck,
        }
    }
}
