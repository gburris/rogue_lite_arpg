//! What scoring reads, answered on demand: [`DecisionInputs`] reads each
//! consideration's input, capability profile and availability straight from the
//! query item when the decision asks for it, so nothing is copied up front.

use bevy::ecs::query::QueryData;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::Brain;
use super::action::{Ability, Action, Bite, Breath, Heal, find_carried_potion};
use super::capability::CapabilityProfile;
use super::engine::Behavior;
use super::motion::{Dash, HomeArea, Motion, Orbit, Swoop, Wander};
use super::perception::Targeting;
use crate::prelude::*;

/// The raw fact a consideration reads; see [`DecisionInputs::read`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ConsiderationInput {
    HasTarget,
    LineOfSight,
    TargetDistance,
    MissingHealth,
    ManaFraction,
    /// Whether this motion's kind is the one running; its payload is ignored.
    MotionRunning(Motion),
    /// Whether the running motion holds the target at a distance.
    Circling,
    /// How far past its home area range the character is.
    HomeAreaPull,
    AttackFinished,
    /// Target distance over this motion's reach; at most `1.0` is in reach.
    MotionReachFraction(Motion),
    /// Target distance over this action's reach; at most `1.0` is in reach.
    ActionReachFraction(Action),
    SecondsSinceTargeted,
}

/// The brain's own earlier choices, as scoring reads them.
#[derive(Clone, Copy, Debug)]
pub(super) struct BrainMemory {
    pub motion: Option<Motion>,
    /// An attack finished since the last decision.
    pub attack_finished: bool,
}

/// The components one decision reads, including every capability an entity may
/// carry. A new capability component is listed here and read in
/// [`DecisionInputs::motion_profile`] or [`DecisionInputs::action_profile`].
#[derive(QueryData)]
pub(super) struct DecisionInputComponents {
    entity: Entity,
    transform: &'static Transform,
    has_target: Has<Targeting>,
    target_info: Option<&'static TargetInfo>,
    health: Option<&'static Health>,
    mana: Option<&'static Mana>,
    home_area: Option<&'static HomeArea>,
    kinematics: Option<&'static Kinematics>,
    items: Option<&'static Items>,
    dash: Option<&'static Dash>,
    swoop: Option<&'static Swoop>,
    orbit: Option<&'static Orbit>,
    wander: Option<&'static Wander>,
    bite: Option<&'static Bite>,
    breath: Option<&'static Breath>,
    heal: Option<&'static Heal>,
}

/// The queries behind equipment profiles and the world-allows-use checks.
#[derive(SystemParam)]
pub(super) struct DecisionQueries<'w, 's> {
    equipment: EquipmentRead<'w, 's>,
    consumables: Query<'w, 's, (), With<Consumable>>,
    items_in_use: Query<'w, 's, (), With<EquipmentInUse>>,
}

/// One decision's view of the world: references only, read on demand.
pub(super) struct DecisionInputs<'a, 'w, 's, 'qw, 'qs> {
    pub item: &'a DecisionInputComponentsItem<'w, 's>,
    pub queries: &'a DecisionQueries<'qw, 'qs>,
    pub now: f32,
}

