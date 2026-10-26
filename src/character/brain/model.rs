//! Core data types for the utility brains.
//!
//! This module only holds plain data: what a brain can sense
//! ([`DecisionContext`]), what it can choose ([`Behavior`]: a [`Motion`] and/or an [`Action`]), and
//! the static definitions it scores against ([`BrainDefinition`]). It knows nothing about
//! ECS; [`super`] wires these types to components, [`super::scoring`] holds
//! the math, [`super::considerations`] holds what brains can sense, and
//! [`super::brains`] holds the shipped definitions.

use bevy::prelude::{Deref, Vec2};

use crate::items::prelude::EquipmentSlot;

/// Distance at which a melee brain considers a target to be in reach.
pub const MELEE_RANGE: f32 = 64.0;
/// Preferred minimum distance a ranged or flying brain keeps from its target.
pub const CAST_MIN_RANGE: f32 = 200.0;
/// Preferred maximum distance a ranged or flying brain keeps from its target.
pub const CAST_MAX_RANGE: f32 = 360.0;
pub const MELEE_DISTANCE_RANGE: Interval = Interval::new(0.0, MELEE_RANGE);
pub const CAST_DISTANCE_RANGE: Interval = Interval::new(CAST_MIN_RANGE, CAST_MAX_RANGE);
/// Core of the cast distance range where shot quality stays full.
pub const CAST_CORE_DISTANCE_RANGE: Interval =
    Interval::new(CAST_MIN_RANGE + 20.0, CAST_MAX_RANGE - 20.0);
/// Distance a caster opens before it stops casting to regenerate mana.
pub const RECOVERY_DISTANCE_RANGE: Interval = Interval::new(300.0, 340.0);
/// Distance from the target at which a wounded warrior may drink its potion.
pub const HEAL_SAFE_DISTANCE: f32 = 200.0;
/// Speed below which a target counts as holding still rather than orbiting.
pub const TARGET_MOVING_SPEED: f32 = 20.0;

/// An ordered `min < max` interval: a distance range, a consideration's input
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

/// How a character moves: a continuous motion that runs until replaced, or a
/// finite committed move that takes exclusive control of steering until its
/// executor reports it finished and is never re-scored while it runs. Attack execution stays with `AttackState`. New behaviors
/// should be decisions over these motions, not new variants, unless the
/// steering contract itself is new.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Motion {
    /// Disengaged: alternates standing and ambling, paced by the `Wander`
    /// component.
    #[default]
    Calm,
    Chase,
    ReturnToAnchor,
    /// Wounded break: still in the fight, just getting away from the target.
    Flee,
    KeepDistance(Interval),
    /// Stands still on purpose. Scored like any motion, so it can win the
    /// slot and keep momentum; an empty slot also stands still, but nothing
    /// chose it.
    Idle,
    /// Circles the target inside the distance range while equipment stays hot.
    Orbit(Interval),
    /// Committed straight run; the `Dash` component's policy sets its aim.
    Dash,
    /// Committed dive through a snapshot of the target and back out.
    Swoop,
}

impl Motion {
    /// Number of distinct motions, for per-motion arrays.
    pub const COUNT: usize = 9;

    /// Every finite motion, the ones that need a capability to carry out. A
    /// payload here only stands in; arrays index by variant.
    pub const FINITE: [Self; 2] = [Self::Dash, Self::Swoop];

    /// Position of this motion in per-motion arrays, ignoring any distance range.
    pub const fn index(self) -> usize {
        match self {
            Self::Calm => 0,
            Self::Chase => 1,
            Self::ReturnToAnchor => 2,
            Self::Flee => 3,
            Self::KeepDistance(_) => 4,
            Self::Idle => 5,
            Self::Orbit(_) => 6,
            Self::Dash => 7,
            Self::Swoop => 8,
        }
    }

    /// Whether this motion ends on its own and reports it, rather than
    /// running until replaced.
    pub const fn is_finite(self) -> bool {
        matches!(self, Self::Dash | Self::Swoop)
    }

    /// Whether steering this motion needs a live target.
    pub fn needs_target(self) -> bool {
        // Flee is still fighting: the target is live, the goal is just distance.
        matches!(
            self,
            Self::Chase
                | Self::KeepDistance(_)
                | Self::Orbit(_)
                | Self::Flee
                | Self::Dash
                | Self::Swoop
        )
    }

    /// One-word label for debug displays.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Calm => "Calm",
            Self::Chase => "Chase",
            Self::ReturnToAnchor => "ReturnToAnchor",
            Self::Flee => "Flee",
            Self::KeepDistance(_) => "KeepDistance",
            Self::Idle => "Idle",
            Self::Orbit(_) => "Orbit",
            Self::Dash => "Dash",
            Self::Swoop => "Swoop",
        }
    }
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

/// The one non-movement thing a character does at a time: use equipment or an
/// ability.
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

/// Position of a motion in a [`MotionActionTable`]: motions come first.
pub const fn motion_table_index(motion: Motion) -> usize {
    motion.index()
}

/// Position of an action in a [`MotionActionTable`]: actions follow the motions.
pub const fn action_table_index(action: Action) -> usize {
    Motion::COUNT + action.index()
}

/// A type with its own entry in a [`MotionActionTable`].
pub trait MotionActionKey: Copy {
    /// Position of this key in a [`MotionActionTable`].
    fn table_index(self) -> usize;
}

impl MotionActionKey for Motion {
    fn table_index(self) -> usize {
        motion_table_index(self)
    }
}

