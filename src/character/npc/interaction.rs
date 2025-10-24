use bevy::prelude::*;

use crate::{
    character::player::{PauseInputEvent, interact::Interaction},
    prelude::PausedState,
};

pub fn on_shop_keeper_store_open(_: On<Interaction>, mut commands: Commands) {
    commands.trigger(PauseInputEvent {
        paused_state: Some(PausedState::Inventory),
    });
}

pub fn on_stat_trainer_store_open(_: On<Interaction>, mut commands: Commands) {
    commands.trigger(PauseInputEvent {
        paused_state: Some(PausedState::StatsShop),
    });
}

pub fn on_game_guide_start(_: On<Interaction>, mut _commands: Commands) {
    warn!("on_game_guide_start");
}
