use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    ai::SimpleMotion,
    combat::{damage::HurtBox, Health},
    configuration::{
        assets::{Shadows, SpriteAssets, SpriteSheetLayouts},
        spawn_shadow, GameCollisionLayer, CHARACTER_FEET_POS_OFFSET,
    },
    items::{equipment::Equipped, inventory::Inventory},
    map::NPCSpawnEvent,
    npc::components::NPC,
    player::interact::InteractionZone,
};

use super::components::NPCType;

pub fn spawn_npcs(
    npc_spawn_trigger: Trigger<NPCSpawnEvent>,
    mut commands: Commands,
    sprites: Res<SpriteAssets>,
    atlases: Res<SpriteSheetLayouts>,
    shadows: Res<Shadows>,
) {
    // Define the NPC types we want to spawn in order
    let npc_types = [NPCType::Helper, NPCType::Shopkeeper, NPCType::StatTrainer];
    let npc_spawn_positions = npc_spawn_trigger.0.clone();

    // Zip the positions with NPC types and spawn them
    for (spawn_position, &npc_type) in npc_spawn_positions.iter().zip(npc_types.iter()) {
        spawn_npc(
            &mut commands,
            npc_type,
            *spawn_position,
            &sprites,
            &atlases,
            &shadows,
        );
    }
}

pub fn spawn_npc(
    commands: &mut Commands,
    npc_type: NPCType,
    spawn_position: Vec2,
    sprites: &SpriteAssets,
    atlases: &SpriteSheetLayouts,
    shadows: &Shadows,
) {
    let mainhand = npc_type.spawn_weapon(commands, sprites, atlases);
    let sprite_sheet_to_use = npc_type.get_sprite_sheet(sprites);
    let on_player_interaction = npc_type.get_interaction_observer();

    let npc = commands
        .spawn((
            NPC,
            SimpleMotion::new(100.0),
            Health::new(1000.0),
            npc_type,
            Inventory::default(),
            Transform::from_translation(spawn_position.extend(0.0)),
            Sprite::from_atlas_image(
                sprite_sheet_to_use,
                TextureAtlas {
                    layout: atlases.enemy_atlas_layout.clone(),
                    ..default()
                },
            ),
        ))
        .observe(on_player_interaction)
        .with_children(|spawner| {
            spawn_shadow(spawner, &shadows, CHARACTER_FEET_POS_OFFSET - 4.0);

            spawner.spawn((
                InteractionZone::NPC,
                Transform::from_xyz(0.0, CHARACTER_FEET_POS_OFFSET, 0.0),
            ));

            spawner.spawn((
                HurtBox,
                Collider::rectangle(26.0, 42.0),
                Transform::from_xyz(0.0, -8.0, 0.0),
                Sensor,
                CollisionLayers::new(
                    [GameCollisionLayer::AllyHurtBox],
                    [GameCollisionLayer::HitBox],
                ),
            ));

            spawner.spawn((
                Transform::from_xyz(0.0, CHARACTER_FEET_POS_OFFSET, 0.0),
                Collider::circle(10.0),
                CollisionLayers::new(
                    [GameCollisionLayer::Grounded],
                    [
                        GameCollisionLayer::Grounded,
                        GameCollisionLayer::HighObstacle,
                        GameCollisionLayer::LowObstacle,
                    ],
                ),
            ));
        })
        .add_child(mainhand)
        .id();

    commands.entity(mainhand).insert(Equipped::new(npc));
}