impl MotionActionKey for Action {
    fn table_index(self) -> usize {
        action_table_index(self)
    }
}

/// A raw table position, for extractors that pick their entry with a const generic.
impl MotionActionKey for usize {
    fn table_index(self) -> usize {
        self
    }
}

impl MotionActionKey for Ability {
    fn table_index(self) -> usize {
        Action::Ability(self).table_index()
    }
}

/// One value per motion and per action, indexed by any [`MotionActionKey`].
#[derive(Clone, Copy, Debug)]
pub struct MotionActionTable<T>([T; Motion::COUNT + Action::COUNT]);

impl<T: Copy> MotionActionTable<T> {
    /// A table with every entry set to `value`.
    pub fn splat(value: T) -> Self {
        Self([value; Motion::COUNT + Action::COUNT])
    }
}

impl<T: Default> Default for MotionActionTable<T> {
    fn default() -> Self {
        Self(std::array::from_fn(|_| T::default()))
    }
}

impl<T, K: MotionActionKey> std::ops::Index<K> for MotionActionTable<T> {
    type Output = T;

    fn index(&self, key: K) -> &T {
        &self.0[key.table_index()]
    }
}

impl<T, K: MotionActionKey> std::ops::IndexMut<K> for MotionActionTable<T> {
    fn index_mut(&mut self, key: K) -> &mut T {
        &mut self.0[key.table_index()]
    }
}

/// One candidate a brain can choose: how the character moves, what it does,
/// and how firmly it holds on. A behavior claims the motion slot, the action
/// slot, or both.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Behavior {
    pub motion: Option<Motion>,
    pub action: Option<Action>,
    pub commit: CommitPolicy,
}

/// How firmly a behavior holds its slots once it wins them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CommitPolicy {
    /// Re-scored every tick; the holder gets momentum and exit-side thresholds.
    Momentum,
    /// Not re-scored until its finite parts finish or an abort guard fails.
    /// Even then it holds for at least `min_seconds` from its start, so a
    /// short action still leaves a beat before the next move.
    Lock { min_seconds: f32 },
}

impl CommitPolicy {
    /// A lock with no minimum hold.
    pub const LOCK: Self = Self::Lock { min_seconds: 0.0 };

    /// Whether this is [`CommitPolicy::Lock`].
    pub const fn is_lock(self) -> bool {
        matches!(self, Self::Lock { .. })
    }

    /// Shortest time a lock holds its slots; zero for momentum.
    pub const fn min_seconds(self) -> f32 {
        match self {
            Self::Lock { min_seconds } => min_seconds,
            Self::Momentum => 0.0,
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
    /// Intended world velocity of the target, read from its movement.
    pub velocity: Vec2,
}

/// Everything a pure brain is allowed to look at during one decision tick:
/// facts sensed from the world and the brain's own earlier choices.
#[derive(Clone, Copy, Debug, Default)]
pub struct DecisionContext {
    pub me: SelfView,
    pub target: Option<TargetView>,
    /// Whether the target is inside each owned capability's reach; all false
    /// without a target.
    pub in_reach: MotionActionTable<bool>,
    pub memory: MemoryView,
}

/// The deciding character's own body state this tick.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SelfView {
    pub health_fraction: f32,
    /// Current mana as a `0.0..=1.0` fraction, or `1.0` for a non-caster.
    pub mana_fraction: f32,
    /// Whether a caster is running out a held mana break.
    pub mana_recovering: bool,
    /// Whether the body is stunned and cannot carry out a committed move.
    pub stunned: bool,
    pub anchor: Option<AnchorView>,
}

impl Default for SelfView {
    fn default() -> Self {
        Self {
            health_fraction: 1.0,
            mana_fraction: 1.0,
            mana_recovering: false,
            stunned: false,
            anchor: None,
        }
    }
}

/// Where the body stands relative to the anchor it is tethered to.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnchorView {
    pub overshoot_fraction: f32,
    pub arrived: bool,
}

/// The brain's own earlier choices, as scoring sees them.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MemoryView {
    /// The motion slot's holder, or `None` while the slot is empty.
    pub motion: Option<Motion>,
    /// An attack finished on the last tick; finite motions key their start off this.
    pub attack_finished: bool,
}

/// Which motions and actions may start right now. Derefs to its table, so
/// `availability[Motion::Swoop]` reads an entry directly.
#[derive(Clone, Copy, Debug, Deref)]
pub struct Availability(pub MotionActionTable<bool>);

/// A complete static brain: the list of behaviors it scores.
#[derive(Debug)]
pub struct BrainDefinition {
    pub behaviors: &'static [Decision<Behavior>],
}

/// One scored candidate: a value, its weight, and the considerations whose
/// geometric mean scales that weight.
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
    pub extract_raw_input: fn(&DecisionContext) -> Option<f32>,
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

/// How a consideration turns its raw input into a `0.0..=1.0` score.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Response {
    /// Normalize over `range`, then shape with `curve`.
    Curve { range: Interval, curve: Curve },
    /// Full once the raw input crosses `enter`, zero before it. While its
    /// decision is current the threshold is `exit` instead, so the gap holds
    /// the decision. `exit < enter` passes raw inputs at or above the threshold;
    /// otherwise, including no gap at all, it passes raw inputs at or below it.
    Threshold { enter: f32, exit: f32 },
}

impl Response {
    /// Score for `raw`, or `None` for a non-finite raw input.
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
