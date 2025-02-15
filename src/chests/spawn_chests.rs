use bevy::prelude::*;

use crate::{
    configuration::assets::SpriteAssets, map::systems::hub::spawn_hub_entities::ChestSpawnEvent,
};

use super::components::{Chest, ChestInteractionRadius};

pub fn spawn_chests(
    chest_spawn_trigger: Trigger<ChestSpawnEvent>,
    mut commands: Commands,
    sprites: Res<SpriteAssets>,
) {
    let chest_spawn_positions = chest_spawn_trigger.0.clone();
    for spawn_position in chest_spawn_positions {
        spawn_chest(&mut commands, &sprites, spawn_position);
    }
}

fn spawn_chest(commands: &mut Commands, sprites: &Res<'_, SpriteAssets>, spawn_position: Vec3) {
    commands
        .spawn((
            Chest,
            Sprite::from_image(sprites.closed_chest.clone()),
            Transform {
                translation: spawn_position,
                ..default()
            },
        ))
        .with_child(ChestInteractionRadius);
}
