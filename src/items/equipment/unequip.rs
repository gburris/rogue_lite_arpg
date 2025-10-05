use avian2d::prelude::Collider;
use bevy::prelude::*;

use super::Equippable;
use crate::{
    combat::melee::ActiveMeleeAttack,
    items::equipment::{EquipmentOf, MainhandOf, OffhandOf},
    prelude::*,
};

pub fn on_item_unequipped(
    trigger: On<Remove, EquipmentOf>,
    mut commands: Commands,
    mut item_query: Query<(&EquipmentOf, &mut Visibility), With<Equippable>>,
    mut holder_query: Query<&ActionState>,
) {
    let item_entity = trigger.target();

    let Ok((equipment_of, mut visibility)) = item_query.get_mut(item_entity) else {
        info!("Equipment was despawned prior to unequip");
        return;
    };

    let Ok(action_state) = holder_query.get_mut(equipment_of.0) else {
        info!("Holder was despawned prior to unequip");
        return;
    };

    if *action_state == ActionState::Defeated {
        info!("Holder was in the death animation prior to unequip");
        return;
    }

    *visibility = Visibility::Hidden;
    commands
        .entity(item_entity)
        .remove::<(Collider, ActiveMeleeAttack, MainhandOf, OffhandOf)>();
}

/// Hold up invariant, if you are no longer Mainhand or Offhand, you ain't equipped!!
pub fn on_equip_slot_removed(
    trigger: On<Remove, (MainhandOf, OffhandOf)>, // this is an OR on these
    mut commands: Commands,
) {
    commands.entity(trigger.target()).remove::<EquipmentOf>();
}
