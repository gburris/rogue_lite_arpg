use crate::{
    combat::Health,
    items::inventory::Inventory,
    labels::states::PausedState,
    player::{Player, PlayerLevel},
    progression::GameProgress,
    ui::{
        constants::{BACKGROUND_COLOR, DARK_GRAY_COLOR, FOOTER_HEIGHT},
        menu_helpers::spawn_header,
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
    player: Single<(&Health, &PlayerLevel, &Inventory), With<Player>>,
    game_progress: Res<GameProgress>,
) {
    let (health, level, inventory) = player.into_inner();

    commands
        .spawn((
            MainMenu,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                row_gap: Val::Px(20.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor::from(BACKGROUND_COLOR),
        ))
        .with_children(|parent| {
            // Header Section
            spawn_header(parent, "PAUSED");

            // Body Section (transparent)
            parent
                .spawn((Node {
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(20.0),
                    ..default()
                },))
                .with_children(|body| {
                    // Spawn all menu buttons
                    let buttons = [MenuButtonConfig::Inventory, MenuButtonConfig::Stats];

                    for button_config in buttons {
                        spawn_menu_button(body, button_config);
                    }
                });

            // Footer Section
            parent
                .spawn((
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
                ))
                .with_children(|footer| {
                    // Player Stats
                    footer
                        .spawn((Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(20.0),
                            ..default()
                        },))
                        .with_children(|stats| {
                            stats.spawn((
                                Text::new(format!("Level: {}", level.current)),
                                TextFont {
                                    font_size: 24.0,
                                    ..default()
                                },
                            ));
                            stats.spawn((
                                Text::new(format!(
                                    "Stat Points: {}",
                                    game_progress.progress_points,
                                )),
                                TextFont {
                                    font_size: 24.0,
                                    ..default()
                                },
                            ));
                            stats.spawn((
                                Text::new(format!("Deaths: {}", game_progress.death_counter,)),
                                TextFont {
                                    font_size: 24.0,
                                    ..default()
                                },
                            ));
                            stats.spawn((
                                Text::new(format!(
                                    "Health: {:.1} / {:.1}",
                                    health.hp, health.max_hp
                                )),
                                TextFont {
                                    font_size: 24.0,
                                    ..default()
                                },
                            ));

                            stats.spawn((
                                Text::new(format!("Total coins: {:.1}", inventory.coins)),
                                TextFont {
                                    font_size: 24.0,
                                    ..default()
                                },
                            ));
                        });

                    // Exit Instructions
                    footer.spawn((
                        Text::new("Press ESC to unpause"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                    ));
                });
        });
}

fn spawn_menu_button(parent: &mut ChildBuilder, config: MenuButtonConfig) {
    let (button_component, button_text) = config.to_component();

    parent
        .spawn((
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
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(button_text),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
            ));
        });
}
