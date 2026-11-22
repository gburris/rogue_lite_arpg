use std::collections::BTreeMap;
use std::io::BufReader;

use bevy::prelude::*;
use rand::Rng;
use rand::distr::Distribution;
use rand::distr::weighted::WeightedIndex;
use ron::Options;
use ron::extensions::Extensions;

use super::blueprint::PlacementKind;
use super::build::{MapRequest, build_from_recipe};
use super::placement::CountRange;
use super::recipe::{
    Location, MapRecipe, RecipeDefaults, RecipeFile, SpawnKind, StructureKind, expand_counts,
    resolved_name,
};
use super::spawn::MAX_MAP_TILES;
use super::structures::{FeatureSize, Structure, structure_from_glyphs};
use crate::prelude::*;

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
                let recipe = self.weighted(&mut **root_rng)?;
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

pub(super) fn load_recipes(mut commands: Commands) -> Result {
    commands.insert_resource(load_recipe_catalog()?);
    Ok(())
}

pub(super) fn load_recipe_catalog() -> Result<RecipeCatalog> {
    catalog_from_file(load_recipe_file()?)
}

/// Resolves, validates and smoke-builds every recipe in a parsed recipe file.
fn catalog_from_file(file: RecipeFile) -> Result<RecipeCatalog> {
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

fn validate_recipe(recipe: &MapRecipe, structures: &BTreeMap<String, Structure>) -> Result<()> {
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
        Location::Random { x_deciles } if x_deciles.min >= x_deciles.max || x_deciles.max > 10 => {
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
fn structures_place(
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
