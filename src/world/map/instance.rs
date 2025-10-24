use std::{collections::HashMap, io::BufReader};

use bevy::{prelude::*, scene::ron::de::from_reader};
use bevy_ecs_tilemap::prelude::*;
use rand::{
    Rng,
    distr::{Distribution, weighted::WeightedIndex},
};
use serde::Deserialize;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_instance_data);
}

#[derive(Resource)]
pub struct InstanceAssets {
    pub instance_config: HashMap<String, InstanceType>,
}

impl InstanceAssets {
    pub fn generate_map_layout(&self) -> Result<MapLayout> {
        let mut rng = rand::rng();

        let instance_names = [
            "Swamp",
            "SwampWithALotOfEmptySquares",
            "LongHallway",
            "TreasureRoom",
        ];
        let weights = [40, 25, 25, 10];

        let dist = WeightedIndex::new(weights)?;
        let selected_index = dist.sample(&mut rng);
        let instance_type = self
            .instance_config
            .get(instance_names[selected_index])
            .ok_or(BevyError::from("Instance name not found"))?;

        let size_x =
            rng.random_range(instance_type.size_x_range.0..=instance_type.size_x_range.1) as u32;
        let size_y =
            rng.random_range(instance_type.size_y_range.0..=instance_type.size_y_range.1) as u32;
        let map_size = TilemapSize {
            x: size_x,
            y: size_y,
        };
        let num_enemies = rng.random_range(
            instance_type.number_of_enemies_range.0..=instance_type.number_of_enemies_range.1,
        ) as u32;
        let num_chests =
            rng.random_range(instance_type.chest_range.0..=instance_type.chest_range.1) as u32;

        let floor_type = match instance_type.floor_type.as_str() {
            "Ground" => TileType::Ground,
            "Cobblestone" => TileType::Cobblestone,
            "Grass" => TileType::Grass,
            _ => {
                warn!(
                    "Unknown floor type: {}, defaulting to Grass",
                    instance_type.floor_type
                );
                TileType::Grass
            }
        };

        let mut builder = MapDataBuilder::new(map_size);

        for prefab_str in &instance_type.prefabs {
            match prefab_str.parse() {
                Ok(prefab) => builder = builder.with_prefab(prefab),
                Err(e) => warn!("{}", e),
            }
        }

        let map_data = builder
            .with_floor(floor_type) //Floor really needs to go first, you don't wanna know what happens if it doesn't
            .with_exterior_walls()
            .with_chests(num_chests)
            .with_exits(instance_type.num_exits)
            .with_enemies(num_enemies)
            .build();

        Ok(MapLayout::from(map_data))
    }
}

#[derive(Deserialize, Debug)]
struct InstanceConfig {
    pub instances: HashMap<String, InstanceType>,
}

#[derive(Deserialize, Debug)]
struct InstanceType {
    pub size_x_range: (f32, f32),
    pub size_y_range: (f32, f32),
    pub number_of_enemies_range: (f32, f32),
    pub num_exits: u32,
    pub chest_range: (f32, f32),
    pub prefabs: Vec<String>,
    pub floor_type: String,
}

fn setup_instance_data(mut commands: Commands) {
    let instance_config = load_instance_data();
    commands.insert_resource(InstanceAssets { instance_config });
}

#[cfg(not(target_arch = "wasm32"))]
use std::fs::File;

use crate::world::map::{MapLayout, TileType, map_data::MapDataBuilder};

#[cfg(not(target_arch = "wasm32"))]
fn fetch_instance_data() -> File {
    File::open("assets/config/instances.ron").expect("Failed to open RON file")
}

#[cfg(target_arch = "wasm32")]
fn fetch_instance_data() -> &'static [u8] {
    include_bytes!("../../../assets/config/instances.ron")
}

fn load_instance_data() -> HashMap<String, InstanceType> {
    let reader = BufReader::new(fetch_instance_data());

    from_reader::<_, InstanceConfig>(reader)
        .unwrap_or_else(|e| {
            error!("Failed to parse RON file: {:?}", e);
            panic!("RON parsing error");
        })
        .instances
}
