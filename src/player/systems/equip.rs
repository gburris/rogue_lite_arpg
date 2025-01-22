use bevy::prelude::*;

use crate::{
    combat::{
        attributes::{mana::ManaCost, Mana},
        weapon::weapon::UseEquipmentEvent,
    },
    items::{EquipmentSlot, Equippable},
    player::{components::PlayerEquipmentSlots, equip_item, Inventory, MainHandActivated, Player},
    ui::pause_menu::button_interactions::TryEquipEvent,
};

#[derive(Event)]
pub struct EquipSuccessEvent {
    pub item_entity: Entity,
    pub previous_item: Option<Entity>,
}

pub fn handle_try_equip_event(
    try_equip_trigger: Trigger<TryEquipEvent>,
    mut commands: Commands,
    mut equipment_query: Query<&mut PlayerEquipmentSlots>,
    mut inventory_query: Query<&mut Inventory>,
    slot_query: Query<&EquipmentSlot>,
) {
    if let Ok(mut equipment_slots) = equipment_query.get_single_mut() {
        if let Some(previous_item) = equip_item(
            &mut equipment_slots,
            try_equip_trigger.item_entity,
            &slot_query,
        ) {
            //Case where there was already something in the slot
            if let Ok(mut inventory) = inventory_query.get_single_mut() {
                let _ = inventory.remove_item(try_equip_trigger.item_entity);
                let _ = inventory.add_item(previous_item);
            }
            commands.trigger(EquipSuccessEvent {
                item_entity: try_equip_trigger.item_entity,
                previous_item: Some(previous_item),
            });
        } else {
            if let Ok(mut inventory) = inventory_query.get_single_mut() {
                let _ = inventory.remove_item(try_equip_trigger.item_entity);
            }
            commands.trigger(EquipSuccessEvent {
                item_entity: try_equip_trigger.item_entity,
                previous_item: None,
            });
        }
    }
}

pub fn handle_equip_success_event(
    equip_success_trigger: Trigger<EquipSuccessEvent>,
    player_query: Query<Entity, With<Player>>,
    mut commands: Commands,
    mut visibility_query: Query<&mut Visibility>,
) {
    let Ok(player_entity) = player_query.get_single() else {
        return;
    };

    // If there was a previous item, remove it from the player
    if let Some(previous_item) = equip_success_trigger.previous_item {
        commands
            .entity(player_entity)
            .remove_children(&[previous_item]);
        if let Ok(mut visibility) = visibility_query.get_mut(previous_item) {
            *visibility = Visibility::Hidden;
        }
    }

    // Add the new item as a child of the player
    commands
        .entity(player_entity)
        .add_child(equip_success_trigger.item_entity);
    if let Ok(mut visibility) = visibility_query.get_mut(equip_success_trigger.item_entity) {
        *visibility = Visibility::Visible;
    }
}

pub fn on_main_hand_activated(
    main_hand_trigger: Trigger<MainHandActivated>,
    mut commands: Commands,
    mut holder_query: Query<(&Children, Option<&mut Mana>)>,
    mut main_hand_query: Query<(&mut Equippable, Option<&ManaCost>)>,
) {
    // Parent needs to have an aim position for equipped item
    let Ok((children, mut holder_mana)) = holder_query.get_mut(main_hand_trigger.entity()) else {
        debug!(
            "Entity: {} tried to use main hand with nothing equipped",
            main_hand_trigger.entity()
        );
        return;
    };

    for &child in children.iter() {
        // if child is equippable
        if let Ok((mut equippable, mana_cost)) = main_hand_query.get_mut(child) {
            if equippable.use_rate.finished() {
                let has_enough_mana = if let (Some(mana), Some(&ManaCost(mana_cost))) =
                    (holder_mana.as_mut(), mana_cost)
                {
                    mana.use_mana(mana_cost)
                } else {
                    true
                };

                if has_enough_mana {
                    commands.trigger_targets(
                        UseEquipmentEvent {
                            holder: main_hand_trigger.entity(),
                        },
                        child,
                    );
                    equippable.use_rate.reset();
                }
            }
        }
    }
}

pub fn tick_equippable_use_rate(mut equippable_query: Query<&mut Equippable>, time: Res<Time>) {
    for mut equippable in equippable_query.iter_mut() {
        equippable.use_rate.tick(time.delta());
    }
}
