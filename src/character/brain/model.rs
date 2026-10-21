//! Core data types for the utility brains.
//!
//! This module only holds plain data: what a brain can sense
//! ([`DecisionContext`]), what it can decide ([`BrainDecision`]), and the
//! static definitions it scores against ([`Brain`]). It knows nothing about
//! ECS; [`super`] wires these types to components, [`super::scoring`] holds
//! the math, [`super::considerations`] holds what brains can sense, and
//! [`super::brains`] holds the shipped definitions.

use crate::items::prelude::EquipmentSlot;

/// Distance at which a melee brain considers a target to be in reach.
pub const MELEE_RANGE: f32 = 64.0;
/// Preferred minimum distance a ranged brain keeps from its target.
pub const MAGE_MIN_RANGE: f32 = 200.0;
/// Preferred maximum distance a ranged brain keeps from its target.
pub const MAGE_MAX_RANGE: f32 = 300.0;
/// Distance from the target at which a wounded warrior may drink its potion.
/// Opening this much ground takes a few seconds of fleeing, so healing
/// never fires the instant health drops under the threshold.
pub const HEAL_SAFE_DISTANCE: f32 = 200.0;

/// A uniformly rolled commitment duration entered when an option becomes current.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CommitWindow {
    pub min: f32,
    pub max: f32,
}

/// Motion selected by a brain. This never represents attack execution; that
/// remains owned by `AttackState` and the equipment use observers.
/// Keep this vocabulary to generic locomotion: new enemy behaviors should be
/// new scored options over these modes (plus considerations), not new variants.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum MotionMode {
    #[default]
    Idle,
    Chase,
    ReturnToAnchor,
    /// Wounded break: still in the fight, just getting away from the target.
    /// Going home happens only once the target is lost (see ReturnToAnchor).
    Flee,
    KeepDistance {
        min: f32,
        max: f32,
    },
    /// Firing-line sidestep: orbits the target while equipment stays hot.
    Strafe,
    Wander,
}

/// Behavior groups used for reasoner-style arbitration in [`super::scoring`].
///
/// Groups are scored against each other before options compete within one,
/// so a passive commitment can never outbid a reactive move no matter how the
/// priorities are tuned.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BehaviorGroup {
    /// Fighting: chase, spacing, strafing, fleeing. Wins whenever a target is live.
    Aggro,
    /// Going home once disengaged. Wins on anchor-range overshoot.
    Return,
    /// Neither engaged nor going home: the wander/idle cycle. Wins when nothing else scores.
    Passive,
}

impl MotionMode {
    /// Whether this mode needs a live target to be driven safely.
    pub fn needs_target(self) -> bool {
        matches!(
            self,
            Self::Chase | Self::KeepDistance { .. } | Self::Strafe | Self::Flee
        )
    }

    /// Whether this mode is part of the idle/wander calm cycle.
    pub fn is_calm(self) -> bool {
        matches!(self, Self::Idle | Self::Wander)
    }

    /// Which behavior group this mode belongs to.
    pub fn group(self) -> BehaviorGroup {
        // Flee is still fighting: the target is live, the goal is just
        // distance. Going home is Return and only scores disengaged.
        match self {
            Self::Chase | Self::KeepDistance { .. } | Self::Strafe | Self::Flee => {
                BehaviorGroup::Aggro
            }
            Self::ReturnToAnchor => BehaviorGroup::Return,
            Self::Idle | Self::Wander => BehaviorGroup::Passive,
        }
    }
}

/// The complete output of one brain tick. Movement and slot intents are
/// independent so a character can keep moving while an item is in use.
/// Healing is separate from equipment: potions are consumed from the
/// inventory, never equipped.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BrainDecision {
    pub motion: MotionMode,
    pub commit: Option<CommitWindow>,
    pub slot_intents: [bool; EquipmentSlot::ALL.len()],
    /// Whether the brain wants to drink a carried potion this tick.
    pub wants_heal: bool,
}

/// Target data visible to a brain this tick.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TargetView {
    pub distance: f32,
    pub line_of_sight: bool,
}

/// Everything a pure brain is allowed to look at during one decision tick.
#[derive(Clone, Copy, Debug)]
pub struct DecisionContext {
    pub target: Option<TargetView>,
    /// Health as a `0.0..=1.0` fraction (`1.0` is unhurt).
    pub health_fraction: f32,
    pub slot_ready: [bool; EquipmentSlot::ALL.len()],
    /// Overshoot beyond the anchor range as a fraction of that range (`0.0`
    /// at the boundary, `1.0` one full range out), or `None` without an anchor.
    pub anchor_overshoot_fraction: Option<f32>,
    /// Whether the character has reached the anchor origin, or `None` without
    /// an anchor.
    pub anchor_arrived: Option<bool>,
    /// Which half of the calm idle/wander cycle should run next.
    pub calm_phase: MotionMode,
    pub within_commitment: bool,
}

impl Default for DecisionContext {
    fn default() -> Self {
        Self {
            target: None,
            health_fraction: 1.0,
            slot_ready: [false; EquipmentSlot::ALL.len()],
            anchor_overshoot_fraction: None,
            anchor_arrived: None,
            calm_phase: MotionMode::Idle,
            within_commitment: false,
        }
    }
}

/// A complete static brain definition.
#[derive(Debug)]
pub struct Brain {
    pub motion_options: &'static [ScoredOption<MotionMode>],
    pub slot_options: [&'static [ScoredOption<()>]; EquipmentSlot::ALL.len()],
    /// When to drink a carried potion, taken straight from the inventory.
    /// Empty for brains that never heal.
    pub heal_options: &'static [ScoredOption<()>],
}

/// One candidate motion or slot request.
#[derive(Clone, Copy, Debug)]
pub struct ScoredOption<T> {
    pub value: T,
    pub priority: f32,
    /// Uniform roll made when this option is entered. `None` is reactive.
    pub commit: Option<CommitWindow>,
    pub considerations: &'static [Consideration],
}

/// One normalized input to an option's score.
#[derive(Clone, Copy, Debug)]
pub struct Consideration {
    pub name: &'static str,
    pub read: fn(&DecisionContext) -> Option<f32>,
    pub range: (f32, f32),
    pub curve: Curve,
}

/// Response curve applied after normalizing a consideration's raw value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Curve {
    Linear,
}

impl Curve {
    pub(crate) fn apply(self, normalized: f32) -> Option<f32> {
        let value = match self {
            Self::Linear => normalized,
        };

        value.is_finite().then(|| value.clamp(0.0, 1.0))
    }
}
