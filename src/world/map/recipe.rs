//! Recipe file types, loading and validation: `map_recipes.ron` to a [`RecipeCatalog`].

use rand::{
    Rng, RngExt,
    distr::{Distribution, weighted::WeightedIndex},
};
use rand_chacha::ChaCha8Rng;
use ron::{Options, extensions::Extensions};
use serde::Deserialize;
use std::{collections::BTreeMap, io::BufReader};

use bevy::prelude::*;

use crate::{
    prelude::EnemyKind,
    world::{
        MapRng,
        map::{
            blueprint::PlacementKind,
            build::{MapRequest, MapResult, build_from_recipe},
            placement::{Anchor, CountRange, PlaceRule, Side},
            spawn::MAX_MAP_TILES,
            structures::{FeatureShape, FeatureSize, Structure, structure_from_glyphs},
            tiles::TileKind,
        },
    },
};

/// Loaded map recipes and the structures available to place in them.
#[derive(Resource, Debug)]
pub struct RecipeCatalog {
    pub(super) hub: MapRecipe,
    pub(super) recipes: BTreeMap<String, MapRecipe>,
    pub(super) structures: BTreeMap<String, Structure>,
    pub(super) weighted_names: Vec<String>,
    /// Every recipe name in file order: hub, dungeons, then arenas.
    #[cfg_attr(not(feature = "dev"), allow(dead_code))]
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

