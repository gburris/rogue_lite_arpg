pub use bevy::prelude::*;

mod simple_motion;
pub mod state;

pub use simple_motion::SimpleMotion;

use crate::labels::sets::{InGameSet, MainSet};

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        state::update_state_on_simple_motion_change.in_set(InGameSet::Simulation),
    )
    .add_systems(
        FixedUpdate,
        simple_motion::to_velocity.in_set(MainSet::InGame),
    );
}
