use rand::{
    RngExt,
    distr::{Distribution, weighted::WeightedIndex},
};
use rand_chacha::ChaCha8Rng;
use ron::de::from_reader;
use serde::Deserialize;
use std::{collections::BTreeMap, io::BufReader};

use bevy::{ecs::system::SystemParam, platform::collections::HashSet, prelude::*};

use crate::{
    prelude::{AppState, EnemyType},
    world::{
        MapRng,
        map::{
            blueprint::{MapBlueprint, Placement, PlacementKind},
            prefabs::{PrefabType, build_empty_square, build_hub, build_temple},
            tiles::{TileGrid, TileType},
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
    catalog: ResMut<'w, MapRecipeCatalog>,
    rng: ResMut<'w, MapRng>,
}

impl NextMap<'_, '_> {
    pub fn set(&mut self, recipe: RecipeSelection) -> Result<()> {
        let (mut map_rng, map_seed) = self.rng.map();
        info!("Generating map with seed: {}", map_seed);

        let blueprint = match recipe {
            RecipeSelection::WeightedRandom => self.catalog.generate_random(&mut map_rng, map_seed),
            RecipeSelection::Named(name) => {
                self.catalog.generate_named(name, &mut map_rng, map_seed)
            }
        }?;

        if let Some(previous_map) = self.previous_map.as_ref() {
            self.commands.entity(previous_map.entity()).despawn();
        }

        self.commands.spawn(blueprint);
        self.next_state.set(AppState::SpawnMap);

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RecipeSelection {
    WeightedRandom,
    Named(&'static str),
}

#[derive(Resource)]
struct MapRecipeCatalog {
    recipes: BTreeMap<String, MapRecipe>,
}

impl MapRecipeCatalog {
    fn get(&self, name: &str) -> Option<&MapRecipe> {
        self.recipes.get(name)
    }

    fn generate_named(
        &self,
        name: &str,
        rng: &mut ChaCha8Rng,
        map_seed: u64,
    ) -> Result<MapBlueprint> {
        let recipe = self
            .get(name)
            .ok_or_else(|| BevyError::from(format!("unknown map recipe: {name}")))?;
        generate_zone(recipe, rng, map_seed)
    }

    fn generate_random(&self, rng: &mut ChaCha8Rng, map_seed: u64) -> Result<MapBlueprint> {
        let weights: Vec<u32> = self.recipes.values().map(|i| i.weight).collect();
        let dist = WeightedIndex::new(weights)?;
        let selected_index = dist.sample(rng);
        let map_recipe = self
            .recipes
            .values()
            .nth(selected_index)
            .ok_or(BevyError::from("instance not found"))?;
        generate_zone(map_recipe, rng, map_seed)
    }
}

impl From<Vec<MapRecipe>> for MapRecipeCatalog {
    fn from(recipes: Vec<MapRecipe>) -> Self {
        Self {
            recipes: recipes
                .into_iter()
                .map(|recipe| (recipe.name.clone(), recipe))
                .collect(),
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct MapConfig {
    pub recipes: Vec<MapRecipe>,
}
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct MapRecipe {
    name: String,
    weight: u32,
    pub size_x_range: (u32, u32),
    pub size_y_range: (u32, u32),
    pub number_of_enemies_range: (u32, u32),
    pub chest_range: (u32, u32),
    pub num_random_exits: u32,
    pub prefabs: Vec<PrefabType>,
    pub floor_type: TileType,
}

impl MapRecipe {
    pub fn validate(&self) -> Result<()> {
        if self.weight == 0 {
            return Err("recipe weight must be positive".into());
        }
        for (name, range) in [("size_x", self.size_x_range), ("size_y", self.size_y_range)] {
            if range.0 == 0 || range.0 > range.1 {
                return Err(format!("invalid {name} range").into());
            }
        }
        for (name, range) in [
            ("enemies", self.number_of_enemies_range),
            ("chests", self.chest_range),
        ] {
            if range.0 > range.1 {
                return Err(format!("invalid {name} range").into());
            }
        }
        if self.num_random_exits > 2 {
            return Err("unsupported exit count".into());
        }
        Ok(())
    }
}

fn setup_instance_data(mut commands: Commands) -> Result {
    let recipes: Vec<MapRecipe> = load_map_recipes();
    for recipe in &recipes {
        recipe.validate()?;
    }

    commands.insert_resource(MapRecipeCatalog::from(recipes));
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
use std::fs::File;

#[cfg(not(target_arch = "wasm32"))]
fn fetch_recipe_data() -> File {
    File::open("assets/config/map_recipes.ron").expect("Failed to open RON file")
}

#[cfg(target_arch = "wasm32")]
fn fetch_recipe_data() -> &'static [u8] {
    include_bytes!("../../../assets/config/map_recipes.ron")
}

fn load_map_recipes() -> Vec<MapRecipe> {
    let reader = BufReader::new(fetch_recipe_data());

    from_reader::<_, MapConfig>(reader)
        .unwrap_or_else(|e| {
            error!("Failed to parse RON file: {:?}", e);
            panic!("RON parsing error");
        })
        .recipes
}

fn generate_zone(
    map_recipe: &MapRecipe,
    rng: &mut ChaCha8Rng,
    map_seed: u64,
) -> Result<MapBlueprint> {
    let size_x = rng.random_range(map_recipe.size_x_range.0..=map_recipe.size_x_range.1);
    let size_y = rng.random_range(map_recipe.size_y_range.0..=map_recipe.size_y_range.1);

    let floor_type = map_recipe.floor_type;

    let mut blueprint = MapBlueprint::new(UVec2::new(size_x, size_y), floor_type, map_seed);

    // Build prefabs
    for prefab in &map_recipe.prefabs {
        match prefab {
            PrefabType::Temple => build_temple(&mut blueprint),
            PrefabType::EmptySquare => build_empty_square(&mut blueprint),
            PrefabType::HubTown => build_hub(&mut blueprint),
        }
    }

    let mut occupied: HashSet<IVec2> = blueprint.placements.iter().map(|p| p.tile).collect();

    // Generate player spawn and exits
    let (player_start, exits) =
        generate_entrance_exit_positions(blueprint.map_size(), map_recipe.num_random_exits, rng);

    for exit in exits {
        if !blueprint
            .tiles
            .get(exit.tile)
            .is_some_and(|t: TileType| t.can_spawn())
        {
            return Err(format!("exit tile {:?} is not spawnable", exit.tile).into());
        }

        if !occupied.insert(exit.tile) {
            return Err(format!("exit tile {:?} is already occupied", exit.tile).into());
        }

        if !blueprint.tiles.contains(exit.doorway) {
            return Err(format!("exit doorway {:?} is out of bounds", exit.doorway).into());
        }

        blueprint.placements.push(Placement {
            tile: exit.tile,
            kind: PlacementKind::Exit,
        });
    }

    // Player start
    let has_existing_player_start = blueprint
        .placements
        .iter()
        .any(|p| matches!(p.kind, PlacementKind::PlayerStart));
    if !has_existing_player_start {
        if !blueprint
            .tiles
            .get(player_start)
            .is_some_and(|t: TileType| t.can_spawn())
        {
            return Err(format!("player start {player_start:?} is not spawnable").into());
        }

        if !occupied.insert(player_start) {
            return Err(format!("player start {player_start:?} is already occupied").into());
        }

        blueprint.placements.push(Placement {
            tile: player_start,
            kind: PlacementKind::PlayerStart,
        });
    }

    // Determine number of enemies and chests from the recipe ranges
    let num_enemies = rng
        .random_range(map_recipe.number_of_enemies_range.0..=map_recipe.number_of_enemies_range.1);
    let num_chests = rng.random_range(map_recipe.chest_range.0..=map_recipe.chest_range.1);

    // Generate enemy positions (avoid walls)
    let enemy_positions =
        find_multiple_positions(&blueprint.tiles, 2..8, num_enemies, rng, &mut occupied);
    for pos in enemy_positions {
        // Randomly choose an enemy type (or use a fixed set; here we pick randomly)
        let enemy_type = match rng.random_range(0..3) {
            0 => EnemyType::FireMage,
            1 => EnemyType::IceMage,
            _ => EnemyType::Warrior,
        };
        blueprint.placements.push(Placement {
            tile: pos,
            kind: PlacementKind::Enemy(enemy_type),
        });
    }

    // Generate chest positions
    let chest_positions =
        find_multiple_positions(&blueprint.tiles, 1..9, num_chests, rng, &mut occupied);
    for pos in chest_positions {
        blueprint.placements.push(Placement {
            tile: pos,
            kind: PlacementKind::Chest,
        });
    }

    blueprint.validate().map_err(|error| error.to_string())?;
    Ok(blueprint)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum MapOrientation {
    Horizontal,
    Vertical,
    Square,
}

struct ExitPlacement {
    tile: IVec2,
    doorway: IVec2,
}

fn generate_entrance_exit_positions(
    map_size: UVec2,
    num_random_exits: u32,
    rng: &mut ChaCha8Rng,
) -> (IVec2, Vec<ExitPlacement>) {
    let orientation = determine_map_orientation(map_size);

    let player_start = match orientation {
        MapOrientation::Horizontal | MapOrientation::Square => {
            // For horizontal and square maps: left to right
            let player_x = 1; // One tile in from left wall
            let player_y = rng.random_range(1..map_size.y - 1) as i32;
            IVec2::new(player_x, player_y)
        }
        MapOrientation::Vertical => {
            // For vertical maps: top to bottom
            let player_y = map_size.y as i32 - 2; // One tile down from top wall
            let player_x = rng.random_range(1..map_size.x - 1) as i32;
            IVec2::new(player_x, player_y)
        }
    };

    let mut exits = match orientation {
        MapOrientation::Horizontal | MapOrientation::Square => {
            // Two exits on the right side
            let exit_y1 = rng.random_range(1..map_size.y / 2) as i32;
            let exit_y2 = rng.random_range(map_size.y / 2..map_size.y - 1) as i32;

            let interior_x = map_size.x as i32 - 2;
            let boundary_x = map_size.x as i32 - 1;

            vec![
                ExitPlacement {
                    tile: IVec2::new(interior_x, exit_y1),
                    doorway: IVec2::new(boundary_x, exit_y1),
                },
                ExitPlacement {
                    tile: IVec2::new(interior_x, exit_y2),
                    doorway: IVec2::new(boundary_x, exit_y2),
                },
            ]
        }
        MapOrientation::Vertical => {
            // Two exits at the bottom
            let exit_x1 = rng.random_range(1..map_size.x / 2) as i32;
            let exit_x2 = rng.random_range(map_size.x / 2..map_size.x - 1) as i32;

            vec![
                ExitPlacement {
                    tile: IVec2::new(exit_x1, 1),
                    doorway: IVec2::new(exit_x1, 0),
                },
                ExitPlacement {
                    tile: IVec2::new(exit_x2, 1),
                    doorway: IVec2::new(exit_x2, 0),
                },
            ]
        }
    };
    exits.truncate(num_random_exits as usize);

    (player_start, exits)
}

fn determine_map_orientation(map_size: UVec2) -> MapOrientation {
    let aspect_ratio = map_size.x as f32 / map_size.y as f32;

    if (aspect_ratio - 1.0).abs() < 0.1 {
        MapOrientation::Square
    } else if aspect_ratio > 1.0 {
        MapOrientation::Horizontal
    } else {
        MapOrientation::Vertical
    }
}

fn find_multiple_positions(
    tiles: &TileGrid,
    x_range: std::ops::Range<u32>,
    count: u32,
    rng: &mut ChaCha8Rng,
    occupied: &mut HashSet<IVec2>,
) -> Vec<IVec2> {
    let mut positions: Vec<IVec2> = Vec::new();
    let mut attempts = 0;

    while positions.len() < count as usize && attempts < 100 {
        if let Some(pos) = find_valid_position(tiles, x_range.clone(), rng, occupied)
            && positions
                .iter()
                .all(|existing| (*existing - pos).length_squared() >= 25)
        {
            positions.push(pos);
            occupied.insert(pos);
        }
        attempts += 1;
    }

    positions
}

fn find_valid_position(
    tiles: &TileGrid,
    x_range: std::ops::Range<u32>,
    rng: &mut ChaCha8Rng,
    occupied: &mut HashSet<IVec2>,
) -> Option<IVec2> {
    let x_start = tiles.map_size().x * x_range.start / 10;
    let x_end = tiles.map_size().x * x_range.end / 10;
    for _ in 0..100 {
        let random_tile = IVec2::new(
            rng.random_range(x_start..x_end) as i32,
            rng.random_range(1..tiles.map_size().y - 1) as i32,
        );

        if !occupied.contains(&random_tile) && tiles.get(random_tile).is_some_and(|t| t.can_spawn())
        {
            return Some(random_tile);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Resource, Default)]
    struct RequestResult(Option<bool>);

    fn request_unknown_recipe(mut next_map: NextMap, mut result: ResMut<RequestResult>) {
        result.0 = Some(next_map.set(RecipeSelection::Named("missing")).is_err());
    }

    #[test]
    fn unknown_recipe_keeps_current_map_and_does_not_request_transition() {
        let mut app = App::new();
        app.insert_resource(MapRecipeCatalog {
            recipes: BTreeMap::new(),
        });
        app.insert_resource(MapRng::seeded(1));
        app.insert_resource(NextState::<AppState>::Unchanged);
        app.insert_resource(RequestResult::default());
        app.world_mut()
            .spawn(MapBlueprint::new(UVec2::ONE, TileType::Grass, 0));
        app.add_systems(Update, request_unknown_recipe);

        app.update();

        assert_eq!(
            app.world().resource::<RequestResult>().0,
            Some(true),
            "unknown recipe should return an error",
        );
        let mut maps = app.world_mut().query::<&MapBlueprint>();
        assert_eq!(
            maps.iter(app.world()).count(),
            1,
            "current map should remain"
        );
        assert!(
            matches!(
                app.world().resource::<NextState<AppState>>(),
                NextState::Unchanged
            ),
            "failed build should not queue a transition",
        );
    }
}
