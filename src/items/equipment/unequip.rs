use avian2d::prelude::Collider;
use bevy::prelude::*;

use crate::{
    combat::melee::ActiveMeleeAttack,
    items::{
        ItemOf,
        equipment::{Equipped, MainhandOf, OffhandOf},
    },
    prelude::*,
};

#[derive(EntityEvent)]
pub struct Unequip {
    pub entity: Entity,
}

pub fn on_item_unequipped(
    trigger: On<Unequip>,
    mut commands: Commands,
    mut item_query: Query<(&ItemOf, &mut Visibility), With<Equipped>>,
    mut holder_query: Query<&mut ActionState>,
) {
    let item_entity = trigger.event().entity;

    let Ok((equipment_of, mut visibility)) = item_query.get_mut(item_entity) else {
        info!("Equipment was despawned prior to unequip");
        return;
    };

    let Ok(mut action_state) = holder_query.get_mut(equipment_of.0) else {
        info!("Holder was despawned prior to unequip");
        return;
    };

    if *action_state == ActionState::Defeated {
        info!("Holder was in the death animation prior to unequip");
        return;
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

    // TODO: We need to re-evaluate action state this has "issues"
    if *action_state == ActionState::Attacking {
        *action_state = ActionState::Movement;
    }
}
