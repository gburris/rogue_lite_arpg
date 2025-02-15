use bevy::prelude::*;

use crate::{
    labels::states::PausedState,
    player::{interact::InteractionEvent, systems::PauseInputEvent},
};

pub fn on_shop_keeper_store_open(_: Trigger<InteractionEvent>, mut commands: Commands) {
    commands.trigger(PauseInputEvent {
        paused_state: Some(PausedState::Inventory),
    });
}

pub fn on_stat_trainer_store_open(_: Trigger<InteractionEvent>, mut commands: Commands) {
    commands.trigger(PauseInputEvent {
        paused_state: Some(PausedState::StatsShop),
    });
}

pub fn on_game_guide_start(_: Trigger<InteractionEvent>, mut _commands: Commands) {
    warn!("on_game_guide_start");
}
