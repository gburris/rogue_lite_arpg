use super::stats_shop::{
    spawn_stats_shop_menu, StatChangeEvent, StatShopButton, StatShopMenu, StatsUIUpdateEvent,
};
use crate::{
    player::{DisplayableStatType, PlayerStats},
    progression::GameProgress,
};
use bevy::prelude::*;

pub fn handle_stat_button_interaction(
    mut interaction_query: Query<(&Interaction, &StatShopButton, &mut BackgroundColor)>,
    mut commands: Commands,
    game_progress: Res<GameProgress>,
    player_stats: Query<&PlayerStats>,
) {
    let stats = player_stats.single();

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
    trigger: Trigger<StatChangeEvent>,
    mut player_stats: Query<&mut PlayerStats>,
    mut game_progress: ResMut<GameProgress>,
    mut commands: Commands,
) {
    let mut stats = player_stats.single_mut();

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
    _: Trigger<StatsUIUpdateEvent>,
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
        commands.entity(entity).despawn_recursive();
    }

    // Respawn with updated values
    spawn_stats_shop_menu(commands, player_stats_query, game_progress);
}
