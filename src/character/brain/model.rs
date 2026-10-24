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

/// Locomotion selected by a brain: where it wants to be. Attack execution
/// stays with `AttackState`, and committed moves are a [`Maneuver`], not a mode.
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
    Wander,
}

/// A committed move that takes exclusive control of steering until it ends.
/// It is chosen on its own channel, so it runs over whatever locomotion mode
/// is current and hands steering back when the executor reports it finished.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Maneuver {
    /// Committed break away from the target along a bearing fixed at the start.
    Kite,
    /// Committed tangent hop after a completed fire volley.
    Burst,
    /// Committed straight slide away from a target that crossed the danger ring.
    Slide,
    /// Committed dive through a snapshot of the target and back out.
    Swoop,
}

impl Maneuver {
    /// Every maneuver, in channel-array order.
    pub const ALL: [Self; 4] = [Self::Kite, Self::Burst, Self::Slide, Self::Swoop];

    /// One-word label for debug displays.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Kite => "Kite",
            Self::Burst => "Burst",
            Self::Slide => "Slide",
            Self::Swoop => "Swoop",
        }
    }
}

/// Something a brain can ask its body to do besides move or use equipment.
/// Each ability has its own channel, scored independently of the others.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ability {
    /// Drink a carried potion.
    Heal,
    /// Loose an innate projectile with no weapon.
    Breathe,
    /// Damage a target in contact reach.
    Bite,
}

impl Ability {
    /// Every ability, in channel-array order.
    pub const ALL: [Self; 3] = [Self::Heal, Self::Breathe, Self::Bite];
}

/// Behavior groups for arbitration in [`super::scoring`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BehaviorGroup {
    Aggro,
    Return,
    Passive,
}

/// The static facts about a [`MotionMode`] that the arbiter and executor
/// consult. Every mode picks exactly one row in [`MotionMode::traits`].
#[derive(Clone, Copy, Debug)]
struct ModeTraits {
    group: BehaviorGroup,
    needs_target: bool,
}

impl ModeTraits {
    /// A sustained fighting mode: re-scored every tick against a live target.
    const REACTIVE: Self = Self {
        group: BehaviorGroup::Aggro,
        needs_target: true,
    };
    /// Going home; only scores while disengaged.
    const HOMEWARD: Self = Self {
        group: BehaviorGroup::Return,
        needs_target: false,
    };
    /// The idle/wander cycle.
    const CALM: Self = Self {
        group: BehaviorGroup::Passive,
        needs_target: false,
    };
}

impl MotionMode {
    /// The single place a mode declares its group and target need.
    fn traits(self) -> ModeTraits {
        match self {
            // Flee is still fighting: the target is live, the goal is just distance.
            Self::Chase | Self::KeepDistance(_) | Self::Hold | Self::Strafe | Self::Flee => {
                ModeTraits::REACTIVE
            }
            Self::ReturnToAnchor => ModeTraits::HOMEWARD,
            Self::Idle | Self::Wander => ModeTraits::CALM,
        }
    }

    pub fn needs_target(self) -> bool {
        self.traits().needs_target
    }

    pub fn is_calm(self) -> bool {
        self.traits().group == BehaviorGroup::Passive
    }

    pub fn next_calm(self) -> MotionMode {
        match self {
            Self::Idle => Self::Wander,
            _ => Self::Idle,
        }
    }

    pub fn group(self) -> BehaviorGroup {
        self.traits().group
    }
}

/// The complete output of one brain tick. Locomotion, maneuver, equipment and
/// ability intents are independent channels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BrainDecision {
    pub motion: MotionMode,
    /// The best maneuver ready to start, if none is already running.
    pub maneuver: Option<Maneuver>,
    pub slot_intents: [bool; EquipmentSlot::ALL.len()],
    pub abilities: [bool; Ability::ALL.len()],
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

