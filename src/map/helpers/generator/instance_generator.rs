use bevy::prelude::*;
use bevy_ecs_tilemap::map::TilemapSize;
use rand::prelude::Distribution;
use rand::{Rng, distr::weighted::WeightedIndex};

use crate::map::components::{InstanceAssets, MapLayout, TileType};

use super::map_data::{MapDataBuilder, PrefabType};

pub fn generate_instance_layout(instance_assets: &Res<InstanceAssets>) -> Result<MapLayout> {
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
    let instance_type = instance_assets
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
        let prefab = match prefab_str.as_str() {
            "Temple" => PrefabType::Temple,
            "EmptySquare" => PrefabType::EmptySquare,
            "NPCHub" => PrefabType::NPCHub, // Added since you mentioned it
            _ => {
                warn!("Unknown prefab type: {}", prefab_str);
                continue;
            }
        };
        builder = builder.with_prefab(prefab);
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
