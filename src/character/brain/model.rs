//! Core data types for the utility brains.
//!
//! This module only holds plain data: what a brain can sense
//! ([`DecisionContext`]), what it can decide ([`BrainDecision`]), and the
//! static definitions it scores against ([`BrainDefinition`]). It knows nothing about
//! ECS; [`super`] wires these types to components, [`super::scoring`] holds
//! the math, [`super::considerations`] holds what brains can sense, and
//! [`super::brains`] holds the shipped definitions.

use bevy::prelude::Vec2;

use crate::items::prelude::EquipmentSlot;

/// Distance at which a melee brain considers a target to be in reach.
pub const MELEE_RANGE: f32 = 64.0;
/// Preferred minimum distance a ranged or flying brain keeps from its target.
pub const CAST_MIN_RANGE: f32 = 200.0;
/// Preferred maximum distance a ranged or flying brain keeps from its target.
pub const CAST_MAX_RANGE: f32 = 360.0;
pub const MELEE_BAND: DistanceBand = DistanceBand::new(0.0, MELEE_RANGE);
pub const CAST_BAND: DistanceBand = DistanceBand::new(CAST_MIN_RANGE, CAST_MAX_RANGE);
/// Core of the cast band where shot quality stays full.
pub const CAST_CORE_BAND: DistanceBand =
    DistanceBand::new(CAST_MIN_RANGE + 20.0, CAST_MAX_RANGE - 20.0);
/// Distance a caster opens before it stops casting to regenerate mana.
pub const RECOVERY_BAND: DistanceBand = DistanceBand::new(300.0, 340.0);
/// Distance from the target at which a wounded warrior may drink its potion.
pub const HEAL_SAFE_DISTANCE: f32 = 200.0;
/// Speed below which a target counts as holding still rather than orbiting.
pub const TARGET_MOVING_SPEED: f32 = 20.0;

/// An ordered `min < max` distance interval in world units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DistanceBand {
    min: f32,
    max: f32,
}

impl DistanceBand {
    /// Builds a band, rejecting bad bounds at compile time.
    pub const fn new(min: f32, max: f32) -> Self {
        assert!(
            min.is_finite() && max.is_finite() && min < max,
            "distance band needs finite min < max"
        );
        Self { min, max }
    }

    pub const fn min(self) -> f32 {
        self.min
    }

    pub const fn max(self) -> f32 {
        self.max
    }

    pub fn error(self, distance: f32) -> f32 {
        if distance < self.min {
            distance - self.min
        } else if distance > self.max {
            distance - self.max
        } else {
            0.0
        }
    }

    pub fn center_error(self, distance: f32) -> f32 {
        let half_width = (self.max - self.min) * 0.5;
        ((distance - (self.min + self.max) * 0.5) / half_width).clamp(-1.0, 1.0)
    }
}

/// A uniformly rolled duration entered when an option becomes current.
/// Only calm options use this today.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CommitWindow {
    min: f32,
    max: f32,
}

impl CommitWindow {
    /// Builds a window, rejecting bad bounds at compile time.
    pub const fn new(min: f32, max: f32) -> Self {
        assert!(
            min.is_finite() && max.is_finite() && min >= 0.0 && min <= max,
            "commit window needs finite, non-negative, ordered bounds"
        );
        Self { min, max }
    }

    pub const fn min(self) -> f32 {
        self.min
    }

    pub const fn max(self) -> f32 {
        self.max
    }
}

/// Motion selected by a brain. Attack execution stays with `AttackState`.
/// New behaviors should be scored options over these modes, not new variants,
/// unless the steering contract itself is new.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum MotionMode {
    #[default]
    Idle,
    Chase,
    ReturnToAnchor,
    /// Wounded break: still in the fight, just getting away from the target.
    Flee,
    KeepDistance(DistanceBand),
    /// Plants in place with a live target: the ice mage's firing stance.
    Hold,
    /// Firing-line sidestep: orbits the target while equipment stays hot.
    Strafe,
    /// Committed bite dive through a snapshot of the target and back out.
    Swoop,
    /// Committed break away from the target along a bearing fixed at the call.
    Kite,
    /// Committed tangent hop after a completed fire volley.
    Burst,
    /// Committed straight slide away from a target that crossed the danger ring.
    Slide,
    Wander,
}

