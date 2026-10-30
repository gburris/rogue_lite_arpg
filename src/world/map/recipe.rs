//! Recipe file types, loading and validation: `map_recipes.ron` to a [`RecipeCatalog`].

use std::collections::BTreeMap;
use std::io::BufReader;

use bevy::prelude::*;
use rand::distr::Distribution;
use rand::distr::weighted::WeightedIndex;
use rand::{Rng, RngExt};
use rand_chacha::ChaCha8Rng;
use ron::Options;
use ron::extensions::Extensions;
use serde::Deserialize;

use super::blueprint::PlacementKind;
use super::build::{MapRequest, MapResult, Stage, build_from_recipe};
use super::placement::{Anchor, CountRange, PlaceRule, Side};
use super::spawn::MAX_MAP_TILES;
use super::structures::{FeatureShape, FeatureSize, Structure, structure_from_glyphs};
use super::tiles::TileKind;
use crate::prelude::*;
use crate::world::MapRng;

/// Loaded map recipes and the structures available to place in them.
#[derive(Resource, Debug)]
pub struct RecipeCatalog {
    pub(super) hub: MapRecipe,
    pub(super) recipes: BTreeMap<String, MapRecipe>,
    pub(super) structures: BTreeMap<String, Structure>,
    pub(super) weighted_names: Vec<String>,
    /// Every recipe name in file order: hub, dungeons, then arenas.
    #[cfg_attr(not(feature = "dev"), expect(dead_code))]
    pub(super) order: Vec<String>,
}

impl RecipeCatalog {
    pub(super) fn hub(&self) -> &MapRecipe {
        &self.hub
    }

