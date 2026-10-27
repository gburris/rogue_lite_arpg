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

#[derive(Resource)]
pub(crate) struct MapRng(ChaCha8Rng);

impl MapRng {
    fn seeded(seed: u64) -> Self {
        Self(ChaCha8Rng::seed_from_u64(seed))
    }

    /// Draws the build seed for the next map.
    pub(crate) fn next_map_seed(&mut self) -> u64 {
        self.0.next_u64()
    }

    /// Root generator for choices made before a map is built, such as the weighted recipe pick.
    pub(crate) fn rng(&mut self) -> &mut ChaCha8Rng {
        &mut self.0
    }
}
