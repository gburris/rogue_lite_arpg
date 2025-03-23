use bevy::prelude::*;

use crate::{
    player::PlayerStats,
    ui::{
        constants::{BACKGROUND_COLOR, DARK_GRAY_ALPHA_COLOR},
        menu_helpers::spawn_header,
    },
};

#[derive(Component)]
pub struct StatsMenu;

#[derive(Component)]
pub struct StatsDisplay;

pub fn spawn_stats_menu(mut commands: Commands, player_stats: Query<&PlayerStats>) {
    debug!("spawn_stats_menu called");

    if let Ok(stats) = player_stats.get_single() {
        commands
            .spawn((
                StatsMenu,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(20.0),
                    ..default()
                },
                BackgroundColor::from(BACKGROUND_COLOR),
            ))
            .with_children(|ChildOf| {
                spawn_header(ChildOf, "STATS");

                // Stats container
                ChildOf
                    .spawn((
                        StatsDisplay,
                        Node {
                            width: Val::Px(600.0),
                            height: Val::Percent(80.0),
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::all(Val::Px(20.0)),
                            ..default()
                        },
                        BackgroundColor::from(DARK_GRAY_ALPHA_COLOR),
                    ))
                    .with_children(|stats_ChildOf| {
                        spawn_stat_row(
                            stats_ChildOf,
                            "Agility",
                            stats.agility,
                            "Movement speed, roll range",
                        );
                        spawn_stat_row(
                            stats_ChildOf,
                            "Strength",
                            stats.strength,
                            "Melee swing damage",
                        );
                        spawn_stat_row(
                            stats_ChildOf,
                            "Dexterity",
                            stats.dexterity,
                            "Critical Strike Chance",
                        );
                        spawn_stat_row(stats_ChildOf, "Intellect", stats.intellect, "Spell damage");
                        spawn_stat_row(stats_ChildOf, "Luck", stats.luck, "Drop rate");
                    });
            });
    }
}

fn spawn_stat_row(builder: &mut ChildBuilder, stat_name: &str, stat_value: u32, description: &str) {
    builder
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(60.0),
                padding: UiRect::all(Val::Px(10.0)),
                margin: UiRect::bottom(Val::Px(5.0)),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor::from(Color::srgba(0.2, 0.2, 0.2, 0.5)),
        ))
        .with_children(|ChildOf| {
            // Left side: Stat name and description
            ChildOf
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    ..default()
                })
                .with_children(|left_side| {
                    // Stat name
                    left_side.spawn((
                        Text::new(stat_name),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                    ));

                    // Description
                    left_side.spawn((
                        Text::new(description),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        Node {
                            margin: UiRect::top(Val::Px(4.0)),
                            ..default()
                        },
                    ));
                });

            // Right side: Stat value
            ChildOf.spawn((
                Text::new(format!("{}/99", stat_value)),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
            ));
        });
}
