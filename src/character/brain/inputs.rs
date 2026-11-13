use avian2d::prelude::LinearVelocity;
use bevy::ecs::query::QueryData;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::Brain;
use super::action::{Abilities, Ability, Action, find_carried_potion};
use super::capability::{AbilityCapabilities, CapabilityProfile, MotionCapabilities};
use super::motion::{HomeArea, Motion};
use super::perception::{Allies, TargetInfo, Targeting};
use crate::prelude::*;

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
    /// Whether the running motion holds the target at a distance.
    Circling,
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
    /// Whether the character owns this action and the world lets it run, whatever its cooldown.
    ActionUsable(Action),
    /// Whether the character is pressed against a wall or a crowd; see
    /// [`Kinematics::is_blocked_at`].
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
    stat_multipliers: &'static StatMultipliers,
    kinematics: &'static Kinematics,
    linear_velocity: Option<&'static LinearVelocity>,
    items: Option<&'static Items>,
    motion_capabilities: MotionCapabilities,
    abilities: Option<&'static Abilities>,
}

/// The queries behind ability and equipment profiles, allies and the world-allows-use checks.
#[derive(SystemParam)]
pub(super) struct DecisionQueries<'w, 's> {
    equipment: EquipmentRead<'w, 's>,
    consumables: Query<'w, 's, (), With<Consumable>>,
    items_in_use: Query<'w, 's, (), With<EquipmentInUse>>,
    ability_capabilities: Query<'w, 's, AbilityCapabilities>,
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

    pub fn is_stunned(&self) -> bool {
        self.components.stat_multipliers.is_stunned()
    }

    /// The profile of the capability that carries out `motion`; see [`MotionCapabilities`].
    pub fn motion_profile(&self, motion: Motion) -> Option<CapabilityProfile> {
        self.components
            .motion_capabilities
            .capability_profile(motion)
    }

    /// The profile of the capability that carries out `action`: the equipped
    /// item's reach, or the component of the ability entity carrying it. `Idle` needs none.
    pub fn action_profile(&self, action: Action) -> Option<CapabilityProfile> {
        match action {
            Action::Use(slot) => self
                .queries
                .equipment
                .equippable_in_slot(self.components.entity, slot)
                .map(|equippable| CapabilityProfile {
                    reach: equippable.reach,
                    ..CapabilityProfile::default()
                }),
            Action::Ability(ability) => self
                .components
                .abilities
                .into_iter()
                .flat_map(Abilities::iter)
                .filter_map(|entity| self.queries.ability_capabilities.get(entity).ok())
                .find_map(|capabilities| capabilities.capability_profile(ability)),
            Action::Idle => Some(CapabilityProfile::default()),
        }
    }

    /// Whether the world lets `action` run now: usable equipment (or a held use
    /// already running), and a carried potion for `Heal`.
    pub fn world_allows_use(&self, action: Action) -> bool {
        let entity = self.components.entity;
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
                find_carried_potion(self.components.items, &queries.consumables).is_some()
            }
            Action::Ability(Ability::Breathe | Ability::Bite | Ability::Rally) | Action::Idle => {
                true
            }
        }
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
            ConsiderationInput::Circling => Some(f32::from(matches!(
                brain.motion(),
                Some(Motion::Orbit(_) | Motion::KeepDistance(_))
            ))),
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
                self.action_profile(action).is_some() && self.world_allows_use(action),
            )),
            ConsiderationInput::MovementBlocked => Some(f32::from(self.is_movement_blocked())),
        }
    }

    /// Whether the character is pressed against a wall or a crowd; see
    /// [`Kinematics::is_blocked_at`].
    fn is_movement_blocked(&self) -> bool {
        let components = self.components;
        components.linear_velocity.is_some_and(|linear_velocity| {
            components
                .kinematics
                .is_blocked_at(linear_velocity.0, components.stat_multipliers.speed)
        })
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
