use avian2d::prelude::LockedAxes;
use bevy::prelude::*;

use crate::{
    movement::components::{IsMoving, SimpleMotion},
    npc::components::NPC,
    resources::assets::SpriteAssets,
};

use super::{components::NPCInteractionRadius, NPCMovement};

pub fn npc_setup(mut commands: Commands, sprites: Res<SpriteAssets>) {
    commands
        .spawn((
            NPC,
            SimpleMotion {
                max_speed: 200.0,
                current_speed: 200.0,
                direction: Vec2::new(1.0, 0.0), // Start by moving right
            },
            IsMoving(true),
            NPCMovement::default(),
            LockedAxes::new().lock_rotation(),
            Sprite::from_image(sprites.npc.clone()),
            Transform {
                translation: Vec3::new(-100., -100., 1.0),
                rotation: Quat::IDENTITY,
                scale: Vec3::splat(0.2),
            },
        ))
        .with_child(NPCInteractionRadius);
}
