use bevy::prelude::*;

use crate::{
    character::player::{interact::InteractionEvent, PauseInputEvent},
    labels::states::PausedState,
};

pub fn on_shop_keeper_store_open(_: On<InteractionEvent>, mut commands: Commands) {
    commands.trigger(PauseInputEvent {
        paused_state: Some(PausedState::Inventory),
    });
}

pub fn on_stat_trainer_store_open(_: On<InteractionEvent>, mut commands: Commands) {
    commands.trigger(PauseInputEvent {
        paused_state: Some(PausedState::StatsShop),
    });
}

pub fn on_game_guide_start(_: On<InteractionEvent>, mut _commands: Commands) {
    warn!("on_game_guide_start");
}
