use bevy::prelude::*;

use crate::{
    character::player::PlayerStats,
    labels::states::PausedState,
    ui::{
        constants::{BACKGROUND_COLOR, DARK_GRAY_ALPHA_COLOR},
        primitives::{menu_header, text},
    },
};

#[derive(Component)]
pub struct StatsMenu;

#[derive(Component)]
pub struct StatsDisplay;

pub fn spawn_stats_menu(mut commands: Commands, player_stats: Query<&PlayerStats>) {
    if let Ok(stats) = player_stats.single() {
        commands.spawn((
            StatsMenu,
            DespawnOnExit(PausedState::Stats),
            Node {
                width: percent(100.0),
                height: percent(100.0),
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: px(20.0),
                ..default()
            },
            BackgroundColor::from(BACKGROUND_COLOR),
            children![
                menu_header("STATS"),
                (
                    StatsDisplay,
                    Node {
                        width: px(600.0),
                        height: percent(80.0),
                        flex_direction: FlexDirection::Column,
                        padding: px(20.0).all(),
                        ..default()
                    },
                    BackgroundColor::from(DARK_GRAY_ALPHA_COLOR),
                    children![
                        stat_row("Agility", stats.agility, "Movement speed, roll range"),
                        stat_row("Strength", stats.strength, "Melee swing damage"),
                        stat_row("Dexterity", stats.dexterity, "Critical Strike Chance"),
                        stat_row("Intellect", stats.intellect, "Spell damage"),
                        stat_row("Luck", stats.luck, "Drop rate"),
                    ],
                )
            ],
        ));
    }
}

fn stat_row(
    stat_name: impl Into<String>,
    stat_value: u32,
    description: impl Into<String>,
) -> impl Bundle {
    (
        Node {
            width: percent(100.0),
            height: px(60.0),
            padding: px(10.0).all(),
            margin: px(5.0).bottom(),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor::from(Color::srgba(0.2, 0.2, 0.2, 0.5)),
        children![
            // left side
            (
                Node {
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                children![
                    text(stat_name, 24.0),
                    (
                        text(description, 16.0),
                        Node {
                            margin: px(4.0).top(),
                            ..default()
                        },
                    )
                ]
            ),
            // right side
            text(format!("{stat_value}/99"), 24.0)
        ],
    )
}
