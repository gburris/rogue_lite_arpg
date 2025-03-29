use bevy::prelude::*;

use crate::{
    ai::{simple_motion, state},
    labels::sets::{InGameSet, MainSet},
};

pub struct AIPlugin;

impl Plugin for AIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            state::update_state_on_simple_motion_change.in_set(InGameSet::Simulation),
        )
        .add_systems(
            FixedUpdate,
            simple_motion::to_velocity.in_set(MainSet::InGame),
        );
    }
}
