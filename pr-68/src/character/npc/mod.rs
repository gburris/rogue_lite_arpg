use bevy::prelude::*;
use bevy_behave::prelude::*;

mod interaction;

use crate::{
    character::{
        behavior::{Idle, Retreat},
        physical_collider,
        player::interact::{InteractionEvent, InteractionZone},
        Character,
    },
    combat::{damage::hurtbox, Health},
    configuration::{
        assets::{Shadows, SpriteAssets, SpriteSheetLayouts},
        shadow, GameCollisionLayer, CHARACTER_FEET_POS_OFFSET,
    },
    items::{equipment::Equipped, inventory::Inventory, spawn_axe, spawn_ice_staff, spawn_sword},
    map::NPCSpawnEvent,
    prelude::*,
};

use super::behavior::{Anchor, Wander};

pub struct NPCPlugin;

impl Plugin for NPCPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(spawn_npcs);
    }
}

#[derive(Component)]
#[require(Character)]
pub struct NPC;

#[derive(Debug, Clone, Component, Copy)]
pub enum NPCType {
    Helper,
    Shopkeeper,
    StatTrainer,
}

impl NPCType {
    pub fn spawn_weapon(
        &self,
        commands: &mut Commands,
        sprites: &SpriteAssets,
        atlases: &SpriteSheetLayouts,
    ) -> Entity {
        match self {
            NPCType::Helper => spawn_ice_staff(commands, sprites, atlases),
            NPCType::Shopkeeper => spawn_axe(commands, sprites),
            NPCType::StatTrainer => spawn_sword(commands, sprites),
        }
    }

    pub fn get_sprite_sheet(&self, sprites: &SpriteAssets) -> Handle<Image> {
        match self {
            NPCType::Helper => sprites.game_guide_sprite_sheet.clone(),
            NPCType::Shopkeeper => sprites.shop_keeper_sprite_sheet.clone(),
            NPCType::StatTrainer => sprites.stat_trainer_sprite_sheet.clone(),
        }
    }

    pub fn get_interaction_observer(&self) -> fn(Trigger<InteractionEvent>, Commands) {
        match self {
            NPCType::Helper => interaction::on_game_guide_start,
            NPCType::Shopkeeper => interaction::on_shop_keeper_store_open,
            NPCType::StatTrainer => interaction::on_stat_trainer_store_open,
        }
    }
}

const TILE_SIZE: f32 = 32.0;
const WANDER_RADIUS: f32 = 2.5 * TILE_SIZE;

fn spawn_npcs(
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

fn spawn_npc(
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

    let npc_behavior = behave! {
        Behave::Forever => {
            Behave::Fallback => {
                Behave::Sequence => {
                    Behave::spawn_named("Idle", Idle::default().timer_range(1.0..4.0)),
                    Behave::spawn_named("Wander", Wander::builder().timer_range(1.0..2.5)),
                },
                Behave::spawn_named("Retreat", Retreat),
            }
        }
    };

    let npc = commands
        .spawn((
            NPC,
            Anchor::new(spawn_position, WANDER_RADIUS),
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
            children![
                shadow(&shadows, CHARACTER_FEET_POS_OFFSET - 4.0),
                (
                    InteractionZone::NPC,
                    Transform::from_xyz(0.0, CHARACTER_FEET_POS_OFFSET, 0.0),
                ),
                hurtbox(Vec2::new(26.0, 42.0), GameCollisionLayer::AllyHurtBox),
                physical_collider(),
                BehaveTree::new(npc_behavior.clone()),
            ],
        ))
        .observe(on_player_interaction)
        .add_child(mainhand)
        .id();

    commands.entity(mainhand).insert(Equipped::new(npc));
}
