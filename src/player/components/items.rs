use bevy::prelude::*;
use std::{collections::HashMap, fmt};

#[derive(Component, Clone, Debug)]
pub struct Item {
    pub name: String,
    pub stats: HashMap<StatType, i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StatType {
    SpellPower,
    CastSpeed,
    AttackDamage,
    Durability,
}

impl fmt::Display for StatType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let stat_name = match self {
            StatType::SpellPower => "Spell Power",
            StatType::CastSpeed => "Cast Speed",
            StatType::AttackDamage => "Attack Damage",
            StatType::Durability => "Durability",
        };
        write!(f, "{}", stat_name)
    }
}

impl Item {
    pub fn new(name: &str) -> Self {
        Item {
            name: name.to_string(),
            stats: HashMap::new(),
        }
    }

    pub fn add_stat(&mut self, stat_type: StatType, value: i32) {
        self.stats.insert(stat_type, value);
    }

    pub fn modify_stat(&mut self, stat_type: StatType, value: i32) {
        if let Some(stat) = self.stats.get_mut(&stat_type) {
            *stat = value;
        }
    }

    pub fn get_stat(&self, stat_type: &StatType) -> Option<i32> {
        self.stats.get(stat_type).copied()
    }
}
