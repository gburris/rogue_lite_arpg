use bevy::prelude::*;

use crate::player::systems::{input::Interact, PauseEvent};

pub fn on_shop_keeper_store_open(_: Trigger<Interact>, mut commands: Commands) {
    commands.trigger(PauseEvent::Inventory);
}

pub fn on_stat_trainer_store_open(_: Trigger<Interact>, mut commands: Commands) {
    commands.trigger(PauseEvent::StatsShop);
}

pub fn on_game_guide_start(_: Trigger<Interact>, mut _commands: Commands) {
    warn!("on_game_guide_start");
}
