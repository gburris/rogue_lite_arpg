mod chest;
mod gold;
mod map;
mod portal;

use bevy::prelude::*;
use rand::{Rng, RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;

pub mod prelude {
    pub use super::chest::SpawnChestsEvent;
    pub use super::gold::GoldDrop;
    pub use super::map::prelude::{Anchor, MapRequest, NextMap, TILE_SIZE, TileToWorld, place};
    pub use super::portal::portal;
}

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((chest::plugin, gold::plugin, portal::plugin, map::plugin));

    let seed = std::env::var("MAP_SEED")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or_else(rand::random);
    info!("Root seed for map: {seed}");
    app.insert_resource(MapRng::seeded(seed));
}

#[derive(Resource)]
pub(crate) struct MapRng {
    rng: ChaCha8Rng,
    root_seed: u64,
}

impl MapRng {
    fn seeded(seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            root_seed: seed,
        }
    }

    /// Draws the build seed for the next map.
    pub(crate) fn next_map_seed(&mut self) -> u64 {
        self.rng.next_u64()
    }

    /// Root generator for choices made before a map is built, such as the weighted recipe pick.
    pub(crate) fn rng(&mut self) -> &mut ChaCha8Rng {
        &mut self.rng
    }

    /// Deterministic coin flip on the map stream for spawn-time variants.
    pub(crate) fn flip(&mut self) -> bool {
        self.rng.random_bool(0.5)
    }

    /// The root seed without advancing map generation.
    pub(crate) fn root_seed(&self) -> u64 {
        self.root_seed
    }
}
