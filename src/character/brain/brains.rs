//! The shipped static brain definitions.
//!
//! All tuning lives here: priorities, commitment windows, and which sensed
//! facts each option weighs. The scoring math stays in [`super::scoring`].

use super::considerations::{
    BADLY_HURT, BEYOND_ANCHOR_RANGE, BEYOND_CAST_RANGE, BEYOND_MIN_CAST_RANGE, FAR_FROM_TARGET,
    HAS_ANCHOR, HAS_TARGET, IN_MELEE_RANGE, LINE_OF_SIGHT, MAINHAND_READY, NO_TARGET,
    OUTSIDE_MELEE_RANGE, PASSIVE_IDLE, PASSIVE_WANDER, WITHIN_CAST_RANGE,
};
use super::model::{
    Brain, CommitWindow, MAGE_MAX_RANGE, MAGE_MIN_RANGE, MELEE_RANGE, MotionMode, ScoredOption,
};
use crate::items::prelude::EquipmentSlot;

const ENEMY_IDLE_OPTION: ScoredOption<MotionMode> = ScoredOption {
    value: MotionMode::Idle,
    priority: 1.0,
    commit: Some(CommitWindow { min: 3.0, max: 5.0 }),
    considerations: &[NO_TARGET, PASSIVE_IDLE],
};
const ENEMY_WANDER_OPTION: ScoredOption<MotionMode> = ScoredOption {
    value: MotionMode::Wander,
    priority: 1.0,
    commit: Some(CommitWindow { min: 1.0, max: 2.0 }),
    considerations: &[NO_TARGET, PASSIVE_WANDER],
};
const VILLAGER_IDLE_OPTION: ScoredOption<MotionMode> = ScoredOption {
    value: MotionMode::Idle,
    priority: 1.0,
    commit: Some(CommitWindow { min: 1.0, max: 4.0 }),
    considerations: &[NO_TARGET, PASSIVE_IDLE],
};
const VILLAGER_WANDER_OPTION: ScoredOption<MotionMode> = ScoredOption {
    value: MotionMode::Wander,
    priority: 1.0,
    commit: Some(CommitWindow { min: 1.0, max: 2.5 }),
    considerations: &[NO_TARGET, PASSIVE_WANDER],
};
const RETURN_TO_ANCHOR_OPTION: ScoredOption<MotionMode> = ScoredOption {
    value: MotionMode::ReturnToAnchor,
    priority: 6.0,
    commit: None,
    considerations: &[NO_TARGET, HAS_ANCHOR, BEYOND_ANCHOR_RANGE],
};
/// Wounded break: still in the fight, just getting away from the target.
/// Fleeing needs no anchor; going home happens only once disengaged.
const FLEE_OPTION: ScoredOption<MotionMode> = ScoredOption {
    value: MotionMode::Flee,
    priority: 7.0,
    commit: None,
    considerations: &[HAS_TARGET, BADLY_HURT],
};
const CHASE_MELEE_OPTION: ScoredOption<MotionMode> = ScoredOption {
    value: MotionMode::Chase,
    priority: 4.0,
    commit: None,
    considerations: &[HAS_TARGET, OUTSIDE_MELEE_RANGE],
};

/// Circles at melee edge while the sword is ready; plants feet to swing while
/// it cools down. Listed before keep so ties strafe.
const WARRIOR_STRAFE_OPTION: ScoredOption<MotionMode> = ScoredOption {
    value: MotionMode::Strafe,
    priority: 5.0,
    commit: None,
    considerations: &[HAS_TARGET, IN_MELEE_RANGE, MAINHAND_READY],
};
const KEEP_MELEE_RANGE_OPTION: ScoredOption<MotionMode> = ScoredOption {
    value: MotionMode::KeepDistance {
        min: 0.0,
        max: MELEE_RANGE,
    },
    priority: 5.0,
    commit: None,
    considerations: &[HAS_TARGET, IN_MELEE_RANGE],
};
const CHASE_CAST_RANGE_OPTION: ScoredOption<MotionMode> = ScoredOption {
    value: MotionMode::Chase,
    priority: 4.0,
    commit: None,
    considerations: &[HAS_TARGET, BEYOND_CAST_RANGE],
};
/// Holds the casting band while the staff cools down; yields to [`STRAFE`]
/// (listed first) whenever a shot is ready.
const KEEP_CAST_RANGE_OPTION: ScoredOption<MotionMode> = ScoredOption {
    value: MotionMode::KeepDistance {
        min: MAGE_MIN_RANGE,
        max: MAGE_MAX_RANGE,
    },
    priority: 5.0,
    commit: None,
    considerations: &[HAS_TARGET, WITHIN_CAST_RANGE],
};
/// Orbits the target while a shot is ready, so the mage fires on the move.
const STRAFE_OPTION: ScoredOption<MotionMode> = ScoredOption {
    value: MotionMode::Strafe,
    priority: 5.0,
    commit: None,
    considerations: &[
        HAS_TARGET,
        WITHIN_CAST_RANGE,
        BEYOND_MIN_CAST_RANGE,
        MAINHAND_READY,
    ],
};

