use rand::{
    RngExt, SeedableRng,
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
    #[cfg(not(target_arch = "wasm32"))]
    app.add_systems(Update, reload_recipe_catalog);
}

/// System parameters for requesting the next map.
#[derive(SystemParam)]
pub struct NextMap<'w, 's> {
    commands: Commands<'w, 's>,
    previous_map: Option<Single<'w, 's, Entity, With<MapBlueprint>>>,
    next_state: ResMut<'w, NextState<AppState>>,
    catalog: Res<'w, RecipeCatalog>,
    root_rng: ResMut<'w, MapRng>,
}

impl NextMap<'_, '_> {
    /// Builds and schedules the requested map, replacing any current map.
    pub fn set(&mut self, request: MapRequest) -> Result<()> {
        let (recipe, seed) = match request {
            MapRequest::Hub => (self.catalog.hub(), self.root_rng.next_seed()),
            MapRequest::Weighted => {
                let seed = self.root_rng.next_seed();
                let mut random_generator = ChaCha8Rng::seed_from_u64(seed);
                (self.catalog.weighted(&mut random_generator)?, seed)
            }
            MapRequest::Named { recipe, seed } => (self.catalog.named(&recipe)?, seed),
        };
        info!("Generating map {} with seed {seed}", recipe.name);
        let blueprint = build_from_recipe(recipe, seed, &self.catalog.structures)?;

        if let Some(previous_map) = self.previous_map.as_ref() {
            self.commands.entity(previous_map.entity()).despawn();
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
    Weighted,
    Named { recipe: String, seed: u64 },
}

/// Loaded map recipes and the structures available to place in them.
#[derive(Resource, Debug)]
pub struct RecipeCatalog {
    hub: MapRecipe,
    recipes: BTreeMap<String, MapRecipe>,
    structures: BTreeMap<String, Structure>,
    weighted_names: Vec<String>,
}

impl RecipeCatalog {
    fn hub(&self) -> &MapRecipe {
        &self.hub
    }

    fn named(&self, name: &str) -> Result<&MapRecipe> {
        if name == self.hub.name {
            return Ok(&self.hub);
        }
        self.recipes
            .get(name)
            .ok_or_else(|| format!("unknown map recipe: {name}").into())
    }

    fn weighted(&self, random_generator: &mut ChaCha8Rng) -> Result<&MapRecipe> {
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
    /// One map row per entry; joined with `'\n'` when the catalog loads.
    glyphs: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
struct RecipeDefaults {
    size_x_range: (u32, u32),
    size_y_range: (u32, u32),
    floor: TileKind,
    structures: Vec<StructureEntry>,
    player_start: PlayerStartPick,
    exits: PlacementPick,
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
    exits: Option<PlacementPick>,
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
    exits: PlacementPick,
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
    Bat,
    Random,
}

impl EnemyPick {
    fn resolve(self, random_generator: &mut ChaCha8Rng) -> EnemyKind {
        match self {
            Self::Warrior => EnemyKind::Warrior,
            Self::IceMage => EnemyKind::IceMage,
            Self::FireMage => EnemyKind::FireMage,
            Self::Bat => EnemyKind::Bat,
            Self::Random => {
                let index = random_generator.random_range(0..EnemyKind::ALL.len());
                EnemyKind::ALL[index]
            }
        }
    }
}

fn load_recipes(mut commands: Commands) -> Result {
    commands.insert_resource(load_recipe_catalog()?);
    Ok(())
}

fn load_recipe_catalog() -> Result<RecipeCatalog> {
    let file = load_recipe_file()?;
    let structures = file
        .structures
        .into_iter()
        .map(|(name, definition)| {
            structure_from_glyphs(definition.floor, &definition.glyphs.join("\n"))
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
        build_from_recipe(&hub, seed, &structures)?;
    }
    let mut recipes = BTreeMap::new();
    let mut weighted_names = Vec::new();
    for entry in file.dungeons.into_iter().chain(file.arenas) {
        let name = entry.name.clone().ok_or("recipe name is required")?;
        if !names.insert(name.clone()) {
            return Err(format!("duplicate recipe name: {name}").into());
        }
        let weight = entry
            .weight
            .ok_or_else(|| format!("recipe {name} requires a weight"))?;
        if weight == 0 {
            return Err(format!("recipe {name} weight must be positive").into());
        }
        let recipe = entry.resolve(name.clone(), weight, &file.defaults);
        validate_recipe(&recipe, &structures)?;
        for seed in 0..=8 {
            build_from_recipe(&recipe, seed, &structures)?;
        }
        weighted_names.push(name.clone());
        recipes.insert(name, recipe);
    }
    if recipes.is_empty() {
        return Err("at least one dungeon or arena recipe is required".into());
    }
    Ok(RecipeCatalog {
        hub,
        recipes,
        structures,
        weighted_names,
    })
}
#[cfg(not(target_arch = "wasm32"))]
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
        Ok(catalog) => commands.insert_resource(catalog),
        Err(error) => error!("Keeping previous recipe catalog after reload error: {error}"),
    }
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
    Options::default()
        .with_default_extension(Extensions::IMPLICIT_SOME)
        .from_reader(reader)
        .map_err(|error| format!("failed to parse assets/config/map_recipes.ron: {error}").into())
}

fn validate_recipe(recipe: &MapRecipe, structures: &BTreeMap<String, Structure>) -> Result<()> {
    validate_recipe_layout(recipe)?;
    let mut earlier_names = std::collections::HashSet::new();
    validate_recipe_structures(recipe, structures, &mut earlier_names)?;
    validate_recipe_enemy_placement(recipe, &earlier_names)?;
    validate_recipe_player_start(recipe, structures)?;
    Ok(())
}

fn validate_recipe_layout(recipe: &MapRecipe) -> Result<()> {
    for (name, range) in [
        ("size_x", recipe.size_x_range),
        ("size_y", recipe.size_y_range),
    ] {
        if range.0 == 0 || range.0 > range.1 || range.1 > 512 {
            return Err(format!("recipe {} has invalid {name} range", recipe.name).into());
        }
    }
    if recipe
        .size_x_range
        .1
        .checked_mul(recipe.size_y_range.1)
        .is_none_or(|area| area > 262_144)
    {
        return Err(format!("recipe {} exceeds the map tile cap", recipe.name).into());
    }
    let (enemy_min, enemy_max) = (recipe.enemies.count.0, recipe.enemies.count.1);
    if enemy_min > enemy_max
        || enemy_max > 512
        || recipe.enemies.fixed.len() > 512
        || recipe.chests.0 > recipe.chests.1
        || recipe.chests.1 > 512
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
    if !matches!(&recipe.exits, PlacementPick::Edge { .. }) {
        return Err(format!("recipe {} exits must use an Edge placement", recipe.name).into());
    }
    if let PlacementPick::Edge { count, .. } = &recipe.exits
        && (count.0 == 0 || count.0 > count.1 || count.1 > 2)
    {
        return Err(format!("recipe {} has invalid exit count range", recipe.name).into());
    }
    Ok(())
}

fn validate_recipe_structures(
    recipe: &MapRecipe,
    structures: &BTreeMap<String, Structure>,
    earlier_names: &mut std::collections::HashSet<String>,
) -> Result<()> {
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
        if structure.radius.is_some_and(|radius| radius > 512) {
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
                || maximum.0 > 512
                || maximum.1 > 512
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
    Ok(())
}

fn validate_recipe_enemy_placement(
    recipe: &MapRecipe,
    earlier_names: &std::collections::HashSet<String>,
) -> Result<()> {
    if let Some(near) = &recipe.enemies.near {
        if !earlier_names.contains(&near.structure) {
            return Err(format!(
                "recipe {} enemy placement references missing structure {}",
                recipe.name, near.structure
            )
            .into());
        }
        if near.radius > 512 {
            return Err(format!("recipe {} enemy radius exceeds the cap", recipe.name).into());
        }
    }
    if recipe.enemies.separation > 512 {
        return Err(format!("recipe {} enemy separation exceeds the cap", recipe.name).into());
    }
    Ok(())
}

fn validate_recipe_player_start(
    recipe: &MapRecipe,
    structures: &BTreeMap<String, Structure>,
) -> Result<()> {
    let structure_starts = recipe.structures.iter().any(|structure_entry| {
        structures
            .get(&structure_entry.kind)
            .is_some_and(|structure| {
                structure
                    .placements
                    .iter()
                    .any(|placement| matches!(&placement.kind, PlacementKind::PlayerStart))
            })
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

/// Shorthand for fallible map generation results.
type MapResult<T> = std::result::Result<T, MapError>;

/// Generates a map blueprint from a recipe and deterministic seed.
pub(crate) fn build_from_recipe(
    recipe: &MapRecipe,
    seed: u64,
    structures: &BTreeMap<String, Structure>,
) -> std::result::Result<MapBlueprint, MapError> {
    let mut random_generator = ChaCha8Rng::seed_from_u64(seed);
    let size_x = random_generator.random_range(recipe.size_x_range.0..=recipe.size_x_range.1);
    let size_y = random_generator.random_range(recipe.size_y_range.0..=recipe.size_y_range.1);
    let mut blueprint = MapBlueprint::new(UVec2::new(size_x, size_y), recipe.floor, seed);
    let mut placement_context = PlaceContext::new(blueprint.map_size());
    place_structures(
        recipe,
        seed,
        structures,
        &mut blueprint,
        &mut placement_context,
        &mut random_generator,
    )?;

    if let Some(player_start) = place_player_start(
        recipe,
        &blueprint,
        &mut placement_context,
        &mut random_generator,
    )? {
        blueprint.placements.push(Placement {
            tile: player_start,
            kind: PlacementKind::PlayerStart,
        });
    } else if !blueprint
        .placements
        .iter()
        .any(|placement| matches!(&placement.kind, PlacementKind::PlayerStart))
    {
        return Err("structure-based PlayerStart source did not place a PlayerStart".into());
    }
    place_exits(
        recipe,
        &mut blueprint,
        &mut placement_context,
        &mut random_generator,
    )?;
    place_enemies(
        recipe,
        seed,
        &mut blueprint,
        &mut placement_context,
        &mut random_generator,
    )?;
    place_chests(
        recipe,
        seed,
        &mut blueprint,
        &mut placement_context,
        &mut random_generator,
    )?;
    blueprint.validate().map_err(MapError::Validation)?;
    Ok(blueprint)
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
        let rule = structure_rule(entry, placement_context, random_generator)?;
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

/// Resolves a structure entry's anchor to a placement rule, rolling any radius offset.
fn structure_rule(
    entry: &StructureEntry,
    placement_context: &PlaceContext,
    random_generator: &mut ChaCha8Rng,
) -> MapResult<PlaceRule> {
    match &entry.anchor {
        StructureAnchor::Random => Ok(PlaceRule::Random {
            count: CountRange(1, 1),
            x_deciles: (1, 9),
            separation: 0,
        }),
        StructureAnchor::Center => Ok(PlaceRule::Fixed(Anchor::Center)),
        StructureAnchor::CenterOffset(offset) => {
            Ok(PlaceRule::Fixed(Anchor::CenterOffset(*offset)))
        }
        StructureAnchor::MapMin(position) => Ok(PlaceRule::Fixed(Anchor::MapMin(*position))),
        StructureAnchor::Structure {
            structure: earlier,
            offset,
        } => {
            if *offset == IVec2::ZERO && entry.radius.is_none() {
                Ok(PlaceRule::Fixed(Anchor::Structure(earlier.clone())))
            } else {
                let center = placement_context
                    .structure_centers_by_name
                    .get(earlier)
                    .copied()
                    .ok_or_else(|| {
                        format!("structure anchor {earlier:?} is missing or was never placed")
                    })?;
                let mut minimum = center + *offset;
                if let Some(radius) = entry.radius {
                    let radius = i32::try_from(radius)
                        .map_err(|_| "structure radius is too large".to_owned())?;
                    minimum += IVec2::new(
                        random_generator.random_range(-radius..=radius),
                        random_generator.random_range(-radius..=radius),
                    );
                }
                Ok(PlaceRule::Fixed(Anchor::MapMin(minimum)))
            }
        }
    }
}

/// Places the player start, returning its tile; `None` means structures provide it.
fn place_player_start(
    recipe: &MapRecipe,
    blueprint: &MapBlueprint,
    placement_context: &mut PlaceContext,
    random_generator: &mut ChaCha8Rng,
) -> MapResult<Option<IVec2>> {
    match &recipe.player_start {
        PlayerStartPick::Structure => Ok(None),
        PlayerStartPick::Placement(pick) => {
            let rule = pick.resolve(random_generator)?;
            let tile = place(
                &rule,
                UVec2::ONE,
                &blueprint.tiles,
                placement_context,
                random_generator,
            )?
            .into_iter()
            .next()
            .ok_or_else(|| "failed to place player start".to_owned())?;
            Ok(Some(tile))
        }
    }
}

/// Places exits along the recipe's edge rule. Portals sit one tile inside the border,
/// so the border tile is only bounds-checked, never carved.
fn place_exits(
    recipe: &MapRecipe,
    blueprint: &mut MapBlueprint,
    placement_context: &mut PlaceContext,
    random_generator: &mut ChaCha8Rng,
) -> MapResult<()> {
    let exit_rule = recipe.exits.resolve(random_generator)?;
    let exits = place(
        &exit_rule,
        UVec2::ONE,
        &blueprint.tiles,
        placement_context,
        random_generator,
    )?;
    for tile in exits {
        let doorway = match &exit_rule {
            PlaceRule::Edge {
                side: Side::Left, ..
            } => IVec2::ZERO.with_y(tile.y),
            PlaceRule::Edge {
                side: Side::Right, ..
            } => IVec2::new(blueprint.map_size().x as i32 - 1, tile.y),
            PlaceRule::Edge {
                side: Side::Top, ..
            } => IVec2::new(tile.x, blueprint.map_size().y as i32 - 1),
            PlaceRule::Edge {
                side: Side::Bottom, ..
            } => IVec2::new(tile.x, 0),
            _ => return Err("exit placement must use an Edge rule".to_owned().into()),
        };
        if !blueprint.tiles.contains(doorway) {
            return Err(format!("exit doorway {doorway:?} is out of bounds").into());
        }
        blueprint.placements.push(Placement {
            tile,
            kind: PlacementKind::Exit,
        });
    }
    Ok(())
}

/// Places fixed enemies, then the random pack. A failed fixed placement aborts the map;
/// a short random pack only warns, since open ground is never guaranteed.
fn place_enemies(
    recipe: &MapRecipe,
    seed: u64,
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
            blueprint.placements.push(Placement {
                tile,
                kind: PlacementKind::Enemy(fixed.pick.resolve(random_generator)),
            });
        }
    }
    let enemy_count = recipe.enemies.count.roll_size(random_generator);
    let enemy_rule = if let Some(nearby_enemy_placement) = &recipe.enemies.near {
        PlaceRule::NearStructure {
            count: CountRange(enemy_count, enemy_count),
            structure: nearby_enemy_placement.structure.clone(),
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
        blueprint.placements.push(Placement {
            tile,
            kind: PlacementKind::Enemy(recipe.enemies.pick.resolve(random_generator)),
        });
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
        blueprint.placements.push(Placement {
            tile,
            kind: PlacementKind::Chest,
        });
    }
    Ok(())
}