/// Chance and spacing of one maneuver, copied from the capability component
/// that executes it so the decision tick never reaches into executor state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ManeuverTuning {
    /// Probability that a ready, scored maneuver actually starts.
    pub chance: f32,
    /// Seconds before the maneuver may be tried again after it ends or misses.
    pub cooldown: f32,
    /// Target distance that triggers the maneuver, when it has one.
    pub trigger_range: Option<f32>,
}

/// What the decision tick needs to know about an ability the entity can carry
/// out, copied from the capability component that applies it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AbilityTuning {
    /// Target distance at which the ability should be armed, when it has a reach.
    pub reach: Option<f32>,
}

/// What the body reports about itself each tick: held calls, cooldowns, and
/// copies of the tuning that considerations need. Unlike a sense, none of it
/// comes from the world; it reflects the brain's own earlier decisions and the
/// capabilities its entity carries.
#[derive(Clone, Copy, Debug)]
pub struct SelfState {
    pub calm_phase: MotionMode,
    /// The locomotion mode currently driving the body.
    pub locomotion: MotionMode,
    /// Whether a caster is running out a held mana break.
    pub mana_recovering: bool,
    pub active_maneuver: Option<Maneuver>,
    /// `None` for a maneuver this entity has no capability for.
    pub maneuver_tuning: [Option<ManeuverTuning>; Maneuver::ALL.len()],
    /// Owned and off cooldown, so it may be scored and started.
    pub maneuver_ready: [bool; Maneuver::ALL.len()],
    /// `None` for an ability this entity has no capability for.
    pub ability_tuning: [Option<AbilityTuning>; Ability::ALL.len()],
}

impl Default for SelfState {
    fn default() -> Self {
        Self {
            calm_phase: MotionMode::Idle,
            locomotion: MotionMode::Idle,
            mana_recovering: false,
            active_maneuver: None,
            maneuver_tuning: [None; Maneuver::ALL.len()],
            maneuver_ready: [false; Maneuver::ALL.len()],
            ability_tuning: [Some(AbilityTuning::default()); Ability::ALL.len()],
        }
    }
}

/// Everything a pure brain is allowed to look at during one decision tick:
/// facts sensed from the world plus the body's [`SelfState`].
#[derive(Clone, Copy, Debug)]
pub struct DecisionContext {
    pub target: Option<TargetView>,
    pub health_fraction: f32,
    /// Current mana as a `0.0..=1.0` fraction, or `1.0` for a non-caster.
    pub mana_fraction: f32,
    pub slot_ready: [bool; EquipmentSlot::ALL.len()],
    pub anchor_overshoot_fraction: Option<f32>,
    pub anchor_arrived: Option<bool>,
    /// An attack finished on the last tick; maneuvers key their start off this.
    pub attack_finished: bool,
    pub self_state: SelfState,
}

impl Default for DecisionContext {
    fn default() -> Self {
        Self {
            target: None,
            health_fraction: 1.0,
            mana_fraction: 1.0,
            slot_ready: [false; EquipmentSlot::ALL.len()],
            anchor_overshoot_fraction: None,
            anchor_arrived: None,
            attack_finished: false,
            self_state: SelfState::default(),
        }
    }
}

/// A complete static brain definition: one scored option list per channel.
#[derive(Debug)]
pub struct BrainDefinition {
    /// Distance band this brain's gates measure against.
    pub band: Option<DistanceBand>,
    pub motion_options: &'static [ScoredOption<MotionMode>],
    pub maneuver_options: &'static [ScoredOption<Maneuver>],
    pub slot_options: [&'static [ScoredOption<()>]; EquipmentSlot::ALL.len()],
    pub ability_options: [&'static [ScoredOption<()>]; Ability::ALL.len()],
    /// Farthest live target distance at which slot intents may fire. `None`
    /// means the decision tick's verdict stands without a live range check.
    pub slot_reach: Option<f32>,
}

/// One candidate option on a channel.
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
