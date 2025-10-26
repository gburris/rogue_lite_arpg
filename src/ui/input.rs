use bevy::prelude::*;

use crate::prelude::PauseInputEvent;

//UN-Pause logic, runs when App State is Paused
pub fn handle_ui_inputs(mut commands: Commands, mut keyboard_input: ResMut<ButtonInput<KeyCode>>) {
    if keyboard_input.clear_just_pressed(KeyCode::Escape) {
        debug!("ui_inputs, enter");
        commands.trigger(PauseInputEvent { menu: None });
    }
}
