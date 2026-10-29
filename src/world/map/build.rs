use std::collections::BTreeMap;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::prelude::AppState;
use crate::world::MapRng;
use crate::world::map::blueprint::{MapBlueprint, MapError, Placement, PlacementKind};
use crate::world::map::placement::{PlaceContext, place};
use crate::world::map::recipe::{
    MapRecipe, RecipeCatalog, StructureKind, load_recipes, resolved_name,
};
use crate::world::map::structures::{
    Structure, apply_structure, build_empty_square, build_pond, build_rubble,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, load_recipes);
    #[cfg(all(feature = "dev_native", not(target_arch = "wasm32")))]
    app.add_systems(
        Update,
        (
            reload_recipe_catalog.run_if(bevy::time::common_conditions::on_timer(
                std::time::Duration::from_millis(500),
            )),
            rebuild_current_map.run_if(resource_exists::<RebuildCurrentMap>),
        )
            .chain()
            .run_if(in_state(AppState::Playing)),
    );
    #[cfg(feature = "dev")]
    app.add_systems(Update, dev_map_keys.run_if(in_state(AppState::Playing)));
}

/// System parameters for requesting the next map.
#[derive(SystemParam)]
pub struct NextMap<'w, 's> {
    commands: Commands<'w, 's>,
    previous_maps: Query<'w, 's, Entity, With<MapBlueprint>>,
    next_state: ResMut<'w, NextState<AppState>>,
    catalog: Res<'w, RecipeCatalog>,
    root_rng: ResMut<'w, MapRng>,
}

impl NextMap<'_, '_> {
    /// Builds and schedules the requested map, replacing any current map.
    pub fn set(&mut self, request: MapRequest) -> Result<()> {
        let (recipe, seed) = self.catalog.resolve(request, &mut self.root_rng)?;
        info!("Generating map {} with seed {seed}", recipe.name);
        let blueprint = build_from_recipe(recipe, seed, &self.catalog.structures)
            .map_err(|error| format!("map {} seed {seed}: {error}", recipe.name))?;

        // Despawn every map, so a stray duplicate cleans itself up instead of wedging `set`.
        for previous_map in &self.previous_maps {
            self.commands.entity(previous_map).despawn();
        }
        self.commands.spawn(blueprint);
        self.next_state.set(AppState::SpawnMap);
        Ok(())
    }
}

/// Selects which recipe and seed to use when generating a map.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MapRequest {
    Hub,
    /// A dungeon picked by weight; arenas are never picked.
    Weighted,
    /// A recipe by name. `seed: None` draws a fresh seed from the root generator.
    Named {
        recipe: String,
        seed: Option<u64>,
    },
}

/// Marks that the recipe catalog was just reloaded and the current map should be rebuilt.
#[cfg(all(feature = "dev_native", not(target_arch = "wasm32")))]
#[derive(Resource)]
struct RebuildCurrentMap;

/// Dev only: polls the recipe file's timestamp and swaps in the catalog when it changes.
#[cfg(all(feature = "dev_native", not(target_arch = "wasm32")))]
fn reload_recipe_catalog(
    mut commands: Commands,
    mut observed: Local<Option<std::time::SystemTime>>,
) {
    let Ok(modified) =
        std::fs::metadata("assets/config/map_recipes.ron").and_then(|metadata| metadata.modified())
    else {
        return;
    };
    if observed.is_none() {
        *observed = Some(modified);
        return;
    }
    if observed.as_ref() == Some(&modified) {
        return;
    }
    *observed = Some(modified);
    match super::recipe::load_recipe_catalog() {
        Ok(catalog) => {
            commands.insert_resource(catalog);
            commands.insert_resource(RebuildCurrentMap);
        }
        Err(error) => error!("Keeping previous recipe catalog after reload error: {error}"),
    }
}

/// Dev only: rebuilds the current map from the reloaded catalog with the same replay key.
#[cfg(all(feature = "dev_native", not(target_arch = "wasm32")))]
fn rebuild_current_map(
    mut commands: Commands,
    blueprint: Single<&MapBlueprint>,
    mut next_map: NextMap,
) {
    commands.remove_resource::<RebuildCurrentMap>();
    let request = MapRequest::Named {
        recipe: blueprint.recipe.clone(),
        seed: Some(blueprint.seed),
    };
    if let Err(error) = next_map.set(request) {
        error!("Keeping the current map after reload: {error}");
    }
}

