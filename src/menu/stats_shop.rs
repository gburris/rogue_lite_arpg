use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_change_stat)
        .add_observer(on_update_stats_ui)
        .add_systems(OnEnter(Menu::StatsShop), spawn_stats_shop_menu);
}

#[derive(Component, Default, Clone)]
pub struct StatShopMenu;

/// Which way a stat shop button moves its stat.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatChange {
    Increase,
    Decrease,
}

#[derive(Component, Clone)]
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
    let points = game_progress.progress_points;
    let rows: Vec<_> = Stat::ALL
        .into_iter()
        .map(|stat| stat_row(stat, stats, points))
        .collect();

    commands.spawn_scene(bsn! {
        @menu_screen(Menu::StatsShop, "STATS SHOP")
        StatShopMenu
        Children [
            @panel(600.0)
            Children [
                {rows}
            ]
            --
            @text({format!("Available Progress Points: {points}")}, 32.0)
        ]
    });
}

fn stat_row(stat: Stat, stats: &PlayerStats, points: u32) -> impl Scene {
    bsn! {
        Node {
            width: percent(100),
            height: px(50),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            padding: {px(10).horizontal()},
        }
        Children [
            @stat_shop_button(stat, StatChange::Decrease, points)
            --
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
            }
            Children [
                @text({format!("{stat:?}: {}", stat.value(stats))}, BODY_FONT_SIZE)
                --
                @muted_text(stat.description(), CAPTION_FONT_SIZE)
            ]
            --
            @stat_shop_button(stat, StatChange::Increase, points)
        ]
    }
}

fn stat_shop_button(stat: Stat, change: StatChange, points: u32) -> impl Scene {
    let (label, highlight) = match change {
        // Highlight increases the player can afford.
        StatChange::Increase if points > 0 => ("+", Color::srgb(1.0, 1.0, 0.0)),
        StatChange::Increase => ("+", Color::srgb(0.5, 0.5, 0.5)),
        StatChange::Decrease => ("-", Color::srgb(0.5, 0.5, 0.5)),
    };
    let colors = ButtonColors {
        idle: ROW_COLOR,
        hovered: highlight,
        pressed: highlight,
    };

    bsn! {
        @button(label, BODY_FONT_SIZE, Color::WHITE, colors)
        ~{StatShopButton { stat, change }}
        Node {
            width: px(30),
            height: px(30),
        }
        on(on_stat_button_activated)
    }
}

fn on_stat_button_activated(
    activate: On<Activate>,
    buttons: Query<&StatShopButton>,
    mut commands: Commands,
    game_progress: Res<GameProgress>,
    player_stats: Single<&PlayerStats>,
) {
    let Ok(button) = buttons.get(activate.entity) else {
        return;
    };

    let can_increase = button.change == StatChange::Increase && game_progress.progress_points > 0;
    let can_decrease =
        button.change == StatChange::Decrease && button.stat.value(*player_stats) > 1;

    if can_increase || can_decrease {
        commands.trigger(ChangeStat {
            stat: button.stat,
            change: button.change,
        });
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
