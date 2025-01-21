use baba_yaga::map::components::TileType;
use baba_yaga::map::helpers::map_layout::generate_map_layout;
use bevy_ecs_tilemap::map::TilemapSize;

/*
These are actually recognized by Rust as an integration test.
But the code is a unit test.
By default, rust wants unit tests to be in the src/ folder, right next to the code it tests
I don't want that.
Downside: The integration tests must be top level in tests/
*/

#[cfg(test)]
mod tests {
    use super::*; // This brings the imports into the test module's scope

    #[test]
    fn test_generate_map_layout() {
        let map_size = TilemapSize { x: 50, y: 50 };
        let map_layout = generate_map_layout(map_size);

        // Check if the map layout has the correct dimensions
        assert_eq!(map_layout.len(), map_size.x as usize);
        assert_eq!(map_layout[0].len(), map_size.y as usize);

        // Verify that the map contains at least some walls
        let wall_count = map_layout
            .iter()
            .flat_map(|row| row.iter())
            .filter(|&&tile| tile == TileType::Wall)
            .count();

        assert!(wall_count > 0, "Map should contain walls");
    }
}
