use bevy::prelude::*;

use crate::labels::sets::InGameSet;

use super::{animate_sprite, update_animation, DefaultAnimationConfig};

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            ((
                animate_sprite,
                update_animation, //Change animation if components change that dictace updating it
            ))
                .in_set(InGameSet::Simulation),
        )
        .insert_resource(DefaultAnimationConfig::default());
    }
}
