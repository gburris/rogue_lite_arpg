use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::collections::BTreeMap;

use bevy::{ecs::system::SystemParam, prelude::*};

use crate::{
    prelude::AppState,
    world::{
        MapRng,
        map::{
            blueprint::{MapBlueprint, MapError, Placement, PlacementKind},
            placement::{Anchor, CountRange, PlaceContext, PlaceRule, place},
            recipe::{
                ExitPick, MapRecipe, PlayerStartPick, RecipeCatalog, StructureAnchor,
                StructureEntry, StructureKind, load_recipes, resolved_name,
            },
            structures::{
                Structure, apply_structure, build_empty_square, build_pond, build_rubble,
            },
        },
    },
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
    app.add_systems(Update, cycle_recipe.run_if(in_state(AppState::Playing)));
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

/// Dev only: F6 steps through every recipe in file order with a fresh seed.
#[cfg(feature = "dev")]
fn cycle_recipe(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    blueprint: Single<&MapBlueprint>,
    mut next_map: NextMap,
) -> Result {
    if !keyboard_input.just_pressed(KeyCode::F6) {
        return Ok(());
    }
    let recipe = next_map.catalog.recipe_after(&blueprint.recipe).to_owned();
    next_map.set(MapRequest::Named { recipe, seed: None })
}

/// Shorthand for fallible map generation results.
pub(super) type MapResult<T> = std::result::Result<T, MapError>;

/// Radius around the player start, in tiles, that no enemy may occupy.
pub(crate) const PLAYER_START_CLEARANCE: u32 = 10;

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

/// Generates a map blueprint from a recipe and deterministic seed.
pub(crate) fn build_from_recipe(
    recipe: &MapRecipe,
    seed: u64,
    structures: &BTreeMap<String, Structure>,
) -> std::result::Result<MapBlueprint, MapError> {
    let mut random_generator = ChaCha8Rng::seed_from_u64(seed);
    let size_x = random_generator.random_range(recipe.size_x_range.0..=recipe.size_x_range.1);
    let size_y = random_generator.random_range(recipe.size_y_range.0..=recipe.size_y_range.1);
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
        &mut random_generator,
    )?;
    // Fixed enemies go first so a placed player start can keep its distance from them.
    place_fixed_enemies(
        recipe,
        &mut blueprint,
        &mut placement_context,
        &mut random_generator,
    )?;
    place_player_start(
        recipe,
        &mut blueprint,
        &mut placement_context,
        &mut random_generator,
    )?;
    let player_start = blueprint.player_start().map_err(MapError::Validation)?;
    place_exits(
        recipe,
        &mut blueprint,
        &mut placement_context,
        &mut random_generator,
    )?;
    placement_context
        .keep_clear
        .push((player_start, PLAYER_START_CLEARANCE));
    place_random_enemies(
        recipe,
        seed,
        &mut blueprint,
        &mut placement_context,
        &mut random_generator,
    )?;
    placement_context.keep_clear.pop();
    place_chests(
        recipe,
        seed,
        &mut blueprint,
        &mut placement_context,
        &mut random_generator,
    )?;
    check_start_clearance(&blueprint, player_start)?;
    blueprint.validate().map_err(MapError::Validation)?;
    Ok(blueprint)
}

