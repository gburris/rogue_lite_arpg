use avian2d::prelude::LockedAxes;
use bevy::prelude::*;

use crate::{
    configuration::assets::SpriteAssets,
    map::systems::hub::spawn_hub_entities::NPCSpawnEvent,
    movement::components::{IsMoving, SimpleMotion},
    npc::components::NPC,
};

use super::{components::NPCInteractionRadius, NPCMovement};

pub fn npc_setup(
    npc_spawn_trigger: Trigger<NPCSpawnEvent>,
    mut commands: Commands,
    sprites: Res<SpriteAssets>,
) {
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
                translation: npc_spawn_trigger.position,
                rotation: Quat::IDENTITY,
                scale: Vec3::splat(0.2),
            },
        ))
        .with_child(NPCInteractionRadius);
}
