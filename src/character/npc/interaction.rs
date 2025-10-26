use bevy::prelude::*;

use crate::prelude::{Menu, PlayerInteraction};

pub(super) fn on_shop_keeper_store_open(
    _: On<PlayerInteraction>,
    mut next_menu_state: ResMut<NextState<Menu>>,
) {
    next_menu_state.set(Menu::Inventory);
}

pub(super) fn on_stat_trainer_store_open(
    _: On<PlayerInteraction>,
    mut next_menu_state: ResMut<NextState<Menu>>,
) {
    next_menu_state.set(Menu::StatsShop);
}

pub(super) fn on_game_guide_start(_: On<PlayerInteraction>, mut _commands: Commands) {
    warn!("on_game_guide_start");
}
