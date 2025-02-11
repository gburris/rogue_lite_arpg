use bevy::prelude::*;

use crate::{
    configuration::assets::SpriteAssets,
    items::{inventory::inventory::Inventory, spawn_health_potion},
};

use super::components::OpenChest;

pub fn open_chest(
    open_chest_trigger: Trigger<OpenChest>,
    player: Single<(Entity, &mut Inventory)>,
    sprites: Res<SpriteAssets>,
    mut commands: Commands,
) {
    let (player, mut inventory) = player.into_inner();

    commands
        .entity(open_chest_trigger.chest_entity)
        .insert(Sprite::from_image(sprites.open_chest.clone()))
        .despawn_descendants();

    let health_potion = spawn_health_potion(&mut commands, &sprites);

    inventory
        .add_item(health_potion)
        .expect("Chest tried adding health potion but inventory was full");

    commands.entity(player).add_child(health_potion);
}
