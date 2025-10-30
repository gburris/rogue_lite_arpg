use bevy::prelude::*;

use crate::prelude::*;

use bevy_enhanced_input::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_player_stopped)
        .add_observer(on_player_movement);
}

#[derive(InputAction)]
#[action_output(Vec2)]

pub(super) struct PlayerMovement;

fn on_player_movement(
    movement: On<Fire<PlayerMovement>>,
    player_motion_query: Single<&mut SimpleMotion, With<Player>>,
) {
    let mut motion = player_motion_query.into_inner();
    motion.start_moving(movement.value);
}

fn on_player_stopped(
    _: On<Complete<PlayerMovement>>,
    mut player_motion: Single<&mut SimpleMotion, (With<Player>, Without<Enemy>, Without<NPC>)>,
) {
    player_motion.stop_moving();
}
