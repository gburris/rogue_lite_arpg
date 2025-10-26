use bevy::prelude::*;

use crate::prelude::{Menu, PauseInputEvent, PlayerInteraction};

pub(super) fn on_shop_keeper_store_open(_: On<PlayerInteraction>, mut commands: Commands) {
    commands.trigger(PauseInputEvent {
        menu: Some(Menu::Inventory),
    });
}

pub(super) fn on_stat_trainer_store_open(_: On<PlayerInteraction>, mut commands: Commands) {
    commands.trigger(PauseInputEvent {
        menu: Some(Menu::StatsShop),
    });
}

pub(super) fn on_game_guide_start(_: On<PlayerInteraction>, mut _commands: Commands) {
    warn!("on_game_guide_start");
}
