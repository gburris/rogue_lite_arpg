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
