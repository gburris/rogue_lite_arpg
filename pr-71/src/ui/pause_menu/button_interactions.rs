use bevy::prelude::*;

use crate::labels::states::PausedState;

use super::main_menu::MenuButton;

pub fn handle_menu_button_pressed(
    mut button_query: Query<(&Interaction, &MenuButton)>,
    mut pause_state: ResMut<NextState<PausedState>>,
) {
    for (interaction, menu_button) in &mut button_query {
        if *interaction == Interaction::Pressed {
            pause_state.set(menu_button.0);
        }
    }
}
