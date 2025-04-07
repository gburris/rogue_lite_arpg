use crate::{
    combat::Health,
    items::inventory::Inventory,
    labels::states::PausedState,
    player::Player,
    progression::GameProgress,
    ui::{
        constants::{BACKGROUND_COLOR, DARK_GRAY_COLOR, FOOTER_HEIGHT},
        primitives::{menu_header, text},
    },
};
use bevy::prelude::*;

#[derive(Component)]
pub struct MainMenu;

#[derive(Component)]
pub struct MenuButton(pub PausedState);

#[derive(Clone, Copy)]
enum MenuButtonConfig {
    Inventory,
    Stats,
}

impl MenuButtonConfig {
    fn to_component(self) -> (MenuButton, &'static str) {
        match self {
            MenuButtonConfig::Inventory => (MenuButton(PausedState::Inventory), "INVENTORY"),
            MenuButtonConfig::Stats => (MenuButton(PausedState::Stats), "STATS"),
        }
    }
}

pub fn spawn_main_menu(
    mut commands: Commands,
    player: Single<(&Health, &Player, &Inventory)>,
    game_progress: Res<GameProgress>,
) {
    let (health, player, inventory) = player.into_inner();

    commands.spawn((
        MainMenu,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            row_gap: Val::Px(20.0),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor::from(BACKGROUND_COLOR),
        children![
            menu_header("PAUSED"),
            // Body Section
            (
                Node {
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(20.0),
                    ..default()
                },
                children![
                    menu_button(MenuButtonConfig::Inventory),
                    menu_button(MenuButtonConfig::Stats),
                ]
            ),
            main_menu_footer(player.get_level(), health, inventory.coins, &game_progress),
        ],
    ));
}

fn menu_button(config: MenuButtonConfig) -> impl Bundle {
    let (button_component, button_text) = config.to_component();

    (
        button_component,
        Button,
        Node {
            width: Val::Px(300.0),
            height: Val::Px(60.0),
            border: UiRect::all(Val::Px(2.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BorderColor(Color::srgb(0.8, 0.8, 0.8)),
        BackgroundColor(DARK_GRAY_COLOR),
        children![(
            Text::new(button_text),
            TextFont {
                font_size: 32.0,
                ..default()
            },
        )],
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
            width: Val::Percent(100.0),
            height: FOOTER_HEIGHT,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            padding: UiRect::horizontal(Val::Px(40.0)),
            ..default()
        },
        BackgroundColor::from(DARK_GRAY_COLOR),
        children![
            // left side player info
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(20.0),
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
