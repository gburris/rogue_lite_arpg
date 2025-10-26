mod inventory;
mod pause;
mod player_stats;
mod stats_shop;

pub mod prelude {
    pub use super::Menu;
}

use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use crate::{prelude::*, ui::constants::BACKGROUND_COLOR};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        inventory::plugin,
        pause::plugin,
        player_stats::plugin,
        stats_shop::plugin,
    ));

    app.init_state::<Menu>();

    app.add_systems(OnExit(Menu::None), pause)
        .add_input_context::<MenuBackground>()
        .add_observer(on_resume);
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

#[derive(InputAction)]
#[action_output(bool)]

struct ResumeGame;

fn on_resume(_: On<Start<ResumeGame>>, mut next_menu_state: ResMut<NextState<Menu>>) {
    next_menu_state.set(Menu::None);
}

fn pause(mut next_pause_state: ResMut<NextState<Pause>>, mut commands: Commands) {
    commands.spawn(menu_background());
    next_pause_state.set(Pause(true));
}

#[derive(Component)]
struct MenuBackground;

fn menu_background() -> impl Bundle {
    (
        Name::new("Pause Overlay"),
        MenuBackground,
        DespawnOnEnter(Menu::None),
        actions!(MenuBackground[
            (
                Action::<ResumeGame>::new(),
                ActionSettings {
                    require_reset: true,
                    ..Default::default()
                },
                bindings![KeyCode::Escape, GamepadButton::Start],
            ),
        ]),
        Node {
            width: percent(100),
            height: percent(100),
            ..default()
        },
        GlobalZIndex(1),
        BackgroundColor::from(BACKGROUND_COLOR),
    )
}
