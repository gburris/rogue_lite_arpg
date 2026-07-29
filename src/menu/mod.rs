mod inventory;
mod pause;
mod player_stats;
mod stats_shop;

pub mod prelude {
    pub use super::Menu;
}

use bevy::{
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
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

fn pause(mut next_pause_state: ResMut<NextState<Pause>>, mut commands: Commands) {
    commands.spawn(menu_background());
    next_pause_state.set(Pause(true));
}

fn unpause(mut next_pause_state: ResMut<NextState<Pause>>) {
    next_pause_state.set(Pause(false));
}

#[derive(InputAction)]
#[action_output(bool)]
struct MenuBack;

fn on_menu_back(_: On<Start<MenuBack>>, mut next_menu_state: ResMut<NextState<Menu>>) {
    next_menu_state.set(Menu::None);
}

#[derive(Component)]
struct MenuBackground;

fn menu_background() -> impl Bundle {
    (
        Name::new("Menu Background"),
        MenuBackground,
        DespawnOnEnter(Menu::None),
        actions!(MenuBackground[
            (
                Action::<MenuBack>::new(),
                ActionSettings {
                    require_reset: true,
                    ..Default::default()
                },
                Bindings::spawn((
                    Spawn(Binding::from(GamepadButton::East)),
                )),
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

#[derive(InputAction)]
#[action_output(bool)]
struct TogglePause;

fn on_toggle_pause(
    _: On<Start<TogglePause>>,
    mut next_menu_state: ResMut<NextState<Menu>>,
    menu: Res<State<Menu>>,
) {
    info!("Toggling pause menu. Current menu state: {:?}", menu.get());
    match menu.get() {
        Menu::None => next_menu_state.set(Menu::Pause),
        _ => next_menu_state.set(Menu::None),
    }
}
#[derive(Component)]
pub(crate) struct PauseControl;

pub(crate) fn pause_actions() -> impl Bundle {
    actions!(
        PauseControl[(
            Action::<TogglePause>::new(),
            // We set `require_reset` to `true` because `ResumeGame` action uses the same input,
            // and we want it to be triggerable only after the button is released.
            ActionSettings {
                require_reset: true,
                ..Default::default()
            },
            // Can't allow escape on wasm because that un-grabs cursor
            Bindings::spawn((
                Spawn(Binding::from(KeyCode::KeyP)),
                Spawn(Binding::from(GamepadButton::Start)),
                #[cfg(not(target_family = "wasm"))]
                Spawn(Binding::from(KeyCode::Escape)),
            )),
        )]
    )
}

fn release_cursor(mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    cursor.visible = true;
    cursor.grab_mode = CursorGrabMode::None;
}
