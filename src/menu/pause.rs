use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use bevy_enhanced_input::prelude::*;

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Pause), spawn_main_menu)
        .add_systems(
            Update,
            handle_menu_button_pressed
                .run_if(in_state(Menu::Pause))
                .in_set(MainSystems::Menu),
        );
}

#[derive(Component)]
struct MainMenu;

#[derive(Component)]
struct MenuButton(pub Menu);

fn handle_menu_button_pressed(
    mut button_query: Query<(&Interaction, &MenuButton)>,
    mut next_menu_state: ResMut<NextState<Menu>>,
) {
    for (interaction, menu_button) in &mut button_query {
        if *interaction == Interaction::Pressed {
            next_menu_state.set(menu_button.0);
        }
    }
}

fn spawn_main_menu(
    mut commands: Commands,
    player: Single<(&Health, &Player, &Purse)>,
    game_progress: Res<GameProgress>,
) {
    let (health, player, purse) = player.into_inner();

    commands.spawn((
        MainMenu,
        DespawnOnExit(Menu::Pause),
        GlobalZIndex(2),
        Node {
            width: percent(100.0),
            height: percent(100.0),
            row_gap: px(20.0),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        children![
            menu_header("PAUSED"),
            // Body Section
            (
                Node {
                    width: percent(100.0),
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    row_gap: px(20.0),
                    ..default()
                },
                children![
                    menu_button(MenuButton(Menu::Inventory), "INVENTORY"),
                    menu_button(MenuButton(Menu::Stats), "STATS"),
                ]
            ),
            main_menu_footer(player.level(), health, purse.amount, &game_progress),
        ],
    ));
}

fn menu_button(marker: impl Bundle, button_text: &str) -> impl Bundle {
    (
        marker,
        Button,
        Node {
            width: px(300.0),
            height: px(60.0),
            border: px(2.0).all(),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BorderColor::all(Color::srgb(0.8, 0.8, 0.8)),
        BackgroundColor(DARK_GRAY_COLOR),
        children![text(button_text, 32.0)],
    )
}

fn main_menu_footer(
    player_level: u32,
    player_health: &Health,
    player_coins: u32,
    game_progress: &GameProgress,
) -> impl Bundle {
    // Footer Section
    (
        Node {
            width: percent(100.0),
            height: FOOTER_HEIGHT,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            padding: px(40.0).horizontal(),
            ..default()
        },
        BackgroundColor::from(DARK_GRAY_COLOR),
        children![
            // left side player info
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: px(20.0),
                    ..default()
                },
                children![
                    text(format!("Level: {player_level}"), 24.0),
                    text(
                        format!("Stat Points: {}", game_progress.progress_points),
                        24.0
                    ),
                    text(format!("Deaths: {}", game_progress.death_counter), 24.0),
                    text(
                        format!(
                            "Health: {:.1} / {:.1}",
                            player_health.hp, player_health.max_hp
                        ),
                        24.0
                    ),
                    text(format!("Total coins: {player_coins}"), 24.0)
                ]
            ),
            // right side exit instructions
            text("Press ESC to unpause", 24.0)
        ],
    )
}

pub(super) fn pause(mut next_pause_state: ResMut<NextState<Pause>>, mut commands: Commands) {
    commands.spawn(menu_background());
    next_pause_state.set(Pause(true));
}

pub(super) fn unpause(mut next_pause_state: ResMut<NextState<Pause>>) {
    next_pause_state.set(Pause(false));
}

#[derive(InputAction)]
#[action_output(bool)]
pub(super) struct MenuBack;

pub(super) fn on_menu_back(_: On<Start<MenuBack>>, mut next_menu_state: ResMut<NextState<Menu>>) {
    next_menu_state.set(Menu::None);
}

#[derive(Component)]
pub(super) struct MenuBackground;

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

pub(super) fn release_cursor(mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    cursor.visible = true;
    cursor.grab_mode = CursorGrabMode::None;
}
