//! A brain's per-run state: the behaviors it holds, its random stream, and the seed
//! that stream grows from.

use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;

use super::perception::DecisionContext;
use super::{Action, ActionFinished, ActionOutcome, Brain, Motion};
use crate::world::MapRng;

/// Longest a committed behavior may run, a safety net against a missing finish.
const COMMIT_TIMEOUT: f32 = 3.0;

/// A behavior that won its slots, with its motion and action copied from the
/// definition and stamped with a run id so a finite motion's
/// [`CurrentBehaviorRunId`](super::motion::CurrentBehaviorRunId) can tell a fresh
/// start from the run it just finished. Appliers report through it, so the
/// committed/continuously scored finish policy lives here.
#[derive(Clone, Copy, Debug)]
pub(super) struct RunningBehavior {
    pub(super) behavior: usize,
    pub(super) motion: Option<Motion>,
    pub(super) action: Option<Action>,
    pub(super) behavior_run_id: u32,
    pub(super) committed: bool,
    pub(super) started_at: f32,
}

impl RunningBehavior {
    pub(super) fn report_completed(self, commands: &mut Commands, entity: Entity) {
        if let Some(finished) = self.build_action_finished(entity, ActionOutcome::Completed) {
            commands.trigger(finished);
        }
    }

    /// Ends a committed use the live world refused; a continuously scored use waits.
    pub(super) fn report_not_started(self, commands: &mut Commands, entity: Entity) {
        if self.committed
            && let Some(finished) = self.build_action_finished(entity, ActionOutcome::NotStarted)
        {
            commands.trigger(finished);
        }
    }

    /// `None` for a record with no action to report on.
    fn build_action_finished(
        self,
        entity: Entity,
        outcome: ActionOutcome,
    ) -> Option<ActionFinished> {
        Some(ActionFinished {
            entity,
            action: self.action?,
            behavior_run_id: self.behavior_run_id,
            outcome,
        })
    }

    pub(super) fn should_drop_committed_behavior(
        &self,
        context: &DecisionContext,
        now: f32,
    ) -> bool {
        self.committed
            && (context.target_view.is_none()
                || context.self_view.stunned
                || now - self.started_at > COMMIT_TIMEOUT)
    }
}

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
#[derive(Resource, Default, Clone, Copy, Debug)]
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
