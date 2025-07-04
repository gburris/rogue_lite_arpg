use bevy::prelude::*;

use crate::{
    character::prelude::Player,
    combat::{damage::DefeatedEvent, Health},
    labels::states::{AppState, PlayingState},
    map::CleanupZone,
    prelude::*,
};

#[derive(Component)]
pub struct GameOverTimer(pub Timer);

pub fn on_player_defeated(
    _: Trigger<DefeatedEvent>,
    player: Single<(Entity, &mut SimpleMotion), With<Player>>,
    mut commands: Commands,
    mut playing_state: ResMut<NextState<PlayingState>>,
) {
    let (player_entity, mut player_motion) = player.into_inner();

    commands
        .entity(player_entity)
        .insert((
            ActionState::Defeated,
            GameOverTimer(Timer::from_seconds(2.0, TimerMode::Once)),
        ))
        .remove::<Health>()
        .despawn_related::<Children>();
    player_motion.stop_moving();
    playing_state.set(PlayingState::Death);
}

pub fn finish_death_animation(
    time: Res<Time>,
    player_death_timer_single: Single<&mut GameOverTimer, With<Player>>,
    mut commands: Commands,
    mut game_over_state: ResMut<NextState<AppState>>,
) {
    let mut death_timer = player_death_timer_single.into_inner();
    death_timer.0.tick(time.delta());
    if death_timer.0.finished() {
        commands.trigger(CleanupZone);
        game_over_state.set(AppState::GameOver);
    }
}
