use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use crate::player::Player;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EnhancedInputPlugin)
            .add_input_context::<Node>()
            .add_input_context::<Player>();
    }
}

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
struct Move;
