//! Core data types for the utility brains.
//!
//! This module only holds plain data: what a brain can sense
//! ([`DecisionContext`]), what it can choose ([`Movement`], [`Action`]), and
//! the static definitions it scores against ([`BrainDefinition`]). It knows nothing about
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
pub const MELEE_BAND: Interval = Interval::new(0.0, MELEE_RANGE);
pub const CAST_BAND: Interval = Interval::new(CAST_MIN_RANGE, CAST_MAX_RANGE);
/// Core of the cast band where shot quality stays full.
pub const CAST_CORE_BAND: Interval = Interval::new(CAST_MIN_RANGE + 20.0, CAST_MAX_RANGE - 20.0);
/// Distance a caster opens before it stops casting to regenerate mana.
pub const RECOVERY_BAND: Interval = Interval::new(300.0, 340.0);
/// Distance from the target at which a wounded warrior may drink its potion.
pub const HEAL_SAFE_DISTANCE: f32 = 200.0;
/// Speed below which a target counts as holding still rather than orbiting.
pub const TARGET_MOVING_SPEED: f32 = 20.0;

/// An ordered `min < max` interval: a distance band, a consideration's input
/// range, or a span of seconds to roll from.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Interval {
    min: f32,
    max: f32,
}

impl Interval {
    /// Builds an interval, rejecting bad bounds at compile time.
    pub const fn new(min: f32, max: f32) -> Self {
        assert!(
            min.is_finite() && max.is_finite() && min < max,
            "interval needs finite min < max"
        );
        Self { min, max }
    }

    pub const fn min(self) -> f32 {
        self.min
    }

    pub const fn max(self) -> f32 {
        self.max
    }

    /// Signed distance outside the interval; zero inside it.
    pub fn error(self, value: f32) -> f32 {
        if value < self.min {
            value - self.min
        } else if value > self.max {
            value - self.max
        } else {
            0.0
        }
    }

    /// Offset from the center as a fraction of the half-width, clamped to `-1.0..=1.0`.
    pub fn center_error(self, value: f32) -> f32 {
        let half_width = (self.max - self.min) * 0.5;
        ((value - (self.min + self.max) * 0.5) / half_width).clamp(-1.0, 1.0)
    }

    /// Maps `raw` onto `0.0..=1.0` across the interval, clamping outside it.
    pub fn normalize(self, raw: f32) -> Option<f32> {
        if !raw.is_finite() {
            return None;
        }
        let normalized = ((raw - self.min) / (self.max - self.min)).clamp(0.0, 1.0);
        normalized.is_finite().then_some(normalized)
    }
}

/// Locomotion selected by a brain: where it wants to be. Attack execution
/// stays with `AttackState`, and committed moves are a [`Maneuver`], not a mode.
/// New behaviors should be decisions over these modes, not new variants,
/// unless the steering contract itself is new.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum MotionMode {
    /// Disengaged: alternates standing and ambling, paced by the `Wander`
    /// component.
    #[default]
    Calm,
    Chase,
    ReturnToAnchor,
    /// Wounded break: still in the fight, just getting away from the target.
    Flee,
    KeepDistance(Interval),
    /// Plants in place with a live target: the ice mage's firing stance.
    Hold,
    /// Firing-line sidestep: orbits the target inside the band while
    /// equipment stays hot.
    Strafe(Interval),
}

impl MotionMode {
    /// Whether steering this mode needs a live target.
    pub fn needs_target(self) -> bool {
        // Flee is still fighting: the target is live, the goal is just distance.
        matches!(
            self,
            Self::Chase | Self::KeepDistance(_) | Self::Hold | Self::Strafe(_) | Self::Flee
        )
    }
}

/// A committed move that takes exclusive control of steering until its
/// executor reports it finished. It competes with the modes on the movement
/// layer but is never re-scored while it runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Maneuver {
    /// Committed straight run; the `Dash` component's policy sets its aim.
    Dash,
    /// Committed dive through a snapshot of the target and back out.
    Swoop,
}

impl Maneuver {
    /// Every maneuver, in array order.
    pub const ALL: [Self; 2] = [Self::Dash, Self::Swoop];

    /// One-word label for debug displays.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Dash => "Dash",
            Self::Swoop => "Swoop",
        }
    }
}

/// What the exclusive movement layer picks between: a sustained mode or a
/// committed maneuver.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Movement {
    Mode(MotionMode),
    Maneuver(Maneuver),
}

/// An innate move a brain can ask its body for, carried out by a capability
/// component in the `ability` module.
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
    /// Every ability, in array order.
    pub const ALL: [Self; 3] = [Self::Heal, Self::Breathe, Self::Bite];
}

/// What the independent action layer can request. Each action fires on its
/// own whenever one of its decisions scores.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// Use the item equipped in a slot.
    Use(EquipmentSlot),
    Ability(Ability),
}

impl Action {
    /// Number of distinct actions, for per-action arrays.
    pub const COUNT: usize = EquipmentSlot::ALL.len() + Ability::ALL.len();