    /// Returns the recipe after `current` in file order (hub, dungeons, arenas), wrapping.
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

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub(super) struct RecipeFile {
    pub(super) defaults: RecipeDefaults,
    pub(super) structures: BTreeMap<String, StructureDefinition>,
    pub(super) hub: RecipeEntry,
    #[serde(default)]
    pub(super) dungeons: Vec<RecipeEntry>,
    #[serde(default)]
    pub(super) arenas: Vec<RecipeEntry>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub(super) struct StructureDefinition {
    pub(super) floor: TileKind,
    pub(super) glyphs: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub(super) struct RecipeDefaults {
    pub(super) size_x_range: (u32, u32),
    pub(super) size_y_range: (u32, u32),
    pub(super) floor: TileKind,
    pub(super) structures: Vec<StructureEntry>,
    pub(super) player_start: PlayerStartPick,
    pub(super) exits: ExitPick,
    pub(super) enemies: EnemyPlacement,
    pub(super) chests: CountRange,
}

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub(super) struct RecipeEntry {
    pub(super) name: Option<String>,
    pub(super) weight: Option<u32>,
    /// Shorthand for a square map: `size: 60` sets both size ranges to `(60, 60)`.
    pub(super) size: Option<u32>,
    pub(super) size_x_range: Option<(u32, u32)>,
    pub(super) size_y_range: Option<(u32, u32)>,
    pub(super) floor: Option<TileKind>,
    pub(super) structures: Option<Vec<StructureEntry>>,
    pub(super) player_start: Option<PlayerStartPick>,
    pub(super) exits: Option<ExitPick>,
    pub(super) enemies: Option<EnemyPlacement>,
    pub(super) chests: Option<CountRange>,
}

/// Resolved map generation settings loaded from a recipe definition.
#[derive(Debug, Clone)]
pub(crate) struct MapRecipe {
    pub(super) name: String,
    pub(super) weight: u32,
    pub(super) size_x_range: (u32, u32),
    pub(super) size_y_range: (u32, u32),
    pub(super) floor: TileKind,
    pub(super) structures: Vec<StructureEntry>,
    pub(super) player_start: PlayerStartPick,
    pub(super) exits: ExitPick,
    pub(super) enemies: EnemyPlacement,
    pub(super) chests: CountRange,
}

impl RecipeEntry {
    pub(super) fn resolve(
        self,
        name: String,
        weight: u32,
        defaults: &RecipeDefaults,
    ) -> Result<MapRecipe> {
        if self.size.is_some() && (self.size_x_range.is_some() || self.size_y_range.is_some()) {
            return Err(format!(
                "recipe {name} sets size together with size_x_range or size_y_range"
            )
            .into());
        }
        let square = self.size.map(|size| (size, size));
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
            player_start: self
                .player_start
                .unwrap_or_else(|| defaults.player_start.clone()),
            exits: self.exits.unwrap_or_else(|| defaults.exits.clone()),
            enemies: self.enemies.unwrap_or_else(|| defaults.enemies.clone()),
            chests: self.chests.unwrap_or(defaults.chests),
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

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub(super) struct StructureEntry {
    pub(super) kind: StructureKind,
    #[serde(default)]
    pub(super) name: String,
    #[serde(default)]
    pub(super) anchor: StructureAnchor,
    pub(super) radius: Option<u32>,
    /// Places this many copies, as if the entry were repeated; resolved names stay
    /// `{kind}_{index}`, so a named entry must keep `count: 1`.
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
#[derive(Deserialize, Debug, Clone)]
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

#[derive(Deserialize, Debug, Clone, Default)]
pub(super) enum StructureAnchor {
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
pub(super) fn resolved_name(kind: &str, index: usize, explicit: &str) -> String {
    if explicit.is_empty() {
        format!("{kind}_{index}")
    } else {
        explicit.to_owned()
    }
}

#[derive(Deserialize, Debug, Clone)]
pub(super) enum PlayerStartPick {
    Structure,
    Placement(PlacementPick),
}

/// Where a recipe's exits come from: `X` glyphs in its structures, or one map edge.
#[derive(Deserialize, Debug, Clone)]
pub(super) enum ExitPick {
    Structure,
    Edge { count: CountRange, side: Side },
}

#[derive(Deserialize, Debug, Clone)]
pub(super) enum PlacementPick {
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
    pub(super) fn resolve(&self, random_generator: &mut ChaCha8Rng) -> MapResult<PlaceRule> {
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

    pub(super) fn valid_player_start(&self) -> bool {
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
pub(super) struct EnemyPlacement {
    pub(super) count: CountRange,
    #[serde(default)]
    pub(super) pick: EnemyPick,
    #[serde(default = "default_enemy_x_deciles")]
    pub(super) x_deciles: (u32, u32),
    #[serde(default)]
    pub(super) separation: u32,
    #[serde(default)]
    pub(super) fixed: Vec<FixedEnemyPlacement>,
    pub(super) near: Option<EnemyNearPlacement>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub(super) struct FixedEnemyPlacement {
    pub(super) pick: EnemyPick,
    pub(super) anchor: Anchor,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub(super) struct EnemyNearPlacement {
    pub(super) structure: String,
    pub(super) radius: u32,
}

pub(super) fn default_enemy_x_deciles() -> (u32, u32) {
    (2, 8)
}

#[derive(Deserialize, Debug, Clone, Copy, Default)]
pub(super) enum EnemyPick {
    #[default]
    Warrior,
    IceMage,
    FireMage,
    Random,
}

impl EnemyPick {
    pub(super) fn resolve(self, random_generator: &mut ChaCha8Rng) -> EnemyKind {
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
            structure_from_glyphs(definition.floor, &definition.glyphs)
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
    let hub = file.hub.resolve(hub_name, 0, &defaults)?;
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
        let recipe = entry.resolve(name.clone(), weight, &defaults)?;
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

#[allow(clippy::too_many_lines)]
pub(super) fn validate_recipe(
    recipe: &MapRecipe,
    structures: &BTreeMap<String, Structure>,
) -> Result<()> {
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
        if let StructureKind::Structure(kind) = &structure.kind
            && !structures.contains_key(kind)
        {
            return Err(
                format!("recipe {} references unknown structure {kind}", recipe.name).into(),
            );
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
        if let Some(size) = structure.kind.feature_size() {
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
        let name = resolved_name(structure.kind.label(), index, &structure.name);
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
            _ => None,
        })
        .any(|structure| {
            structure
                .placements
                .iter()
                .any(|placement| wanted(&placement.kind))
        })
}
