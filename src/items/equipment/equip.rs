use avian2d::prelude::Collider;
use bevy::prelude::*;

use crate::{
    combat::{damage::components::DamageSource, melee::components::MeleeWeapon},
    enemy::Enemy,
    items::{
        equipment::{EquipmentSlot, EquipmentSlots},
        inventory::inventory::Inventory,
    },
    player::Player,
    ui::pause_menu::button_interactions::{AttemptEquipEvent, AttemptUnequipEvent},
};

#[derive(Component, Clone)]
pub struct Equippable {
    pub slot: EquipmentSlot,
    pub use_rate: Timer, // swing a sword, shoot a weapon, etc...
}

impl Default for Equippable {
    fn default() -> Self {
        Self {
            slot: EquipmentSlot::Mainhand,
            use_rate: Timer::from_seconds(0.4, TimerMode::Once),
        }
    }
}

impl Equippable {
    pub fn new(slot: EquipmentSlot) -> Self {
        Equippable { slot, ..default() }
    }
}

#[derive(Event)]
pub struct UnequipSuccessEvent {
    pub item_entity: Entity,
}

pub fn attempt_equip_from_inventory(
    try_equip_trigger: Trigger<AttemptEquipEvent>,
    mut commands: Commands,
    mut holder_query: Query<(&mut EquipmentSlots, &mut Inventory), With<Player>>,
    equipment_query: Query<&Equippable>,
) {
    if let Ok((mut equipment_slots, mut inventory)) =
        holder_query.get_mut(try_equip_trigger.entity())
    {
        if let Ok(equippable) = equipment_query.get(try_equip_trigger.item_entity) {
            if let Some(previous_item) =
                equipment_slots.equip(try_equip_trigger.item_entity, &equippable.slot)
            {
                if inventory.add_item(previous_item).is_ok() {
                    commands.trigger(UnequipSuccessEvent {
                        item_entity: previous_item,
                    });
                } else {
                    // TODO: handle this scenario, need to prevent equip if inventory is going to be full
                    error!("Inventory was full! Already equipped new item before previous one was unequiped");
                }
            }

            // Regardless of if there was a previous item or not, we need to remove equipped item from inventory
            inventory
                .remove_item_by_value(try_equip_trigger.item_entity)
                .expect("Failed to remove newly equipped item from inventory");
        }
    }
}

pub fn handle_try_unequip_event(
    try_unequip_trigger: Trigger<AttemptUnequipEvent>,
    mut commands: Commands,
    mut holder_query: Query<(&mut EquipmentSlots, &mut Inventory)>,
    equipment_query: Query<&Equippable>,
) {
    if let Ok((mut equipment_slots, mut inventory)) =
        holder_query.get_mut(try_unequip_trigger.entity())
    {
        if let Ok(equipped) = equipment_query.get(try_unequip_trigger.item_entity) {
            if inventory.add_item(try_unequip_trigger.item_entity).is_ok() {
                equipment_slots.unequip(&equipped.slot);
                commands.trigger(UnequipSuccessEvent {
                    item_entity: try_unequip_trigger.item_entity,
                });
            } else {
                // TODO: add UI to inform player this failed
                error!("Inventory was full! Cannot unequip weapon");
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
        commands
            .entity(unequip_success_trigger.item_entity)
            .remove::<Collider>();
    }
}

pub fn on_equipment_slot_equip(
    mut commands: Commands,
    mut item_query: Query<(Entity, &mut Visibility, Option<&MeleeWeapon>)>,
    mut holder_query: Query<(Entity, &EquipmentSlots, Option<&Enemy>), Changed<EquipmentSlots>>,
) {
    for (holder_entity, equipment_slot, enemy) in holder_query.iter_mut() {
        if let Some(mainhand) = equipment_slot.mainhand {
            //The mainhand exists (equip)
            if let Ok((item_entity, mut visibility, melee_weapon)) = item_query.get_mut(mainhand) {
                // Add equipment as child to holder, this will make the entity visible
                //Just kidding, player somehow still requires visibility toggle
                *visibility = Visibility::Visible;
                commands.entity(holder_entity).add_child(item_entity);

                if let Some(melee_weapon) = melee_weapon {
                    let damage_source = if enemy.is_some() {
                        DamageSource::Enemy
                    } else {
                        DamageSource::Player
                    };

                    // If melee weapon, we need to add collider and new collision layers on equip
                    commands.entity(mainhand).insert((
                        melee_weapon.hitbox.clone(),
                        MeleeWeapon::collision_layers(damage_source),
                    ));
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
