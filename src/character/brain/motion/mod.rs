//! Per-frame steering execution: translates [`Brain`](super::Brain) locomotion
//! and maneuvers into [`SimpleMotion`].
//!
//! [`super`] decides at ~10 Hz what to do; this module drives every frame how
//! to move. Each behavior owns its component, its memory and its steering in
//! one file: [`approach`] (stateless), [`strafe`], [`wander`], [`dash`],
//! [`swoop`] and [`cast_plant`]. The memories are opt-in capabilities that
//! persist across mode switches, so they stay separate components rather than
//! one enum.
//!
//! A running maneuver takes exclusive control of steering. Maneuver
//! components implement [`ManeuverExecutor`] and are listed once, in
//! [`Maneuvers`]; the brain's active maneuver is the single source of truth,
//! and an executor reports back only with [`ManeuverFinished`], never by
//! writing decision state.

mod approach;
mod cast_plant;
mod dash;
mod strafe;
mod swoop;
mod wander;

use bevy::{ecs::query::QueryData, prelude::*};

pub use cast_plant::CastPlant;
pub use dash::Dash;
pub use strafe::Strafe;
pub use swoop::Swoop;
pub use wander::Wander;

use super::{ActiveManeuver, Brain, BrainRng, Maneuver, MotionMode, Tuning};
use crate::character::{
    anchor::Anchor,
    state::SimpleMotion,
    vision::{TargetInfo, Targeting},
};
use crate::prelude::{AttackState, TILE_SIZE};

pub(crate) const ANCHOR_ARRIVAL_RADIUS: f32 = TILE_SIZE as f32 / 2.0;

/// Maximum turn rate in radians per second. Absent means snap.
#[derive(Component, Clone, Copy, Debug)]
pub struct TurnRate(pub f32);

/// A maneuver ran to its end. The brain releases the maneuver and starts its
/// cooldown when it reads this.
#[derive(Message, Clone, Copy, Debug)]
pub struct ManeuverFinished {
    pub entity: Entity,
    pub maneuver: Maneuver,
}

/// Read-only world input shared by every steering behavior this frame.
pub(super) struct SteerInput<'a> {
    pub position: Vec2,
    pub now: f32,
    pub target_info: Option<&'a TargetInfo>,
    pub anchor: Option<&'a Anchor>,
}

/// How hard to push along the steered heading: a fraction of max speed for
/// gaits, a multiple for committed bursts.
#[derive(Clone, Copy)]
pub(super) enum Effort {
    Scaled(f32),
    Boosted(f32),
}

/// One frame of a maneuver executor.
pub(super) enum ManeuverStep {
    /// Still running: steer this way.
    Steer {
        direction: Option<Vec2>,
        effort: Effort,
    },
    /// Ended this frame; the brain has not heard yet.
    Finished,
    /// Ended earlier and is waiting for the brain to release it.
    Spent,
}

/// Which brain-issued run an executor is carrying out. The brain stamps every
/// start with a fresh epoch, so an executor can tell a new run of the same
/// maneuver from the one it just finished, even with no frame in between.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct RunTag {
    epoch: Option<u32>,
    /// The run ended and the brain has not yet released it.
    spent: bool,
}

/// Everything a maneuver step may look at this frame. The random stream is
/// the only thing it can mutate.
pub(super) struct ManeuverContext<'a, 'w> {
    pub input: &'a SteerInput<'w>,
    pub motion: &'a SimpleMotion,
    pub delta_secs: f32,
    pub rng: &'a mut BrainRng,
}

impl ManeuverContext<'_, '_> {
    /// World units one straight-run steering step may consume this frame.
    /// Stunned bodies hold still so committed passes neither advance nor finish.
    pub fn run_step(&self, speed_boost: f32) -> f32 {
        if self.motion.is_stunned() {
            0.0
        } else {
            self.motion.max_speed * speed_boost * self.motion.speed_scale() * self.delta_secs
        }
    }
}

/// A capability component that carries out one [`Maneuver`]. Implementing this
/// and listing the component in [`Maneuvers`] is all the executor side of a
/// new maneuver needs.
pub(super) trait ManeuverExecutor {
    fn maneuver(&self) -> Maneuver;
    /// Chance, spacing and reach the decision tick applies to this maneuver.
    fn tuning(&self) -> Tuning;
    fn tag(&mut self) -> &mut RunTag;
    /// Forgets a run the brain is no longer asking for.
    fn reset(&mut self, now: f32);
    fn step(&mut self, context: &mut ManeuverContext) -> ManeuverStep;
    /// Extra speed locomotion should keep while settling out of the maneuver.
    fn recovery_boost(&self, _now: f32) -> f32 {
        1.0
    }
}