impl DecisionInputs<'_, '_, '_, '_, '_> {
    /// The tracked target while targeting one at a finite distance. `TargetInfo`
    /// also tracks a watched entity, so every target input reads through here.
    pub fn target(&self) -> Option<&TargetInfo> {
        self.item
            .target_info
            .filter(|info| self.item.has_target && info.distance.is_finite())
    }

    pub fn stunned(&self) -> bool {
        self.item.kinematics.is_some_and(Kinematics::is_stunned)
    }

    /// The profile of the capability that carries out `motion`; `None` when the
    /// entity lacks it. Motions steered without a component are always owned.
    pub fn motion_profile(&self, motion: Motion) -> Option<CapabilityProfile> {
        match motion {
            // Their components carry no decision settings, only ownership.
            Motion::Wander => self.item.wander.map(|_| CapabilityProfile::default()),
            Motion::Orbit(_) => self.item.orbit.map(|_| CapabilityProfile::default()),
            Motion::Dash => self.item.dash.map(Dash::profile),
            Motion::Swoop => self.item.swoop.map(Swoop::profile),
            Motion::Chase
            | Motion::ReturnToHome
            | Motion::Flee
            | Motion::KeepDistance(_)
            | Motion::Idle => Some(CapabilityProfile::default()),
        }
    }

    /// The profile of the capability that carries out `action`: the equipped
    /// item's reach, or the ability component's. `Idle` needs none.
    pub fn action_profile(&self, action: Action) -> Option<CapabilityProfile> {
        match action {
            Action::Use(slot) => self
                .queries
                .equipment
                .equippable_in_slot(self.item.entity, slot)
                .map(|equippable| CapabilityProfile {
                    reach: equippable.reach,
                    ..CapabilityProfile::default()
                }),
            Action::Ability(Ability::Heal) => self.item.heal.map(Heal::profile),
            Action::Ability(Ability::Breathe) => self.item.breath.map(Breath::profile),
            Action::Ability(Ability::Bite) => self.item.bite.map(Bite::profile),
            Action::Idle => Some(CapabilityProfile::default()),
        }
    }

    /// Whether the world lets `action` run now: usable equipment (or a held use
    /// already running), and a carried potion for `Heal`.
    fn can_use(&self, action: Action) -> bool {
        let entity = self.item.entity;
        let queries = self.queries;
        match action {
            Action::Use(slot) => {
                let held_item_in_use = !slot.is_instant_use()
                    && queries
                        .equipment
                        .item_in_slot(entity, slot)
                        .is_some_and(|item| queries.items_in_use.contains(item));
                held_item_in_use || queries.equipment.can_use(entity, slot).is_ok()
            }
            Action::Ability(Ability::Heal) => {
                find_carried_potion(self.item.items, &queries.consumables).is_some()
            }
            Action::Ability(Ability::Breathe | Ability::Bite) | Action::Idle => true,
        }
    }

    /// Whether `behavior` may start now: each part's capability is owned, off its
    /// cooldown on `brain`, and allowed by the world.
    pub fn is_available(&self, behavior: &Behavior, brain: &Brain) -> bool {
        let motion_available = |motion: Motion| {
            self.motion_profile(motion).is_some() && brain.is_cooldown_ready(motion, self.now)
        };
        let action_available = |action: Action| {
            self.action_profile(action).is_some()
                && brain.is_cooldown_ready(action, self.now)
                && self.can_use(action)
        };
        behavior.motion.is_none_or(motion_available) && behavior.action.is_none_or(action_available)
    }

    /// Answers `input`; `None` when it is absent, which vetoes the behavior.
    pub fn read(&self, input: ConsiderationInput, memory: BrainMemory) -> Option<f32> {
        match input {
            ConsiderationInput::HasTarget => flag(self.target().is_some()),
            ConsiderationInput::LineOfSight => {
                self.target().and_then(|target| flag(target.line_of_sight))
            }
            ConsiderationInput::TargetDistance => self.target().map(|target| target.distance),
            ConsiderationInput::MissingHealth => {
                Some(1.0 - self.item.health.and_then(Health::fraction).unwrap_or(1.0))
            }
            ConsiderationInput::ManaFraction => {
                Some(self.item.mana.and_then(Mana::fraction).unwrap_or(1.0))
            }
            ConsiderationInput::MotionRunning(motion) => flag(
                memory
                    .motion
                    .is_some_and(|running| running.index() == motion.index()),
            ),
            ConsiderationInput::Circling => flag(matches!(
                memory.motion,
                Some(Motion::Orbit(_) | Motion::KeepDistance(_))
            )),
            ConsiderationInput::HomeAreaPull => self.home_area_pull(memory),
            ConsiderationInput::AttackFinished => flag(memory.attack_finished),
            ConsiderationInput::MotionReachFraction(motion) => {
                self.reach_fraction(self.motion_profile(motion))
            }
            ConsiderationInput::ActionReachFraction(action) => {
                self.reach_fraction(self.action_profile(action))
            }
            ConsiderationInput::SecondsSinceTargeted => {
                self.target().map(|target| self.now - target.targeted_at)
            }
        }
    }

    /// How far past its home area range the character is. Once already heading
    /// home it reads full until arrival, so crossing back inside the range
    /// mid-return cannot hand the character straight back to wandering.
    fn home_area_pull(&self, memory: BrainMemory) -> Option<f32> {
        let home_area = self.item.home_area?;
        let position = self.item.transform.translation.xy();
        let overshoot_fraction = home_area.overshoot_fraction(position)?;
        if memory.motion == Some(Motion::ReturnToHome) {
            let arrived = home_area.distance_from(position) <= home_area.arrival_radius();
            flag(!arrived)
        } else {
            Some(overshoot_fraction)
        }
    }

    fn reach_fraction(&self, profile: Option<CapabilityProfile>) -> Option<f32> {
        self.target()
            .map(|target| reach_fraction(target.distance, profile))
    }
}

/// A yes-or-no input as `1.0` or `0.0`.
fn flag(set: bool) -> Option<f32> {
    Some(if set { 1.0 } else { 0.0 })
}

/// `distance` over the capability's reach; at most `1.0` is in reach.
/// `f32::MAX` for a capability that is missing or has no positive reach.
fn reach_fraction(distance: f32, profile: Option<CapabilityProfile>) -> f32 {
    profile
        .and_then(|profile| profile.reach)
        .filter(|&reach| reach > 0.0)
        .map_or(f32::MAX, |reach| distance / reach)
}
