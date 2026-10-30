use bevy::prelude::*;
use bevy::ui_widgets::observe;

use super::{Npc, NpcKind, VILLAGER, interaction};
use crate::character::brain::{Brain, BrainDefinition, HomeArea, Interval, Wander};
use crate::character::physical_collider;
use crate::prelude::*;

const WANDER_RADIUS: f32 = 2.5 * TILE_SIZE as f32;

#[derive(Event)]
pub struct SpawnNpcs(pub Vec<NpcSpawnData>);

#[derive(Debug, Clone, Copy)]
pub struct NpcSpawnData {
    pub position: Vec2,
    pub npc_kind: NpcKind,
}

pub(super) fn spawn_npcs(
    npc_spawn_trigger: On<SpawnNpcs>,
    mut commands: Commands,
    sprites: Res<SpriteAssets>,
    sprite_layouts: Res<SpriteSheetLayouts>,
    shadows: Res<Shadows>,
) {
    for NpcSpawnData {
        position: spawn_position,
        npc_kind,
    } in &npc_spawn_trigger.0
    {
        spawn_npc(
            &mut commands,
            *npc_kind,
            *spawn_position,
            &sprites,
            &sprite_layouts,
            &shadows,
        );
    }
}

fn spawn_npc(
    commands: &mut Commands,
    npc_kind: NpcKind,
    spawn_position: Vec2,
    sprites: &SpriteAssets,
    sprite_layouts: &SpriteSheetLayouts,
    shadows: &Shadows,
) {
    let brain = &VILLAGER;

    match npc_kind {
        NpcKind::Helper => spawn_npc_with_equipment(
            commands,
            (
                base_npc(spawn_position, shadows, brain),
                helper(sprites, sprite_layouts),
            ),
            ice_staff(sprites, sprite_layouts),
        ),
        NpcKind::Shopkeeper => spawn_npc_with_equipment(
            commands,
            (
                base_npc(spawn_position, shadows, brain),
                shopkeeper(sprites, sprite_layouts),
            ),
            axe(sprites),
        ),
        NpcKind::StatTrainer => spawn_npc_with_equipment(
            commands,
            (
                base_npc(spawn_position, shadows, brain),
                stat_trainer(sprites, sprite_layouts),
            ),
            sword(sprites),
        ),
    }
}

fn spawn_npc_with_equipment(commands: &mut Commands, npc: impl Bundle, mainhand: impl Bundle) {
    let npc = commands.spawn(npc).id();

    let mainhand = commands.spawn(mainhand).id();

    commands.trigger(Equip {
        item: mainhand,
        holder: npc,
    });
}

fn base_npc(
    spawn_position: Vec2,
    shadows: &Shadows,
    brain: &'static BrainDefinition,
) -> impl Bundle {
    (
        Npc,
        Brain::new(brain),
        // Villagers stand briefly between strolls.
        Wander::new(Interval::new(1.0, 4.0), Interval::new(1.0, 2.5)),
        HomeArea::new(spawn_position, WANDER_RADIUS, TILE_SIZE as f32 / 2.0),
        Kinematics::new(100.0),
        Health::new(1000.0),
        Transform::from_translation(spawn_position.extend(ZLayer::OnGround.z())),
        children![
            shadow(shadows, CHARACTER_FEET_POS_OFFSET - 4.0),
            (
                InteractionZone::NPC,
                Transform::from_xyz(0.0, CHARACTER_FEET_POS_OFFSET, 0.0),
            ),
            hurtbox(Vec2::new(26.0, 42.0), GameCollisionLayer::AllyHurtBox),
            physical_collider(),
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
    )
}
