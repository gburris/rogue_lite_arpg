use bevy::prelude::*;

use crate::{labels::sets::InGameSet, movement::motion::*};

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_facing_direction_and_action_state_on_motion_change, //Handle idle to walking states here
                simple_movement_to_velocity,
            )
                .in_set(InGameSet::Physics),
        );
    }
}
