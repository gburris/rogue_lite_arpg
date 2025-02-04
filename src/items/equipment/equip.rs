use avian2d::prelude::Collider;
use bevy::prelude::*;

use crate::{
    combat::melee::components::MeleeWeapon,
    items::{
        equipment::{equipment_slots::*, EquipmentSlot, EquipmentSlots},
        inventory::inventory::Inventory,
    },
    player::Player,
    ui::pause_menu::button_interactions::{TryEquipFromUIEvent, TryUnequipFromUIEvent},
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
}

pub fn handle_try_equip_event(
    try_equip_trigger: Trigger<TryEquipFromUIEvent>,
    mut commands: Commands,
    mut equipment_query: Query<&mut EquipmentSlots, With<Player>>,
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
            commands.trigger(UnequipSuccessEvent {
                item_entity: previous_item,
            });
        } else {
            if let Ok(mut inventory) = inventory_query.get_single_mut() {
                let _ = inventory.remove_item(try_equip_trigger.item_entity);
            }
        }
    }
}

pub fn handle_try_unequip_event(
    try_equip_trigger: Trigger<TryUnequipFromUIEvent>,
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

//Legacy unequip code.
//This will not work for NPCs, Enemies, whatever, player only
pub fn handle_unequip_success_event(
    unequip_success_trigger: Trigger<UnequipSuccessEvent>,
    mut commands: Commands,
    player_query: Query<Entity, With<Player>>,
    mut visibility_query: Query<&mut Visibility>,
) {
    if let Ok(player_entity) = player_query.get_single() {
        commands
            .entity(player_entity)
            .remove_children(&[unequip_success_trigger.item_entity]);

        if let Ok(mut visibility) = visibility_query.get_mut(unequip_success_trigger.item_entity) {
            *visibility = Visibility::Hidden
        }
        warn!("removing hitbox");
        commands
            .entity(unequip_success_trigger.item_entity)
            .remove::<Collider>();
    }
}

pub fn on_equipment_slot_equip(
    mut commands: Commands,
    mut item_query: Query<(Entity, &mut Visibility, Option<&MeleeWeapon>)>,
    mut holder_query: Query<(Entity, &EquipmentSlots), Changed<EquipmentSlots>>,
) {
    for (holder_entity, equipment_slot) in holder_query.iter_mut() {
        warn!("equipment slots changed");
        if let Some(mainhand) = equipment_slot.mainhand {
            //The mainhand exists (equip)
            if let Ok((item_entity, mut visibility, mainhand_item)) = item_query.get_mut(mainhand) {
                *visibility = Visibility::Visible;
                commands.entity(holder_entity).add_child(item_entity);
                if let Some(melee_weapon) = mainhand_item {
                    let hitbox = &melee_weapon.hitbox;
                    warn!("adding hitbox");
                    commands
                        .entity(mainhand)
                        .insert(Collider::rectangle(hitbox.width, hitbox.length));
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
