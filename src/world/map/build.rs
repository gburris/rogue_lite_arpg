use rand::{
    Rng, RngExt, SeedableRng,
    distr::{Distribution, weighted::WeightedIndex},
};
use rand_chacha::ChaCha8Rng;
use ron::{Options, extensions::Extensions};
use serde::Deserialize;
use std::{collections::BTreeMap, io::BufReader};

use bevy::{ecs::system::SystemParam, prelude::*};

use crate::{
    prelude::{AppState, EnemyKind},
    world::{
        MapRng,
        map::{
            blueprint::{MapBlueprint, MapError, Placement, PlacementKind},
            placement::{Anchor, CountRange, PlaceContext, PlaceRule, Side, place},
            spawn::MAX_MAP_TILES,
            structures::{
                BuiltinFeature, FeatureShape, FeatureSize, Structure, apply_structure,
                build_empty_square, build_pond, build_rubble, structure_from_glyphs,
            },
            tiles::TileKind,
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

/// Loaded map recipes and the structures available to place in them.
#[derive(Resource, Debug)]
pub struct RecipeCatalog {
    hub: MapRecipe,
    recipes: BTreeMap<String, MapRecipe>,
    structures: BTreeMap<String, Structure>,
    weighted_names: Vec<String>,
    /// Every recipe name in file order: hub, dungeons, then arenas.
    #[cfg_attr(not(feature = "dev"), allow(dead_code))]
    order: Vec<String>,
}

impl RecipeCatalog {
    fn hub(&self) -> &MapRecipe {
        &self.hub
    }

    /// Chooses the recipe and build seed for a request.
    /// The weighted pick draws from the root generator itself, so it never shares a draw
    /// with the build seed's own generator (which rolls the map size first).
    fn resolve(&self, request: MapRequest, root_rng: &mut MapRng) -> Result<(&MapRecipe, u64)> {
        Ok(match request {
            MapRequest::Hub => (self.hub(), root_rng.next_map_seed()),
            MapRequest::Weighted => {
                let recipe = self.weighted(root_rng.rng())?;
                (recipe, root_rng.next_map_seed())
            }
            MapRequest::Named { recipe, seed } => (
                self.named(&recipe)?,
                seed.unwrap_or_else(|| root_rng.next_map_seed()),
            ),
        })
    }

    /// Returns the recipe after `current` in file order (hub, dungeons, arenas), wrapping.
    #[cfg(feature = "dev")]
    fn recipe_after(&self, current: &str) -> &str {
        let index = self
            .order
            .iter()
            .position(|name| name == current)
            .map_or(0, |index| (index + 1) % self.order.len());
        &self.order[index]
    }

    fn named(&self, name: &str) -> Result<&MapRecipe> {
        if name == self.hub.name {
            return Ok(&self.hub);
        }
        self.recipes
            .get(name)
            .ok_or_else(|| format!("unknown map recipe: {name}").into())
    }

    fn weighted(&self, random_generator: &mut impl Rng) -> Result<&MapRecipe> {
        let weights = self
            .weighted_names
            .iter()
            .map(|name| self.recipes[name].weight)
            .collect::<Vec<_>>();
        let index = WeightedIndex::new(weights)?;
        let name = &self.weighted_names[index.sample(random_generator)];
        self.recipes
            .get(name)
            .ok_or_else(|| "weighted recipe catalog is empty".into())
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct RecipeFile {
    defaults: RecipeDefaults,
    structures: BTreeMap<String, StructureDefinition>,
    hub: RecipeEntry,
    #[serde(default)]
    dungeons: Vec<RecipeEntry>,
    #[serde(default)]
    arenas: Vec<RecipeEntry>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct StructureDefinition {
    floor: TileKind,
    glyphs: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
struct RecipeDefaults {
    size_x_range: (u32, u32),
    size_y_range: (u32, u32),
    floor: TileKind,
    structures: Vec<StructureEntry>,
    player_start: PlayerStartPick,
    exits: ExitPick,
    enemies: EnemyPlacement,
    chests: CountRange,
}

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
struct RecipeEntry {
    name: Option<String>,
    weight: Option<u32>,
    size_x_range: Option<(u32, u32)>,
    size_y_range: Option<(u32, u32)>,
    floor: Option<TileKind>,
    structures: Option<Vec<StructureEntry>>,
    player_start: Option<PlayerStartPick>,
    exits: Option<ExitPick>,
    enemies: Option<EnemyPlacement>,
    chests: Option<CountRange>,
}

/// Resolved map generation settings loaded from a recipe definition.
#[derive(Debug, Clone)]
pub(crate) struct MapRecipe {
    name: String,
    weight: u32,
    size_x_range: (u32, u32),
    size_y_range: (u32, u32),
    floor: TileKind,
    structures: Vec<StructureEntry>,
    player_start: PlayerStartPick,
    exits: ExitPick,
    enemies: EnemyPlacement,
    chests: CountRange,
}

impl RecipeEntry {
    fn resolve(self, name: String, weight: u32, defaults: &RecipeDefaults) -> MapRecipe {
        MapRecipe {
            name,
            weight,
            size_x_range: self.size_x_range.unwrap_or(defaults.size_x_range),
            size_y_range: self.size_y_range.unwrap_or(defaults.size_y_range),
            floor: self.floor.unwrap_or(defaults.floor),
            structures: self
                .structures
                .unwrap_or_else(|| defaults.structures.clone()),
            player_start: self
                .player_start
                .unwrap_or_else(|| defaults.player_start.clone()),
            exits: self.exits.unwrap_or_else(|| defaults.exits.clone()),
            enemies: self.enemies.unwrap_or_else(|| defaults.enemies.clone()),
            chests: self.chests.unwrap_or(defaults.chests),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
struct StructureEntry {
    kind: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    anchor: StructureAnchor,
    radius: Option<u32>,
    size: Option<FeatureSize>,
    #[serde(default)]
    shape: FeatureShape,
}

#[derive(Deserialize, Debug, Clone, Default)]
enum StructureAnchor {
    #[default]
    Random,
    Center,
    CenterOffset(IVec2),
    MapMin(IVec2),
    Structure {
        structure: String,
        offset: IVec2,
    },
}
fn resolved_name(kind: &str, index: usize, explicit: &str) -> String {
    if explicit.is_empty() {
        format!("{kind}_{index}")
    } else {
        explicit.to_owned()
    }
}

#[derive(Deserialize, Debug, Clone)]
enum PlayerStartPick {
    Structure,
    Placement(PlacementPick),
}

/// Where a recipe's exits come from: `X` glyphs in its structures, or one map edge.
#[derive(Deserialize, Debug, Clone)]
enum ExitPick {
    Structure,
    Edge { count: CountRange, side: Side },
}

#[derive(Deserialize, Debug, Clone)]
enum PlacementPick {
    Fixed {
        anchor: Anchor,
    },
    Random {
        count: CountRange,
        x_deciles: (u32, u32),
        separation: u32,
    },
    Edge {
        count: CountRange,
        side: Side,
    },
    Choose(Vec<PlacementPick>),
}

impl PlacementPick {
    fn resolve(&self, random_generator: &mut ChaCha8Rng) -> MapResult<PlaceRule> {
        match self {
            Self::Fixed { anchor } => Ok(PlaceRule::Fixed(anchor.clone())),
            Self::Random {
                count,
                x_deciles,
                separation,
            } => Ok(PlaceRule::Random {
                count: *count,
                x_deciles: *x_deciles,
                separation: *separation,
            }),
            Self::Edge { count, side } => Ok(PlaceRule::Edge {
                count: *count,
                side: *side,
            }),
            Self::Choose(picks) => {
                if picks.is_empty() {
                    return Err("placement choice list is empty".to_owned().into());
                }
                Ok(picks[random_generator.random_range(0..picks.len())]
                    .resolve(random_generator)?)
            }
        }
    }

    fn valid_player_start(&self) -> bool {
        match self {
            Self::Random {
                count, x_deciles, ..
            } => count.0 == 1 && count.1 == 1 && x_deciles.0 < x_deciles.1 && x_deciles.1 <= 10,
            Self::Edge { count, .. } => count.0 == 1 && count.1 == 1,
            Self::Choose(picks) => !picks.is_empty() && picks.iter().all(Self::valid_player_start),
            Self::Fixed { .. } => false,
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
struct EnemyPlacement {
    count: CountRange,
    #[serde(default)]
    pick: EnemyPick,
    #[serde(default = "default_enemy_x_deciles")]
    x_deciles: (u32, u32),
    #[serde(default)]
    separation: u32,
    #[serde(default)]
    fixed: Vec<FixedEnemyPlacement>,
    near: Option<EnemyNearPlacement>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
struct FixedEnemyPlacement {
    pick: EnemyPick,
    anchor: Anchor,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
struct EnemyNearPlacement {
    structure: String,
    radius: u32,
}

fn default_enemy_x_deciles() -> (u32, u32) {
    (2, 8)
}

#[derive(Deserialize, Debug, Clone, Copy, Default)]
enum EnemyPick {
    #[default]
    Warrior,
    IceMage,
    FireMage,
    Random,
}

impl EnemyPick {
    fn resolve(self, random_generator: &mut ChaCha8Rng) -> EnemyKind {
        match self {
            Self::Warrior => EnemyKind::Warrior,
            Self::IceMage => EnemyKind::IceMage,
            Self::FireMage => EnemyKind::FireMage,
            Self::Random => match random_generator.random_range(0..3) {
                0 => EnemyKind::Warrior,
                1 => EnemyKind::IceMage,
                _ => EnemyKind::FireMage,
            },
        }
    }
}

fn load_recipes(mut commands: Commands) -> Result {
    commands.insert_resource(load_recipe_catalog()?);
    Ok(())
}

fn load_recipe_catalog() -> Result<RecipeCatalog> {
    catalog_from_file(load_recipe_file()?)
}

/// Resolves, validates and smoke-builds every recipe in a parsed recipe file.
fn catalog_from_file(file: RecipeFile) -> Result<RecipeCatalog> {
    let structures = file
        .structures
        .into_iter()
        .map(|(name, definition)| {
            structure_from_glyphs(definition.floor, &definition.glyphs)
                .map(|structure| (name, structure))
                .map_err(|error| format!("invalid structure glyphs: {error:?}"))
        })
        .collect::<std::result::Result<BTreeMap<_, _>, _>>()?;
    if let Some(reserved) = structures
        .keys()
        .find(|name| BuiltinFeature::from_kind(name).is_some())
    {
        return Err(
            format!("structure name {reserved:?} is reserved for a builtin feature").into(),
        );
    }
    let mut names = std::collections::HashSet::new();
    let hub_name = file
        .hub
        .name
        .clone()
        .unwrap_or_else(|| "HubTown".to_owned());
    if !names.insert(hub_name.clone()) {
        return Err(format!("duplicate recipe name: {hub_name}").into());
    }
    let hub = file.hub.resolve(hub_name, 0, &file.defaults);
    validate_recipe(&hub, &structures)?;
    for seed in 0..=8 {
        build_from_recipe(&hub, seed, &structures)
            .map_err(|error| format!("map {} seed {seed}: {error}", hub.name))?;
    }
    let mut recipes = BTreeMap::new();
    let mut weighted_names = Vec::new();
    let mut order = vec![hub.name.clone()];
    let dungeons = file.dungeons.into_iter().map(|entry| (entry, true));
    let arenas = file.arenas.into_iter().map(|entry| (entry, false));
    for (entry, is_dungeon) in dungeons.chain(arenas) {
        let name = entry.name.clone().ok_or("recipe name is required")?;
        if !names.insert(name.clone()) {
            return Err(format!("duplicate recipe name: {name}").into());
        }
        // Only dungeons join the portal rotation; arenas are reached by name, so their
        // weight is optional and unused.
        let weight = if is_dungeon {
            let weight = entry
                .weight
                .ok_or_else(|| format!("dungeon {name} requires a weight"))?;
            if weight == 0 {
                return Err(format!("dungeon {name} weight must be positive").into());
            }
            weighted_names.push(name.clone());
            weight
        } else {
            entry.weight.unwrap_or(0)
        };
        let recipe = entry.resolve(name.clone(), weight, &file.defaults);
        validate_recipe(&recipe, &structures)?;
        for seed in 0..=8 {
            build_from_recipe(&recipe, seed, &structures)
                .map_err(|error| format!("map {name} seed {seed}: {error}"))?;
        }
        order.push(name.clone());
        recipes.insert(name, recipe);
    }
    if weighted_names.is_empty() {
        return Err("at least one dungeon recipe is required".into());
    }
    Ok(RecipeCatalog {
        hub,
        recipes,
        structures,
        weighted_names,
        order,
    })
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
    match load_recipe_catalog() {
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

#[cfg(not(target_arch = "wasm32"))]
use std::fs::File;

#[cfg(not(target_arch = "wasm32"))]
fn fetch_recipe_data() -> Result<File> {
    File::open("assets/config/map_recipes.ron")
        .map_err(|error| format!("failed to open assets/config/map_recipes.ron: {error}").into())
}

#[cfg(target_arch = "wasm32")]
fn fetch_recipe_data() -> &'static [u8] {
    include_bytes!("../../../assets/config/map_recipes.ron")
}

fn load_recipe_file() -> Result<RecipeFile> {
    #[cfg(not(target_arch = "wasm32"))]
    let reader = BufReader::new(fetch_recipe_data()?);
    #[cfg(target_arch = "wasm32")]
    let reader = BufReader::new(fetch_recipe_data());
    recipe_options()
        .from_reader(reader)
        .map_err(|error| format!("failed to parse assets/config/map_recipes.ron: {error}").into())
}

fn recipe_options() -> Options {
    Options::default().with_default_extension(Extensions::IMPLICIT_SOME)
}

#[allow(clippy::too_many_lines)]
fn validate_recipe(recipe: &MapRecipe, structures: &BTreeMap<String, Structure>) -> Result<()> {
    for (name, range) in [
        ("size_x", recipe.size_x_range),
        ("size_y", recipe.size_y_range),
    ] {
        if range.0 == 0 || range.0 > range.1 || range.1 > MAX_MAP_TILES {
            return Err(format!(
                "recipe {} has invalid {name} range (each side must be 1..={MAX_MAP_TILES})",
                recipe.name
            )
            .into());
        }
    }
    let (enemy_min, enemy_max) = (recipe.enemies.count.0, recipe.enemies.count.1);
    if enemy_min > enemy_max
        || enemy_max > MAX_MAP_TILES
        || recipe.enemies.fixed.len() > MAX_MAP_TILES as usize
        || recipe.chests.0 > recipe.chests.1
        || recipe.chests.1 > MAX_MAP_TILES
    {
        return Err(format!("recipe {} has invalid placement count range", recipe.name).into());
    }
    if recipe.enemies.near.is_none()
        && enemy_min > 0
        && (recipe.enemies.x_deciles.0 >= recipe.enemies.x_deciles.1
            || recipe.enemies.x_deciles.1 > 10)
    {
        return Err(format!("recipe {} has invalid enemy x-decile range", recipe.name).into());
    }
    if let ExitPick::Edge { count, .. } = &recipe.exits
        && (count.0 == 0 || count.0 > count.1 || count.1 > 2)
    {
        return Err(format!("recipe {} has invalid exit count range", recipe.name).into());
    }
    let structure_exits = structures_place(recipe, structures, |kind| {
        matches!(kind, PlacementKind::Exit)
    });
    if matches!(
        (structure_exits, &recipe.exits),
        (true, ExitPick::Edge { .. }) | (false, ExitPick::Structure)
    ) {
        return Err(format!(
            "recipe {} must define exactly one exit source, structure glyphs or an edge",
            recipe.name
        )
        .into());
    }
    let mut earlier_names = std::collections::HashSet::new();
    for (index, structure) in recipe.structures.iter().enumerate() {
        if !structures.contains_key(&structure.kind)
            && BuiltinFeature::from_kind(&structure.kind).is_none()
        {
            return Err(format!(
                "recipe {} references unknown structure {}",
                recipe.name, structure.kind
            )
            .into());
        }
        if let StructureAnchor::Structure {
            structure: target, ..
        } = &structure.anchor
        {
            if !earlier_names.contains(target) {
                return Err(format!(
                    "recipe {} structure {index} anchors to missing or later structure {target}",
                    recipe.name
                )
                .into());
            }
        } else if structure.radius.is_some() {
            return Err(format!(
                "recipe {} structure {index} radius requires a Structure anchor",
                recipe.name
            )
            .into());
        }
        if structure
            .radius
            .is_some_and(|radius| radius > MAX_MAP_TILES)
        {
            return Err(format!(
                "recipe {} structure {index} radius exceeds the cap",
                recipe.name
            )
            .into());
        }
        if let Some(size) = structure.size {
            if BuiltinFeature::from_kind(&structure.kind).is_none() {
                return Err(format!(
                    "recipe {} structure {index} size requires a builtin feature kind",
                    recipe.name
                )
                .into());
            }
            let (minimum, maximum) = match size {
                FeatureSize::Exact(dimensions) => (dimensions, dimensions),
                FeatureSize::Ranged(minimum, maximum) => (minimum, maximum),
            };
            if minimum.0 == 0
                || minimum.1 == 0
                || minimum.0 > maximum.0
                || minimum.1 > maximum.1
                || maximum.0 > MAX_MAP_TILES
                || maximum.1 > MAX_MAP_TILES
            {
                return Err(format!(
                    "recipe {} structure {index} has an invalid feature size",
                    recipe.name
                )
                .into());
            }
        }
        if !matches!(structure.shape, FeatureShape::Square)
            && BuiltinFeature::from_kind(&structure.kind) != Some(BuiltinFeature::Pond)
        {
            return Err(format!(
                "recipe {} structure {index} shape requires the Pond feature",
                recipe.name
            )
            .into());
        }
        let name = resolved_name(&structure.kind, index, &structure.name);
        if !earlier_names.insert(name.clone()) {
            return Err(format!("recipe {} repeats structure name {name}", recipe.name).into());
        }
    }
    if let Some(near) = &recipe.enemies.near {
        if !earlier_names.contains(&near.structure) {
            return Err(format!(
                "recipe {} enemy placement references missing structure {}",
                recipe.name, near.structure
            )
            .into());
        }
        if near.radius > MAX_MAP_TILES {
            return Err(format!("recipe {} enemy radius exceeds the cap", recipe.name).into());
        }
    }
    if recipe.enemies.separation > MAX_MAP_TILES {
        return Err(format!("recipe {} enemy separation exceeds the cap", recipe.name).into());
    }
    let structure_starts = structures_place(recipe, structures, |kind| {
        matches!(kind, PlacementKind::PlayerStart)
    });
    if matches!(
        (structure_starts, &recipe.player_start),
        (true, PlayerStartPick::Placement(_)) | (false, PlayerStartPick::Structure)
    ) {
        return Err(format!(
            "recipe {} must define exactly one PlayerStart source",
            recipe.name
        )
        .into());
    }
    if let PlayerStartPick::Placement(PlacementPick::Choose(picks)) = &recipe.player_start
        && picks.is_empty()
    {
        return Err(format!("recipe {} has an empty player-start choice", recipe.name).into());
    }
    if let PlayerStartPick::Placement(pick) = &recipe.player_start
        && !pick.valid_player_start()
    {
        return Err(format!(
            "recipe {} player-start choices must use single-placement Random or Edge rules",
            recipe.name
        )
        .into());
    }
    Ok(())
}

/// Returns whether any structure in the recipe carries a glyph placement matching `wanted`.
fn structures_place(
    recipe: &MapRecipe,
    structures: &BTreeMap<String, Structure>,
    wanted: impl Fn(&PlacementKind) -> bool,
) -> bool {
    recipe.structures.iter().any(|structure_entry| {
        structures
            .get(&structure_entry.kind)
            .is_some_and(|structure| {
                structure
                    .placements
                    .iter()
                    .any(|placement| wanted(&placement.kind))
            })
    })
}

/// Shorthand for fallible map generation results.
type MapResult<T> = std::result::Result<T, MapError>;

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
        let resolved_name = resolved_name(&entry.kind, index, &entry.name);
        let rule = structure_rule(entry);
        let placed_center = match BuiltinFeature::from_kind(&entry.kind) {
            Some(BuiltinFeature::EmptySquare) => build_empty_square(
                blueprint,
                placement_context,
                &rule,
                entry.size,
                random_generator,
            )?,
            Some(BuiltinFeature::Pond) => build_pond(
                blueprint,
                placement_context,
                &rule,
                entry.size,
                entry.shape,
                random_generator,
            )?,
            Some(BuiltinFeature::Rubble) => build_rubble(
                blueprint,
                placement_context,
                &rule,
                entry.size,
                random_generator,
            )?,
            None => {
                let structure = structures.get(&entry.kind).ok_or_else(|| {
                    format!("structure {:?} is missing from the catalog", entry.kind)
                })?;
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
                        format!(
                            "failed to place structure {index} ({:?}): {error}",
                            entry.kind
                        )
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
                "Recipe {} seed {} could not place structure {index} ({:?})",
                recipe.name, seed, entry.kind
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
            let seen = widths.entry(recipe.name.as_str()).or_insert((u32::MAX, 0));
            *seen = (seen.0.min(width), seen.1.max(width));
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
