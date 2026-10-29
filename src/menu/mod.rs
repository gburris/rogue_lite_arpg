mod inventory;
mod pause;
mod player_stats;
mod stats_shop;

pub mod prelude {
    pub use super::Menu;
}

use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
pub(crate) use pause::{PauseControl, pause_actions};

use self::pause::{MenuBackground, on_menu_back, on_toggle_pause, pause, release_cursor, unpause};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        inventory::plugin,
        pause::plugin,
        player_stats::plugin,
        stats_shop::plugin,
    ));

    app.init_state::<Menu>();

    app.add_systems(OnEnter(Menu::None), unpause)
        .add_systems(OnExit(Menu::None), (pause, release_cursor))
        .add_input_context::<PauseControl>()
        .add_input_context::<MenuBackground>()
        .add_observer(on_menu_back)
        .add_observer(on_toggle_pause);
}

#[derive(States, Eq, Default, Hash, Clone, Copy, Debug, PartialEq)]
pub enum Menu {
    #[default]
    None, // In Game
    Pause,
    Inventory,
    StatsShop,
    ItemsShop,
    Stats,
}
