use baba_yaga::map::components::{MapMarker, TileType};
use baba_yaga::map::helpers::map_layout::generate_map_layout;
use bevy_ecs_tilemap::map::TilemapSize;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_map_layout() {
        let map_size = TilemapSize { x: 50, y: 50 };
        let map_layout = generate_map_layout(map_size);

        // Check if the map layout has the correct dimensions
        assert_eq!(map_layout.tiles.len(), map_size.x as usize);
        assert_eq!(map_layout.tiles[0].len(), map_size.y as usize);

        // Verify that the map contains at least some walls
        let wall_count = map_layout
            .tiles
            .iter()
            .flat_map(|row| row.iter())
            .filter(|&&tile| tile == TileType::Wall)
            .count();

        assert!(wall_count > 0, "Map should contain walls");

        // Verify that the map contains at least some water bodies (ponds)
        let water_count = map_layout
            .tiles
            .iter()
            .flat_map(|row| row.iter())
            .filter(|&&tile| tile == TileType::Water)
            .count();

        assert!(water_count > 0, "Map should contain water bodies");

        // Ensure the map contains at least one player spawn marker
        let player_spawn_exists = map_layout
            .markers
            .iter()
            .any(|marker| matches!(marker, MapMarker::PlayerSpawn(_)));

        assert!(
            player_spawn_exists,
            "Map should contain player spawn marker"
        );
    }
}
