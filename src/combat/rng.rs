use bevy::prelude::*;
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::prelude::*;

/// Random stream for combat rolls, seeded from the map's root seed.
#[derive(Resource)]
pub(super) struct CombatRng {
    random: ChaCha8Rng,
}

impl Default for CombatRng {
    fn default() -> Self {
        Self {
            random: ChaCha8Rng::seed_from_u64(0),
        }
    }
}

impl CombatRng {
    /// Uniform value in `min..max`.
    pub(super) fn range(&mut self, min: f32, max: f32) -> f32 {
        self.random.random_range(min..max)
    }
}

pub(super) fn seed_combat_rng_from_map(map_rng: Res<MapRng>, mut combat_rng: ResMut<CombatRng>) {
    // Offset keeps this stream independent of the map stream drawn from the same seed.
    combat_rng.random = ChaCha8Rng::seed_from_u64(map_rng.root_seed() ^ 0xC0FF_EE00_DAA6_E5ED);
}
