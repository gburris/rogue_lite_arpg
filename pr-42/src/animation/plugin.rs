use bevy::prelude::*;

use crate::labels::sets::InGameSet;

use super::{
    animate_sprite,
    spells::{on_healing_tome_visual_added, on_shield_effect_added},
    update_animation, DefaultAnimationConfig,
};

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_healing_tome_visual_added)
            .add_observer(on_shield_effect_added)
            .add_systems(
                Update,
                ((animate_sprite, update_animation)).in_set(InGameSet::Simulation),
            )
            .insert_resource(DefaultAnimationConfig::default());
    }
}
