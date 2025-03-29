use bevy::prelude::*;

#[derive(Component, Clone)]
pub struct PlayerStats {
    pub agility: u32,   //Movement speed, roll range
    pub strength: u32,  //Melee swing damage
    pub dexterity: u32, //Critical Stike Change
    pub intellect: u32, //Spell damage
    pub luck: u32,      //Drop rate
}

impl Default for PlayerStats {
    fn default() -> Self {
        PlayerStats {
            agility: 1,
            strength: 1,
            dexterity: 1,
            intellect: 1,
            luck: 99,
        }
    }
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum DisplayableStatType {
    Agility,
    Strength,
    Dexterity,
    Intellect,
    Luck,
}

impl DisplayableStatType {
    pub fn get_description(&self) -> &'static str {
        match self {
            DisplayableStatType::Agility => "Movement speed, roll range",
            DisplayableStatType::Strength => "Melee swing damage",
            DisplayableStatType::Dexterity => "Critical Strike Chance",
            DisplayableStatType::Intellect => "Spell damage",
            DisplayableStatType::Luck => "Drop rate",
        }
    }

    pub fn get_value(&self, stats: &PlayerStats) -> u32 {
        match self {
            DisplayableStatType::Agility => stats.agility,
            DisplayableStatType::Strength => stats.strength,
            DisplayableStatType::Dexterity => stats.dexterity,
            DisplayableStatType::Intellect => stats.intellect,
            DisplayableStatType::Luck => stats.luck,
        }
    }
}
