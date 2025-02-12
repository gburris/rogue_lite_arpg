use bevy::prelude::*;

use crate::items::Autoloot;

// This is needed to tell auto loot systems to not treat gold as an item
#[derive(Component)]
#[require(Autoloot)]
pub struct Currency {
    pub value: u32,
}

#[derive(Event)]
pub struct GoldDropEvent {
    pub drop_location: Transform,
    pub amount: u32,
}
