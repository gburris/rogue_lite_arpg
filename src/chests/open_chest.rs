use bevy::prelude::*;

use crate::{chests::Chest, configuration::assets::SpriteAssets, econ::components::GoldDropEvent};

#[derive(Event)]
pub struct OpenChest;

pub fn open_chest(
    open_chest_trigger: Trigger<Interaction>, // does not work
    chest_transforms: Query<&Transform, With<Chest>>,
    sprites: Res<SpriteAssets>,
    mut commands: Commands,
) {
    let chest_entity = open_chest_trigger.entity();

    commands
        .entity(chest_entity)
        .insert(Sprite::from_image(sprites.open_chest.clone()))
        .despawn_descendants();
    if let Ok(chest_transform) = chest_transforms.get(chest_entity) {
        commands.trigger(GoldDropEvent {
            amount: 999,
            drop_location: *chest_transform,
        });
    };
}
