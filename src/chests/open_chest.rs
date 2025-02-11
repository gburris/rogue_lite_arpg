use bevy::prelude::*;

use crate::{configuration::assets::SpriteAssets, econ::components::GoldDropEvent};

use super::components::{Chest, OpenChest};

pub fn open_chest(
    open_chest_trigger: Trigger<OpenChest>,
    chest_transforms: Query<&Transform, With<Chest>>,
    sprites: Res<SpriteAssets>,
    mut commands: Commands,
) {
    commands
        .entity(open_chest_trigger.chest_entity)
        .insert(Sprite::from_image(sprites.open_chest.clone()))
        .despawn_descendants();
    if let Ok(chest_transform) = chest_transforms.get(open_chest_trigger.chest_entity) {
        commands.trigger(GoldDropEvent {
            amount: 999,
            drop_location: *chest_transform,
        });
    };
}
