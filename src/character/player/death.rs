use bevy::prelude::*;

use crate::{
    combat::{Health, damage::Defeated},
    prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        finish_death_animation
            .in_set(InGameSystems::Vfx)
            .run_if(in_state(PlayingState::Death)),
    );
}

#[derive(Component)]
struct GameOverTimer(Timer);

pub(super) fn on_player_defeated(
    _: On<Defeated>,
    player: Single<(Entity, &mut SimpleMotion), With<Player>>,
    mut commands: Commands,
    mut playing_state: ResMut<NextState<PlayingState>>,
) {
    let (player_entity, mut player_motion) = player.into_inner();

    commands
        .entity(player_entity)
        .insert(GameOverTimer(Timer::from_seconds(2.0, TimerMode::Once)))
        .remove::<Health>()
        .despawn_related::<Children>();
    player_motion.stop_moving();
    playing_state.set(PlayingState::Death);
}

fn finish_death_animation(
    time: Res<Time>,
    player_death_timer_single: Single<&mut GameOverTimer, With<Player>>,
    mut game_over_state: ResMut<NextState<AppState>>,
) {
    let mut death_timer = player_death_timer_single.into_inner();
    death_timer.0.tick(time.delta());
    if death_timer.0.is_finished() {
        game_over_state.set(AppState::GameOver);
    }
}
