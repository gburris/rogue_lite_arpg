//! Per-brain random streams and the root seed they grow from.

use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;

use super::Brain;
use crate::prelude::*;

/// A brain's random stream, seeded from [`BrainRngRootSeed`] and the entity. Kept apart
/// from [`Brain`] so steering gets randomness without the rest of the brain.
#[derive(Component, Debug)]
pub struct BrainRng {
    random: ChaCha8Rng,
}

impl Default for BrainRng {
    fn default() -> Self {
        Self {
            random: ChaCha8Rng::seed_from_u64(0),
        }
    }
}

impl BrainRng {
    /// Uniform value in `min..max`, or `min` for an empty/invalid range.
    pub fn range(&mut self, min: f32, max: f32) -> f32 {
        if max > min {
            self.random.random_range(min..max)
        } else {
            min
        }
    }

    pub fn direction(&mut self) -> Vec2 {
        let angle = self.random.random_range(0.0..std::f32::consts::TAU);
        Vec2::new(angle.cos(), angle.sin())
    }
}

pub(super) fn seed_brain_rng_and_schedule_first_decision(
    mut world: DeferredWorld,
    context: HookContext,
) {
    let root_seed = world
        .get_resource::<BrainRngRootSeed>()
        .map_or(0, |seed| seed.0);
    let now = world
        .get_resource::<Time<Virtual>>()
        .map_or(0.0, Time::elapsed_secs);
    let entity_seed = context.entity.to_bits().wrapping_mul(0x9E37_79B9_7F4A_7C15);
    let Some(mut rng) = world.get_mut::<BrainRng>(context.entity) else {
        return;
    };
    rng.random = ChaCha8Rng::seed_from_u64(root_seed ^ entity_seed);
    let phase = rng.range(0.0, DECISION_PERIOD);
    if let Some(mut brain) = world.get_mut::<Brain>(context.entity) {
        brain.next_decision_at = now + phase;
    }
}

/// Seconds between a brain's scheduled decisions.
pub(super) const DECISION_PERIOD: f32 = 0.1;

/// Root seed used to seed each brain's independent random stream.
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct BrainRngRootSeed(pub u64);

pub(super) fn seed_brain_rng_root_from_map(
    map_rng: Option<Res<MapRng>>,
    mut root_seed: ResMut<BrainRngRootSeed>,
) {
    if let Some(map_rng) = map_rng {
        root_seed.0 = map_rng.root_seed();
    } else {
        warn!("MapRng is missing; seeding AI randomness with 0");
    }
}
