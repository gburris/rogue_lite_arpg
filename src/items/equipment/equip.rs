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

fn on_equip(
    equipped: On<Add, Equipped>,
    mut commands: Commands,
    mut item_query: Query<(&ItemOf, &Equippable, &mut Visibility), With<Equipped>>,
    mut holder_query: Query<(Option<&Mainhand>, Option<&Offhand>), With<Character>>,
) {
    let (item_of, equippable, mut visibility) = item_query
        .get_mut(equipped.entity)
        .expect("Added Equipped to non-equippable item");

    let holder_entity = item_of.0;

    let (mainhand, offhand) = holder_query
        .get_mut(holder_entity)
        .expect("Added Equipment to holder that is not a character");

    commands
        .entity(equipped.entity)
        .insert(ChildOf(holder_entity));

    match equippable.slot {
        EquipmentSlot::Mainhand => {
            if let Some(mainhand) = mainhand {
                commands.trigger(Unequip { entity: mainhand.0 })
            }

            commands
                .entity(equipped.entity)
                .insert(MainhandOf(holder_entity));
        }
        EquipmentSlot::Offhand => {
            if let Some(offhand) = offhand {
                commands.trigger(Unequip { entity: offhand.0 })
            }

            commands
                .entity(equipped.entity)
                .insert(OffhandOf(holder_entity));
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
