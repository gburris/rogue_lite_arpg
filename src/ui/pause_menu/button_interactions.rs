use bevy::prelude::*;

use crate::{
    items::{
        equipment::{
            equip::{AttemptEquipEvent, AttemptUnequipEvent},
            Equippable,
        },
        Consumable, Item,
    },
    labels::states::PausedState,
    player::{systems::ConsumeEvent, Player},
    ui::display_case::{DisplayCaseSlot, FilledDisplaySlot},
};

use super::{
    equipment_menu::{EquipmentButton, UpdateEquipmentUIEvent},
    inventory_menu::ItemText,
    main_menu::MenuButton,
};

#[derive(Event)]
pub struct EquipmentItemClicked {
    pub item_entity: Option<Entity>,
}

#[derive(Event)]
pub struct ItemRemovedUIEvent {
    pub item_entity: Entity,
    pub index: usize,
}

/// Trigger on entity with Inventory component (i.e. the player entity)
#[derive(Event)]
pub struct UpdateInventoryUIEvent;

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
        // commands.trigger(UpdateInventoryUIEvent);
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
    slot_query: Query<(&FilledDisplaySlot, &Parent)>,
    slot_index_query: Query<&DisplayCaseSlot>,
    item_query: Query<(Has<Equippable>, Has<Consumable>), With<Item>>,
    player: Single<Entity, With<Player>>,
) {
    let (item_slot, slot_parent) = slot_query.get(trigger.entity()).unwrap();

    let item_entity = item_slot.item;

    if let Ok((equippable, consumable)) = item_query.get(item_entity) {
        if equippable {
            commands.trigger_targets(AttemptEquipEvent { item_entity }, *player);
        } else if consumable {
            commands.trigger_targets(ConsumeEvent { item_entity }, *player);
        }

        commands.trigger_targets(UpdateInventoryUIEvent, *player);
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