const EMPTY_SLOT_OPTIONS: &[ScoredOption<()>] = &[];

const WARRIOR_MAINHAND: &[ScoredOption<()>] = &[ScoredOption {
    value: (),
    priority: 1.0,
    commit: None,
    considerations: &[HAS_TARGET, MAINHAND_READY, IN_MELEE_RANGE],
}];

const RANGED_MAGE_MAINHAND: &[ScoredOption<()>] = &[ScoredOption {
    value: (),
    priority: 1.0,
    commit: None,
    considerations: &[HAS_TARGET, LINE_OF_SIGHT, MAINHAND_READY, WITHIN_CAST_RANGE],
}];

const fn combat_slots(
    mainhand: &'static [ScoredOption<()>],
    offhand: &'static [ScoredOption<()>],
) -> [&'static [ScoredOption<()>]; EquipmentSlot::ALL.len()] {
    let mut slots = [EMPTY_SLOT_OPTIONS; EquipmentSlot::ALL.len()];
    slots[EquipmentSlot::Mainhand as usize] = mainhand;
    slots[EquipmentSlot::Offhand as usize] = offhand;
    slots
}

/// Drinks the carried potion only once the flee has opened safe distance,
/// or the target is gone entirely: never mid-melee.
const WARRIOR_HEAL: &[ScoredOption<()>] = &[
    ScoredOption {
        value: (),
        priority: 1.0,
        commit: None,
        considerations: &[BADLY_HURT, FAR_FROM_TARGET],
    },
    ScoredOption {
        value: (),
        priority: 1.0,
        commit: None,
        considerations: &[BADLY_HURT, NO_TARGET],
    },
];

static WARRIOR_MOTION: &[ScoredOption<MotionMode>] = &[
    ENEMY_WANDER_OPTION,
    ENEMY_IDLE_OPTION,
    RETURN_TO_ANCHOR_OPTION,
    FLEE_OPTION,
    CHASE_MELEE_OPTION,
    WARRIOR_STRAFE_OPTION,
    KEEP_MELEE_RANGE_OPTION,
];

static RANGED_MAGE_MOTION: &[ScoredOption<MotionMode>] = &[
    ENEMY_WANDER_OPTION,
    ENEMY_IDLE_OPTION,
    RETURN_TO_ANCHOR_OPTION,
    CHASE_CAST_RANGE_OPTION,
    STRAFE_OPTION,
    KEEP_CAST_RANGE_OPTION,
];

static VILLAGER_MOTION: &[ScoredOption<MotionMode>] = &[
    VILLAGER_IDLE_OPTION,
    VILLAGER_WANDER_OPTION,
    RETURN_TO_ANCHOR_OPTION,
];

static WARRIOR_SLOTS: [&[ScoredOption<()>]; EquipmentSlot::ALL.len()] =
    combat_slots(WARRIOR_MAINHAND, EMPTY_SLOT_OPTIONS);
static RANGED_MAGE_SLOTS: [&[ScoredOption<()>]; EquipmentSlot::ALL.len()] =
    combat_slots(RANGED_MAGE_MAINHAND, EMPTY_SLOT_OPTIONS);
pub(crate) static NO_SLOTS: [&[ScoredOption<()>]; EquipmentSlot::ALL.len()] =
    [EMPTY_SLOT_OPTIONS; EquipmentSlot::ALL.len()];

/// The big bruiser: closes to melee reach, swings, flees from its target
/// when badly hurt, and walks home only once disengaged.
pub static WARRIOR: Brain = Brain {
    motion_options: WARRIOR_MOTION,
    slot_options: WARRIOR_SLOTS,
    heal_options: WARRIOR_HEAL,
};
/// Strafes the preferred casting band while a shot is ready, holds it while
/// cooling down, and fires whenever it has sight and mana.
pub static RANGED_MAGE: Brain = Brain {
    motion_options: RANGED_MAGE_MOTION,
    slot_options: RANGED_MAGE_SLOTS,
    heal_options: EMPTY_SLOT_OPTIONS,
};
/// Wanders around its anchor and never requests equipment.
pub static VILLAGER: Brain = Brain {
    motion_options: VILLAGER_MOTION,
    slot_options: NO_SLOTS,
    heal_options: EMPTY_SLOT_OPTIONS,
};
