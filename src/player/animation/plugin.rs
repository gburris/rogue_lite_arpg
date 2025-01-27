use bevy::prelude::*;

use crate::{
    labels::sets::InGameSet,
    player::animation::{
        animate_player::update_player_animation, components::PlayerAnimationConfig,
    },
};

pub struct PlayerAnimationPlugin;

impl Plugin for PlayerAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            ((
                update_player_animation, //Change animation if animation component changes
            ))
                .in_set(InGameSet::Simulation),
        )
        .insert_resource(PlayerAnimationConfig::default());
    }
}
