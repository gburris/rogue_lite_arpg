use bevy::prelude::*;

use crate::{
    items::{Consumable, Equipable, ItemName},
    labels::states::PausedState,
};

use super::{
    inventory_menu::{InventoryItemButton, InventoryItemName},
    main_menu::MenuButton,
};

#[derive(Event)]
pub struct InventoryItemClicked {
    pub item_entity: Option<Entity>,
}

#[derive(Event)]
pub struct EquipEvent {
    pub item_entity: Entity,
}

#[derive(Event)]
pub struct InventoryUpdatedEvent;

#[derive(Event)]
pub struct ConsumeEvent {
    pub item_entity: Entity,
}

pub fn handle_inventory_interactions(
    mut interaction_query: Query<(&Interaction, &InventoryItemButton, Entity)>,
    mut menu_item_text: Query<(&mut TextColor, &Parent), With<InventoryItemName>>,
    mut commands: Commands,
) {
    for (interaction, button, entity) in interaction_query.iter_mut() {
        // Find the text color component for this button's text
        let text_color = menu_item_text
            .iter_mut()
            .find(|(_, parent)| parent.get() == entity);

        match *interaction {
            Interaction::Hovered => {
                if let Some((mut color, _)) = text_color {
                    // Change text color to a brighter shade when hovering
                    *color = TextColor::from(Color::srgb(0.0, 1.0, 1.0));
                }
            }
            Interaction::Pressed => {
                commands.trigger(InventoryItemClicked {
                    item_entity: button.item_entity,
                });
            }
            Interaction::None => {
                if let Some((mut color, _)) = text_color {
                    // Change text color to a brighter shade when hovering
                    *color = TextColor::default();
                }
            }
        }
    }
}

pub fn handle_inventory_click(
    trigger: Trigger<InventoryItemClicked>,
    mut commands: Commands,
    equipable_query: Query<&Equipable>,
    consumable_query: Query<&Consumable>,
    item_name_query: Query<&ItemName>,
) {
    if let Some(item_entity) = trigger.item_entity {
        // Check if item is equipable
        if let Ok(item_name) = item_name_query.get(item_entity) {
            debug!("User clicked on {:?}", item_name);
        }

        if equipable_query.contains(item_entity) {
            commands.trigger(EquipEvent {
                item_entity: item_entity,
            });
        }
        // Check if item is consumable
        else if consumable_query.contains(item_entity) {
            commands.trigger(ConsumeEvent {
                item_entity: item_entity,
            });
        }
        //Redraw inventory
        commands.trigger(InventoryUpdatedEvent);
    }
}

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
