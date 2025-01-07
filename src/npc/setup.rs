use avian2d::prelude::{Collider, CollidingEntities, Friction, Restitution, RigidBody};
use bevy::prelude::*;

use crate::{npc::components::NPC, resources::assets::SpriteAssets};

use super::{components::NPCInteractionRadius, NPCMovement};
//This function spawns all the NPCs.
//This assumes we are only going to have them on the overworld.

//Setup an NPC.
//He has a child entity with a collider that is a circle.
//This represents the interaction radius
pub fn npc_setup(mut commands: Commands, sprites: Res<SpriteAssets>) {
    commands
        .spawn((
            NPC,
            NPCMovement::default(),
            Sprite::from_image(sprites.npc.clone()),
            RigidBody::Static,
            Collider::rectangle(100.0, 100.0),
            Transform {
                translation: Vec3::new(-100., -100., 1.0),
                rotation: Quat::IDENTITY,
                scale: Vec3::splat(0.2),
            },
        ))
        .with_child((NPCInteractionRadius, CollidingEntities::default()));
}