/// Behavior groups for arbitration in [`super::scoring`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BehaviorGroup {
    Aggro,
    Return,
    Passive,
}

impl MotionMode {
    /// Whether this mode is a one-shot commitment that ends by clearing its
    /// call latch rather than by re-scoring. Commitments are always Aggro and
    /// always need a target.
    pub fn is_commitment(self) -> bool {
        matches!(self, Self::Swoop | Self::Kite | Self::Burst | Self::Slide)
    }

    pub fn needs_target(self) -> bool {
        self.is_commitment()
            || matches!(
                self,
                Self::Chase | Self::KeepDistance(_) | Self::Hold | Self::Strafe | Self::Flee
            )
    }

    pub fn is_calm(self) -> bool {
        matches!(self, Self::Idle | Self::Wander)
    }

    pub fn next_calm(self) -> MotionMode {
        match self {
            Self::Idle => Self::Wander,
            _ => Self::Idle,
        }
    }

    pub fn group(self) -> BehaviorGroup {
        // Flee is still fighting: the target is live, the goal is just
        // distance. Going home is Return and only scores disengaged.
        // Commitments (`is_commitment`) are Aggro moves: clearing their call
        // latch zeroes their score, which the arbiter treats as an ineligible
        // incumbent and exits at once.
        match self {
            Self::Chase
            | Self::KeepDistance(_)
            | Self::Hold
            | Self::Strafe
            | Self::Swoop
            | Self::Kite
            | Self::Burst
            | Self::Slide
            | Self::Flee => BehaviorGroup::Aggro,
            Self::ReturnToAnchor => BehaviorGroup::Return,
            Self::Idle | Self::Wander => BehaviorGroup::Passive,
        }
    }
}

/// The complete output of one brain tick. Movement, equipment, healing, and
/// breathing intents are independent.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BrainDecision {
    pub motion: MotionMode,
    pub slot_intents: [bool; EquipmentSlot::ALL.len()],
    pub wants_heal: bool,
    pub wants_breathe: bool,
}

/// Target data visible to a brain this tick.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TargetView {
    pub distance: f32,
    pub line_of_sight: bool,
    /// Unit direction from the viewer to the target.
    pub bearing: Vec2,
    /// Intended world velocity of the target, read from its locomotion.
    pub velocity: Vec2,
    /// Signed tangential component of the target's velocity around the viewer.
    /// Positive means the target circles counter-clockwise.
    pub tangential_velocity: f32,
}

/// Everything a pure brain is allowed to look at during one decision tick.
#[derive(Clone, Copy, Debug)]
pub struct DecisionContext {
    pub target: Option<TargetView>,
    pub health_fraction: f32,
    /// Current mana as a `0.0..=1.0` fraction, or `1.0` for a non-caster.
    pub mana_fraction: f32,
    /// Whether a caster is running out a latched mana break.
    pub mana_recovering: bool,
    pub slot_ready: [bool; EquipmentSlot::ALL.len()],
    pub anchor_overshoot_fraction: Option<f32>,
    pub anchor_arrived: Option<bool>,
    pub calm_phase: MotionMode,
    pub strafe_band: Option<DistanceBand>,
    pub swoop_called: bool,
    /// A warrior is currently running out a break from its target.
    pub kite_called: bool,
    /// A fire mage has a tangent burst left over from its last volley.
    pub burst_called: bool,
    /// An ice mage has been pushed inside its danger ring and called a slide.
    pub slide_called: bool,
    /// An attack finished on the last tick; dashes latch their call off this.
    pub attack_finished: bool,
    /// A dash is currently latched, under any policy.
    pub dash_called: bool,
    /// The latched dash is off cooldown and may rearm.
    pub dash_ready: bool,
    /// Target distance calling an escape dash, when the brain dashes to escape.
    pub dash_trigger_range: Option<f32>,
    /// The swoop lottery is due for another roll.
    pub swoop_due: bool,
}

