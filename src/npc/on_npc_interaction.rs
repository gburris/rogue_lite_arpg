use bevy::prelude::*;

use crate::{
    controller::plugin::PauseInputEvent, labels::states::PausedState,
    player::interact::InteractionEvent,
};

pub fn on_shop_keeper_store_open(_: Trigger<InteractionEvent>, mut commands: Commands) {
    commands.trigger(PauseInputEvent(Some(PausedState::Inventory)));
}

pub fn on_stat_trainer_store_open(_: Trigger<InteractionEvent>, mut commands: Commands) {
    commands.trigger(PauseInputEvent(Some(PausedState::StatsShop)));
}

pub fn on_game_guide_start(_: Trigger<InteractionEvent>, mut _commands: Commands) {
    warn!("on_game_guide_start");
}
