use std::io::BufReader;

use crate::world::map::{blueprint::MapBlueprint, generation::generate_zone, prefabs::PrefabType};
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
    recipes: Vec<MapRecipe>,
}

impl MapRecipeCatalog {
    pub fn generate_blueprint(
        &self,
        mut rng: &mut ChaCha8Rng,
        zone_seed: u64,
    ) -> Result<MapBlueprint> {
        let weights: Vec<u32> = self.recipes.iter().map(|i| i.weight).collect();
        let dist = WeightedIndex::new(weights)?;
        let selected_index = dist.sample(&mut rng);
        let map_recipe = self
            .recipes
            .get(selected_index)
            .ok_or(BevyError::from("Instance not found"))?;

        let blueprint = generate_zone(map_recipe, &mut rng, zone_seed)?;

        Ok(blueprint)
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
    pub size_x_range: (f32, f32),
    pub size_y_range: (f32, f32),
    pub number_of_enemies_range: (f32, f32),
    pub chest_range: (f32, f32),
    pub num_exits: u32,
    pub prefabs: Vec<PrefabType>,
    pub floor_type: String,
}

fn setup_instance_data(mut commands: Commands) {
    let recipes = load_instance_data();
    commands.insert_resource(MapRecipeCatalog { recipes });
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
