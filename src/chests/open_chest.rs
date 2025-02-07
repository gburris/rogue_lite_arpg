use bevy::prelude::*;

use crate::{
    configuration::assets::SpriteAssets,
    items::{inventory::inventory::Inventory, spawn_health_potion},
};

use super::components::OpenChest;

pub fn open_chest(
    open_chest_trigger: Trigger<OpenChest>,
    mut player_inventory: Single<&mut Inventory>,
    sprites: Res<SpriteAssets>,
    mut commands: Commands,
) {
    commands
        .entity(open_chest_trigger.chest_entity)
        .insert(Sprite::from_image(sprites.open_chest.clone()))
        .despawn_descendants();

    player_inventory
        .add_item(spawn_health_potion(&mut commands))
        .expect("Chest tried adding health potion but inventory was full");
}
