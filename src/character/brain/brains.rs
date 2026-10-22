//! The shipped static brain definitions.
//!
//! All tuning lives here: priorities, commitment windows, and which sensed
//! facts each option weighs. The scoring math stays in [`super::scoring`].

use super::considerations::{
    BADLY_HURT, BEYOND_ANCHOR_RANGE, BURST_CALLED, CAST_QUALITY, FAR_FROM_TARGET, HAS_ANCHOR,
    HAS_TARGET, IN_MELEE_RANGE, KITE_CALLED, LINE_OF_SIGHT, MAINHAND_READY, NO_TARGET, NOT_KITING,
    NOT_RECOVERING, NOT_SWOOPING, PASSIVE_IDLE, PASSIVE_WANDER, RECOVERING, SLIDE_CALLED,
    SWOOP_CALLED, WITHIN_CAST_RANGE,
};
use super::model::{
    BandEdge, Brain, CAST_CORE_BAND, CommitWindow, Gate, MELEE_BAND, MotionMode, RECOVERY_BAND,
    ScoredOption,
};
use crate::items::prelude::EquipmentSlot;

pub(crate) const MELEE_HYSTERESIS: f32 = 4.0;
const CAST_HYSTERESIS: f32 = 5.0;

const CHASE_MELEE_GATE: Gate =
    Gate::new("outside melee edge", BandEdge::Max, MELEE_HYSTERESIS, 0.0);
const HOLD_MELEE_GATE: Gate = Gate::new("inside melee edge", BandEdge::Max, 0.0, MELEE_HYSTERESIS);
const CHASE_CAST_GATE: Gate = Gate::new(
    "outside cast band",
    BandEdge::Max,
    CAST_HYSTERESIS,
    -CAST_HYSTERESIS,
);
const HOLD_CAST_GATE: Gate = Gate::new(
    "inside outer cast edge",
    BandEdge::Max,
    -CAST_HYSTERESIS,
    CAST_HYSTERESIS,
);
const INNER_CAST_GATE: Gate = Gate::new(
    "past inner cast edge",
    BandEdge::Min,
    CAST_HYSTERESIS,
    -CAST_HYSTERESIS,
);

const ENEMY_IDLE_OPTION: ScoredOption<MotionMode> = ScoredOption {
    value: MotionMode::Idle,
    priority: 1.0,
    commit: Some(CommitWindow::new(3.0, 5.0)),
    considerations: &[NO_TARGET, PASSIVE_IDLE],
    gates: &[],
};
const ENEMY_WANDER_OPTION: ScoredOption<MotionMode> = ScoredOption {
    value: MotionMode::Wander,
    priority: 1.0,
    commit: Some(CommitWindow::new(1.0, 2.0)),
    considerations: &[NO_TARGET, PASSIVE_WANDER],
    gates: &[],
};
const VILLAGER_IDLE_OPTION: ScoredOption<MotionMode> = ScoredOption {
    value: MotionMode::Idle,
    priority: 1.0,
    commit: Some(CommitWindow::new(1.0, 4.0)),
    considerations: &[NO_TARGET, PASSIVE_IDLE],
    gates: &[],
};
const VILLAGER_WANDER_OPTION: ScoredOption<MotionMode> = ScoredOption {
    value: MotionMode::Wander,
    priority: 1.0,
    commit: Some(CommitWindow::new(1.0, 2.5)),
    considerations: &[NO_TARGET, PASSIVE_WANDER],
    gates: &[],
};
const RETURN_TO_ANCHOR_OPTION: ScoredOption<MotionMode> = ScoredOption {
    value: MotionMode::ReturnToAnchor,
    priority: 6.0,
    commit: None,
    considerations: &[NO_TARGET, HAS_ANCHOR, BEYOND_ANCHOR_RANGE],
    gates: &[],
};
const FLEE_OPTION: ScoredOption<MotionMode> = ScoredOption {
    value: MotionMode::Flee,
    priority: 7.0,
    commit: None,
    considerations: &[HAS_TARGET, BADLY_HURT],
    gates: &[],
};
const CHASE_MELEE_OPTION: ScoredOption<MotionMode> = ScoredOption {
    value: MotionMode::Chase,
    priority: 4.0,
    commit: None,
    considerations: &[HAS_TARGET],
    gates: &[CHASE_MELEE_GATE],
};

/// Runs out the break rolled after a swing, then the normal chase/hold options
/// bring it back in for the next swing.
const WARRIOR_KITE_OPTION: ScoredOption<MotionMode> = ScoredOption {
    value: MotionMode::Kite,
    priority: 6.0,
    commit: None,
    considerations: &[HAS_TARGET, KITE_CALLED],
    gates: &[],
};

