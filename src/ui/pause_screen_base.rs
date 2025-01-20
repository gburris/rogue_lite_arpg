use bevy::prelude::*;

use crate::{combat::damage::components::Health, player::{Player, PlayerLevel}};

#[derive(Component)]
pub struct PauseScreen;

pub fn spawn_pause_screen(
    mut commands: Commands,
    player_level: Query<&PlayerLevel>,
    player_health: Query<&Health, With<Player>>,
) {
    debug!("spawn_pause_screen called");

    let level = player_level.single();
    let health = player_health.single();

    commands
        .spawn((
            PauseScreen,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor::from(Color::BLACK.with_alpha(0.9)),
            Visibility::Visible,
            GlobalZIndex(1),
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

pub fn despawn_pause_screen(
    mut commands: Commands,
    pause_menu_background_query: Query<Entity, With<PauseScreen>>,
) {
    debug!("despawn_pause_screen called");
    for entity in pause_menu_background_query.iter() {
        commands.entity(entity).despawn();
    }
}
