use avian2d::prelude::LockedAxes;
use bevy::prelude::*;

use crate::{
    animation::{AnimationTimer, DefaultAnimationConfig, DefaultAnimations, MovementDirection},
    combat::{attributes::Health, components::ActionState},
    configuration::assets::{SpriteAssets, SpriteSheetLayouts},
    items::{
        equipment::EquipmentSlots, spawn_axe, spawn_ice_staff, spawn_random_mainhand_weapon,
        spawn_sword,
    },
    map::systems::hub::spawn_hub_entities::NPCSpawnEvent,
    movement::components::SimpleMotion,
    npc::components::NPC,
};

use super::components::NPCInteractionRadius;

#[derive(Debug, Clone, Copy)]
pub enum NPCType {
    Helper,
    Shopkeeper,
    StatTrainer,
}

pub fn spawn_npcs(
    npc_spawn_trigger: Trigger<NPCSpawnEvent>,
    mut commands: Commands,
    animation_config: Res<DefaultAnimationConfig>,
    sprites: Res<SpriteAssets>,
    atlases: Res<SpriteSheetLayouts>,
) {
    let npc_spawn_positions = npc_spawn_trigger.0.clone();
    let mut npc_counter = 0;

    for spawn_position in npc_spawn_positions {
        let npc_type = match npc_counter {
            0 => NPCType::Helper,
            1 => NPCType::Shopkeeper,
            2 => NPCType::StatTrainer,
            _ => {
                warn!("No NPC type defined for index: {}", npc_counter);
                NPCType::Helper // Default type
            }
        };

        spawn_npc(
            &mut commands,
            npc_type,
            spawn_position,
            &animation_config,
            &sprites,
            &atlases,
        );
        npc_counter += 1;
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
    let mainhand_to_weild = match npc_type {
        NPCType::Helper => spawn_ice_staff(commands, &sprites, &atlases),
        NPCType::Shopkeeper => spawn_axe(commands, &sprites),
        NPCType::StatTrainer => spawn_sword(commands, &sprites),
    };
    let sprite_sheet_to_use = match npc_type {
        NPCType::Helper => sprites.game_guide_sprite_sheet.clone(),
        NPCType::Shopkeeper => sprites.shop_keeper_sprite_sheet.clone(),
        NPCType::StatTrainer => sprites.stat_trainer_sprite_sheet.clone(),
    };
    let sprite = Sprite::from_atlas_image(
        sprite_sheet_to_use,
        TextureAtlas {
            layout: atlases.enemy_atlas_layout.clone(),
            index: animation_config
                .get_indices(&DefaultAnimations::IdleDown)
                .first,
        },
    );
    commands
        .spawn((
            NPC,
            SimpleMotion::new(100.0),
            Health::new(1000.0),
            LockedAxes::new().lock_rotation(),
            ActionState::None,
            EquipmentSlots {
                mainhand: Some(mainhand_to_weild),
                head: None,
            },
            (
                Transform::from_translation(spawn_position),
                animation_config.get_indices(&DefaultAnimations::IdleDown),
                AnimationTimer(animation_config.get_timer(&DefaultAnimations::IdleDown)),
                sprite,
                DefaultAnimations::IdleDown,
                MovementDirection::None,
            ),
        ))
        .with_child(NPCInteractionRadius);
}
