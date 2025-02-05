use avian2d::prelude::LockedAxes;
use bevy::prelude::*;

use crate::{
    configuration::assets::SpriteAssets, map::systems::hub::spawn_hub_entities::NPCSpawnEvent,
    movement::components::SimpleMotion, npc::components::NPC,
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
            SimpleMotion::new(250.0),
            NPCMovement::default(),
            LockedAxes::new().lock_rotation(),
            Sprite::from_image(sprites.npc.clone()),
            Transform {
                translation: npc_spawn_trigger.position,
                rotation: Quat::IDENTITY,
                scale: Vec3::splat(0.02),
            },
        ))
        .with_child(NPCInteractionRadius);
}
