use ::bevy::prelude::*;

use crate::{
    configuration::YSort,
    despawn::components::LiveDuration,
    items::{inventory::inventory::Inventory, Lootable},
    player::{interact::InteractionEvent, Player},
};

pub fn on_lootable_item_interaction(
    trigger: Trigger<InteractionEvent>,
    mut commands: Commands,
    player: Single<(Entity, &mut Inventory), With<Player>>,
) {
    let item_entity = trigger.target();

    let (player_entity, mut inventory) = player.into_inner();

    if inventory.add_item(item_entity).is_ok() {
        commands.entity(player_entity).add_child(item_entity);

        // Make sure item doesn't despawn and is hidden (since its in inventory)
        commands
            .entity(item_entity)
            .remove::<Lootable>()
            .remove::<LiveDuration>()
            .remove::<YSort>()
            .insert(Visibility::Hidden);

        // Remove interaction zone once itme is picked up
        commands.entity(trigger.interaction_zone_entity).despawn();
    } else {
        warn!("Inventory is full!")
    }
}
