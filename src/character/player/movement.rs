use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_player_stopped)
        .add_observer(on_player_movement);
}

#[derive(InputAction)]
#[action_output(Vec2)]
pub(super) struct PlayerMovement;

fn on_player_movement(
    movement: On<Fire<PlayerMovement>>,
    mut player_kinematics: Single<&mut Kinematics, With<Player>>,
) {
    player_kinematics.move_in_direction(movement.value, 1.0);
}

fn on_player_stopped(
    _: On<Complete<PlayerMovement>>,
    mut player_kinematics: Single<&mut Kinematics, (With<Player>, Without<Enemy>, Without<Npc>)>,
) {
    player_kinematics.stop_moving();
}
