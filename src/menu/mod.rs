mod inventory;
mod pause;
mod player_stats;
mod stats_shop;

pub mod prelude {
    pub use super::Menu;
    pub(crate) use super::{PauseControl, pause_actions};
}

use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use self::pause::MenuBackground;
pub(crate) use self::pause::{PauseControl, pause_actions};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        inventory::plugin,
        pause::plugin,
        player_stats::plugin,
        stats_shop::plugin,
    ));

    app.init_state::<Menu>();

    app.add_systems(OnEnter(Menu::None), pause::unpause)
        .add_systems(OnExit(Menu::None), (pause::pause, pause::release_cursor))
        .add_input_context::<PauseControl>()
        .add_input_context::<MenuBackground>()
        .add_observer(pause::on_menu_back)
        .add_observer(pause::on_toggle_pause);
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
