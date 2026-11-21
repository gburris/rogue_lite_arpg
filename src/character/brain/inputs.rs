use bevy::ecs::query::QueryData;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::Action;
use super::perception::{TargetInfo, Targeting};
use super::{Brain, Home, HomeArea, MotionKey, MotionStatus};
use crate::character::Allies;
use crate::character::state::BlockedState;
use crate::prelude::*;

/// Whether an action could start now; see [`DecisionInputs::action_readiness`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ActionReadiness {
    Ready,
    /// Only waiting prevents it, such as for a cooldown, mana or an interruption to end.
    Waiting,
    /// The character lacks what the action needs, such as the item, the ability or a potion.
    Missing,
}

/// The raw fact a consideration reads; see [`DecisionInputs::read_consideration_input`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ConsiderationInput {
    HasTarget,
    LineOfSight,
    TargetDistance,
    /// The target's intended speed.
    TargetSpeed,
    MissingHealth,
    ManaFraction,
    /// Whether this motion is running.
    MotionRunning(MotionKey),
    /// Whether the character can start the motion of this source (see [`MotionStatus::is_ready`])
    /// and no behavior with it is in a start roll retry delay.
    MotionReady(MotionKey),
    /// Seconds the running motion has held its slot when it is this motion, else zero.
    SecondsInMotion(MotionKey),
    /// How far past its home area range the character is.
    HomeAreaPull,
    AttackFinished,
    /// Target distance over the reach of this motion source; at most `1.0` is in reach.
    MotionReachFraction(MotionKey),
    /// Target distance over this action's reach; at most `1.0` is in reach.
    ActionReachFraction(Action),
    /// The closest ally's distance over this action's reach; absent with no ally alive.
    AllyReachFraction(Action),
    SecondsSinceTargeted,
    /// Whether the character owns this action and has what it needs, whatever only waiting would
    /// fix.
    ActionUsable(Action),
    /// Whether the character is blocked; see [`BlockedState`].
    MovementBlocked,
}

/// The components of the deciding character that its inputs read.
#[derive(QueryData)]
pub(super) struct DecisionInputComponents {
    entity: Entity,
    transform: &'static Transform,
    has_target: Has<Targeting>,
    target_info: Option<&'static TargetInfo>,
    health: Option<&'static Health>,
    mana: Option<&'static Mana>,
    home: Option<&'static Home>,
    blocked: BlockedState,
}

/// The queries behind action and motion readiness, reach and allies.
#[derive(SystemParam)]
pub(super) struct DecisionQueries<'w, 's> {
    actions: ActionStatus<'w, 's>,
    motions: MotionStatus<'w, 's>,
    allies: Allies<'w, 's>,
    home_areas: Query<'w, 's, (&'static Transform, &'static HomeArea)>,
}

/// One decision's view of the world: references only, read on demand.
pub(super) struct DecisionInputs<'a, 'w, 's> {
    pub components: &'a DecisionInputComponentsItem<'w, 's>,
    pub queries: &'a DecisionQueries<'a, 'a>,
    pub now: f32,
}

