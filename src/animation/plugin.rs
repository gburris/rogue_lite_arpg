use bevy::prelude::*;

use crate::labels::sets::InGameSystems;

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
                .in_set(InGameSystems::Vfx),
        )
        .insert_resource(DefaultAnimationConfig::default());
    }
}
