//Query for my chest
//"Open it" by changing it's sprite to be open_chest.png

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
        .insert(Sprite::from_image(sprites.open_chest.clone()));
    commands
        .entity(open_chest_trigger.chest_entity)
        .despawn_descendants();
    let _ = player_inventory.add_item(spawn_health_potion(&mut commands, &sprites));
}
