use bevy::prelude::*;

use crate::labels::sets::InGameSet;

use super::animate_sprite;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            ((
                animate_sprite, //Change animation if animation component changes
            ))
                .in_set(InGameSet::Simulation),
        );
    }
}
