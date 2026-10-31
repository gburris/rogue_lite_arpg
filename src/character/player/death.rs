use bevy::prelude::*;

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        finish_death_animation
            .in_set(VfxSystems::Animate)
            .run_if(in_state(PlayingState::Death)),
    );
}

#[derive(Component, Deref, DerefMut)]
struct GameOverTimer(Timer);

pub(super) fn on_player_defeated(
    _: On<Defeated>,
    player: Single<Entity, With<Player>>,
    mut commands: Commands,
    mut next_playing_state: ResMut<NextState<PlayingState>>,
) {
    commands
        .entity(*player)
        .insert(GameOverTimer(Timer::from_seconds(2.0, TimerMode::Once)))
        .remove::<Health>()
        .despawn_related::<Children>();
    next_playing_state.set(PlayingState::Death);
}

fn finish_death_animation(
    time: Res<Time>,
    player_death_timer_single: Single<&mut GameOverTimer, With<Player>>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    let mut death_timer = player_death_timer_single.into_inner();
    death_timer.tick(time.delta());
    if death_timer.is_finished() {
        next_app_state.set(AppState::GameOver);
    }
}