/// Arena that F1 opens in dev builds.
#[cfg(feature = "dev")]
const AI_TEST_ARENA: &str = "AiTestArena";

/// Dev only: F1 opens the AI test arena; F6 steps through every recipe in file order.
/// Both use a fresh seed.
#[cfg(feature = "dev")]
fn dev_map_keys(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    blueprint: Single<&MapBlueprint>,
    mut next_map: NextMap,
) -> Result {
    let recipe = if keyboard_input.just_pressed(KeyCode::F1) {
        AI_TEST_ARENA.to_owned()
    } else if keyboard_input.just_pressed(KeyCode::F6) {
        next_map.catalog.recipe_after(&blueprint.recipe).to_owned()
    } else {
        return Ok(());
    };
    next_map.set(MapRequest::Named { recipe, seed: None })
}

/// Shorthand for fallible map generation results.
pub(super) type MapResult<T> = std::result::Result<T, MapError>;

/// Records a point placement on the blueprint and in the placement context.
fn push_point(
    blueprint: &mut MapBlueprint,
    placement_context: &mut PlaceContext,
    tile: IVec2,
    kind: PlacementKind,
) {
    placement_context.points.push(tile);
    blueprint.placements.push(Placement { tile, kind });
}

/// A build stage. Each stage draws from its own `ChaCha8` stream of the map seed, so adding or
/// changing spawns never shifts the terrain, and changing one spawn kind leaves the others.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Stage {
    Size,
    Structures,
    /// Spawns at fixed anchors, of any kind, in file order.
    Fixed,
    PlayerStart,
    Exits,
    Enemies,
    Chests,
    Npcs,
}

impl Stage {
    /// Stages that place spawn entries, in build order.
    const SPAWNS: [Self; 6] = [
        Self::Fixed,
        Self::PlayerStart,
        Self::Exits,
        Self::Enemies,
        Self::Chests,
        Self::Npcs,
    ];

    /// This stage's generator: the map seed on the stage's own `ChaCha8` stream.
    fn random_generator(self, seed: u64) -> ChaCha8Rng {
        let mut random_generator = ChaCha8Rng::seed_from_u64(seed);
        random_generator.set_stream(self as u64);
        random_generator
    }
}

/// Generates a map blueprint from a recipe and deterministic seed.
pub(crate) fn build_from_recipe(
    recipe: &MapRecipe,
    seed: u64,
    structures: &BTreeMap<String, Structure>,
) -> std::result::Result<MapBlueprint, MapError> {
    let mut size_generator = Stage::Size.random_generator(seed);
    let size_x = size_generator.random_range(recipe.size_x_range.0..=recipe.size_x_range.1);
    let size_y = size_generator.random_range(recipe.size_y_range.0..=recipe.size_y_range.1);
    let mut blueprint = MapBlueprint::new(
        UVec2::new(size_x, size_y),
        recipe.floor,
        recipe.name.clone(),
        seed,
    );
    let mut placement_context = PlaceContext::new(blueprint.map_size());
    place_structures(
        recipe,
        seed,
        structures,
        &mut blueprint,
        &mut placement_context,
        &mut Stage::Structures.random_generator(seed),
    )?;
    for stage in Stage::SPAWNS {
        // Fixed spawns come first so a sampled player start can keep clear of fixed
        // enemies; sampled enemies then keep clear of the start.
        let keep_clear_len = placement_context.keep_clear.len();
        match stage {
            Stage::PlayerStart => placement_context.keep_clear.extend(
                blueprint
                    .placements
                    .iter()
                    .filter(|placement| matches!(placement.kind, PlacementKind::Enemy(_)))
                    .map(|placement| (placement.tile, recipe.start_clearance)),
            ),
            Stage::Enemies => {
                let player_start = blueprint.player_start().map_err(MapError::Validation)?;
                placement_context
                    .keep_clear
                    .push((player_start, recipe.start_clearance));
            }
            _ => {}
        }
        place_spawns(recipe, seed, stage, &mut blueprint, &mut placement_context)?;
        placement_context.keep_clear.truncate(keep_clear_len);
    }
    let player_start = blueprint.player_start().map_err(MapError::Validation)?;
    check_start_clearance(&blueprint, player_start, recipe.start_clearance)?;
    blueprint.validate().map_err(MapError::Validation)?;
    Ok(blueprint)
}