    /// Position of this action in per-action arrays.
    pub const fn index(self) -> usize {
        match self {
            Self::Use(slot) => slot as usize,
            Self::Ability(ability) => EquipmentSlot::ALL.len() + ability as usize,
        }
    }
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

/// What the decision tick needs to know about one capability, copied from the
/// component that carries it out so scoring never reaches into executor state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tuning {
    /// Target distance within which the capability should be used, when it has one.
    pub reach: Option<f32>,
    /// Probability that a winning maneuver actually starts.
    pub chance: f32,
    /// Seconds before a maneuver may be tried again after it ends or misses.
    pub cooldown: f32,
}

impl Default for Tuning {
    fn default() -> Self {
        Self {
            reach: None,
            chance: 1.0,
            cooldown: 0.0,
        }
    }
}

/// Everything a pure brain is allowed to look at during one decision tick:
/// facts sensed from the world, the brain's own earlier choices, and the
/// tuning of the capabilities its entity carries.
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
    /// The mode currently driving the body.
    pub mode: MotionMode,
    /// Whether a caster is running out a held mana break.
    pub mana_recovering: bool,
    pub active_maneuver: Option<Maneuver>,
    /// `None` for a maneuver this entity has no capability for.
    pub maneuver_tuning: [Option<Tuning>; Maneuver::ALL.len()],
    /// Owned and off cooldown, so it may be scored and started.
    pub maneuver_ready: [bool; Maneuver::ALL.len()],
    /// `None` for an action this entity cannot carry out.
    pub action_tuning: [Option<Tuning>; Action::COUNT],
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
            mode: MotionMode::Calm,
            mana_recovering: false,
            active_maneuver: None,
            maneuver_tuning: [None; Maneuver::ALL.len()],
            maneuver_ready: [false; Maneuver::ALL.len()],
            action_tuning: [Some(Tuning::default()); Action::COUNT],
        }
    }
}

impl DecisionContext {
    /// Whether the body can start `movement` right now: modes always, a
    /// maneuver only when owned and off cooldown.
    pub fn can_start(&self, movement: Movement) -> bool {
        match movement {
            Movement::Mode(_) => true,
            Movement::Maneuver(maneuver) => self.maneuver_ready[maneuver as usize],
        }
    }

    /// Whether the body carries the capability for `action`.
    pub fn can_perform(&self, action: Action) -> bool {
        self.action_tuning[action.index()].is_some()
    }
}

/// A complete static brain: one decision list per layer.
#[derive(Debug)]
pub struct BrainDefinition {
    /// Exclusive layer: the best decision wins, and the current mode keeps momentum.
    pub movement: &'static [Decision<Movement>],
    /// Independent layer: every action with a scoring decision fires.
    pub actions: &'static [Decision<Action>],
    /// Farthest target distance at which equipment use may fire. Scored as
    /// the equipment's reach and re-checked live by the applier. `None` means
    /// the brain never uses equipment.
    pub equipment_reach: Option<f32>,
}

/// One scored candidate on a layer: an action, its weight, and the
/// considerations whose geometric mean scales that weight.
#[derive(Clone, Copy, Debug)]
pub struct Decision<T> {
    pub value: T,
    pub weight: f32,
    pub considerations: &'static [Consideration],
}

/// One normalized input to a decision's score.
#[derive(Clone, Copy, Debug)]
pub struct Consideration {
    pub name: &'static str,
    pub read: fn(&DecisionContext) -> Option<f32>,
    pub response: Response,
    /// Scores `1 - response` instead, so one reader serves a fact and its opposite.
    pub inverted: bool,
}

impl Consideration {
    /// The opposite fact under a new name. A missing input stays neutral.
    pub const fn inverted(self, name: &'static str) -> Self {
        Self {
            name,
            inverted: !self.inverted,
            ..self
        }
    }
}

/// How a consideration turns its raw reading into a `0.0..=1.0` score.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Response {
    /// Normalize over `range`, then shape with `curve`.
    Curve { range: Interval, curve: Curve },
    /// Full once the reading crosses `enter`, zero before it. While its
    /// decision is current the threshold is `exit` instead, so the gap holds
    /// the decision. `exit < enter` passes readings at or above the threshold;
    /// otherwise, including no gap at all, it passes readings at or below it.
    Threshold { enter: f32, exit: f32 },
}

impl Response {
    /// Score for `raw`, or `None` for a non-finite reading.
    pub fn score(self, raw: f32, is_current: bool) -> Option<f32> {
        match self {
            Self::Curve { range, curve } => range.normalize(raw).map(|value| curve.apply(value)),
            Self::Threshold { enter, exit } => {
                if !raw.is_finite() {
                    return None;
                }
                let threshold = if is_current { exit } else { enter };
                let passes = if exit < enter {
                    raw >= threshold
                } else {
                    raw <= threshold
                };
                Some(if passes { 1.0 } else { 0.0 })
            }
        }
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
