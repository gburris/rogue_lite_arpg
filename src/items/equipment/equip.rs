use bevy::prelude::*;

use crate::{
    character::Character,
    combat::{damage::DamageSource, melee::MeleeWeapon},
    items::{
        ItemOf,
        equipment::{Equipped, MainhandOf, OffhandOf},
    },
    prelude::Enemy,
};

use super::{EquipmentSlot, Equippable};

pub fn on_item_equipped(
    trigger: On<Add, Equipped>,
    mut commands: Commands,
    mut item_query: Query<
        (&ItemOf, &Equippable, &mut Visibility, Option<&MeleeWeapon>),
        With<Equipped>,
    >,
    mut holder_query: Query<Has<Enemy>, With<Character>>,
) {
    let equipped_entity = trigger.event().entity;
    let (item_of, equippable, mut visibility, melee_weapon) = item_query
        .get_mut(equipped_entity)
        .expect("Added Equipped to non-equippable item");

    let holder_entity = item_of.0;

    let is_enemy = holder_query
        .get_mut(holder_entity)
        .expect("Added Equipment to holder that is not a character");

    commands
        .entity(equipped_entity)
        .insert(ChildOf(holder_entity));

    match equippable.slot {
        EquipmentSlot::Mainhand => {
            commands
                .entity(equipped_entity)
                .insert(MainhandOf(holder_entity));
        }
        EquipmentSlot::Offhand => {
            commands
                .entity(equipped_entity)
                .insert(OffhandOf(holder_entity));
        }
    }

    if equippable.slot == EquipmentSlot::Mainhand || equippable.slot == EquipmentSlot::Offhand {
        // Make sure item is now visible, since it is hidden while in inventory
        *visibility = Visibility::Visible;
    }

    if let Some(melee_weapon) = melee_weapon {
        let damage_source = if is_enemy {
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
