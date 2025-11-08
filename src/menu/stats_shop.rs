use crate::{
    prelude::{DisplayableStatType, GameProgress, MainSystems, Menu, PlayerStats},
    ui_primitives::{
        constants::DARK_GRAY_ALPHA_COLOR,
        primitives::{menu_header, text},
    },
};
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app
        // Pause Related Systems
        .add_observer(handle_player_stat_change)
        .add_observer(handle_stats_shop_ui_update)
        .add_systems(OnEnter(Menu::StatsShop), spawn_stats_shop_menu)
        .add_systems(
            Update,
            handle_stat_button_interaction
                .run_if(in_state(Menu::StatsShop))
                .in_set(MainSystems::Menu),
        );
}

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
            width: percent(100.0),
            height: px(50.0),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            padding: px(10.0).horizontal(),
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
            width: px(30.0),
            height: px(30.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor::from(Color::srgba(0.2, 0.2, 0.2, 0.5)),
        children![text(if is_increase { "+" } else { "-" }, 24.0)],
    )
}

pub fn handle_stat_button_interaction(
    mut interaction_query: Query<(&Interaction, &StatShopButton, &mut BackgroundColor)>,
    mut commands: Commands,
    game_progress: Res<GameProgress>,
    player_stats: Query<&PlayerStats>,
) {
    let stats = player_stats.single().expect("Player stats not found");

    for (interaction, button, mut background_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                let can_increase = button.is_increase && game_progress.progress_points > 0;
                let can_decrease = !button.is_increase && button.stat_type.get_value(stats) > 1;

                if can_increase || can_decrease {
                    commands.trigger(StatChangeEvent {
                        stat_type: button.stat_type,
                        is_increase: button.is_increase,
                    });
                }
            }
            Interaction::Hovered => {
                if button.is_increase && game_progress.progress_points > 0 {
                    *background_color = BackgroundColor(Color::srgb(1.0, 1.0, 0.0));
                } else {
                    *background_color = BackgroundColor(Color::srgb(0.5, 0.5, 0.5));
                }
            }
            Interaction::None => {
                *background_color = BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.5));
            }
        }
    }
}

pub fn handle_player_stat_change(
    trigger: On<StatChangeEvent>,
    mut player_stats: Query<&mut PlayerStats>,
    mut game_progress: ResMut<GameProgress>,
    mut commands: Commands,
) {
    let mut stats = player_stats.single_mut().expect("Player stats not found");

    match (trigger.stat_type, trigger.is_increase) {
        (stat_type, true) if game_progress.progress_points > 0 => {
            match stat_type {
                DisplayableStatType::Agility => stats.agility += 1,
                DisplayableStatType::Strength => stats.strength += 1,
                DisplayableStatType::Dexterity => stats.dexterity += 1,
                DisplayableStatType::Intellect => stats.intellect += 1,
                DisplayableStatType::Luck => stats.luck += 1,
            }
            game_progress.progress_points -= 1;
            commands.trigger(StatsUIUpdateEvent);
        }
        (stat_type, false) => {
            let current_value = stat_type.get_value(&stats);
            if current_value > 1 {
                match stat_type {
                    DisplayableStatType::Agility => stats.agility -= 1,
                    DisplayableStatType::Strength => stats.strength -= 1,
                    DisplayableStatType::Dexterity => stats.dexterity -= 1,
                    DisplayableStatType::Intellect => stats.intellect -= 1,
                    DisplayableStatType::Luck => stats.luck -= 1,
                }
                game_progress.progress_points += 1;
                commands.trigger(StatsUIUpdateEvent);
            }
        }
        _ => {}
    }
}

pub fn handle_stats_shop_ui_update(
    _: On<StatsUIUpdateEvent>,
    mut commands: Commands,
    stats_menu_query: Query<Entity, With<StatShopMenu>>,
    player_stats_query: Single<&PlayerStats>,
    mut game_progress: ResMut<GameProgress>,
) {
    //Set Game Progress to current player stats
    let player_stats = player_stats_query.clone();
    game_progress.base_stats = player_stats.clone();

    // Despawn existing menu
    for entity in stats_menu_query.iter() {
        commands.entity(entity).despawn();
    }

    // Respawn with updated values
    spawn_stats_shop_menu(commands, player_stats_query, game_progress);
}
