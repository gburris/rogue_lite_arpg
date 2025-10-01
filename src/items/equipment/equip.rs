use bevy::{prelude::*, state::commands};

use crate::{
    character::Character,
    combat::{damage::DamageSource, melee::MeleeWeapon},
    items::{
        equipment::{EquipmentOf, Mainhand, MainhandOf, Offhand, OffhandOf},
        ItemOf,
    },
    prelude::Enemy,
};

use super::{EquipmentSlot, Equippable};

pub fn on_item_equipped(
    trigger: Trigger<OnAdd, EquipmentOf>,
    mut commands: Commands,
    mut item_query: Query<(
        &Equippable,
        &EquipmentOf,
        &mut Visibility,
        Option<&MeleeWeapon>,
    )>,
    mut holder_query: Query<Option<&Enemy>, With<Character>>,
) {
    let equipped_entity = trigger.target();
    let (equippable, equipment_of, mut visibility, melee_weapon) = item_query
        .get_mut(equipped_entity)
        .expect("Added Equipped to non-equippable item");

    let (enemy) = holder_query
        .get_mut(equipment_of.0)
        .expect("Added Equipped to item with holder that is missing an inventory");

    commands
        .entity(equipped_entity)
        .insert((ItemOf(equipment_of.0), ChildOf(equipment_of.0)));

    match equippable.slot {
        EquipmentSlot::Mainhand => {
            // remove bevy 0.17
            commands.entity(equipment_of.0).remove::<Mainhand>();

            commands
                .entity(equipped_entity)
                .insert(MainhandOf(equipment_of.0));
        }
        EquipmentSlot::Offhand => {
            commands.entity(equipment_of.0).remove::<Offhand>();

            commands
                .entity(equipped_entity)
                .insert(OffhandOf(equipment_of.0));
        }
    }

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