/// Places the recipe's spawn entries that belong to `stage`, in file order.
/// A fixed anchor that cannot be placed aborts the map; a sampled spawn that places fewer
/// than it rolled only warns, since open ground is never guaranteed. A missing player
/// start or exit is still caught by validation.
fn place_spawns(
    recipe: &MapRecipe,
    seed: u64,
    stage: Stage,
    blueprint: &mut MapBlueprint,
    placement_context: &mut PlaceContext,
) -> MapResult<()> {
    let mut random_generator = stage.random_generator(seed);
    let entries = recipe.spawns.iter().filter(|spawn| {
        if spawn.at.is_fixed() {
            stage == Stage::Fixed
        } else {
            spawn.what.stage() == stage
        }
    });
    for spawn in entries {
        let count = spawn.count.roll_size(&mut random_generator);
        let rule = spawn
            .at
            .rule(count, spawn.separation, &mut random_generator)?;
        let tiles = place(
            &rule,
            UVec2::ONE,
            &blueprint.tiles,
            placement_context,
            &mut random_generator,
        )?;
        if tiles.len() < count as usize {
            warn!(
                "Recipe {} seed {seed} placed {} of {count} {:?}",
                recipe.name,
                tiles.len(),
                spawn.what
            );
        }
        for tile in tiles {
            let kind = spawn.what.placement(&mut random_generator);
            push_point(blueprint, placement_context, tile, kind);
        }
    }
    Ok(())
}

/// Fails if any enemy sits within `clearance` tiles of the player start.
fn check_start_clearance(
    blueprint: &MapBlueprint,
    player_start: IVec2,
    clearance: u32,
) -> MapResult<()> {
    let limit = (clearance as i32).pow(2);
    if let Some(enemy) = blueprint.placements.iter().find(|placement| {
        matches!(placement.kind, PlacementKind::Enemy(_))
            && placement.tile.distance_squared(player_start) < limit
    }) {
        return Err(format!(
            "enemy at {} is within {clearance} tiles of the player start {player_start}",
            enemy.tile
        )
        .into());
    }
    Ok(())
}

/// Places every structure in recipe order, recording each center by resolved name.
/// Unplaceable structures warn and record nothing; later anchors to them fail loudly.
fn place_structures(
    recipe: &MapRecipe,
    seed: u64,
    structures: &BTreeMap<String, Structure>,
    blueprint: &mut MapBlueprint,
    placement_context: &mut PlaceContext,
    random_generator: &mut ChaCha8Rng,
) -> MapResult<()> {
    for (index, entry) in recipe.structures.iter().enumerate() {
        let resolved_name = resolved_name(entry.kind.label(), index, &entry.name);
        let rule = entry.at.rule(1, 0, random_generator)?;
        let placed_center = match &entry.kind {
            StructureKind::EmptySquare { size } => {
                build_empty_square(blueprint, placement_context, &rule, *size, random_generator)?
            }
            StructureKind::Pond { size, shape } => build_pond(
                blueprint,
                placement_context,
                &rule,
                *size,
                *shape,
                random_generator,
            )?,
            StructureKind::Rubble { size } => {
                build_rubble(blueprint, placement_context, &rule, *size, random_generator)?
            }
            StructureKind::Structure(kind) => {
                let structure = structures
                    .get(kind)
                    .ok_or_else(|| format!("structure {kind:?} is missing from the catalog"))?;
                let footprint = structure.tiles.map_size();
                let placed_origin = place(
                    &rule,
                    footprint,
                    &blueprint.tiles,
                    placement_context,
                    random_generator,
                )?
                .into_iter()
                .next();
                if let Some(origin) = placed_origin {
                    apply_structure(blueprint, structure, origin).map_err(|error| {
                        format!("failed to place structure {index} ({kind:?}): {error}")
                    })?;
                    placement_context.points.extend(
                        structure
                            .placements
                            .iter()
                            .map(|placement| origin + placement.tile),
                    );
                    Some(origin + footprint.as_ivec2() / 2)
                } else {
                    None
                }
            }
        };
        if let Some(center) = placed_center {
            placement_context.record_structure_center(resolved_name, center);
        } else {
            warn!(
                "Recipe {} seed {} could not place structure {index} ({})",
                recipe.name,
                seed,
                entry.kind.label()
            );
        }
    }
    Ok(())
}