    /// Chooses the recipe and build seed for a request.
    /// The weighted pick draws from the root generator itself, so it never shares a draw
    /// with the build seed's own generator (which rolls the map size first).
    pub(super) fn resolve(
        &self,
        request: MapRequest,
        root_rng: &mut MapRng,
    ) -> Result<(&MapRecipe, u64)> {
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

    /// The recipe after `current` in file order, wrapping.
    #[cfg(feature = "dev")]
    pub(super) fn recipe_after(&self, current: &str) -> &str {
        let index = self
            .order
            .iter()
            .position(|name| name == current)
            .map_or(0, |index| (index + 1) % self.order.len());
        &self.order[index]
    }

    pub(super) fn named(&self, name: &str) -> Result<&MapRecipe> {
        if name == self.hub.name {
            return Ok(&self.hub);
        }
        self.recipes
            .get(name)
            .ok_or_else(|| format!("unknown map recipe: {name}").into())
    }

    pub(super) fn weighted(&self, random_generator: &mut impl Rng) -> Result<&MapRecipe> {
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

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RecipeFile {
    /// Named spawn lists that recipes include with `presets: ["Name"]`.
    #[serde(default)]
    pub(super) spawn_presets: BTreeMap<String, Vec<SpawnEntry>>,
    pub(super) defaults: RecipeDefaults,
    pub(super) structures: BTreeMap<String, StructureDefinition>,
    pub(super) hub: RecipeEntry,
    #[serde(default)]
    pub(super) dungeons: Vec<RecipeEntry>,
    #[serde(default)]
    pub(super) arenas: Vec<RecipeEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct StructureDefinition {
    pub(super) floor: TileKind,
    /// One row per entry, top row first; joined with `'\n'` when the catalog loads.
    pub(super) glyphs: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RecipeDefaults {
    pub(super) size_x_range: CountRange,
    pub(super) size_y_range: CountRange,
    pub(super) floor: TileKind,
    pub(super) structures: Vec<StructureEntry>,
    /// Radius around the player start, in tiles, that no enemy may occupy.
    pub(super) start_clearance: u32,
    #[serde(default)]
    pub(super) presets: Vec<String>,
    pub(super) spawns: Vec<SpawnEntry>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RecipeEntry {
    pub(super) name: Option<String>,
    pub(super) weight: Option<u32>,
    /// Shorthand for a square map: `size: 60` sets both size ranges to `(60, 60)`.
    pub(super) size: Option<u32>,
    pub(super) size_x_range: Option<CountRange>,
    pub(super) size_y_range: Option<CountRange>,
    pub(super) floor: Option<TileKind>,
    pub(super) structures: Option<Vec<StructureEntry>>,
    pub(super) start_clearance: Option<u32>,
    /// Replaces the default presets; their spawns come before the recipe's own.
    pub(super) presets: Option<Vec<String>>,
    /// Replaces the default spawns entirely, like `structures`.
    pub(super) spawns: Option<Vec<SpawnEntry>>,
}

/// Resolved map generation settings loaded from a recipe definition.
#[derive(Clone, Debug)]
pub(crate) struct MapRecipe {
    pub(super) name: String,
    pub(super) weight: u32,
    pub(super) size_x_range: CountRange,
    pub(super) size_y_range: CountRange,
    pub(super) floor: TileKind,
    pub(super) structures: Vec<StructureEntry>,
    pub(super) start_clearance: u32,
    /// Preset spawns followed by the recipe's own.
    pub(super) spawns: Vec<SpawnEntry>,
}

impl RecipeEntry {
    pub(super) fn resolve(
        self,
        name: String,
        weight: u32,
        defaults: &RecipeDefaults,
        spawn_presets: &BTreeMap<String, Vec<SpawnEntry>>,
    ) -> Result<MapRecipe> {
        if self.size.is_some() && (self.size_x_range.is_some() || self.size_y_range.is_some()) {
            return Err(format!(
                "recipe {name} sets size together with size_x_range or size_y_range"
            )
            .into());
        }
        let square = self.size.map(CountRange::exactly);
        let mut spawns = Vec::new();
        for preset in self.presets.as_ref().unwrap_or(&defaults.presets) {
            let preset_spawns = spawn_presets
                .get(preset)
                .ok_or_else(|| format!("recipe {name} uses unknown spawn preset {preset}"))?;
            spawns.extend(preset_spawns.iter().cloned());
        }
        spawns.extend(self.spawns.unwrap_or_else(|| defaults.spawns.clone()));
        Ok(MapRecipe {
            name,
            weight,
            size_x_range: self
                .size_x_range
                .or(square)
                .unwrap_or(defaults.size_x_range),
            size_y_range: self
                .size_y_range
                .or(square)
                .unwrap_or(defaults.size_y_range),
            floor: self.floor.unwrap_or(defaults.floor),
            structures: expand_counts(
                self.structures
                    .unwrap_or_else(|| defaults.structures.clone()),
            ),
            start_clearance: self.start_clearance.unwrap_or(defaults.start_clearance),
            spawns,
        })
    }
}

/// Replaces each entry with `count` single copies, as if the line were repeated.
pub(super) fn expand_counts(entries: Vec<StructureEntry>) -> Vec<StructureEntry> {
    entries
        .into_iter()
        .flat_map(|entry| {
            let copies = entry.count as usize;
            std::iter::repeat_n(StructureEntry { count: 1, ..entry }, copies)
        })
        .collect()
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct StructureEntry {
    pub(super) kind: StructureKind,
    #[serde(default)]
    pub(super) name: String,
    /// Where the structure goes, from the same options as spawns; `Random()` by default.
    #[serde(default = "random_location")]
    pub(super) at: Location,
    /// Resolved names stay `{kind}_{index}`, so a named entry must keep `count: 1`.
    #[serde(default = "one")]
    pub(super) count: u32,
}

pub(super) fn one() -> u32 {
    1
}

/// What a structure entry places: a glyph structure from the file's `structures` table,
/// or a builtin procedural feature. Each feature takes only the fields it uses.
///
/// To add a feature: add a variant with its fields, write its builder from
/// `claim_feature_footprint` in structures.rs (see rubble for a one-tile feature), and
/// handle the variant in the exhaustive `place_structures` match.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) enum StructureKind {
    /// A named glyph structure: `Structure("Temple")`.
    Structure(String),
    /// Walled dead-zone square, a 3–10 tile square by default: `EmptySquare()`.
    EmptySquare {
        #[serde(default)]
        size: Option<FeatureSize>,
    },
    /// Water that blocks ground movement and spawns, a 3–8 tile square by default:
    /// `Pond(size: Exact((14, 9)), shape: Blob)`.
    Pond {
        #[serde(default)]
        size: Option<FeatureSize>,
        #[serde(default)]
        shape: FeatureShape,
    },
    /// Wall rubble, a single tile by default: `Rubble()`.
    Rubble {
        #[serde(default)]
        size: Option<FeatureSize>,
    },
}

impl StructureKind {
    /// Name used for `{kind}_{index}` structure names and in messages.
    pub(super) fn label(&self) -> &str {
        match self {
            Self::Structure(name) => name,
            Self::EmptySquare { .. } => "EmptySquare",
            Self::Pond { .. } => "Pond",
            Self::Rubble { .. } => "Rubble",
        }
    }

    pub(super) fn feature_size(&self) -> Option<FeatureSize> {
        match self {
            Self::Structure(_) => None,
            Self::EmptySquare { size } | Self::Pond { size, .. } | Self::Rubble { size } => *size,
        }
    }
}

fn random_location() -> Location {
    Location::Random {
        x_deciles: interior_deciles(),
    }
}

pub(super) fn resolved_name(kind: &str, index: usize, explicit: &str) -> String {
    if explicit.is_empty() {
        format!("{kind}_{index}")
    } else {
        explicit.to_owned()
    }
}

/// One line of a recipe's `spawns`: what to place, where, and how many.
///
/// ```ron
/// (what: PlayerStart, at: Edge(Left)),
/// (what: Enemy(FireMage), at: MapMin((5, 5))),
/// (what: RandomEnemy, count: (10, 15), at: Random(x_deciles: (2, 8)), separation: 5),
/// (what: Chest, count: (3, 5), at: Near(structure: "Temple", radius: 6)),
/// ```
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SpawnEntry {
    pub(super) what: SpawnKind,
    /// How many to place; a bare number or an inclusive `(min, max)`. Defaults to 1.
    #[serde(default = "one_spawn")]
    pub(super) count: CountRange,
    pub(super) at: Location,
    /// Minimum tile distance from every point placed so far (`Random` and `Near` only).
    #[serde(default)]
    pub(super) separation: u32,
}

fn one_spawn() -> CountRange {
    CountRange::exactly(1)
}

/// What a spawn entry places.
#[derive(Clone, Copy, Debug, Deserialize)]
pub(super) enum SpawnKind {
    PlayerStart,
    Exit,
    Chest,
    Enemy(EnemyKind),
    /// An enemy picked uniformly from `EnemyKind::ALL`, rolled per spawn.
    RandomEnemy,
    Npc(NpcKind),
}

impl SpawnKind {
    /// Build stage this kind's sampled (non-fixed) spawns run in.
    pub(super) fn stage(self) -> Stage {
        match self {
            Self::PlayerStart => Stage::PlayerStart,
            Self::Exit => Stage::Exits,
            Self::Enemy(_) | Self::RandomEnemy => Stage::Enemies,
            Self::Chest => Stage::Chests,
            Self::Npc(_) => Stage::Npcs,
        }
    }

    /// Resolves to a concrete placement, rolling `RandomEnemy`.
    pub(super) fn placement(self, random_generator: &mut ChaCha8Rng) -> PlacementKind {
        match self {
            Self::PlayerStart => PlacementKind::PlayerStart,
            Self::Exit => PlacementKind::Exit,
            Self::Chest => PlacementKind::Chest,
            Self::Enemy(kind) => PlacementKind::Enemy(kind),
            Self::RandomEnemy => {
                let index = random_generator.random_range(0..EnemyKind::ALL.len());
                PlacementKind::Enemy(EnemyKind::ALL[index])
            }
            Self::Npc(kind) => PlacementKind::Npc(kind),
        }
    }
}

/// Where a spawn or structure goes. `Random`, `Edge`, `Near` and `Choose` sample a spot;
/// the rest are fixed anchors, which place exactly one. Positions are measured to the
/// center of what is placed, except the `Map*` anchors and `Edge`, which place its
/// bottom-left corner.
#[derive(Clone, Debug, Deserialize)]
pub(super) enum Location {
    /// Anywhere inside the half-open x-decile band, `(1, 9)` by default: `Random()`.
    Random {
        #[serde(default = "interior_deciles")]
        x_deciles: (u32, u32),
    },
    /// One tile inside a wall; several spawns spread along it: `Edge(Left)`.
    Edge(Side),
    /// Within `radius` tiles (a square) of a named structure's center plus `offset`.
    Near {
        structure: String,
        #[serde(default)]
        offset: IVec2,
        radius: u32,
    },
    /// One of these, picked per map.
    Choose(Vec<Self>),
    Center,
    CenterOffset(IVec2),
    MapMin(IVec2),
    /// Offset from the top-right tile: `MapMax((-6, -6))`.
    MapMax(IVec2),
    /// An earlier structure's center plus `offset`.
    Structure {
        structure: String,
        #[serde(default)]
        offset: IVec2,
    },
}

fn interior_deciles() -> (u32, u32) {
    (1, 9)
}

impl Location {
    /// Fixed anchors place exactly one and run before every sampled spawn.
    pub(super) fn is_fixed(&self) -> bool {
        !matches!(
            self,
            Self::Random { .. } | Self::Edge(_) | Self::Near { .. } | Self::Choose(_)
        )
    }

    /// Builds the placement rule for `count` spawns, picking one option of a `Choose`.
    pub(super) fn rule(
        &self,
        count: u32,
        separation: u32,
        random_generator: &mut ChaCha8Rng,
    ) -> MapResult<PlaceRule> {
        let count_range = CountRange::exactly(count);
        Ok(match self {
            Self::Random { x_deciles } => PlaceRule::Random {
                count: count_range,
                x_deciles: *x_deciles,
                separation,
            },
            Self::Edge(side) => PlaceRule::Edge {
                count: count_range,
                side: *side,
            },
            Self::Near {
                structure,
                offset,
                radius,
            } => PlaceRule::NearStructure {
                count: count_range,
                structure: structure.clone(),
                offset: *offset,
                radius: *radius,
                separation,
            },
            Self::Choose(options) => {
                if options.is_empty() {
                    return Err("spawn choice list is empty".into());
                }
                let index = random_generator.random_range(0..options.len());
                return options[index].rule(count, separation, random_generator);
            }
            Self::Center => PlaceRule::Fixed(Anchor::Center),
            Self::CenterOffset(offset) => PlaceRule::Fixed(Anchor::CenterOffset(*offset)),
            Self::MapMin(offset) => PlaceRule::Fixed(Anchor::MapMin(*offset)),
            Self::MapMax(offset) => PlaceRule::Fixed(Anchor::MapMax(*offset)),
            Self::Structure { structure, offset } => PlaceRule::Fixed(Anchor::Structure {
                name: structure.clone(),
                offset: *offset,
            }),
        })
    }
}

pub(super) fn load_recipes(mut commands: Commands) -> Result {
    commands.insert_resource(load_recipe_catalog()?);
    Ok(())
}

pub(super) fn load_recipe_catalog() -> Result<RecipeCatalog> {
    catalog_from_file(load_recipe_file()?)
}

/// Resolves, validates and smoke-builds every recipe in a parsed recipe file.
pub(super) fn catalog_from_file(file: RecipeFile) -> Result<RecipeCatalog> {
    let structures = file
        .structures
        .into_iter()
        .map(|(name, definition)| {
            structure_from_glyphs(definition.floor, &definition.glyphs.join("\n"))
                .map(|structure| (name, structure))
                .map_err(|error| format!("invalid structure glyphs: {error:?}"))
        })
        .collect::<std::result::Result<BTreeMap<_, _>, _>>()?;
    let mut names = std::collections::HashSet::new();
    let hub_name = file
        .hub
        .name
        .clone()
        .unwrap_or_else(|| "HubTown".to_owned());
    if !names.insert(hub_name.clone()) {
        return Err(format!("duplicate recipe name: {hub_name}").into());
    }
    let defaults = RecipeDefaults {
        structures: expand_counts(file.defaults.structures.clone()),
        ..file.defaults
    };
    let hub = file
        .hub
        .resolve(hub_name, 0, &defaults, &file.spawn_presets)?;
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
        let recipe = entry.resolve(name.clone(), weight, &defaults, &file.spawn_presets)?;
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

#[cfg(not(target_arch = "wasm32"))]
use std::fs::File;

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn fetch_recipe_data() -> Result<File> {
    File::open("assets/config/map_recipes.ron")
        .map_err(|error| format!("failed to open assets/config/map_recipes.ron: {error}").into())
}

#[cfg(target_arch = "wasm32")]
pub(super) fn fetch_recipe_data() -> &'static [u8] {
    include_bytes!("../../../assets/config/map_recipes.ron")
}

pub(super) fn load_recipe_file() -> Result<RecipeFile> {
    #[cfg(not(target_arch = "wasm32"))]
    let reader = BufReader::new(fetch_recipe_data()?);
    #[cfg(target_arch = "wasm32")]
    let reader = BufReader::new(fetch_recipe_data());
    recipe_options()
        .from_reader(reader)
        .map_err(|error| format!("failed to parse assets/config/map_recipes.ron: {error}").into())
}

pub(super) fn recipe_options() -> Options {
    Options::default().with_default_extension(Extensions::IMPLICIT_SOME)
}

pub(super) fn validate_recipe(
    recipe: &MapRecipe,
    structures: &BTreeMap<String, Structure>,
) -> Result<()> {
    for (name, range) in [
        ("size_x", recipe.size_x_range),
        ("size_y", recipe.size_y_range),
    ] {
        if range.min == 0 || range.min > range.max || range.max > MAX_MAP_TILES {
            return Err(format!(
                "recipe {} has invalid {name} range (each side must be 1..={MAX_MAP_TILES})",
                recipe.name
            )
            .into());
        }
    }
    if recipe.start_clearance > MAX_MAP_TILES {
        return Err(format!("recipe {} start_clearance exceeds the cap", recipe.name).into());
    }
    let mut earlier_names = std::collections::HashSet::new();
    for (index, structure) in recipe.structures.iter().enumerate() {
        if let StructureKind::Structure(kind) = &structure.kind
            && !structures.contains_key(kind)
        {
            return Err(
                format!("recipe {} references unknown structure {kind}", recipe.name).into(),
            );
        }
        validate_location(
            &structure.at,
            CountRange::exactly(1),
            &earlier_names,
            &format!("recipe {} structure {index}", recipe.name),
        )?;
        if let Some(size) = structure.kind.feature_size() {
            let (minimum, maximum) = match size {
                FeatureSize::Exact(size) => (size, size),
                FeatureSize::Ranged(minimum, maximum) => (minimum, maximum),
            };
            if minimum.width == 0
                || minimum.height == 0
                || minimum.width > maximum.width
                || minimum.height > maximum.height
                || maximum.width > MAX_MAP_TILES
                || maximum.height > MAX_MAP_TILES
            {
                return Err(format!(
                    "recipe {} structure {index} has an invalid feature size",
                    recipe.name
                )
                .into());
            }
        }
        let name = resolved_name(structure.kind.label(), index, &structure.name);
        if !earlier_names.insert(name.clone()) {
            return Err(format!("recipe {} repeats structure name {name}", recipe.name).into());
        }
    }
    validate_spawns(recipe, structures, &earlier_names)
}

/// Checks each spawn entry, and that the player start and exits each come from exactly
/// one source: structure glyphs or spawn entries.
fn validate_spawns(
    recipe: &MapRecipe,
    structures: &BTreeMap<String, Structure>,
    structure_names: &std::collections::HashSet<String>,
) -> Result<()> {
    for (index, spawn) in recipe.spawns.iter().enumerate() {
        let context = format!("recipe {} spawn {index}", recipe.name);
        let CountRange { min, max } = spawn.count;
        if min > max || max > MAX_MAP_TILES || spawn.separation > MAX_MAP_TILES {
            return Err(format!("{context} has an invalid count or separation").into());
        }
        if matches!(spawn.what, SpawnKind::PlayerStart) && spawn.count != CountRange::exactly(1) {
            return Err(format!("{context} must place exactly one player start").into());
        }
        validate_location(&spawn.at, spawn.count, structure_names, &context)?;
    }
    let structure_starts = structures_place(recipe, structures, |kind| {
        matches!(kind, PlacementKind::PlayerStart)
    });
    let spawn_starts = recipe
        .spawns
        .iter()
        .filter(|spawn| matches!(spawn.what, SpawnKind::PlayerStart))
        .count();
    if (structure_starts && spawn_starts > 0) || (!structure_starts && spawn_starts != 1) {
        return Err(format!(
            "recipe {} must define exactly one player start source, a structure glyph or one spawn",
            recipe.name
        )
        .into());
    }
    let structure_exits = structures_place(recipe, structures, |kind| {
        matches!(kind, PlacementKind::Exit)
    });
    let exit_spawns = recipe
        .spawns
        .iter()
        .filter(|spawn| matches!(spawn.what, SpawnKind::Exit));
    let has_exit_spawns = exit_spawns.clone().next().is_some();
    let guarantees_an_exit = exit_spawns.clone().any(|spawn| spawn.count.min > 0);
    if (structure_exits && has_exit_spawns) || (!structure_exits && !guarantees_an_exit) {
        return Err(format!(
            "recipe {} must define exactly one exit source, structure glyphs or exit spawns",
            recipe.name
        )
        .into());
    }
    Ok(())
}

fn validate_location(
    at: &Location,
    count: CountRange,
    structure_names: &std::collections::HashSet<String>,
    context: &str,
) -> Result<()> {
    match at {
        Location::Random { x_deciles } if x_deciles.0 >= x_deciles.1 || x_deciles.1 > 10 => {
            Err(format!("{context} has an invalid x-decile range").into())
        }
        Location::Near {
            structure, radius, ..
        } => {
            if !structure_names.contains(structure) {
                return Err(
                    format!("{context} references missing or later structure {structure}").into(),
                );
            }
            if *radius > MAX_MAP_TILES {
                return Err(format!("{context} radius exceeds the cap").into());
            }
            Ok(())
        }
        Location::Choose(options) => {
            if options.is_empty() {
                return Err(format!("{context} has an empty choice").into());
            }
            options
                .iter()
                .try_for_each(|option| validate_location(option, count, structure_names, context))
        }
        Location::Structure { structure, .. } if !structure_names.contains(structure) => {
            Err(format!("{context} references missing or later structure {structure}").into())
        }
        fixed if fixed.is_fixed() && count != CountRange::exactly(1) => {
            Err(format!("{context} uses a fixed anchor, so it must place exactly one").into())
        }
        _ => Ok(()),
    }
}

/// Returns whether any structure in the recipe carries a glyph placement matching `wanted`.
pub(super) fn structures_place(
    recipe: &MapRecipe,
    structures: &BTreeMap<String, Structure>,
    wanted: impl Fn(&PlacementKind) -> bool,
) -> bool {
    recipe
        .structures
        .iter()
        .filter_map(|structure_entry| match &structure_entry.kind {
            StructureKind::Structure(kind) => structures.get(kind),
            StructureKind::EmptySquare { .. }
            | StructureKind::Pond { .. }
            | StructureKind::Rubble { .. } => None,
        })
        .any(|structure| {
            structure
                .placements
                .iter()
                .any(|placement| wanted(&placement.kind))
        })
}
