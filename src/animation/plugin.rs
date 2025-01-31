use bevy::prelude::*;

use crate::labels::sets::InGameSet;

use super::{animate_sprite, update_animation_on_movement_direction_change, DefaultAnimationConfig};

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            ((
                animate_sprite,
                update_animation_on_movement_direction_change, //Change animation if animation component changes
            ))
                .in_set(InGameSet::Simulation),
        )
        .insert_resource(DefaultAnimationConfig::default());
    }
}
