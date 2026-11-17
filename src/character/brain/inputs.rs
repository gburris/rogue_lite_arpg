use bevy::ecs::query::QueryData;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::Brain;
use super::action::Action;
use super::capability::{CapabilityProfile, MotionCapabilities};
use super::motion::{HomeArea, Motion};
use super::perception::{TargetInfo, Targeting};
use crate::character::Allies;
use crate::character::state::BlockedState;
use crate::prelude::*;

/// Whether an action could start now; see [`DecisionInputs::action_readiness`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ActionReadiness {
    Ready,
    /// Only waiting stands in the way, such as a cooldown, mana or an interruption.
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
    MissingHealth,
    ManaFraction,
    /// Whether this motion's kind is the one running; its payload is ignored.
    MotionRunning(Motion),
    /// Whether the character owns this motion and its cooldown has run out; the payload is
    /// ignored.
    MotionReady(Motion),
    /// Seconds the running motion has held its slot when it is this motion's kind, else zero;
    /// the payload is ignored.
    SecondsInMotion(Motion),
    /// How far past its home area range the character is.
    HomeAreaPull,
    AttackFinished,
    /// Target distance over this motion's reach; at most `1.0` is in reach.
    MotionReachFraction(Motion),
    /// Target distance over this action's reach; at most `1.0` is in reach.
    ActionReachFraction(Action),
    /// The closest ally's distance over this action's reach; absent with no ally alive.
    AllyReachFraction(Action),
    SecondsSinceTargeted,
    /// Whether the character owns this action and has what it needs, whatever only waiting would
    /// fix.
    ActionUsable(Action),
    /// Whether the character is pressed against a wall or a crowd; see [`BlockedState`].
    MovementBlocked,
}

/// The components one decision reads, including every capability a character may carry.
#[derive(QueryData)]
pub(super) struct DecisionInputComponents {
    entity: Entity,
    transform: &'static Transform,
    has_target: Has<Targeting>,
    target_info: Option<&'static TargetInfo>,
    health: Option<&'static Health>,
    mana: Option<&'static Mana>,
    home_area: Option<&'static HomeArea>,
    blocked: BlockedState,
    motion_capabilities: MotionCapabilities,
}

/// The queries behind action profiles, allies and the can-run checks.
#[derive(SystemParam)]
pub(super) struct DecisionQueries<'w, 's> {
    actions: ActionStatus<'w, 's>,
    allies: Allies<'w, 's>,
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

    /// The profile of the capability that carries out `motion`; see [`MotionCapabilities`].
    pub fn motion_profile(&self, motion: Motion) -> Option<CapabilityProfile> {
        self.components
            .motion_capabilities
            .capability_profile(motion)
    }

    /// The profile of the action source that carries out `action`: its reach. `Idle` needs none.
    pub fn action_profile(&self, action: Action) -> Option<CapabilityProfile> {
        let actions = &self.queries.actions;
        let entity = self.components.entity;
        let key = action.key()?;
        actions.source_for_key(entity, key)?;
        Some(CapabilityProfile {
            reach: actions.reach(entity, key),
            ..CapabilityProfile::default()
        })
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

    /// Whether the character could start `action` now, or what stands in the way.
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
                brain
                    .motion()
                    .is_some_and(|running| running.is_same_kind(motion)),
            )),
            ConsiderationInput::MotionReady(motion) => Some(f32::from(
                self.motion_profile(motion).is_some() && brain.is_cooldown_ready(motion, self.now),
            )),
            ConsiderationInput::SecondsInMotion(motion) => Some(
                brain
                    .current_motion_behavior()
                    .filter(|running| running.motion.is_some_and(|held| held.is_same_kind(motion)))
                    .map_or(0.0, |running| self.now - running.started_at),
            ),
            ConsiderationInput::HomeAreaPull => self.home_area_pull(brain.motion()),
            ConsiderationInput::AttackFinished => Some(f32::from(brain.attack_finish_pending())),
            ConsiderationInput::MotionReachFraction(motion) => {
                self.target_reach_fraction(self.motion_profile(motion))
            }
            ConsiderationInput::ActionReachFraction(action) => {
                self.target_reach_fraction(self.action_profile(action))
            }
            ConsiderationInput::AllyReachFraction(action) => {
                let closest = self.queries.allies.closest(self.components.entity)?;
                Some(reach_fraction(
                    closest.distance,
                    self.action_profile(action),
                ))
            }
            ConsiderationInput::SecondsSinceTargeted => {
                self.target().map(|target| self.now - target.targeted_at)
            }
            ConsiderationInput::ActionUsable(action) => Some(f32::from(
                self.action_profile(action).is_some()
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
    fn home_area_pull(&self, running_motion: Option<Motion>) -> Option<f32> {
        let home_area = self.components.home_area?;
        let position = self.components.transform.translation.xy();
        let overshoot_fraction = home_area.overshoot_fraction(position)?;
        if running_motion == Some(Motion::ReturnToHome) {
            Some(f32::from(!home_area.has_arrived(position)))
        } else {
            Some(overshoot_fraction)
        }
    }

    /// Target distance over the capability's reach; see [`reach_fraction`].
    fn target_reach_fraction(&self, profile: Option<CapabilityProfile>) -> Option<f32> {
        let target = self.target()?;
        Some(reach_fraction(target.distance, profile))
    }
}

/// `distance` over the capability's reach; at most `1.0` is in reach. `f32::MAX` for a
/// capability that is missing or has no positive reach.
fn reach_fraction(distance: f32, profile: Option<CapabilityProfile>) -> f32 {
    profile
        .and_then(|profile| profile.reach)
        .filter(|&reach| reach > 0.0)
        .map_or(f32::MAX, |reach| distance / reach)
}
