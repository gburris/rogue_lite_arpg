use bevy::prelude::*;

use crate::labels::states::PausedState;

use super::{inventory_menu::InventoryItemButton, main_menu::MenuButton};

// Add this new event
#[derive(Event)]
pub struct InventoryItemClicked {
    pub item_entity: Option<Entity>,
}

pub fn handle_inventory_interactions(
    mut interaction_query: Query<(&Interaction, &InventoryItemButton, &mut BackgroundColor)>,
    mut commands: Commands,
) {
    warn!("listening");
    for (interaction, button, mut background_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Hovered => {
                *background_color = BackgroundColor::from(Color::srgb(0.95, 1.0, 0.0));
            }
            Interaction::Pressed => {
                commands.trigger(InventoryItemClicked {
                    item_entity: button.item_entity,
                });
            }
            Interaction::None => {}
        }
    }
}

// pub fn handle_inventory_click(
//     _: Trigger<InventoryItemClicked>,
//     mut commands: Commands,
//     player_equipment: Query<&mut PlayerEquipmentSlots>,
// ) {
//     warn!("handling inventory click");
// }

pub fn handle_menu_button_pressed(
    mut button_query: Query<(&Interaction, &MenuButton)>,
    mut pause_state: ResMut<NextState<PausedState>>,
) {
    for (interaction, menu_button) in &mut button_query {
        if *interaction == Interaction::Pressed {
            debug!("handle_menu_button_pressed");
            pause_state.set(menu_button.0);
        }
    }
}
