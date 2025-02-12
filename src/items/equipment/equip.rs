use bevy::prelude::*;

use crate::{
    combat::{damage::components::DamageSource, melee::components::MeleeWeapon},
    enemy::Enemy,
    items::{equipment::UnequipEvent, inventory::Inventory},
};

use super::{equippable::Equipped, EquipmentSlot, Equippable};

#[derive(Event)]
pub struct EquipEvent {
    pub item_entity: Entity,
}

impl EquipEvent {
    pub fn new(item_entity: Entity) -> Self {
        Self { item_entity }
    }
}

pub fn on_equip_event(equip_trigger: Trigger<EquipEvent>, mut commands: Commands) {
    commands
        .entity(equip_trigger.item_entity)
        .insert(Equipped::new(equip_trigger.entity()));
}

pub fn on_item_equipped(
    trigger: Trigger<OnAdd, Equipped>,
    mut commands: Commands,
    mut item_query: Query<(
        &Equippable,
        &Equipped,
        &mut Visibility,
        Option<&MeleeWeapon>,
    )>,
    mut holder_query: Query<(&mut Inventory, Option<&Enemy>)>,
) {
    let equipped_entity = trigger.entity();
    let (equippable, equipped, mut visibility, melee_weapon) = item_query
        .get_mut(equipped_entity)
        .expect("Added Equipped to non-equippable item");

    let (mut inventory, enemy) = holder_query
        .get_mut(equipped.get_equipped_to())
        .expect("Added Equipped to item with holder that is missing an inventory");

    // If previously equipped, must handle it!
    if let Some(previous) = inventory.get_equipped(equippable.slot) {
        commands.trigger_targets(
            UnequipEvent {
                item_entity: previous,
            },
            equipped.get_equipped_to(),
        );
    }

    inventory.equip(equipped_entity, equippable.slot);

    if equippable.slot == EquipmentSlot::Mainhand {
        // Make sure item is now visible, since it is hidden while in inventory
        *visibility = Visibility::Visible;
    }

    if let Some(melee_weapon) = melee_weapon {
        let damage_source = if enemy.is_some() {
            DamageSource::Enemy
        } else {
            DamageSource::Player
        };

        // If melee weapon, we need to add collider and new collision layers on equip
        commands.entity(equipped_entity).insert((
            melee_weapon.hitbox.clone(),
            MeleeWeapon::collision_layers(damage_source),
        ));
    }
}
