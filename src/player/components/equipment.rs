use bevy::prelude::*;

use crate::items::get_default_sword;

use crate::items::Item;

#[derive(Component)]
pub struct PlayerEquipmentSlots {
    pub mainhand: Option<Item>,
    pub offhand: Option<Item>,
    pub head: Option<Item>,
    pub chest: Option<Item>,
    pub legs: Option<Item>,
    pub feet: Option<Item>,
    pub hands: Option<Item>,
    pub shoulders: Option<Item>,
    pub neck: Option<Item>,
    pub ring: Option<Item>,
    pub trinket: Option<Item>,
}

impl Default for PlayerEquipmentSlots {
    fn default() -> Self {
        PlayerEquipmentSlots {
            mainhand: Some(get_default_sword()),
            offhand: None,
            head: None,
            chest: None,
            legs: None,
            feet: None,
            hands: None,
            shoulders: None,
            neck: None,
            ring: None,
            trinket: None,
        }
    }
}