/// Every maneuver executor an entity may carry. The one place to list a new one.
#[derive(QueryData)]
#[query_data(mutable)]
pub(super) struct Maneuvers {
    dash: Option<&'static mut Dash>,
    swoop: Option<&'static mut Swoop>,
}

impl ManeuversItem<'_, '_> {
    fn executors(&mut self) -> impl Iterator<Item = &mut dyn ManeuverExecutor> {
        [
            self.dash
                .as_deref_mut()
                .map(|executor| executor as &mut dyn ManeuverExecutor),
            self.swoop
                .as_deref_mut()
                .map(|executor| executor as &mut dyn ManeuverExecutor),
        ]
        .into_iter()
        .flatten()
    }

    /// Resets any executor holding a run the brain no longer asks for, so the
    /// brain's active maneuver stays the single source of truth.
    fn sync(&mut self, active: Option<ActiveManeuver>, now: f32) {
        for executor in self.executors() {
            let epoch = active
                .filter(|active| active.maneuver == executor.maneuver())
                .map(|active| active.epoch);
            if executor.tag().epoch != epoch {
                executor.reset(now);
                *executor.tag() = RunTag { epoch, spent: false };
            }
        }
    }

    /// Runs the executor for `maneuver`, if this entity has one.
    fn step(&mut self, maneuver: Maneuver, context: &mut ManeuverContext) -> Option<ManeuverStep> {
        self.executors()
            .find(|executor| executor.maneuver() == maneuver)
            .map(|executor| executor.step(context))
    }

    /// Largest recovery boost any executor still holds.
    fn recovery_boost(&mut self, now: f32) -> f32 {
        self.executors()
            .map(|executor| executor.recovery_boost(now))
            .fold(1.0, f32::max)
    }
}

impl ManeuversReadOnlyItem<'_, '_> {
    /// Tuning of every maneuver this entity can carry out, by maneuver index.
    pub(super) fn tuning(&self) -> [Option<Tuning>; Maneuver::ALL.len()] {
        let executors: [Option<&dyn ManeuverExecutor>; 2] = [
            self.dash.map(|executor| executor as &dyn ManeuverExecutor),
            self.swoop.map(|executor| executor as &dyn ManeuverExecutor),
        ];
        let mut tuning = [None; Maneuver::ALL.len()];
        for executor in executors.into_iter().flatten() {
            tuning[executor.maneuver() as usize] = Some(executor.tuning());
        }
        tuning
    }
}

/// Everything one steered character needs this frame.
#[derive(QueryData)]
#[query_data(mutable)]
pub(super) struct Steered {
    entity: Entity,
    brain: &'static Brain,
    rng: &'static mut BrainRng,
    motion: &'static mut SimpleMotion,
    transform: &'static Transform,
    target_info: Option<&'static TargetInfo>,
    has_target: Has<Targeting>,
    anchor: Option<&'static Anchor>,
    turn_rate: Option<&'static TurnRate>,
    attack_state: Option<&'static AttackState>,
    maneuvers: Maneuvers,
    wander: Option<&'static mut Wander>,
    strafe: Option<&'static mut Strafe>,
    cast_plant: Option<&'static mut CastPlant>,
}

