use avian2d::prelude::Collider;
use bevy::prelude::*;

use crate::{
    character::Character,
    combat::damage::DamageSource,
    items::{
        ItemOf,
        equipment::{Equipped, Mainhand, MainhandOf, Offhand, OffhandOf},
        melee::{ActiveMeleeAttack, MeleeWeapon},
    },
    prelude::{AttackState, Enemy},
};

use super::{EquipmentSlot, Equippable};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_equip).add_observer(on_unequip);
}

fn on_equip(
    trigger: On<Add, Equipped>,
    mut commands: Commands,
    mut item_query: Query<
        (&ItemOf, &Equippable, &mut Visibility, Option<&MeleeWeapon>),
        With<Equipped>,
    >,
    mut holder_query: Query<(Option<&Mainhand>, Option<&Offhand>, Has<Enemy>), With<Character>>,
) {
    let equipped_entity = trigger.entity;
    let (item_of, equippable, mut visibility, melee_weapon) = item_query
        .get_mut(equipped_entity)
        .expect("Added Equipped to non-equippable item");

    let holder_entity = item_of.0;

    let (mainhand, offhand, is_enemy) = holder_query
        .get_mut(holder_entity)
        .expect("Added Equipment to holder that is not a character");

    commands
        .entity(equipped_entity)
        .insert(ChildOf(holder_entity));

    match equippable.slot {
        EquipmentSlot::Mainhand => {
            if let Some(mainhand) = mainhand {
                commands.trigger(Unequip { entity: mainhand.0 })
            }

            commands
                .entity(equipped_entity)
                .insert(MainhandOf(holder_entity));
        }
        EquipmentSlot::Offhand => {
            if let Some(offhand) = offhand {
                commands.trigger(Unequip { entity: offhand.0 })
            }

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

#[derive(EntityEvent)]
pub struct Unequip {
    pub entity: Entity,
}

fn on_unequip(
    trigger: On<Unequip>,
    mut commands: Commands,
    mut item_query: Query<(&ItemOf, &mut Visibility, Has<ActiveMeleeAttack>), With<Equipped>>,
    mut holder_query: Query<&mut AttackState>,
) {
    let item_entity = trigger.entity;

    let Ok((equipment_of, mut visibility, is_active_attack)) = item_query.get_mut(item_entity)
    else {
        info!("Equipment was despawned prior to unequip");
        return;
    };

    let Ok(mut attack_state) = holder_query.get_mut(equipment_of.0) else {
        info!("Holder was despawned prior to unequip");
        return;
    };

    // If you are in the menu and unequip a weapon while you were mid-swing,
    // we need to handle leaving attack state
    // TODO: Consider just cancelling attacks upon pausing
    if is_active_attack {
        attack_state.is_attacking = false;
    }

    *visibility = Visibility::Hidden;
    commands.entity(item_entity).remove::<(
        Equipped,
        Collider,
        ActiveMeleeAttack,
        MainhandOf,
        OffhandOf,
        ChildOf,
    )>();
}
