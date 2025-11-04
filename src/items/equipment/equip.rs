use avian2d::prelude::Collider;
use bevy::prelude::*;

use crate::{
    character::Character,
    items::{
        ItemOf,
        equipment::{Equipped, Mainhand, MainhandOf, Offhand, OffhandOf},
        melee::ActiveMeleeAttack,
    },
    prelude::AttackState,
};

use super::{EquipmentSlot, Equippable};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_equip).add_observer(on_unequip);
}

#[derive(EntityEvent)]
pub struct Equip {
    #[event_target]
    pub item: Entity,
    pub holder: Entity,
}

fn on_equip(
    equipped: On<Equip>,
    mut commands: Commands,
    mut item_query: Query<(&Equippable, &mut Visibility, Has<ItemOf>)>,
    mut holder_query: Query<(Option<&Mainhand>, Option<&Offhand>), With<Character>>,
) {
    let (equippable, mut visibility, in_inventory) = item_query
        .get_mut(equipped.item)
        .expect("Added Equipped to non-equippable item");

    let (mainhand, offhand) = holder_query
        .get_mut(equipped.holder)
        .expect("Added Equipment to holder that is not a character");

    commands
        .entity(equipped.item)
        .insert((Equipped, ChildOf(equipped.holder)));

    // Without this condition items get "re-added" to item relationship and order gets rearranged
    if !in_inventory {
        commands
            .entity(equipped.item)
            .insert(ItemOf(equipped.holder));
    }

    match equippable.slot {
        EquipmentSlot::Mainhand => {
            if let Some(mainhand) = mainhand {
                commands.trigger(Unequip { entity: mainhand.0 })
            }

            commands
                .entity(equipped.item)
                .insert(MainhandOf(equipped.holder));
        }
        EquipmentSlot::Offhand => {
            if let Some(offhand) = offhand {
                commands.trigger(Unequip { entity: offhand.0 })
            }

            commands
                .entity(equipped.item)
                .insert(OffhandOf(equipped.holder));
        }
    }

    // Make sure item is now visible, since it is hidden while in inventory
    *visibility = Visibility::Visible;
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
