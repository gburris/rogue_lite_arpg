use bevy::prelude::*;

use crate::{
    items::{equipment::Equippable, Consumable, Item},
    labels::states::PausedState,
    player::Player,
};

use super::{
    equipment_menu::{EquipmentButton, EquipmentItemText, UpdateEquipmentUIEvent},
    inventory_menu::{InventorySlot, ItemText},
    main_menu::MenuButton,
};

#[derive(Event)]
pub struct EquipmentItemClicked {
    pub item_entity: Option<Entity>,
}
#[derive(Event)]
pub struct AttemptEquipEvent {
    pub item_entity: Entity,
}

#[derive(Event)]
pub struct AttemptUnequipEvent {
    pub item_entity: Entity,
}

#[derive(Event)]
pub struct UpdateInventoryUIEvent;

#[derive(Event)]
pub struct ConsumeEvent {
    pub item_entity: Entity,
}

pub fn on_equipped_clicked(
    trigger: Trigger<Pointer<Click>>,
    mut commands: Commands,
    slot_query: Query<&EquipmentButton>,
    player: Single<Entity, With<Player>>,
) {
    let slot = slot_query.get(trigger.entity());

    if let Ok(EquipmentButton {
        item_entity: Some(item_entity),
    }) = slot
    {
        commands.trigger_targets(
            AttemptUnequipEvent {
                item_entity: *item_entity,
            },
            *player,
        );
        //Redraw item lists
        commands.trigger(UpdateEquipmentUIEvent);
        commands.trigger(UpdateInventoryUIEvent);
    }
}

pub fn on_item_done_hovering(
    trigger: Trigger<Pointer<Out>>,
    mut menu_item_text: Query<(&mut TextColor, &Parent), With<ItemText>>,
) {
    // Find the text color component for this button's text
    if let Some((mut text_color, _)) = menu_item_text
        .iter_mut()
        .find(|(_, parent)| parent.get() == trigger.entity())
    {
        *text_color = TextColor::default();
    }
}

pub fn on_item_hover(
    trigger: Trigger<Pointer<Over>>,
    mut menu_item_text: Query<(&mut TextColor, &Parent), With<ItemText>>,
) {
    // Find the text color component for this button's text
    if let Some((mut text_color, _)) = menu_item_text
        .iter_mut()
        .find(|(_, parent)| parent.get() == trigger.entity())
    {
        // Change text color to a brighter shade when hovering
        text_color.0 = Color::srgb(0.0, 1.0, 1.0);
    }
}

pub fn on_item_clicked(
    trigger: Trigger<Pointer<Click>>,
    mut commands: Commands,
    slot_query: Query<&InventorySlot>,
    item_query: Query<(Has<Equippable>, Has<Consumable>), With<Item>>,
    player: Single<Entity, With<Player>>,
) {
    let slot = slot_query.get(trigger.entity());

    if let Ok(InventorySlot {
        item: Some(item_entity),
        index: _,
    }) = slot
    {
        if let Ok((equippable, consumable)) = item_query.get(*item_entity) {
            if equippable {
                commands.trigger_targets(
                    AttemptEquipEvent {
                        item_entity: *item_entity,
                    },
                    *player,
                );
            } else if consumable {
                commands.trigger_targets(
                    ConsumeEvent {
                        item_entity: *item_entity,
                    },
                    *player,
                );
            }

            commands.trigger(UpdateInventoryUIEvent);
        }
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
