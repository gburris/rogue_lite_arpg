use crate::{
    player::{DisplayableStatType, PlayerStats},
    progression::GameProgress,
    ui::{
        constants::{BACKGROUND_COLOR, DARK_GRAY_ALPHA_COLOR},
        primitives::{menu_header, text},
    },
};
use bevy::prelude::*;

#[derive(Component)]
pub struct StatShopMenu;

#[derive(Component)]
pub struct StatShopButton {
    pub stat_type: DisplayableStatType,
    pub is_increase: bool,
}

#[derive(Event)]
pub struct StatChangeEvent {
    pub stat_type: DisplayableStatType,
    pub is_increase: bool,
}

#[derive(Event)]
pub struct StatsUIUpdateEvent;

pub fn spawn_stats_shop_menu(
    mut commands: Commands,
    player_stats: Single<&PlayerStats>,
    game_progress: ResMut<GameProgress>,
) {
    let stats = player_stats.into_inner();

    commands.spawn((
        StatShopMenu,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(20.0),
            ..default()
        },
        BackgroundColor::from(BACKGROUND_COLOR),
        GlobalZIndex(1),
        children![
            menu_header("STATS SHOP"),
            // stats shop body
            (
                Node {
                    width: Val::Px(600.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(20.0)),
                    row_gap: Val::Px(10.0),
                    ..default()
                },
                BackgroundColor::from(DARK_GRAY_ALPHA_COLOR),
                children![
                    stat_row(DisplayableStatType::Agility, stats),
                    stat_row(DisplayableStatType::Strength, stats),
                    stat_row(DisplayableStatType::Dexterity, stats),
                    stat_row(DisplayableStatType::Intellect, stats),
                    stat_row(DisplayableStatType::Luck, stats),
                ]
            ),
            // Progress Points Display
            text(
                format!(
                    "Available Progress Points: {}",
                    game_progress.progress_points
                ),
                32.0
            )
        ],
    ));
}

fn stat_row(stat_type: DisplayableStatType, stats: &PlayerStats) -> impl Bundle {
    (
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(50.0),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            padding: UiRect::horizontal(Val::Px(10.0)),
            ..default()
        },
        children![
            // Decrease button
            stat_shop_button(stat_type, false),
            (
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
                children![
                    text(
                        format!("{:?}: {}", stat_type, stat_type.get_value(stats)),
                        24.0
                    ),
                    (
                        text(stat_type.get_description(), 16.0),
                        TextColor::from(Color::srgb(0.5, 0.5, 0.5)),
                    )
                ]
            ),
            // Increase button
            stat_shop_button(stat_type, true)
        ],
    )
}

fn stat_shop_button(stat_type: DisplayableStatType, is_increase: bool) -> impl Bundle {
    (
        StatShopButton {
            stat_type,
            is_increase,
        },
        Button,
        Node {
            width: Val::Px(30.0),
            height: Val::Px(30.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor::from(Color::srgba(0.2, 0.2, 0.2, 0.5)),
        children![text(if is_increase { "+" } else { "-" }, 24.0)],
    )
}
