use crate::{
    combat::attributes::Health, econ::components::Wallet, labels::states::PausedState, player::{Player, PlayerLevel}, progression::GameProgress
};
use bevy::prelude::*;

#[derive(Component)]
pub struct MainMenu;

#[derive(Component)]
pub struct MenuButton(pub PausedState);

#[derive(Clone, Copy)]
enum MenuButtonConfig {
    Equipment,
    Inventory,
    Stats,
}

impl MenuButtonConfig {
    fn to_component(self) -> (MenuButton, &'static str) {
        match self {
            MenuButtonConfig::Equipment => (MenuButton(PausedState::Equipment), "EQUIPMENT"),
            MenuButtonConfig::Inventory => (MenuButton(PausedState::Inventory), "INVENTORY"),
            MenuButtonConfig::Stats => (MenuButton(PausedState::Stats), "STATS"),
        }
    }
}

pub fn spawn_main_menu(
    mut commands: Commands,
    player_level: Query<&PlayerLevel>,
    player_health: Query<&Health, With<Player>>,
    player_wallet: Query<&Wallet, With<Player>>,
    game_progress: Res<GameProgress>,
) {
    debug!("spawn_main_menu called");
    // Get the current values
    let level = player_level.single();
    let health = player_health.single();
    let wallet = player_wallet.single();
    commands
        .spawn((
            MainMenu,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            GlobalZIndex(1),
            Visibility::Visible,
        ))
        .with_children(|parent| {
            // Header Section
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(120.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor::from(Color::srgb(0.1, 0.1, 0.1)),
                ))
                .with_children(|header| {
                    header.spawn((
                        Text::new("GAME PAUSED"),
                        TextFont {
                            font_size: 80.0,
                            ..default()
                        },
                        Node::default(),
                    ));
                });

            // Body Section (transparent)
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        flex_grow: 1.0,
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(20.0),
                        padding: UiRect::vertical(Val::Px(40.0)),
                        ..default()
                    },
                    BackgroundColor::from(Color::srgba(0.0, 0.0, 0.0, 0.7)),
                ))
                .with_children(|body| {
                    // Spawn all menu buttons
                    let buttons = [
                        MenuButtonConfig::Equipment,
                        MenuButtonConfig::Inventory,
                        MenuButtonConfig::Stats,
                    ];

                    for button_config in buttons {
                        spawn_menu_button(body, button_config);
                    }
                });

            // Footer Section
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(80.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        padding: UiRect::horizontal(Val::Px(40.0)),
                        ..default()
                    },
                    BackgroundColor::from(Color::srgb(0.1, 0.1, 0.1)),
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
                                Node::default(),
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
                                Node::default(),
                            ));
                            stats.spawn((
                                Text::new(format!("Deaths: {}", game_progress.death_counter,)),
                                TextFont {
                                    font_size: 24.0,
                                    ..default()
                                },
                                Node::default(),
                            ));

                            stats.spawn((
                                Text::new(format!("Wallet: {:.1}", wallet.coins)),
                                TextFont {
                                    font_size: 24.0,
                                    ..default()
                                },
                                Node::default(),
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
                                Node::default(),
                            ));
                        });

                    // Exit Instructions
                    footer.spawn((
                        Text::new("Press ESC to unpause"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        Node::default(),
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
            BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(button_text),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                Node::default(),
            ));
        });
}

pub fn despawn_main_menu(
    mut commands: Commands,
    pause_menu_background_query: Query<Entity, With<MainMenu>>,
) {
    debug!("despawn_main_menu called");
    for entity in pause_menu_background_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
