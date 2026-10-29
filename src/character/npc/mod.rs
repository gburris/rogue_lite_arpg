use bevy::{prelude::*, ui_widgets::observe};

mod interaction;

use super::brain::considerations::{ACTION_BASELINE_IDLE, RETURN_TO_HOME, WANDER};
use super::brain::{Behavior, Brain, BrainDefinition, HomeArea, Interval, Wander};
use crate::{
    character::{Character, physical_collider},
    prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(spawn_npcs);
}

#[derive(Event)]
pub struct SpawnNpcs(pub Vec<NpcSpawnData>);

#[derive(Debug, Clone)]
pub struct NpcSpawnData {
    pub position: Vec2,
    pub npc_kind: NpcKind,
}

#[derive(Component)]
#[require(Character, CombatSide = CombatSide::Ally, DespawnOnExit::<AppState>(AppState::Playing))]
pub struct NPC;

/// Recipes name these directly, e.g. `(what: Npc(Shopkeeper), ...)` in `map_recipes.ron`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, serde::Deserialize)]
pub enum NpcKind {
    Helper,
    Shopkeeper,
    StatTrainer,
}

static VILLAGER_BEHAVIORS: &[Behavior] = &[WANDER, ACTION_BASELINE_IDLE, RETURN_TO_HOME];

/// Wanders around its home area and never requests equipment.
pub(in crate::character) static VILLAGER: BrainDefinition = BrainDefinition {
    behaviors: VILLAGER_BEHAVIORS,
};

impl NpcKind {
    pub fn brain(&self) -> &'static BrainDefinition {
        &VILLAGER
    }
}

const WANDER_RADIUS: f32 = 2.5 * TILE_SIZE as f32;

fn spawn_npcs(
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
    let brain = npc_kind.brain();

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
        NPC,
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