impl DecisionInputs<'_, '_, '_> {
    /// The tracked target while targeting one at a finite distance. `TargetInfo`
    /// also tracks a watched entity, so every target input reads through here.
    pub fn target(&self) -> Option<&TargetInfo> {
        self.components
            .target_info
            .filter(|info| self.components.has_target && info.has_finite_distance())
    }

    /// See [`MotionStatus::has_source`].
    pub fn has_motion_source(&self, key: MotionKey) -> bool {
        self.queries.motions.has_source(self.components.entity, key)
    }

    /// See [`MotionStatus::is_ready`].
    pub fn is_motion_ready(&self, key: MotionKey) -> bool {
        self.queries.motions.is_ready(self.components.entity, key)
    }

    /// See [`MotionStatus::ends_on_its_own`].
    pub fn motion_ends_on_its_own(&self, key: MotionKey) -> bool {
        self.queries
            .motions
            .ends_on_its_own(self.components.entity, key)
    }

    /// Whether the character has an action source for `action`; never for `Idle`, which has none.
    pub fn has_action_source(&self, action: Action) -> bool {
        action.key().is_some_and(|key| {
            self.queries
                .actions
                .source_for_key(self.components.entity, key)
                .is_some()
        })
    }

    /// How far the action source for `action` reaches; `None` without a source or a reach.
    fn action_reach(&self, action: Action) -> Option<f32> {
        let key = action.key()?;
        self.queries.actions.reach(self.components.entity, key)
    }

    /// Whether `action` ends on its own, so a committed behavior over it can finish; see
    /// [`ActionUseStyle`].
    pub fn action_ends_on_its_own(&self, action: Action) -> bool {
        let entity = self.components.entity;
        action
            .key()
            .is_some_and(|key| self.queries.actions.ends_on_its_own(entity, key))
    }

    /// Whether a use of `action` is an attack; see [`Attack`].
    pub fn action_is_attack(&self, action: Action) -> bool {
        let entity = self.components.entity;
        action
            .key()
            .is_some_and(|key| self.queries.actions.is_attack(entity, key))
    }

    /// Whether the character can start `action` now, or why not.
    fn action_readiness(&self, action: Action) -> ActionReadiness {
        let Some(key) = action.key() else {
            return ActionReadiness::Ready;
        };
        match self.queries.actions.can_use(self.components.entity, key) {
            Ok(_) => ActionReadiness::Ready,
            Err(failure) if failure.is_temporary() => ActionReadiness::Waiting,
            Err(_) => ActionReadiness::Missing,
        }
    }

    /// Whether `action` can run now: it is already in progress, or it is ready and no other action
    /// is in progress, so a drink the brain dropped still blocks a swing. `Idle` always can.
    pub fn can_run(&self, action: Action) -> bool {
        let actions = &self.queries.actions;
        let entity = self.components.entity;
        let Some(key) = action.key() else {
            return true;
        };
        if actions.is_using(entity, key) {
            return true;
        }
        !actions.is_using_any(entity) && self.action_readiness(action) == ActionReadiness::Ready
    }

    /// Answers `input`; `None` when it is absent, which vetoes the behavior.
    pub fn read_consideration_input(
        &self,
        input: ConsiderationInput,
        brain: &Brain,
    ) -> Option<f32> {
        match input {
            ConsiderationInput::HasTarget => Some(f32::from(self.target().is_some())),
            ConsiderationInput::LineOfSight => {
                self.target().map(|target| f32::from(target.line_of_sight))
            }
            ConsiderationInput::TargetDistance => self.target().map(|target| target.distance),
            ConsiderationInput::TargetSpeed => self.target().map(|target| target.velocity.length()),
            ConsiderationInput::MissingHealth => Some(
                1.0 - self
                    .components
                    .health
                    .and_then(Health::fraction)
                    .unwrap_or(1.0),
            ),
            ConsiderationInput::ManaFraction => {
                Some(self.components.mana.and_then(Mana::fraction).unwrap_or(1.0))
            }
            ConsiderationInput::MotionRunning(motion) => Some(f32::from(
                brain.motion().is_some_and(|running| running == motion),
            )),
            ConsiderationInput::MotionReady(key) => Some(f32::from(
                self.is_motion_ready(key) && !brain.is_start_roll_waiting(key, self.now),
            )),
            ConsiderationInput::SecondsInMotion(motion) => Some(
                brain
                    .current_motion_behavior()
                    .filter(|(_, held)| *held == motion)
                    .map_or(0.0, |(running, _)| self.now - running.started_at),
            ),
            ConsiderationInput::HomeAreaPull => self.home_area_pull(brain.motion()),
            ConsiderationInput::AttackFinished => Some(f32::from(brain.attack_finish_pending())),
            ConsiderationInput::MotionReachFraction(key) => {
                let reach = self.queries.motions.reach(self.components.entity, key);
                self.target_reach_fraction(reach)
            }
            ConsiderationInput::ActionReachFraction(action) => {
                self.target_reach_fraction(self.action_reach(action))
            }
            ConsiderationInput::AllyReachFraction(action) => {
                let closest = self.queries.allies.closest(self.components.entity)?;
                Some(reach_fraction(closest.distance, self.action_reach(action)))
            }
            ConsiderationInput::SecondsSinceTargeted => {
                self.target().map(|target| self.now - target.targeted_at)
            }
            ConsiderationInput::ActionUsable(action) => Some(f32::from(
                self.has_action_source(action)
                    && self.action_readiness(action) != ActionReadiness::Missing,
            )),
            ConsiderationInput::MovementBlocked => {
                Some(f32::from(self.components.blocked.is_blocked()))
            }
        }
    }

    /// How far past its home area range the character is. Once already heading
    /// home it reads full until arrival, so crossing back inside the range
    /// mid-return cannot hand the character straight back to wandering.
    fn home_area_pull(&self, running_motion: Option<MotionKey>) -> Option<f32> {
        let home = self.components.home?;
        let (home_transform, home_area) = self.queries.home_areas.get(home.get()).ok()?;
        let distance = self
            .components
            .transform
            .translation
            .xy()
            .distance(home_transform.translation.xy());
        let overshoot_fraction = home_area.overshoot_fraction(distance)?;
        if running_motion == Some(MotionKey::ReturnToHome) {
            Some(f32::from(!home_area.has_arrived(distance)))
        } else {
            Some(overshoot_fraction)
        }
    }

    /// Target distance over `reach`; see [`reach_fraction`].
    fn target_reach_fraction(&self, reach: Option<f32>) -> Option<f32> {
        let target = self.target()?;
        Some(reach_fraction(target.distance, reach))
    }
}

/// `distance` over `reach`; at most `1.0` is in reach. `f32::MAX` for a source that is missing or
/// has no positive reach.
fn reach_fraction(distance: f32, reach: Option<f32>) -> f32 {
    reach
        .filter(|&reach| reach > 0.0)
        .map_or(f32::MAX, |reach| distance / reach)
}
