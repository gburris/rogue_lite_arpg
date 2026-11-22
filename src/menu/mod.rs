mod inventory;
mod pause;
mod stats_shop;

pub mod prelude {
    pub use super::Menu;
    pub use super::pause::pause_actions;
}

use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use self::pause::{MenuBackground, PauseControl};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((inventory::plugin, pause::plugin, stats_shop::plugin));

    app.init_state::<Menu>();

    app.add_systems(OnEnter(Menu::None), pause::unpause)
        .add_systems(OnExit(Menu::None), (pause::pause, pause::release_cursor))
        .add_input_context::<PauseControl>()
        .add_input_context::<MenuBackground>()
        .add_observer(pause::on_menu_back)
        .add_observer(pause::on_toggle_pause)
        .add_observer(pause::on_toggle_inventory);
}

#[derive(States, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Menu {
    #[default]
    None, // In Game
    Pause,
    Inventory,
    StatsShop,
    ItemsShop,
    Stats,
}
