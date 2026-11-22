use bevy::prelude::*;
use serde::Deserialize;

use super::brain::{
    ACTION_BASELINE_IDLE, Behavior, Brain, BrainDefinition, RETURN_TO_HOME, WANDER,
};
use super::motion::{Home, HomeArea, MotionSources, ReturnToHome, Wander};
use super::{Character, physical_collider};
use crate::interval::Interval;
use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_spawn_npcs);
}

#[derive(Component, Default, Clone)]
#[require(Character, CombatSide = CombatSide::Ally, DespawnOnExit::<AppState>(AppState::Playing))]
pub struct Npc;

/// Recipes name these directly, e.g. `(what: Npc(Shopkeeper), ...)` in `map_recipes.ron`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum NpcKind {
    Helper,
    Shopkeeper,
    StatTrainer,
}

static VILLAGER_BEHAVIORS: &[Behavior] = &[WANDER, ACTION_BASELINE_IDLE, RETURN_TO_HOME];

/// Wanders around its home area and never requests equipment.
pub(super) static VILLAGER: BrainDefinition = BrainDefinition {
    behaviors: VILLAGER_BEHAVIORS,
};

const WANDER_RADIUS: f32 = 2.5 * TILE_SIZE as f32;

#[derive(Event)]
pub struct SpawnNpcs(pub Vec<NpcSpawn>);

#[derive(Clone, Copy, Debug)]
pub struct NpcSpawn {
    pub position: Vec2,
    pub kind: NpcKind,
}

fn on_spawn_npcs(
    npc_spawn_trigger: On<SpawnNpcs>,
    mut commands: Commands,
    sprites: Res<SpriteAssets>,
    sprite_layouts: Res<SpriteSheetLayouts>,
    shadows: Res<Shadows>,
) {
    for NpcSpawn {
        position: spawn_position,
        kind,
    } in &npc_spawn_trigger.0
    {
        spawn_npc(
            &mut commands,
            *kind,
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
    let base = base_npc(npc_kind, spawn_position, shadows, &VILLAGER);

    match npc_kind {
        NpcKind::Helper => spawn_npc_with_equipment(
            commands,
            bsn! {
                @{base}
                @villager_sprite(sprites.game_guide_sprite_sheet.clone(), sprite_layouts)
                on(on_game_guide_start)
            },
            ice_staff(sprites, sprite_layouts),
        ),
        NpcKind::Shopkeeper => spawn_npc_with_equipment(
            commands,
            bsn! {
                @{base}
                @villager_sprite(sprites.shop_keeper_sprite_sheet.clone(), sprite_layouts)
                on(on_shop_keeper_store_open)
            },
            axe(sprites),
        ),
        NpcKind::StatTrainer => spawn_npc_with_equipment(
            commands,
            bsn! {
                @{base}
                @villager_sprite(sprites.stat_trainer_sprite_sheet.clone(), sprite_layouts)
                on(on_stat_trainer_store_open)
            },
            sword(sprites),
        ),
    }
}

fn spawn_npc_with_equipment(commands: &mut Commands, npc: impl Scene, mainhand: impl Scene) {
    let npc = commands.spawn_scene(npc).id();

    let mainhand = commands.spawn_scene(mainhand).id();

    commands.trigger(Equip {
        item: mainhand,
        holder: npc,
    });
}

fn base_npc(
    npc_kind: NpcKind,
    spawn_position: Vec2,
    shadows: &Shadows,
    brain: &'static BrainDefinition,
) -> impl Scene {
    let name = Name::new(format!("{npc_kind:?}"));
    bsn! {
        Npc
        template(move |_| Ok(name.clone()))
        template(move |_| Ok(Brain::new(brain)))
        MotionSources [
            // Villagers stand briefly between strolls.
            ~Wander::new(Interval::new(1.0, 4.0), Interval::new(1.0, 2.5))
            --
            ReturnToHome
        ]
        Home [
            Name::new("Home")
            ~HomeArea::new(WANDER_RADIUS, TILE_SIZE as f32 / 2.0)
            Transform::from_translation({spawn_position.extend(0.0)})
        ]
        ~Kinematics::new(100.0)
        ~Health::new(1000.0)
        Transform::from_translation({spawn_position.extend(ZLayer::OnGround.z())})
        Children [
            @shadow(shadows, CHARACTER_FEET_POS_OFFSET - 4.0)
            --
            ~InteractionZone::NPC
            Transform::from_xyz(0.0, CHARACTER_FEET_POS_OFFSET, 0.0)
            --
            @hurtbox(Vec2::new(26.0, 42.0), GameCollisionLayer::AllyHurtBox)
            --
            @physical_collider()
        ]
    }
}

/// A villager drawn from the shared enemy sheet layout.
fn villager_sprite(image: Handle<Image>, sprite_layouts: &SpriteSheetLayouts) -> impl Scene {
    let sprite = Sprite::from_atlas_image(
        image,
        TextureAtlas {
            layout: sprite_layouts.enemy.clone(),
            ..default()
        },
    );
    bsn! {
        template(move |_| Ok(sprite.clone()))
    }
}

fn on_shop_keeper_store_open(
    _: On<PlayerInteraction>,
    mut next_menu_state: ResMut<NextState<Menu>>,
) {
    next_menu_state.set(Menu::Inventory);
}

fn on_stat_trainer_store_open(
    _: On<PlayerInteraction>,
    mut next_menu_state: ResMut<NextState<Menu>>,
) {
    next_menu_state.set(Menu::StatsShop);
}

fn on_game_guide_start(_: On<PlayerInteraction>) {
    info!("The Helper has nothing to say yet");
}
