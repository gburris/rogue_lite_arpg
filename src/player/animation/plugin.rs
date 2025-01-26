use bevy::prelude::*;

use crate::{
    labels::{sets::InGameSet, states::AppState},
    player::animation::{
        animate_player::update_player_animation, components::PlayerAnimationConfig,
        run_player_animation::run_player_animation,
    },
};

pub struct PlayerAnimationPlugin;

impl Plugin for PlayerAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            ((
                update_player_animation, //Change animation if animation component changes
                run_player_animation,    //Process the frames of the animation
            ))
                .in_set(InGameSet::Simulation)
                .run_if(in_state(AppState::Playing)),
        )
        .insert_resource(PlayerAnimationConfig::default());
    }
}
