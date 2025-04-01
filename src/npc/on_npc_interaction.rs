use bevy::prelude::*;

use crate::{
    controller::plugin::{Interact, PauseEvent},
    labels::states::PausedState,
};

pub fn on_shop_keeper_store_open(_: Trigger<Interact>, mut commands: Commands) {
    commands.trigger(PauseEvent(Some(PausedState::Inventory)));
}

pub fn on_stat_trainer_store_open(_: Trigger<Interact>, mut commands: Commands) {
    commands.trigger(PauseEvent(Some(PausedState::StatsShop)));
}

pub fn on_game_guide_start(_: Trigger<Interact>, mut _commands: Commands) {
    warn!("on_game_guide_start");
}
