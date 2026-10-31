use bevy::prelude::*;

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app
        // Pause Related Systems
        .add_observer(on_change_stat)
        .add_observer(on_update_stats_ui)
        .add_systems(OnEnter(Menu::StatsShop), spawn_stats_shop_menu)
        .add_systems(
            Update,
            update_stat_buttons
                .run_if(in_state(Menu::StatsShop))
                .in_set(MainSystems::Menu),
        );
}

#[derive(Component)]
pub struct StatShopMenu;

/// Which way a stat shop button moves its stat.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatChange {
    Increase,
    Decrease,
}

#[derive(Component)]
pub struct StatShopButton {
    pub stat: Stat,
    pub change: StatChange,
}

#[derive(Event)]
pub struct ChangeStat {
    pub stat: Stat,
    pub change: StatChange,
}

#[derive(Event)]
pub struct UpdateStatsUi;

pub fn spawn_stats_shop_menu(
    mut commands: Commands,
    player_stats: Single<&PlayerStats>,
    game_progress: ResMut<GameProgress>,
) {
    let stats = player_stats.into_inner();

    commands.spawn((
        StatShopMenu,
        DespawnOnExit(Menu::StatsShop),
        GlobalZIndex(2),
        Node {
            width: percent(100.0),
            height: percent(100.0),
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            row_gap: px(20.0),
            ..default()
        },
        children![
            menu_header("STATS SHOP"),
            // stats shop body
            (
                Node {
                    width: px(600.0),
                    flex_direction: FlexDirection::Column,
                    padding: px(20.0).all(),
                    row_gap: px(10.0),
                    ..default()
                },
                BackgroundColor::from(DARK_GRAY_ALPHA_COLOR),
                children![
                    stat_row(Stat::Agility, stats),
                    stat_row(Stat::Strength, stats),
                    stat_row(Stat::Dexterity, stats),
                    stat_row(Stat::Intellect, stats),
                    stat_row(Stat::Luck, stats),
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

fn stat_row(stat: Stat, stats: &PlayerStats) -> impl Bundle {
    (
        Node {
            width: percent(100.0),
            height: px(50.0),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            padding: px(10.0).horizontal(),
            ..default()
        },
        children![
            // Decrease button
            stat_shop_button(stat, StatChange::Decrease),
            (
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
                children![
                    text(format!("{stat:?}: {}", stat.value(stats)), 24.0),
                    (
                        text(stat.description(), 16.0),
                        TextColor::from(Color::srgb(0.5, 0.5, 0.5)),
                    )
                ]
            ),
            // Increase button
            stat_shop_button(stat, StatChange::Increase)
        ],
    )
}

fn stat_shop_button(stat: Stat, change: StatChange) -> impl Bundle {
    (
        StatShopButton { stat, change },
        Button,
        Node {
            width: px(30.0),
            height: px(30.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor::from(Color::srgba(0.2, 0.2, 0.2, 0.5)),
        children![text(
            match change {
                StatChange::Increase => "+",
                StatChange::Decrease => "-",
            },
            24.0
        )],
    )
}

pub fn update_stat_buttons(
    mut interaction_query: Query<(&Interaction, &StatShopButton, &mut BackgroundColor)>,
    mut commands: Commands,
    game_progress: Res<GameProgress>,
    player_stats: Query<&PlayerStats>,
) {
    let stats = player_stats.single().expect("Player stats not found");

    for (interaction, button, mut background_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                let can_increase =
                    button.change == StatChange::Increase && game_progress.progress_points > 0;
                let can_decrease =
                    button.change == StatChange::Decrease && button.stat.value(stats) > 1;

                if can_increase || can_decrease {
                    commands.trigger(ChangeStat {
                        stat: button.stat,
                        change: button.change,
                    });
                }
            }
            Interaction::Hovered => {
                if button.change == StatChange::Increase && game_progress.progress_points > 0 {
                    *background_color = BackgroundColor::from(Color::srgb(1.0, 1.0, 0.0));
                } else {
                    *background_color = BackgroundColor::from(Color::srgb(0.5, 0.5, 0.5));
                }
            }
            Interaction::None => {
                *background_color = BackgroundColor::from(Color::srgba(0.2, 0.2, 0.2, 0.5));
            }
        }
    }
}

pub fn on_change_stat(
    trigger: On<ChangeStat>,
    mut player_stats: Query<&mut PlayerStats>,
    mut game_progress: ResMut<GameProgress>,
    mut commands: Commands,
) {
    let mut stats = player_stats.single_mut().expect("Player stats not found");

    match (trigger.stat, trigger.change) {
        (stat, StatChange::Increase) if game_progress.progress_points > 0 => {
            match stat {
                Stat::Agility => stats.agility += 1,
                Stat::Strength => stats.strength += 1,
                Stat::Dexterity => stats.dexterity += 1,
                Stat::Intellect => stats.intellect += 1,
                Stat::Luck => stats.luck += 1,
            }
            game_progress.progress_points -= 1;
            commands.trigger(UpdateStatsUi);
        }
        (stat, StatChange::Decrease) => {
            let current_value = stat.value(&stats);
            if current_value > 1 {
                match stat {
                    Stat::Agility => stats.agility -= 1,
                    Stat::Strength => stats.strength -= 1,
                    Stat::Dexterity => stats.dexterity -= 1,
                    Stat::Intellect => stats.intellect -= 1,
                    Stat::Luck => stats.luck -= 1,
                }
                game_progress.progress_points += 1;
                commands.trigger(UpdateStatsUi);
            }
        }
        (_, StatChange::Increase) => {}
    }
}

pub fn on_update_stats_ui(
    _: On<UpdateStatsUi>,
    mut commands: Commands,
    stats_menu_query: Query<Entity, With<StatShopMenu>>,
    player_stats_query: Single<&PlayerStats>,
    mut game_progress: ResMut<GameProgress>,
) {
    // Set Game Progress to current player stats
    let player_stats = **player_stats_query;
    game_progress.base_stats = player_stats;

    // Despawn existing menu
    for entity in &stats_menu_query {
        commands.entity(entity).despawn();
    }

    // Respawn with updated values
    spawn_stats_shop_menu(commands, player_stats_query, game_progress);
}
