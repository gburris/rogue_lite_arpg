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
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor::from(BACKGROUND_COLOR),
            children![
                menu_header("STATS"),
                (
                    StatsDisplay,
                    Node {
                        width: Val::Px(600.0),
                        height: Val::Percent(80.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(20.0)),
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
            width: Val::Percent(100.0),
            height: Val::Px(60.0),
            padding: UiRect::all(Val::Px(10.0)),
            margin: UiRect::bottom(Val::Px(5.0)),
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
                            margin: UiRect::top(Val::Px(4.0)),
                            ..default()
                        },
                    )
                ]
            ),
            // right side
            text(format!("{}/99", stat_value), 24.0)
        ],
    )
}
