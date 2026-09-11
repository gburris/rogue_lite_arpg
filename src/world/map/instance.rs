use std::{collections::BTreeMap, io::BufReader};

use crate::world::map::{
    TileType, blueprint::MapBlueprint, generation::generate_zone, prefabs::PrefabType,
};
use bevy::prelude::*;
use rand::distr::{Distribution, weighted::WeightedIndex};
use rand_chacha::ChaCha8Rng;
use ron::de::from_reader;
use serde::Deserialize;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_instance_data);
}

#[derive(Resource)]
pub struct MapRecipeCatalog {
    recipes: BTreeMap<String, MapRecipe>,
}

impl MapRecipeCatalog {
    pub fn get(&self, name: &str) -> Option<&MapRecipe> {
        self.recipes.get(name)
    }

    pub fn generate_named(
        &self,
        name: &str,
        rng: &mut ChaCha8Rng,
        zone_seed: u64,
    ) -> Result<MapBlueprint> {
        let recipe = self
            .get(name)
            .ok_or_else(|| BevyError::from(format!("unknown map recipe: {name}")))?;
        generate_zone(recipe, rng, zone_seed)
    }

    pub fn generate_random(&self, rng: &mut ChaCha8Rng, zone_seed: u64) -> Result<MapBlueprint> {
        let weights: Vec<u32> = self.recipes.values().map(|i| i.weight).collect();
        let dist = WeightedIndex::new(weights)?;
        let selected_index = dist.sample(rng);
        let map_recipe = self
            .recipes
            .values()
            .nth(selected_index)
            .ok_or(BevyError::from("instance not found"))?;
        generate_zone(map_recipe, rng, zone_seed)
    }

    pub fn generate_blueprint(&self, rng: &mut ChaCha8Rng, zone_seed: u64) -> Result<MapBlueprint> {
        self.generate_random(rng, zone_seed)
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
pub struct MapRecipe {
    name: String,
    weight: u32,
    pub size_x_range: (u32, u32),
    pub size_y_range: (u32, u32),
    pub number_of_enemies_range: (u32, u32),
    pub chest_range: (u32, u32),
    pub num_exits: u32,
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
        if self.num_exits > 2 {
            return Err("unsupported exit count".into());
        }
        Ok(())
    }
}

fn setup_instance_data(mut commands: Commands) {
    let recipes: Vec<MapRecipe> = load_instance_data();

    commands.insert_resource(MapRecipeCatalog::from(recipes));
}

#[cfg(not(target_arch = "wasm32"))]
use std::fs::File;

#[cfg(not(target_arch = "wasm32"))]
fn fetch_instance_data() -> File {
    File::open("assets/config/map_recipes.ron").expect("Failed to open RON file")
}

#[cfg(target_arch = "wasm32")]
fn fetch_instance_data() -> &'static [u8] {
    include_bytes!("../../../assets/config/map_recipes.ron")
}

fn load_instance_data() -> Vec<MapRecipe> {
    let reader = BufReader::new(fetch_instance_data());

    from_reader::<_, MapConfig>(reader)
        .unwrap_or_else(|e| {
            error!("Failed to parse RON file: {:?}", e);
            panic!("RON parsing error");
        })
        .recipes
}
