use bevy::prelude::*;

use crate::{
    combat::{damage::DamageSource, melee::MeleeWeapon},
    enemy::Enemy,
    items::inventory::Inventory,
};

use super::{equippable::Equipped, EquipmentSlot, Equippable};

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
    let equipped_entity = trigger.target();
    let (equippable, equipped, mut visibility, melee_weapon) = item_query
        .get_mut(equipped_entity)
        .expect("Added Equipped to non-equippable item");

    let (mut inventory, enemy) = holder_query
        .get_mut(equipped.get_equipped_to())
        .expect("Added Equipped to item with holder that is missing an inventory");

    // If previously equipped, must handle it!
    if let Some(previous) = inventory.get_equipped(equippable.slot) {
        commands.entity(previous).remove::<Equipped>();
    }

    inventory.equip(equipped_entity, equippable.slot);

    if equippable.slot == EquipmentSlot::Mainhand || equippable.slot == EquipmentSlot::Offhand {
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
