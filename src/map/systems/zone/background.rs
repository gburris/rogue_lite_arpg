use bevy::prelude::*;
use rand::{thread_rng, Rng};

#[derive(Component)]
pub struct ZoneBackground;

#[derive(Component)]
pub struct GemCluster {
    pub glow_timer: f32, // New field for the glow effect
}

const FLOOR_SIZE: f32 = 5000.0;
const GRID_SIZE: f32 = 50.0;
const GRID_COUNT: i32 = (FLOOR_SIZE / GRID_SIZE) as i32;

// Colors for variety in rocks and gems
const ROCK_COLORS: [(f32, f32, f32); 4] = [
    (0.2, 0.2, 0.2),       // Medium gray (50% darker)
    (0.225, 0.225, 0.225), // Lighter gray (50% darker)
    (0.175, 0.175, 0.175), // Darker gray (50% darker)
    (0.21, 0.21, 0.21),    // Another gray variation (50% darker)
];

const GEM_COLORS: [(f32, f32, f32); 3] = [
    (0.2, 0.2, 0.8), // Blue
    (0.8, 0.2, 0.2), // Red
    (0.6, 0.2, 0.8), // Purple
];

pub fn spawn_background(mut commands: Commands) {
    let mut rng = thread_rng();

    // Spawn base floor
    commands.spawn((
        Sprite::from_color(
            Color::srgb(0.3, 0.3, 0.3), // Dark gray base
            Vec2::new(FLOOR_SIZE, FLOOR_SIZE),
        ),
        Transform::from_xyz(0.0, 0.0, -1.0),
        ZoneBackground,
    ));

    // Generate grid of rock piles
    for x in -GRID_COUNT / 2..GRID_COUNT / 2 {
        for y in -GRID_COUNT / 2..GRID_COUNT / 2 {
            let base_x = x as f32 * GRID_SIZE;
            let base_y = y as f32 * GRID_SIZE;

            // Spawn rock pile (4-6 rocks per pile)
            let num_rocks = rng.gen_range(4..7);
            for _ in 0..num_rocks {
                let offset_x = rng.gen_range(-20.0..20.0);
                let offset_y = rng.gen_range(-20.0..20.0);

                let color_idx = rng.gen_range(0..ROCK_COLORS.len());
                let (r, g, b) = ROCK_COLORS[color_idx];
                let variation = 0.95 + (rng.gen::<f32>() * 0.1);

                let rock_size = Vec2::new(rng.gen_range(15.0..25.0), rng.gen_range(15.0..25.0));

                commands.spawn((
                    Sprite::from_color(
                        Color::srgb(r * variation, g * variation, b * variation),
                        rock_size,
                    ),
                    Transform::from_xyz(base_x + offset_x, base_y + offset_y, -0.9),
                    ZoneBackground,
                ));
            }

            // 1% chance to spawn gem cluster
            if rng.gen::<f32>() < 0.001 {
                spawn_gem_cluster(&mut commands, &mut rng, base_x, base_y);
            }
        }
    }
}

fn spawn_gem_cluster(
    commands: &mut Commands,
    rng: &mut rand::rngs::ThreadRng,
    base_x: f32,
    base_y: f32,
) {
    let num_gems = rng.gen_range(3..7);

    for _ in 0..num_gems {
        let offset_x = rng.gen_range(-30.0..30.0);
        let offset_y = rng.gen_range(-30.0..30.0);

        let color_idx = rng.gen_range(0..GEM_COLORS.len());
        let (r, g, b) = GEM_COLORS[color_idx];

        let gem_size = Vec2::new(rng.gen_range(8.0..12.0), rng.gen_range(8.0..12.0));

        commands.spawn((
            Sprite::from_color(Color::srgba(r, g, b, 0.9), gem_size),
            Transform::from_xyz(base_x + offset_x, base_y + offset_y, -0.8),
            ZoneBackground,
            GemCluster { glow_timer: 0.0 },
        ));
    }
}
