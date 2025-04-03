use avian2d::prelude::Collider;
use bevy::prelude::*;

use super::{equippable::Equipped, Equippable};
use crate::{
    ai::state::ActionState, combat::melee::ActiveMeleeAttack, items::inventory::Inventory,
};

pub fn on_item_unequipped(
    trigger: Trigger<OnRemove, Equipped>,
    mut commands: Commands,
    mut item_query: Query<(&Equippable, &Parent, &mut Visibility)>,
    mut holder_query: Query<(&ActionState, &mut Inventory)>,
) {
    let item_entity = trigger.entity();

    let Ok((equippable, holder, mut visibility)) = item_query.get_mut(item_entity) else {
        info!("Item was despawned prior to unequip");
        return;
    };

    let Ok((action_state, mut inventory)) = holder_query.get_mut(holder.get()) else {
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
        .remove::<Collider>()
        .remove::<ActiveMeleeAttack>();

    inventory.unequip(item_entity, equippable.slot);

    info!("Item Unequipped!");
}