impl Default for DecisionContext {
    fn default() -> Self {
        Self {
            target: None,
            health_fraction: 1.0,
            mana_fraction: 1.0,
            mana_recovering: false,
            slot_ready: [false; EquipmentSlot::ALL.len()],
            anchor_overshoot_fraction: None,
            anchor_arrived: None,
            calm_phase: MotionMode::Idle,
            strafe_band: None,
            swoop_called: false,
            kite_called: false,
            burst_called: false,
            slide_called: false,
            attack_finished: false,
            dash_called: false,
            dash_ready: true,
            dash_trigger_range: None,
            swoop_due: false,
        }
    }
}

/// A complete static brain definition.
#[derive(Debug)]
pub struct BrainDefinition {
    pub motion_options: &'static [ScoredOption<MotionMode>],
    pub slot_options: [&'static [ScoredOption<()>]; EquipmentSlot::ALL.len()],
    pub heal_options: &'static [ScoredOption<()>],
    pub breath_options: &'static [ScoredOption<()>],
}

/// One candidate motion or slot request.
#[derive(Clone, Copy, Debug)]
pub struct ScoredOption<T> {
    pub value: T,
    pub priority: f32,
    pub commit: Option<CommitWindow>,
    pub considerations: &'static [Consideration],
    pub gates: &'static [Gate],
}

/// Which edge of the character's distance band a [`Gate`] measures from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BandEdge {
    Min,
    Max,
}

/// Dual-threshold eligibility gate over target distance. Offsets measure from
/// one edge of the character's band; challengers enter at `enter_offset`,
/// incumbents leave past `exit_offset`, and the gap between them holds the mode.
#[derive(Clone, Copy, Debug)]
pub struct Gate {
    pub name: &'static str,
    pub edge: BandEdge,
    pub enter_offset: f32,
    pub exit_offset: f32,
}

impl Gate {
    /// Builds a band-relative gate.
    pub const fn new(
        name: &'static str,
        edge: BandEdge,
        enter_offset: f32,
        exit_offset: f32,
    ) -> Self {
        Self {
            name,
            edge,
            enter_offset,
            exit_offset,
        }
    }
}

/// One normalized input to an option's score.
#[derive(Clone, Copy, Debug)]
pub struct Consideration {
    pub name: &'static str,
    pub read: fn(&DecisionContext) -> Option<f32>,
    pub range: NormalizedRange,
    pub curve: Curve,
}

/// An ordered `min < max` raw-value range that normalizes to `0.0..=1.0`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NormalizedRange {
    min: f32,
    max: f32,
}

impl NormalizedRange {
    /// Builds a range, rejecting bad bounds at compile time.
    pub const fn new(min: f32, max: f32) -> Self {
        assert!(
            min.is_finite() && max.is_finite() && min < max,
            "consideration range needs finite min < max"
        );
        Self { min, max }
    }

    pub const fn min(self) -> f32 {
        self.min
    }

    pub const fn max(self) -> f32 {
        self.max
    }

    pub fn normalize(self, raw: f32) -> Option<f32> {
        if !raw.is_finite() {
            return None;
        }
        let normalized = ((raw - self.min) / (self.max - self.min)).clamp(0.0, 1.0);
        normalized.is_finite().then_some(normalized)
    }
}

/// Response curve applied after normalizing a consideration's raw value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Curve {
    Linear,
    /// Steepens a partial input (`x^3`).
    Cubic,
    /// Full score through the middle, easing to zero at each end.
    Plateau {
        edge: f32,
    },
}

impl Curve {
    pub(crate) fn apply(self, normalized: f32) -> f32 {
        let value = match self {
            Self::Linear => normalized,
            Self::Cubic => normalized * normalized * normalized,
            Self::Plateau { edge } => {
                let edge = edge.clamp(0.0, 0.5);
                if edge <= 0.0 {
                    1.0
                } else {
                    let ramp = if normalized < edge {
                        normalized / edge
                    } else if normalized > 1.0 - edge {
                        (1.0 - normalized) / edge
                    } else {
                        1.0
                    };
                    smoothstep(ramp)
                }
            }
        };

        value.clamp(0.0, 1.0)
    }
}

fn smoothstep(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * value * (3.0 - 2.0 * value)
}
