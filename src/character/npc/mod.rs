use bevy::{prelude::*, ui_widgets::observe};
use bevy_behave::prelude::*;

mod interaction;

use crate::{
    character::{
        Character,
        behavior::{Idle, Retreat},
        physical_collider,
        player::interact::InteractionZone,
    },
    combat::{Health, damage::hurtbox},
    configuration::{
        CHARACTER_FEET_POS_OFFSET, GameCollisionLayer,
        assets::{Shadows, SpriteAssets, SpriteSheetLayouts},
        shadow,
    },
    items::{Items, axe, equipment::Equipped, ice_staff, sword},
    prelude::*,
};

use super::behavior::{Anchor, Wander};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(spawn_npcs);
}

#[derive(Event)]
pub struct SpawnNpcs(pub Vec<Vec2>);

#[derive(Component)]
#[require(
    Character,     
    DespawnOnExit::<AppState>(AppState::Playing),
)]
pub struct NPC;

#[derive(Component, Clone, Copy, Debug)]
enum NPCType {
    Helper,
    Shopkeeper,
    StatTrainer,
}

const TILE_SIZE: f32 = 32.0;
const WANDER_RADIUS: f32 = 2.5 * TILE_SIZE;

fn spawn_npcs(
    npc_spawn_trigger: On<SpawnNpcs>,
    mut commands: Commands,
    sprites: Res<SpriteAssets>,
    sprite_layouts: Res<SpriteSheetLayouts>,
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
            &sprite_layouts,
            &shadows,
        );
    }
}

fn spawn_npc(
    commands: &mut Commands,
    npc_type: NPCType,
    spawn_position: Vec2,
    sprites: &SpriteAssets,
    sprite_layouts: &SpriteSheetLayouts,
    shadows: &Shadows,
) {
    match npc_type {
        NPCType::Helper => commands.spawn((
            npc_type,
            base_npc(spawn_position, shadows),
            helper(sprites, sprite_layouts),
        )),
        NPCType::Shopkeeper => commands.spawn((
            npc_type,
            base_npc(spawn_position, shadows),
            shopkeeper(sprites, sprite_layouts),
        )),
        NPCType::StatTrainer => commands.spawn((
            npc_type,
            base_npc(spawn_position, shadows),
            stat_trainer(sprites, sprite_layouts),
        )),
    };
}

fn base_npc(spawn_position: Vec2, shadows: &Shadows) -> impl Bundle {
    (
        NPC,
        Anchor::new(spawn_position, WANDER_RADIUS),
        SimpleMotion::new(100.0),
        Health::new(1000.0),
        Transform::from_translation(spawn_position.extend(0.0)),
        children![
            shadow(shadows, CHARACTER_FEET_POS_OFFSET - 4.0),
            (
                InteractionZone::NPC,
                Transform::from_xyz(0.0, CHARACTER_FEET_POS_OFFSET, 0.0),
            ),
            hurtbox(Vec2::new(26.0, 42.0), GameCollisionLayer::AllyHurtBox),
            physical_collider(),
            BehaveTree::new(wander_and_retreat_behavior()),
        ],
    )
}

fn helper(sprites: &SpriteAssets, sprite_layouts: &SpriteSheetLayouts) -> impl Bundle {
    (
        Sprite::from_atlas_image(
            sprites.game_guide_sprite_sheet.clone(),
            TextureAtlas {
                layout: sprite_layouts.enemy_atlas_layout.clone(),
                ..default()
            },
        ),
        observe(interaction::on_game_guide_start),
        related!(Items[(Equipped, ice_staff(sprites, sprite_layouts))]),
    )
}

fn shopkeeper(sprites: &SpriteAssets, sprite_layouts: &SpriteSheetLayouts) -> impl Bundle {
    (
        Sprite::from_atlas_image(
            sprites.shop_keeper_sprite_sheet.clone(),
            TextureAtlas {
                layout: sprite_layouts.enemy_atlas_layout.clone(),
                ..default()
            },
        ),
        observe(interaction::on_shop_keeper_store_open),
        related!(Items[(Equipped, axe(sprites))]),
    )
}

fn stat_trainer(sprites: &SpriteAssets, sprite_layouts: &SpriteSheetLayouts) -> impl Bundle {
    (
        Sprite::from_atlas_image(
            sprites.stat_trainer_sprite_sheet.clone(),
            TextureAtlas {
                layout: sprite_layouts.enemy_atlas_layout.clone(),
                ..default()
            },
        ),
        observe(interaction::on_stat_trainer_store_open),
        related!(Items[(Equipped, sword(sprites))]),
    )
}

fn wander_and_retreat_behavior() -> Tree<Behave> {
    behave! {
        Behave::Forever => {
            Behave::Fallback => {
                Behave::Sequence => {
                    Behave::spawn_named("Idle", Idle::default().timer_range(1.0..4.0)),
                    Behave::spawn_named("Wander", Wander::builder().timer_range(1.0..2.5)),
                },
                Behave::spawn_named("Retreat", Retreat),
            }
        }
    }
}
