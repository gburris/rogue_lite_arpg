use bevy::prelude::*;

use crate::{
    character::Purse,
    prelude::*,
    ui::{
        constants::{DARK_GRAY_COLOR, FOOTER_HEIGHT},
        primitives::{menu_header, text},
    },
};

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
            main_menu_footer(player.get_level(), health, purse.amount, &game_progress),
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
