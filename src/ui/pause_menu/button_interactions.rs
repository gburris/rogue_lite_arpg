use bevy::prelude::*;

use crate::{
    items::{
        equipment::{Equippable, Equipped},
        inventory::Inventory,
        Consumable, Item,
    },
    labels::states::PausedState,
    player::{systems::ConsumeEvent, Player},
    ui::display_case::DisplayCaseSlot,
};

use super::main_menu::MenuButton;

const HOVER_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.3);

/// Trigger on entity with Inventory component (i.e. the player entity)
#[derive(Event)]
pub struct UpdateInventoryUIEvent;

pub fn on_item_done_hovering(
    trigger: Trigger<Pointer<Out>>,
    mut item_slot: Query<&mut BackgroundColor, With<DisplayCaseSlot>>,
) {
    if let Ok(mut background_color) = item_slot.get_mut(trigger.entity()) {
        background_color.0 = Color::NONE;
    }
}

pub fn on_item_hover(
    trigger: Trigger<Pointer<Over>>,
    mut item_slot: Query<&mut BackgroundColor, With<DisplayCaseSlot>>,
) {
    if let Ok(mut background_color) = item_slot.get_mut(trigger.entity()) {
        background_color.0 = HOVER_COLOR;
    }
}

pub fn on_item_clicked(
    trigger: Trigger<Pointer<Click>>,
    mut commands: Commands,
    slot_query: Query<&DisplayCaseSlot>,
    item_query: Query<(Has<Equippable>, Has<Consumable>), With<Item>>,
    player: Single<(Entity, &Inventory), With<Player>>,
) {
    let item_slot = slot_query.get(trigger.entity()).unwrap();
    let (player_entity, inventory) = player.into_inner();
    let item_entity = inventory.items[item_slot.index];

    if let Ok((equippable, consumable)) = item_query.get(item_entity) {
        if equippable {
            commands
                .entity(item_entity)
                .insert(Equipped::new(player_entity));
        } else if consumable {
            commands.trigger_targets(ConsumeEvent { item_entity }, player_entity);
        }

        commands.trigger_targets(UpdateInventoryUIEvent, player_entity);
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