/// Holds melee reach instead of orbiting; sword readiness only gates the swing.
const KEEP_MELEE_RANGE_OPTION: ScoredOption<MotionMode> = ScoredOption {
    value: MotionMode::KeepDistance(MELEE_BAND),
    priority: 5.0,
    commit: None,
    considerations: &[HAS_TARGET],
    gates: &[HOLD_MELEE_GATE],
};
const CHASE_CAST_RANGE_OPTION: ScoredOption<MotionMode> = ScoredOption {
    value: MotionMode::Chase,
    priority: 4.0,
    commit: None,
    considerations: &[HAS_TARGET],
    gates: &[CHASE_CAST_GATE],
};
/// Sidesteps for a good firing lane; the target's tangent picks the orbit side
/// in `strafe_direction`. The fire mage keeps this moving even against a
/// stationary target, so its turns come from its own volleys instead of the
/// player's movement.
const FIRE_STRAFE_OPTION: ScoredOption<MotionMode> = ScoredOption {
    value: MotionMode::Strafe,
    priority: 5.5,
    commit: None,
    considerations: &[HAS_TARGET],
    gates: &[HOLD_CAST_GATE],
};
/// Plants in the cast band and lets the ice bolt do the work. The danger-ring
/// slide is the only time this mage gives ground.
const ICE_PLANT_OPTION: ScoredOption<MotionMode> = ScoredOption {
    value: MotionMode::Hold,
    priority: 5.0,
    commit: None,
    considerations: &[HAS_TARGET],
    gates: &[HOLD_CAST_GATE],
};
/// A tangent hop called when the fire staff finishes a volley.
const FIRE_BURST_OPTION: ScoredOption<MotionMode> = ScoredOption {
    value: MotionMode::Burst,
    priority: 6.5,
    commit: None,
    considerations: &[HAS_TARGET, BURST_CALLED],
    gates: &[],
};
/// A straight escape called when the player crosses the ice mage's danger ring.
/// The latched call alone scores it: re-adding a distance consideration would
/// cancel the run as soon as it opens range, the same way a band-edge gate
/// would cancel a dive mid-flight.
const ICE_SLIDE_OPTION: ScoredOption<MotionMode> = ScoredOption {
    value: MotionMode::Slide,
    priority: 6.8,
    commit: None,
    considerations: &[HAS_TARGET, SLIDE_CALLED],
    gates: &[],
};
/// Breaks contact until the caster's latched mana break releases. The two
/// casters share the mode but enter it on their own mana rhythms.
const FIRE_RECOVER_OPTION: ScoredOption<MotionMode> = ScoredOption {
    value: MotionMode::KeepDistance(RECOVERY_BAND),
    priority: 6.8,
    commit: None,
    considerations: &[HAS_TARGET, RECOVERING],
    gates: &[],
};
const ICE_RECOVER_OPTION: ScoredOption<MotionMode> = ScoredOption {
    value: MotionMode::KeepDistance(RECOVERY_BAND),
    priority: 6.6,
    commit: None,
    considerations: &[HAS_TARGET, RECOVERING],
    gates: &[],
};

/// Orbits the casting band; the bat breathes innately, no ready weapon needed.
const BAT_STRAFE_OPTION: ScoredOption<MotionMode> = ScoredOption {
    value: MotionMode::Strafe,
    priority: 5.5,
    commit: None,
    considerations: &[HAS_TARGET],
    gates: &[INNER_CAST_GATE, HOLD_CAST_GATE],
};
/// Holds the core so it keeps repositioning toward the middle.
const BAT_KEEP_OPTION: ScoredOption<MotionMode> = ScoredOption {
    value: MotionMode::KeepDistance(CAST_CORE_BAND),
    priority: 5.0,
    commit: None,
    considerations: &[HAS_TARGET],
    gates: &[HOLD_CAST_GATE],
};
/// Bite dive on a called tick. No band gates: a band-edge gate would cancel
/// the dive mid-flight as it closes inside the inner edge. `WITHIN_CAST_RANGE`
/// only vetoes the call; the latch carries the pass.
const BAT_SWOOP_OPTION: ScoredOption<MotionMode> = ScoredOption {
    value: MotionMode::Swoop,
    priority: 6.0,
    commit: None,
    considerations: &[HAS_TARGET, SWOOP_CALLED, WITHIN_CAST_RANGE],
    gates: &[],
};

const NO_ACTIONS: &[ScoredOption<()>] = &[];

const WARRIOR_MAINHAND: &[ScoredOption<()>] = &[ScoredOption {
    value: (),
    priority: 1.0,
    commit: None,
    considerations: &[HAS_TARGET, MAINHAND_READY, IN_MELEE_RANGE, NOT_KITING],
    gates: &[],
}];

const MAGE_MAINHAND: &[ScoredOption<()>] = &[ScoredOption {
    value: (),
    priority: 1.0,
    commit: None,
    considerations: &[
        HAS_TARGET,
        LINE_OF_SIGHT,
        MAINHAND_READY,
        WITHIN_CAST_RANGE,
        CAST_QUALITY,
        NOT_RECOVERING,
    ],
    gates: &[],
}];

const BAT_BREATH: &[ScoredOption<()>] = &[ScoredOption {
    value: (),
    priority: 1.0,
    commit: None,
    considerations: &[HAS_TARGET, LINE_OF_SIGHT, WITHIN_CAST_RANGE, NOT_SWOOPING],
    gates: &[],
}];

