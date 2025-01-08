use bevy::prelude::*;

use crate::{npc::components::NPC, resources::assets::SpriteAssets};

use super::{components::NPCInteractionRadius, NPCMovement};

pub fn npc_setup(mut commands: Commands, sprites: Res<SpriteAssets>) {
    commands
        .spawn((
            NPC,
            NPCMovement::default(),
            Sprite::from_image(sprites.npc.clone()),
            Transform {
                translation: Vec3::new(-100., -100., 1.0),
                rotation: Quat::IDENTITY,
                scale: Vec3::splat(0.2),
            },
        ))
        .with_child(NPCInteractionRadius);
}
