use avian2d::prelude::LockedAxes;
use bevy::prelude::*;

use crate::{
    ai::{
        state::{ActionState, FacingDirection},
        SimpleMotion,
    },
    animation::{AnimationTimer, DefaultAnimationConfig},
    combat::Health,
    configuration::assets::{SpriteAssets, SpriteSheetLayouts},
    items::{equipment::Equipped, inventory::Inventory},
    map::NPCSpawnEvent,
    npc::components::NPC,
    player::interact::InteractionZone,
};

use super::components::NPCType;

pub fn spawn_npcs(
    npc_spawn_trigger: Trigger<NPCSpawnEvent>,
    mut commands: Commands,
    animation_config: Res<DefaultAnimationConfig>,
    sprites: Res<SpriteAssets>,
    atlases: Res<SpriteSheetLayouts>,
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
            &animation_config,
            &sprites,
            &atlases,
        );
    }
}

pub fn spawn_npc(
    commands: &mut Commands,
    npc_type: NPCType,
    spawn_position: Vec3,
    animation_config: &Res<DefaultAnimationConfig>,
    sprites: &Res<SpriteAssets>,
    atlases: &Res<SpriteSheetLayouts>,
) {
    let mainhand = npc_type.spawn_weapon(commands, sprites, atlases);
    let sprite_sheet_to_use = npc_type.get_sprite_sheet(sprites);
    let on_player_interaction = npc_type.get_interaction_observer();
    let sprite = Sprite::from_atlas_image(
        sprite_sheet_to_use,
        TextureAtlas {
            layout: atlases.enemy_atlas_layout.clone(),
            index: animation_config
                .get_indices(ActionState::Idle, FacingDirection::Down)
                .first,
        },
    );
    let npc = commands
        .spawn((
            NPC,
            SimpleMotion::new(100.0),
            Health::new(1000.0),
            LockedAxes::new().lock_rotation(),
            ActionState::Idle,
            npc_type,
            Inventory::default(),
            (
                Transform::from_translation(spawn_position),
                animation_config.get_indices(ActionState::Idle, FacingDirection::Down),
                AnimationTimer(
                    animation_config.get_timer(ActionState::Idle, FacingDirection::Down),
                ),
                sprite,
                FacingDirection::Down,
            ),
        ))
        .observe(on_player_interaction)
        .with_child(InteractionZone::NPC)
        .add_child(mainhand)
        .id();

    commands.entity(mainhand).insert(Equipped::new(npc));
}