/// Fails if any enemy sits within `PLAYER_START_CLEARANCE` tiles of the player start.
fn check_start_clearance(blueprint: &MapBlueprint, player_start: IVec2) -> MapResult<()> {
    let limit = (PLAYER_START_CLEARANCE as i32).pow(2);
    if let Some(enemy) = blueprint.placements.iter().find(|placement| {
        matches!(placement.kind, PlacementKind::Enemy(_))
            && placement.tile.distance_squared(player_start) < limit
    }) {
        return Err(format!(
            "enemy at {} is within {PLAYER_START_CLEARANCE} tiles of the player start {player_start}",
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
        let rule = structure_rule(entry);
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

/// Resolves a structure entry's anchor to a placement rule. A `Structure` anchor is
/// measured between centers; with a radius it samples near that point instead.
fn structure_rule(entry: &StructureEntry) -> PlaceRule {
    match &entry.anchor {
        StructureAnchor::Random => PlaceRule::Random {
            count: CountRange(1, 1),
            x_deciles: (1, 9),
            separation: 0,
        },
        StructureAnchor::Center => PlaceRule::Fixed(Anchor::Center),
        StructureAnchor::CenterOffset(offset) => PlaceRule::Fixed(Anchor::CenterOffset(*offset)),
        StructureAnchor::MapMin(position) => PlaceRule::Fixed(Anchor::MapMin(*position)),
        StructureAnchor::Structure { structure, offset } => match entry.radius {
            None => PlaceRule::Fixed(Anchor::Structure {
                name: structure.clone(),
                offset: *offset,
            }),
            Some(radius) => PlaceRule::NearStructure {
                count: CountRange(1, 1),
                structure: structure.clone(),
                offset: *offset,
                radius,
                separation: 0,
            },
        },
    }
}

/// Places a recipe-chosen player start, keeping it clear of enemies already placed.
/// A `Structure` start was already placed by its structure's glyph.
fn place_player_start(
    recipe: &MapRecipe,
    blueprint: &mut MapBlueprint,
    placement_context: &mut PlaceContext,
    random_generator: &mut ChaCha8Rng,
) -> MapResult<()> {
    let PlayerStartPick::Placement(pick) = &recipe.player_start else {
        return Ok(());
    };
    let rule = pick.resolve(random_generator)?;
    let keep_clear_len = placement_context.keep_clear.len();
    placement_context.keep_clear.extend(
        blueprint
            .placements
            .iter()
            .filter(|placement| matches!(placement.kind, PlacementKind::Enemy(_)))
            .map(|placement| (placement.tile, PLAYER_START_CLEARANCE)),
    );
    let placed = place(
        &rule,
        UVec2::ONE,
        &blueprint.tiles,
        placement_context,
        random_generator,
    );
    placement_context.keep_clear.truncate(keep_clear_len);
    let tile = placed?
        .into_iter()
        .next()
        .ok_or_else(|| "failed to place player start".to_owned())?;
    push_point(
        blueprint,
        placement_context,
        tile,
        PlacementKind::PlayerStart,
    );
    Ok(())
}

/// Places exits along the recipe's edge rule, one tile inside the solid border.
/// A `Structure` exit was already placed by its structure's glyph.
fn place_exits(
    recipe: &MapRecipe,
    blueprint: &mut MapBlueprint,
    placement_context: &mut PlaceContext,
    random_generator: &mut ChaCha8Rng,
) -> MapResult<()> {
    let ExitPick::Edge { count, side } = recipe.exits else {
        return Ok(());
    };
    let exits = place(
        &PlaceRule::Edge { count, side },
        UVec2::ONE,
        &blueprint.tiles,
        placement_context,
        random_generator,
    )?;
    for tile in exits {
        push_point(blueprint, placement_context, tile, PlacementKind::Exit);
    }
    Ok(())
}

/// Places fixed enemies; a failed fixed placement aborts the map.
fn place_fixed_enemies(
    recipe: &MapRecipe,
    blueprint: &mut MapBlueprint,
    placement_context: &mut PlaceContext,
    random_generator: &mut ChaCha8Rng,
) -> MapResult<()> {
    for fixed in &recipe.enemies.fixed {
        let positions = place(
            &PlaceRule::Fixed(fixed.anchor.clone()),
            UVec2::ONE,
            &blueprint.tiles,
            placement_context,
            random_generator,
        )?;
        for tile in positions {
            let kind = PlacementKind::Enemy(fixed.pick.resolve(random_generator));
            push_point(blueprint, placement_context, tile, kind);
        }
    }
    Ok(())
}

/// Places the random enemy pack. A short pack only warns, since open ground is never
/// guaranteed.
fn place_random_enemies(
    recipe: &MapRecipe,
    seed: u64,
    blueprint: &mut MapBlueprint,
    placement_context: &mut PlaceContext,
    random_generator: &mut ChaCha8Rng,
) -> MapResult<()> {
    let enemy_count = recipe.enemies.count.roll_size(random_generator);
    let enemy_rule = if let Some(nearby_enemy_placement) = &recipe.enemies.near {
        PlaceRule::NearStructure {
            count: CountRange(enemy_count, enemy_count),
            structure: nearby_enemy_placement.structure.clone(),
            offset: IVec2::ZERO,
            radius: nearby_enemy_placement.radius,
            separation: recipe.enemies.separation,
        }
    } else {
        PlaceRule::Random {
            count: CountRange(enemy_count, enemy_count),
            x_deciles: recipe.enemies.x_deciles,
            separation: recipe.enemies.separation,
        }
    };
    let enemy_positions = place(
        &enemy_rule,
        UVec2::ONE,
        &blueprint.tiles,
        placement_context,
        random_generator,
    )?;
    if enemy_positions.len() < enemy_count as usize {
        warn!(
            "Recipe {} seed {} placed {} of {} enemies",
            recipe.name,
            seed,
            enemy_positions.len(),
            enemy_count
        );
    }
    for tile in enemy_positions {
        let kind = PlacementKind::Enemy(recipe.enemies.pick.resolve(random_generator));
        push_point(blueprint, placement_context, tile, kind);
    }
    Ok(())
}

/// Places the random chest pack, warning when fewer than requested fit.
fn place_chests(
    recipe: &MapRecipe,
    seed: u64,
    blueprint: &mut MapBlueprint,
    placement_context: &mut PlaceContext,
    random_generator: &mut ChaCha8Rng,
) -> MapResult<()> {
    let chest_count = recipe.chests.roll_size(random_generator);
    let chest_positions = place(
        &PlaceRule::Random {
            count: CountRange(chest_count, chest_count),
            x_deciles: (1, 9),
            separation: 0,
        },
        UVec2::ONE,
        &blueprint.tiles,
        placement_context,
        random_generator,
    )?;
    if chest_positions.len() < chest_count as usize {
        warn!(
            "Recipe {} seed {} placed {} of {} chests",
            recipe.name,
            seed,
            chest_positions.len(),
            chest_count
        );
    }
    for tile in chest_positions {
        push_point(blueprint, placement_context, tile, PlacementKind::Chest);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::map::{
        placement::Side,
        recipe::{
            RecipeFile, catalog_from_file, load_recipe_catalog, recipe_options, validate_recipe,
        },
    };

    /// Seeds every recipe is built with; cheap enough to run on every `cargo test`.
    const SEEDS: std::ops::Range<u64> = 0..64;
    /// Seed whose hub layout is pinned in `tests/snapshots/hub_layout.txt`.
    const HUB_SNAPSHOT_SEED: u64 = 0;
    const HUB_SNAPSHOT_PATH: &str = "tests/snapshots/hub_layout.txt";

    fn catalog() -> RecipeCatalog {
        load_recipe_catalog().expect("assets/config/map_recipes.ron should load")
    }

    fn all_recipes(catalog: &RecipeCatalog) -> impl Iterator<Item = &MapRecipe> {
        std::iter::once(&catalog.hub).chain(catalog.recipes.values())
    }

    fn build(catalog: &RecipeCatalog, recipe: &MapRecipe, seed: u64) -> MapBlueprint {
        build_from_recipe(recipe, seed, &catalog.structures)
            .unwrap_or_else(|error| panic!("map {} seed {seed}: {error}", recipe.name))
    }

    fn count(blueprint: &MapBlueprint, wanted: impl Fn(&PlacementKind) -> bool) -> usize {
        blueprint
            .placements
            .iter()
            .filter(|placement| wanted(&placement.kind))
            .count()
    }

    /// Renders tiles top row first, one glyph per tile.
    fn render(blueprint: &MapBlueprint) -> String {
        let size = blueprint.map_size().as_ivec2();
        let mut text = String::new();
        for y in (0..size.y).rev() {
            for x in 0..size.x {
                let tile = blueprint
                    .tiles
                    .get(IVec2::new(x, y))
                    .expect("tile is in bounds");
                text.push(tile.glyph());
            }
            text.push('\n');
        }
        text
    }

    #[test]
    fn every_recipe_builds() {
        let catalog = catalog();
        for recipe in all_recipes(&catalog) {
            validate_recipe(recipe, &catalog.structures)
                .unwrap_or_else(|error| panic!("recipe {} is invalid: {error}", recipe.name));
            for seed in SEEDS {
                let blueprint = build(&catalog, recipe, seed);
                assert!(
                    blueprint == build(&catalog, recipe, seed),
                    "map {} seed {seed} is not deterministic",
                    recipe.name
                );
                assert_eq!(blueprint.recipe, recipe.name);
                assert_eq!(blueprint.seed, seed);
            }
        }
    }

    #[test]
    fn weighted_maps_cover_their_full_size_range() {
        let catalog = catalog();
        let checked = ["LongHallway", "Swamp"];
        let mut widths = BTreeMap::<&str, (u32, u32)>::new();
        let mut root_rng = MapRng::seeded(0);
        for _ in 0..1_000 {
            let (recipe, seed) = catalog
                .resolve(MapRequest::Weighted, &mut root_rng)
                .expect("weighted pick succeeds");
            if !checked.contains(&recipe.name.as_str()) {
                continue;
            }
            let width = build(&catalog, recipe, seed).map_size().x;
            let observed = widths.entry(recipe.name.as_str()).or_insert((u32::MAX, 0));
            *observed = (observed.0.min(width), observed.1.max(width));
        }
        for name in checked {
            let (low, high) = catalog.recipes[name].size_x_range;
            let margin = (high - low) / 10;
            let (seen_low, seen_high) = widths[name];
            assert!(
                seen_low <= low + margin && seen_high >= high - margin,
                "{name} widths {seen_low}..={seen_high} do not cover {low}..={high}"
            );
        }
    }

    #[test]
    fn enemies_keep_clear_of_the_player_start() {
        let catalog = catalog();
        let limit = (PLAYER_START_CLEARANCE as i32).pow(2);
        for recipe in all_recipes(&catalog) {
            for seed in SEEDS {
                let blueprint = build(&catalog, recipe, seed);
                let start = blueprint.player_start().expect("one player start");
                for placement in &blueprint.placements {
                    if matches!(placement.kind, PlacementKind::Enemy(_)) {
                        assert!(
                            placement.tile.distance_squared(start) >= limit,
                            "map {} seed {seed}: enemy at {} is too close to the start {start}",
                            recipe.name,
                            placement.tile
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn hub_has_exactly_one_exit() {
        let catalog = catalog();
        for seed in SEEDS {
            let blueprint = build(&catalog, &catalog.hub, seed);
            let exits = count(&blueprint, |kind| matches!(kind, PlacementKind::Exit));
            assert_eq!(exits, 1, "hub seed {seed} has {exits} exits");
        }
    }

    #[test]
    fn recipes_need_exactly_one_exit_source() {
        let catalog = catalog();
        let mut hub = catalog.hub.clone();
        hub.exits = ExitPick::Edge {
            count: CountRange(1, 1),
            side: Side::Right,
        };
        assert!(validate_recipe(&hub, &catalog.structures).is_err());

        let mut swamp = catalog.recipes["Swamp"].clone();
        swamp.exits = ExitPick::Structure;
        assert!(validate_recipe(&swamp, &catalog.structures).is_err());
    }

    #[test]
    fn arenas_are_named_only_and_need_no_weight() {
        let text = std::fs::read_to_string("assets/config/map_recipes.ron")
            .expect("recipe file is readable");
        let text = text.replace(
            "arenas: [],",
            "arenas: [(name: \"TestArena\", size_x_range: (40, 40), size_y_range: (40, 40))],",
        );
        assert!(
            text.contains("TestArena"),
            "recipe file has no empty arenas list"
        );
        let file: RecipeFile = recipe_options()
            .from_str(&text)
            .expect("edited recipe file parses");
        let catalog = catalog_from_file(file).expect("catalog with an arena loads");

        assert!(catalog.named("TestArena").is_ok());
        assert!(
            !catalog
                .weighted_names
                .iter()
                .any(|name| name == "TestArena")
        );
        assert_eq!(catalog.order.last().map(String::as_str), Some("TestArena"));
        let mut root_rng = MapRng::seeded(0);
        for _ in 0..500 {
            let (recipe, _) = catalog
                .resolve(MapRequest::Weighted, &mut root_rng)
                .expect("weighted pick succeeds");
            assert_ne!(recipe.name, "TestArena");
        }
    }

    #[test]
    fn named_requests_replay_the_same_map() {
        let catalog = catalog();
        let mut root_rng = MapRng::seeded(5);
        let request = MapRequest::Named {
            recipe: "FireShrine".to_owned(),
            seed: Some(42),
        };
        let (recipe, seed) = catalog.resolve(request, &mut root_rng).unwrap();
        assert_eq!((recipe.name.as_str(), seed), ("FireShrine", 42));

        let fresh = MapRequest::Named {
            recipe: "FireShrine".to_owned(),
            seed: None,
        };
        let (_, first) = catalog.resolve(fresh.clone(), &mut root_rng).unwrap();
        let (_, second) = catalog.resolve(fresh, &mut root_rng).unwrap();
        assert_ne!(first, second, "seed: None draws a fresh seed each time");
    }

    #[test]
    fn hub_layout_matches_snapshot() {
        let catalog = catalog();
        let actual = render(&build(&catalog, &catalog.hub, HUB_SNAPSHOT_SEED));
        if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
            std::fs::write(HUB_SNAPSHOT_PATH, &actual).expect("snapshot is writable");
            return;
        }
        let expected = std::fs::read_to_string(HUB_SNAPSHOT_PATH)
            .expect("snapshot is readable")
            .replace("\r\n", "\n");
        assert!(
            expected == actual,
            "hub layout changed; if intended, rerun with UPDATE_SNAPSHOTS=1.\nactual:\n{actual}"
        );
    }
}