const fn combat_slots(
    mainhand: &'static [ScoredOption<()>],
    offhand: &'static [ScoredOption<()>],
) -> [&'static [ScoredOption<()>]; EquipmentSlot::ALL.len()] {
    let mut slots = [NO_ACTIONS; EquipmentSlot::ALL.len()];
    slots[EquipmentSlot::Mainhand as usize] = mainhand;
    slots[EquipmentSlot::Offhand as usize] = offhand;
    slots
}

/// Drinks the carried potion only once the flee has opened safe distance.
const WARRIOR_HEAL: &[ScoredOption<()>] = &[
    ScoredOption {
        value: (),
        priority: 1.0,
        commit: None,
        considerations: &[BADLY_HURT, FAR_FROM_TARGET],
        gates: &[],
    },
    ScoredOption {
        value: (),
        priority: 1.0,
        commit: None,
        considerations: &[BADLY_HURT, NO_TARGET],
        gates: &[],
    },
];

static WARRIOR_MOTION: &[ScoredOption<MotionMode>] = &[
    ENEMY_WANDER_OPTION,
    ENEMY_IDLE_OPTION,
    RETURN_TO_ANCHOR_OPTION,
    FLEE_OPTION,
    WARRIOR_KITE_OPTION,
    CHASE_MELEE_OPTION,
    KEEP_MELEE_RANGE_OPTION,
];

static FIRE_MAGE_MOTION: &[ScoredOption<MotionMode>] = &[
    ENEMY_WANDER_OPTION,
    ENEMY_IDLE_OPTION,
    RETURN_TO_ANCHOR_OPTION,
    FIRE_RECOVER_OPTION,
    FIRE_BURST_OPTION,
    CHASE_CAST_RANGE_OPTION,
    FIRE_STRAFE_OPTION,
];

static ICE_MAGE_MOTION: &[ScoredOption<MotionMode>] = &[
    ENEMY_WANDER_OPTION,
    ENEMY_IDLE_OPTION,
    RETURN_TO_ANCHOR_OPTION,
    ICE_SLIDE_OPTION,
    ICE_RECOVER_OPTION,
    CHASE_CAST_RANGE_OPTION,
    ICE_PLANT_OPTION,
];

static BAT_MOTION: &[ScoredOption<MotionMode>] = &[
    ENEMY_WANDER_OPTION,
    ENEMY_IDLE_OPTION,
    RETURN_TO_ANCHOR_OPTION,
    CHASE_CAST_RANGE_OPTION,
    BAT_STRAFE_OPTION,
    BAT_KEEP_OPTION,
    BAT_SWOOP_OPTION,
];

static VILLAGER_MOTION: &[ScoredOption<MotionMode>] = &[
    VILLAGER_IDLE_OPTION,
    VILLAGER_WANDER_OPTION,
    RETURN_TO_ANCHOR_OPTION,
];

static WARRIOR_SLOTS: [&[ScoredOption<()>]; EquipmentSlot::ALL.len()] =
    combat_slots(WARRIOR_MAINHAND, NO_ACTIONS);
static MAGE_SLOTS: [&[ScoredOption<()>]; EquipmentSlot::ALL.len()] =
    combat_slots(MAGE_MAINHAND, NO_ACTIONS);
pub(crate) static NO_SLOTS: [&[ScoredOption<()>]; EquipmentSlot::ALL.len()] =
    [NO_ACTIONS; EquipmentSlot::ALL.len()];

/// Closes to melee reach, plants to swing, usually runs a break away after a
/// swing, flees when badly hurt, and walks home once disengaged.
pub static WARRIOR: Brain = Brain {
    motion_options: WARRIOR_MOTION,
    slot_options: WARRIOR_SLOTS,
    heal_options: WARRIOR_HEAL,
    breath_options: NO_ACTIONS,
};
/// Holds the cast band on a moving orbit, bursts to a fresh tangent after each
/// volley, and backs off only long enough to refill a drained mana pool.
pub static FIRE_MAGE: Brain = Brain {
    motion_options: FIRE_MAGE_MOTION,
    slot_options: MAGE_SLOTS,
    heal_options: NO_ACTIONS,
    breath_options: NO_ACTIONS,
};
/// Plants on a firing lane, slides straight out when the player crosses its
/// danger ring, and retreats until its expensive bolts are affordable again.
pub static ICE_MAGE: Brain = Brain {
    motion_options: ICE_MAGE_MOTION,
    slot_options: MAGE_SLOTS,
    heal_options: NO_ACTIONS,
    breath_options: NO_ACTIONS,
};
/// Wanders around its anchor and never requests equipment.
pub static VILLAGER: Brain = Brain {
    motion_options: VILLAGER_MOTION,
    slot_options: NO_SLOTS,
    heal_options: NO_ACTIONS,
    breath_options: NO_ACTIONS,
};
/// Circles its target and breathes fireballs; no weapon needed.
pub static BAT: Brain = Brain {
    motion_options: BAT_MOTION,
    slot_options: NO_SLOTS,
    heal_options: NO_ACTIONS,
    breath_options: BAT_BREATH,
};
