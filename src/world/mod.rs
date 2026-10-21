mod chest;
mod gold;
mod map;
mod portal;

use bevy::prelude::*;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

pub mod prelude {
    pub use super::chest::*;
    pub use super::gold::*;
    pub use super::map::prelude::*;
    pub use super::portal::*;
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

/// Deterministic source of map seeds derived from the game's root seed.
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

    /// Advances the map stream and returns the next map seed.
    pub(crate) fn next_seed(&mut self) -> u64 {
        self.rng.next_u64()
    }

    /// The root seed without advancing map generation.
    pub(crate) fn root_seed(&self) -> u64 {
        self.root_seed
    }
}
