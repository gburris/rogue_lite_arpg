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
    prelude::{AppState, EnemyType},
    world::{
        MapRng,
        map::{
            blueprint::{MapBlueprint, MapError, Placement, PlacementKind},
            place::{Anchor, CountRange, PlaceContext, PlaceRule, Side},
            prefabs::{PrefabType, build_empty_square, build_temple},
            structures::{StructureKind, apply_structure},
            tiles::TileType,
        },
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_instance_data);
}

#[derive(SystemParam)]
pub struct NextMap<'w, 's> {
    commands: Commands<'w, 's>,
    previous_map: Option<Single<'w, 's, Entity, With<MapBlueprint>>>,
    next_state: ResMut<'w, NextState<AppState>>,
    catalog: Res<'w, RecipeCatalog>,
    rng: ResMut<'w, MapRng>,
}

impl NextMap<'_, '_> {
    pub fn set(&mut self, request: MapRequest) -> Result<()> {
        let (recipe, seed) = match request {
            MapRequest::Hub => (self.catalog.hub(), self.rng.map().1),
            MapRequest::Weighted => {
                let seed = self.rng.map().1;
                let mut rng = ChaCha8Rng::seed_from_u64(seed);
                (self.catalog.weighted(&mut rng)?, seed)
            }
            MapRequest::Named { recipe, seed } => (self.catalog.named(&recipe)?, seed),
        };
        let blueprint = build_from_recipe(recipe, seed)?;

        if let Some(previous_map) = self.previous_map.as_ref() {
            self.commands.entity(previous_map.entity()).despawn();
        }
        self.commands.spawn(blueprint);
        self.next_state.set(AppState::SpawnMap);
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MapRequest {
    Hub,
    Weighted,
    Named { recipe: String, seed: u64 },
}

#[derive(Resource, Debug)]
pub struct RecipeCatalog {
    hub: MapRecipe,
    recipes: BTreeMap<String, MapRecipe>,
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

    fn weighted(&self, rng: &mut ChaCha8Rng) -> Result<&MapRecipe> {
        let weights = self
            .weighted_names
            .iter()
            .map(|name| self.recipes[name].weight)
            .collect::<Vec<_>>();
        let index = WeightedIndex::new(weights)?;
        let name = &self.weighted_names[index.sample(rng)];
        self.recipes
            .get(name)
            .ok_or_else(|| "weighted recipe catalog is empty".into())
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct RecipeFile {
    defaults: RecipeDefaults,
    hub: RecipeEntry,
    #[serde(default)]
    dungeons: Vec<RecipeEntry>,
    #[serde(default)]
    arenas: Vec<RecipeEntry>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
struct RecipeDefaults {
    size_x_range: (u32, u32),
    size_y_range: (u32, u32),
    floor_type: TileType,
    structures: Vec<StructureEntry>,
    player_start: PlayerStartPick,
    exits: PlacementPick,
    enemies: EnemyPlacement,
    chests: RecipeCountRange,
}

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
struct RecipeEntry {
    name: Option<String>,
    weight: Option<u32>,
    size_x_range: Option<(u32, u32)>,
    size_y_range: Option<(u32, u32)>,
    floor_type: Option<TileType>,
    structures: Option<Vec<StructureEntry>>,
    player_start: Option<PlayerStartPick>,
    exits: Option<PlacementPick>,
    enemies: Option<EnemyPlacement>,
    chests: Option<RecipeCountRange>,
}

#[derive(Debug, Clone)]
pub(crate) struct MapRecipe {
    name: String,
    weight: u32,
    size_x_range: (u32, u32),
    size_y_range: (u32, u32),
    floor_type: TileType,
    structures: Vec<StructureEntry>,
    player_start: PlayerStartPick,
    exits: PlacementPick,
    enemies: EnemyPlacement,
    chests: RecipeCountRange,
}

impl RecipeEntry {
    fn resolve(self, name: String, weight: u32, defaults: &RecipeDefaults) -> MapRecipe {
        MapRecipe {
            name,
            weight,
            size_x_range: self.size_x_range.unwrap_or(defaults.size_x_range),
            size_y_range: self.size_y_range.unwrap_or(defaults.size_y_range),
            floor_type: self.floor_type.unwrap_or(defaults.floor_type),
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
    kind: PrefabType,
    #[serde(default)]
    anchor: StructureAnchor,
    radius: Option<u32>,
}

#[derive(Deserialize, Debug, Clone, Copy, Default)]
enum StructureAnchor {
    #[default]
    Random,
    Center,
    CenterOffset(IVec2),
    At(IVec2),
    Near {
        structure: usize,
        offset: IVec2,
    },
}

#[derive(Deserialize, Debug, Clone, Copy)]
enum PlacementSide {
    Left,
    Right,
    Top,
    Bottom,
}

impl From<PlacementSide> for Side {
    fn from(value: PlacementSide) -> Self {
        match value {
            PlacementSide::Left => Self::Left,
            PlacementSide::Right => Self::Right,
            PlacementSide::Top => Self::Top,
            PlacementSide::Bottom => Self::Bottom,
        }
    }
}

#[derive(Deserialize, Debug, Clone, Copy)]
enum PlacementAnchor {
    Center,
    CenterOffset(IVec2),
    At(IVec2),
}

impl From<PlacementAnchor> for Anchor {
    fn from(value: PlacementAnchor) -> Self {
        match value {
            PlacementAnchor::Center => Self::Center,
            PlacementAnchor::CenterOffset(offset) => Self::CenterOffset(offset),
            PlacementAnchor::At(position) => Self::At(position),
        }
    }
}

#[derive(Deserialize, Debug, Clone, Copy)]
struct RecipeCountRange(u32, u32);

impl RecipeCountRange {
    fn roll_size(self, rng: &mut ChaCha8Rng) -> u32 {
        rng.random_range(self.0..=self.1)
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
        anchor: PlacementAnchor,
    },
    Random {
        count: RecipeCountRange,
        x_deciles: (u32, u32),
        separation: u32,
    },
    Edge {
        count: RecipeCountRange,
        side: PlacementSide,
    },
    Choose(Vec<PlacementPick>),
}

impl PlacementPick {
    fn resolve(&self, rng: &mut ChaCha8Rng) -> PlaceRule {
        match self {
            Self::Fixed { anchor } => PlaceRule::Fixed((*anchor).into()),
            Self::Random {
                count,
                x_deciles,
                separation,
            } => PlaceRule::Random {
                count: CountRange(count.0, count.1),
                x_deciles: *x_deciles,
                separation: *separation,
            },
            Self::Edge { count, side } => PlaceRule::Edge {
                count: CountRange(count.0, count.1),
                side: (*side).into(),
            },
            Self::Choose(picks) => picks[rng.random_range(0..picks.len())].resolve(rng),
        }
    }

    fn random_only(&self) -> bool {
        match self {
            Self::Random {
                count, x_deciles, ..
            } => count.0 == 1 && count.1 == 1 && x_deciles.0 < x_deciles.1 && x_deciles.1 <= 10,
            Self::Choose(picks) => !picks.is_empty() && picks.iter().all(Self::random_only),
            _ => false,
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
struct EnemyPlacement {
    count: RecipeCountRange,
    #[serde(default)]
    pick: EnemyPick,
    #[serde(default = "default_enemy_x_deciles")]
    x_deciles: (u32, u32),
    #[serde(default)]
    separation: u32,
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
    fn resolve(self, rng: &mut ChaCha8Rng) -> EnemyType {
        match self {
            Self::Warrior => EnemyType::Warrior,
            Self::IceMage => EnemyType::IceMage,
            Self::FireMage => EnemyType::FireMage,
            Self::Random => match rng.random_range(0..3) {
                0 => EnemyType::Warrior,
                1 => EnemyType::IceMage,
                _ => EnemyType::FireMage,
            },
        }
    }
}

fn setup_instance_data(mut commands: Commands) -> Result {
    let file = load_recipe_file()?;
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
    validate_recipe(&hub, true)?;
    for seed in 0..=8 {
        build_from_recipe(&hub, seed)?;
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
        validate_recipe(&recipe, false)?;
        for seed in 0..=8 {
            build_from_recipe(&recipe, seed)?;
        }
        weighted_names.push(name.clone());
        recipes.insert(name, recipe);
    }
    if recipes.is_empty() {
        return Err("at least one dungeon or arena recipe is required".into());
    }
    commands.insert_resource(RecipeCatalog {
        hub,
        recipes,
        weighted_names,
    });
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
use std::fs::File;

#[cfg(not(target_arch = "wasm32"))]
fn fetch_recipe_data() -> Result<File> {
    File::open("assets/config/map_recipes.ron").map_err(Into::into)
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
        .map_err(Into::into)
}

fn validate_recipe(recipe: &MapRecipe, hub: bool) -> Result<()> {
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
        || recipe.chests.0 > recipe.chests.1
        || recipe.chests.1 > 512
    {
        return Err(format!("recipe {} has invalid placement count range", recipe.name).into());
    }
    if enemy_min > 0
        && (recipe.enemies.x_deciles.0 >= recipe.enemies.x_deciles.1
            || recipe.enemies.x_deciles.1 > 10)
    {
        return Err(format!("recipe {} has invalid enemy x-decile range", recipe.name).into());
    }
    if !matches!(&recipe.exits, PlacementPick::Edge { .. }) {
        return Err(format!("recipe {} exits must use an Edge placement", recipe.name).into());
    }
    if let PlacementPick::Edge { count, .. } = &recipe.exits {
        if count.0 == 0 || count.0 > count.1 || count.1 > 2 {
            return Err(format!("recipe {} has invalid exit count range", recipe.name).into());
        }
    }
    for (index, structure) in recipe.structures.iter().enumerate() {
        if let StructureAnchor::Near {
            structure: target, ..
        } = structure.anchor
        {
            if target >= index {
                return Err(format!(
                    "recipe {} structure {index} anchors to a structure that is not earlier",
                    recipe.name
                )
                .into());
            }
        } else if structure.radius.is_some() {
            return Err(format!(
                "recipe {} structure {index} radius requires a Near anchor",
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
    }
    let structure_starts = recipe
        .structures
        .iter()
        .any(|s| matches!(s.kind, PrefabType::HubTown));
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
    if let PlayerStartPick::Placement(PlacementPick::Choose(picks)) = &recipe.player_start {
        if picks.is_empty() {
            return Err(format!("recipe {} has an empty player-start choice", recipe.name).into());
        }
    }
    if !hub {
        if let PlayerStartPick::Placement(pick) = &recipe.player_start {
            if !pick.random_only() {
                return Err(format!(
                    "recipe {} weighted player-start choices must use Random rules",
                    recipe.name
                )
                .into());
            }
        }
    }
    Ok(())
}

pub(crate) fn build_from_recipe(
    recipe: &MapRecipe,
    seed: u64,
) -> std::result::Result<MapBlueprint, MapError> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let size_x = rng.random_range(recipe.size_x_range.0..=recipe.size_x_range.1);
    let size_y = rng.random_range(recipe.size_y_range.0..=recipe.size_y_range.1);
    let mut blueprint = MapBlueprint::new(UVec2::new(size_x, size_y), recipe.floor_type, seed);
    let mut placement_context = PlaceContext::new(blueprint.map_size());
    let mut structure_centers = Vec::with_capacity(recipe.structures.len());
    for (index, entry) in recipe.structures.iter().enumerate() {
        let center_count = placement_context.structure_centers.len();
        let rule = match entry.anchor {
            StructureAnchor::Random => PlaceRule::Random {
                count: CountRange(1, 1),
                x_deciles: (1, 9),
                separation: 0,
            },
            StructureAnchor::Center => PlaceRule::Fixed(Anchor::Center),
            StructureAnchor::CenterOffset(offset) => PlaceRule::Fixed(Anchor::CenterOffset(offset)),
            StructureAnchor::At(position) => PlaceRule::Fixed(Anchor::At(position)),
            StructureAnchor::Near {
                structure: earlier,
                offset,
            } => {
                let center = structure_centers
                    .get(earlier)
                    .and_then(|center| *center)
                    .ok_or("structure anchor missing")?;
                let mut min = center + offset;
                if let Some(radius) = entry.radius {
                    let radius =
                        i32::try_from(radius).map_err(|_| "structure radius is too large")?;
                    min += IVec2::new(
                        rng.random_range(-radius..=radius),
                        rng.random_range(-radius..=radius),
                    );
                }
                PlaceRule::Fixed(Anchor::At(min))
            }
        };
        match entry.kind {
            PrefabType::Temple => {
                build_temple(&mut blueprint, &mut placement_context, &rule, &mut rng)?
            }
            PrefabType::EmptySquare => {
                build_empty_square(&mut blueprint, &mut placement_context, &rule, &mut rng)?
            }
            PrefabType::HubTown => {
                let structure = StructureKind::HubTown.picture();
                let positions = super::place::place(
                    &rule,
                    structure.size().as_uvec2(),
                    &blueprint.tiles,
                    &mut placement_context,
                    &mut rng,
                )?;
                if let Some(min) = positions.first() {
                    apply_structure(&mut blueprint, structure, *min)
                        .map_err(|error| format!("failed to place structure {index}: {error:?}"))?;
                }
            }
        }
        if placement_context.structure_centers.len() == center_count {
            warn!(
                "Recipe {} seed {} could not place structure {index} ({:?})",
                recipe.name, seed, entry.kind
            );
        }
        structure_centers.push(
            placement_context
                .structure_centers
                .get(center_count)
                .copied(),
        );
    }

    let mut occupied = blueprint
        .placements
        .iter()
        .map(|placement| placement.tile)
        .collect::<std::collections::HashSet<_>>();
    placement_context.occupied.extend(occupied.iter().copied());
    let player_start = match &recipe.player_start {
        PlayerStartPick::Structure => None,
        PlayerStartPick::Placement(pick) => {
            let rule = pick.resolve(&mut rng);
            Some(
                super::place::place(
                    &rule,
                    UVec2::ONE,
                    &blueprint.tiles,
                    &mut placement_context,
                    &mut rng,
                )?
                .into_iter()
                .next()
                .ok_or("failed to place player start")?,
            )
        }
    };
    let exit_rule = recipe.exits.resolve(&mut rng);
    let exits = super::place::place(
        &exit_rule,
        UVec2::ONE,
        &blueprint.tiles,
        &mut placement_context,
        &mut rng,
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
            } => IVec2::new(tile.x, 0),
            PlaceRule::Edge {
                side: Side::Bottom, ..
            } => IVec2::new(tile.x, blueprint.map_size().y as i32 - 1),
            _ => return Err("exit placement must use an Edge rule".into()),
        };
        if !occupied.insert(tile) {
            return Err(format!("exit tile {tile:?} is already occupied").into());
        }
        if !blueprint.tiles.contains(doorway) {
            return Err(format!("exit doorway {doorway:?} is out of bounds").into());
        }
        blueprint.placements.push(Placement {
            tile,
            kind: PlacementKind::Exit,
        });
    }
    if let Some(player_start) = player_start {
        if !occupied.insert(player_start) {
            return Err(format!("player start {player_start:?} is already occupied").into());
        }
        blueprint.placements.push(Placement {
            tile: player_start,
            kind: PlacementKind::PlayerStart,
        });
    } else if !blueprint
        .placements
        .iter()
        .any(|p| matches!(p.kind, PlacementKind::PlayerStart))
    {
        return Err("structure-based PlayerStart source did not place a PlayerStart".into());
    }

    let enemy_count = recipe.enemies.count.roll_size(&mut rng);
    let enemy_positions = super::place::place(
        &PlaceRule::Random {
            count: CountRange(enemy_count, enemy_count),
            x_deciles: recipe.enemies.x_deciles,
            separation: recipe.enemies.separation,
        },
        UVec2::ONE,
        &blueprint.tiles,
        &mut placement_context,
        &mut rng,
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
            kind: PlacementKind::Enemy(recipe.enemies.pick.resolve(&mut rng)),
        });
    }
    let chest_count = recipe.chests.roll_size(&mut rng);
    let chest_positions = super::place::place(
        &PlaceRule::Random {
            count: CountRange(chest_count, chest_count),
            x_deciles: (1, 9),
            separation: 0,
        },
        UVec2::ONE,
        &blueprint.tiles,
        &mut placement_context,
        &mut rng,
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
    blueprint.validate().map_err(MapError::Validation)?;
    Ok(blueprint)
}