pub(super) fn drive_motion(
    time: Res<Time<Virtual>>,
    mut characters: Query<Steered>,
    mut finished: MessageWriter<ManeuverFinished>,
) {
    let now = time.elapsed_secs();
    let delta_secs = time.delta_secs();

    for character in &mut characters {
        let SteeredItem {
            entity,
            brain,
            mut rng,
            mut motion,
            transform,
            target_info,
            has_target,
            anchor,
            turn_rate,
            attack_state,
            mut maneuvers,
            mut wander,
            mut strafe,
            mut cast_plant,
        } = character;

        let active = brain.active_maneuver();
        maneuvers.sync(active, now);

        // Casters plant through the read of their own wind-up. Keeping this
        // ahead of the steering also means a committed burst or slide
        // cannot drag the caster out from under a staff that is already firing.
        let attacking = attack_state.is_some_and(|state| state.is_attacking);
        let planted = cast_plant
            .as_deref_mut()
            .is_some_and(|plant| plant.update(attacking, now));
        let missing_target = (active.is_some() || brain.mode().needs_target())
            && (!has_target || target_info.is_none());
        if planted || missing_target {
            stop(&mut motion);
            continue;
        }

        let turn_cap = turn_rate.map(|rate| rate.0 * delta_secs);
        let input = SteerInput {
            position: transform.translation.xy(),
            now,
            target_info,
            anchor,
        };

        if let Some(active) = active {
            let mut context = ManeuverContext {
                input: &input,
                motion: &motion,
                delta_secs,
                rng: &mut rng,
            };
            match maneuvers.step(active.maneuver, &mut context) {
                Some(ManeuverStep::Steer { direction, effort }) => {
                    drive(&mut motion, direction, effort, turn_cap);
                    continue;
                }
                // Fall through to locomotion this frame so the body never
                // coasts for a frame between a maneuver and its settle-out.
                Some(ManeuverStep::Finished) => {
                    finished.write(ManeuverFinished {
                        entity,
                        maneuver: active.maneuver,
                    });
                }
                Some(ManeuverStep::Spent) | None => {}
            }
        }

        let recovery_boost = maneuvers.recovery_boost(now);
        let (direction, effort) = steer_locomotion(
            brain.mode(),
            &input,
            LocomotionMemory {
                wander: wander.as_deref_mut(),
                strafe: strafe.as_deref_mut(),
                recovery_boost,
            },
            &mut rng,
        );
        drive(&mut motion, direction, effort, turn_cap);
    }
}

/// The opt-in memories and tuning locomotion steering may use.
struct LocomotionMemory<'a> {
    wander: Option<&'a mut Wander>,
    strafe: Option<&'a mut Strafe>,
    /// Extra speed held while settling out of a maneuver; also freezes the
    /// orbit side so re-entering the circle does not flip the tangent.
    recovery_boost: f32,
}

/// Where and how hard the current locomotion mode wants to push.
fn steer_locomotion(
    mode: MotionMode,
    input: &SteerInput,
    memory: LocomotionMemory,
    rng: &mut BrainRng,
) -> (Option<Vec2>, Effort) {
    let LocomotionMemory {
        mut wander,
        strafe,
        recovery_boost,
    } = memory;
    if mode != MotionMode::Calm
        && let Some(wander) = wander.as_deref_mut()
    {
        wander.leave_calm();
    }
    match mode {
        MotionMode::Hold => (None, Effort::Scaled(1.0)),
        MotionMode::Chase => (approach::chase(input), Effort::Scaled(1.0)),
        // Still in the fight: run from the target, not toward home.
        // Going home happens only once the target is lost.
        MotionMode::Flee => (approach::flee(input), Effort::Scaled(1.0)),
        MotionMode::ReturnToAnchor => (approach::home(input), Effort::Scaled(1.0)),
        MotionMode::KeepDistance(keep_band) => (
            approach::keep_distance(input, keep_band, wander, rng),
            Effort::Boosted(recovery_boost),
        ),
        MotionMode::Strafe(band) => (
            strafe::steer(input, strafe, band, rng, recovery_boost > 1.0),
            Effort::Boosted(recovery_boost),
        ),
        MotionMode::Calm => {
            let direction = wander::steer(input, wander.as_deref_mut(), rng);
            let scale = wander::speed_scale(wander.as_deref());
            (direction, Effort::Scaled(scale))
        }
    }
}

fn stop(motion: &mut SimpleMotion) {
    if !motion.is_idle() {
        motion.stop_moving();
    }
}

fn drive(
    motion: &mut SimpleMotion,
    direction: Option<Vec2>,
    effort: Effort,
    turn_cap: Option<f32>,
) {
    let Some(desired) = direction else {
        stop(motion);
        return;
    };
    let heading = turn_towards(motion, desired, turn_cap);
    match effort {
        Effort::Scaled(scale) => motion.start_moving_scaled(heading, scale),
        Effort::Boosted(speed_boost) => motion.start_moving_boosted(heading, speed_boost),
    }
}

fn turn_towards(motion: &SimpleMotion, desired: Vec2, turn_cap: Option<f32>) -> Vec2 {
    if let Some(max_angle) = turn_cap
        && motion.direction.length_squared() > f32::EPSILON
        && desired.length_squared() > f32::EPSILON
    {
        motion.direction.rotate_towards(desired, max_angle)
    } else {
        desired
    }
}
