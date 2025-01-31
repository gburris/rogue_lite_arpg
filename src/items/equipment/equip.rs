use avian2d::prelude::Collider;
use bevy::prelude::*;

use crate::{
    combat::melee::components::MeleeHitbox,
    items::{
        equipment::{equipment_slots::*, EquipmentSlot, EquipmentSlots},
        inventory::inventory::Inventory,
    },
    player::Player,
    ui::pause_menu::button_interactions::{TryEquipEvent, TryUnequipEvent},
};

#[derive(Component, Clone, Debug)]
pub struct Equippable {
    pub use_rate: Timer, // swing a sword, shoot a weapon, etc...
}

impl Default for Equippable {
    fn default() -> Self {
        Self {
            use_rate: Timer::from_seconds(0.4, TimerMode::Once),
        }
    }
}

#[derive(Event)]
pub struct UnequipSuccessEvent {
    pub item_entity: Entity,
}

#[derive(Event)]
pub struct EquipSuccessEvent {
    pub item_entity: Entity,
    pub previous_item: Option<Entity>,
}

fn update_item_visibility(
    entity: Entity,
    visible: bool,
    commands: &mut Commands,
    visibility_query: &mut Query<&mut Visibility>,
) {
    if let Ok(mut visibility) = visibility_query.get_mut(entity) {
        *visibility = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    if visible {
        commands.entity(entity).insert(Collider::rectangle(
            MeleeHitbox::default().width,
            MeleeHitbox::default().length,
        ));
    } else {
        commands.entity(entity).remove::<Collider>();
    }
}

pub fn handle_try_equip_event(
    try_equip_trigger: Trigger<TryEquipEvent>,
    mut commands: Commands,
    mut equipment_query: Query<&mut EquipmentSlots>,
    mut inventory_query: Query<&mut Inventory>,
    slot_query: Query<&EquipmentSlot>,
) {
    if let Ok(mut equipment_slots) = equipment_query.get_single_mut() {
        if let Some(previous_item) = equip_item(
            &mut equipment_slots,
            try_equip_trigger.item_entity,
            &slot_query,
        ) {
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

pub fn handle_try_unequip_event(
    try_equip_trigger: Trigger<TryUnequipEvent>,
    mut commands: Commands,
    mut equipment_query: Query<&mut EquipmentSlots>,
    mut inventory_query: Query<&mut Inventory>,
    slot_query: Query<&EquipmentSlot>,
) {
    if let Ok(mut equipment_slots) = equipment_query.get_single_mut() {
        if let Ok(mut inventory) = inventory_query.get_single_mut() {
            let did_add_item = inventory.add_item(try_equip_trigger.item_entity);
            if did_add_item.is_ok() {
                unequip_item(
                    &mut equipment_slots,
                    try_equip_trigger.item_entity,
                    &slot_query,
                );
                commands.trigger(UnequipSuccessEvent {
                    item_entity: try_equip_trigger.item_entity,
                });
            } else {
                warn!("Inventory was full! Cannot unequip weapon");
            }
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

    if let Some(previous_item) = equip_success_trigger.previous_item {
        commands
            .entity(player_entity)
            .remove_children(&[previous_item]);
        update_item_visibility(previous_item, false, &mut commands, &mut visibility_query);
    }

    commands
        .entity(player_entity)
        .add_child(equip_success_trigger.item_entity);

    update_item_visibility(
        equip_success_trigger.item_entity,
        true,
        &mut commands,
        &mut visibility_query,
    );
}

pub fn handle_unequip_success_event(
    unequip_success_trigger: Trigger<UnequipSuccessEvent>,
    player_query: Query<Entity, With<Player>>,
    mut commands: Commands,
    mut visibility_query: Query<&mut Visibility>,
) {
    if let Ok(player_entity) = player_query.get_single() {
        commands
            .entity(player_entity)
            .remove_children(&[unequip_success_trigger.item_entity]);

        update_item_visibility(
            unequip_success_trigger.item_entity,
            false,
            &mut commands,
            &mut visibility_query,
        );
    }
}
pub fn tick_equippable_use_rate(mut equippable_query: Query<&mut Equippable>, time: Res<Time>) {
    for mut equippable in equippable_query.iter_mut() {
        equippable.use_rate.tick(time.delta());
    }
}
