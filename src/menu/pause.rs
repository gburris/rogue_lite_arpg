use bevy::prelude::*;
use bevy::ui_widgets::Activate;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use bevy_enhanced_input::prelude::*;

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Pause), spawn_main_menu);
}

#[derive(Component, Default, Clone)]
struct MainMenu;

fn spawn_main_menu(
    mut commands: Commands,
    player: Single<(&Health, &Player, &Purse)>,
    game_progress: Res<GameProgress>,
) {
    let (health, player, purse) = player.into_inner();
    let footer = main_menu_footer(player.level(), health, purse.amount, &game_progress);

    commands.spawn_scene(bsn! {
        @menu_screen(Menu::Pause, "PAUSED")
        MainMenu
        Children [
            Node {
                width: percent(100),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: px(20),
            }
            Children [
                @menu_button("INVENTORY")
                on(open_menu(Menu::Inventory))
                --
                @menu_button("STATS")
                on(open_menu(Menu::Stats))
            ]
            --
            @{footer}
        ]
    });
}

/// An observer that switches to `menu` when its button is activated.
fn open_menu(menu: Menu) -> impl Fn(On<Activate>, ResMut<NextState<Menu>>) + Clone {
    move |_, mut next_menu_state| next_menu_state.set(menu)
}

fn main_menu_footer(
    player_level: u32,
    player_health: &Health,
    player_coins: u32,
    game_progress: &GameProgress,
) -> impl Scene {
    let stats = [
        format!("Level: {player_level}"),
        format!("Stat Points: {}", game_progress.progress_points),
        format!("Deaths: {}", game_progress.death_counter),
        format!(
            "Health: {:.1} / {:.1}",
            player_health.hp, player_health.max_hp
        ),
        format!("Total coins: {player_coins}"),
    ]
    .map(|stat| text(stat, BODY_FONT_SIZE));

    bsn! {
        @menu_footer()
        Children [
            // left side player info
            Node {
                flex_direction: FlexDirection::Row,
                column_gap: px(20),
            }
            Children [
                {Vec::from(stats)}
            ]
            --
            @unpause_hint()
        ]
    }
}

pub(super) fn pause(mut next_pause_state: ResMut<NextState<PauseState>>, mut commands: Commands) {
    commands.spawn_scene(menu_background());
    next_pause_state.set(PauseState::Paused);
}

pub(super) fn unpause(mut next_pause_state: ResMut<NextState<PauseState>>) {
    next_pause_state.set(PauseState::Running);
}

#[derive(InputAction)]
#[action_output(bool)]
pub(super) struct MenuBack;

pub(super) fn on_menu_back(_: On<Start<MenuBack>>, mut next_menu_state: ResMut<NextState<Menu>>) {
    next_menu_state.set(Menu::None);
}

#[derive(Component, Default, Clone)]
pub(super) struct MenuBackground;

/// Dims the game behind every menu and lets a gamepad back out of it.
fn menu_background() -> impl Scene {
    bsn! {
        #MenuBackground
        MenuBackground
        ~{DespawnOnEnter(Menu::None)}
        Node {
            width: percent(100),
            height: percent(100),
        }
        GlobalZIndex(1)
        BackgroundColor(BACKGROUND_COLOR)
        Actions<MenuBackground> [
            Action::<MenuBack>::new()
            ActionSettings { require_reset: true }
            Bindings [
                Binding(GamepadButton::East)
            ]
        ]
    }
}

#[derive(InputAction)]
#[action_output(bool)]
pub(super) struct TogglePause;

pub(super) fn on_toggle_pause(
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

#[derive(Component, Default, Clone)]
pub(crate) struct PauseControl;

/// The pause toggle, added to the player.
pub(crate) fn pause_actions() -> impl Scene {
    // Escape un-grabs the cursor on the web, so it can't pause there.
    let escape = (!cfg!(target_family = "wasm")).then(|| {
        bsn_list! {
            Binding(KeyCode::Escape)
        }
    });

    bsn! {
        PauseControl
        Actions<PauseControl> [
            Action::<TogglePause>::new()
            // `MenuBack` shares this input, so wait for a release before toggling again.
            ActionSettings { require_reset: true }
            Bindings [
                Binding(KeyCode::KeyP)
                --
                Binding(GamepadButton::Start)
                --
                {escape}
            ]
        ]
    }
}

pub(super) fn release_cursor(mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    cursor.visible = true;
    cursor.grab_mode = CursorGrabMode::None;
}
