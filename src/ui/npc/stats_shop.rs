use crate::{
    player::{DisplayableStatType, PlayerStats},
    progression::GameProgress,
    ui::{
        constants::{BACKGROUND_COLOR, DARK_GRAY_ALPHA_COLOR},
        menu_helpers::spawn_header,
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

    commands
        .spawn((
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
        ))
        .with_children(|parent| {
            // Title
            spawn_header(parent, "STATS SHOP");

            // Stats container
            parent
                .spawn((
                    Node {
                        width: Val::Px(600.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(20.0)),
                        row_gap: Val::Px(10.0),
                        ..default()
                    },
                    BackgroundColor::from(DARK_GRAY_ALPHA_COLOR),
                ))
                .with_children(|container| {
                    // Spawn each stat row
                    for stat_type in [
                        DisplayableStatType::Agility,
                        DisplayableStatType::Strength,
                        DisplayableStatType::Dexterity,
                        DisplayableStatType::Intellect,
                        DisplayableStatType::Luck,
                    ] {
                        spawn_stat_row(container, stat_type, stats);
                    }
                });

            // Progress Points Display
            parent.spawn((
                Text::new(format!(
                    "Available Progress Points: {}",
                    game_progress.progress_points
                )),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
            ));
        });
}

fn spawn_stat_row(parent: &mut ChildBuilder, stat_type: DisplayableStatType, stats: &PlayerStats) {
    parent
        .spawn((Node {
            width: Val::Percent(100.0),
            height: Val::Px(50.0),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            padding: UiRect::horizontal(Val::Px(10.0)),
            ..default()
        },))
        .with_children(|row| {
            // Decrease button
            spawn_stat_shop_button(row, stat_type, false);

            // Stat info
            row.spawn((Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },))
                .with_children(|info| {
                    info.spawn((
                        Text::new(format!("{:?}: {}", stat_type, stat_type.get_value(stats))),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                    ));
                    info.spawn((
                        Text::new(stat_type.get_description()),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor::from(Color::srgb(0.5, 0.5, 0.5)),
                    ));
                });

            // Increase button
            spawn_stat_shop_button(row, stat_type, true);
        });
}

fn spawn_stat_shop_button(
    parent: &mut ChildBuilder,
    stat_type: DisplayableStatType,
    is_increase: bool,
) {
    parent
        .spawn((
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
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(if is_increase { "+" } else { "-" }),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
            ));
        });
}
