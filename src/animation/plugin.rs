use bevy::prelude::*;

use crate::labels::sets::InGameSet;

use super::{DefaultAnimationConfig, animate_sprite, update_animation};

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                animate_sprite,
                update_animation, //Change animation if components change that dictace updating it
            )
                .chain()
                .in_set(InGameSet::Vfx),
        )
        .insert_resource(DefaultAnimationConfig::default());
    }
}
