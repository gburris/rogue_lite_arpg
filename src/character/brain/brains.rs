use super::considerations::{
    ANCHOR_PULL, ATTACK_FINISHED, BADLY_HURT, BEYOND_CAST_BAND, CAST_QUALITY, CIRCLING,
    FAR_FROM_TARGET, HAS_TARGET, IN_BITE_REACH, IN_BREATH_REACH, IN_DASH_REACH, IN_MAINHAND_REACH,
    IN_SWOOP_REACH, INSIDE_MELEE_REACH, INSIDE_OUTER_CAST_EDGE, LINE_OF_SIGHT, MAINHAND_READY,
    MELEE_REACH, NO_TARGET, NOT_DASHING, NOT_RECOVERING, NOT_SWOOPING, OUTSIDE_MELEE_REACH,
    PAST_INNER_CAST_EDGE, RECOVERING,
};
use super::model::{
    Ability, Action, BrainDefinition, CAST_BAND, CAST_CORE_BAND, CAST_MAX_RANGE, Consideration,
    Decision, MELEE_BAND, Maneuver, MotionMode, Movement, RECOVERY_BAND,
};
use crate::items::prelude::EquipmentSlot;

/// Weight every maneuver carries: above any mode's weight with momentum, so a
/// maneuver whose considerations all pass always wins the movement layer.
const MANEUVER_WEIGHT: f32 = 10.0;

/// Builds a decision, rejecting a bad weight at compile time.
const fn decision<T>(
    value: T,
    weight: f32,
    considerations: &'static [Consideration],
) -> Decision<T> {
    assert!(
        weight.is_finite() && weight > 0.0,
        "decision weight must be finite and positive"
    );
    Decision {
        value,
        weight,
        considerations,
    }
}

const fn mode(
    mode: MotionMode,
    weight: f32,
    considerations: &'static [Consideration],
) -> Decision<Movement> {
    decision(Movement::Mode(mode), weight, considerations)
}

// Maneuvers carry no band thresholds: one would cancel the move mid-flight
// as it opens or closes range. Their start conditions are scored once; the
// executor reports when they end.
const fn maneuver(
    maneuver: Maneuver,
    considerations: &'static [Consideration],
) -> Decision<Movement> {
    decision(
        Movement::Maneuver(maneuver),
        MANEUVER_WEIGHT,
        considerations,
    )
}

const fn action(action: Action, considerations: &'static [Consideration]) -> Decision<Action> {
    decision(action, 1.0, considerations)
}

const MAINHAND: Action = Action::Use(EquipmentSlot::Mainhand);

/// Disengaged ambling; each body's `Wander` component sets the rhythm.
const CALM: Decision<Movement> = mode(MotionMode::Calm, 1.0, &[NO_TARGET]);
const RETURN_TO_ANCHOR: Decision<Movement> =
    mode(MotionMode::ReturnToAnchor, 6.0, &[NO_TARGET, ANCHOR_PULL]);
const FLEE: Decision<Movement> = mode(MotionMode::Flee, 7.0, &[HAS_TARGET, BADLY_HURT]);
const CHASE_MELEE: Decision<Movement> = mode(MotionMode::Chase, 4.0, &[OUTSIDE_MELEE_REACH]);

/// Holds melee reach instead of orbiting; sword readiness only gates the swing.
const KEEP_MELEE_RANGE: Decision<Movement> = mode(
    MotionMode::KeepDistance(MELEE_BAND),
    5.0,
    &[INSIDE_MELEE_REACH],
);
const CHASE_CAST_RANGE: Decision<Movement> = mode(MotionMode::Chase, 4.0, &[BEYOND_CAST_BAND]);
/// Sidesteps for a good firing lane; the target's tangent picks the orbit side
/// in `strafe_direction`. The fire mage keeps this moving even against a
/// stationary target, so its turns come from its own volleys instead of the
/// player's movement.
const FIRE_STRAFE: Decision<Movement> = mode(
    MotionMode::Strafe(CAST_BAND),
    5.5,
    &[INSIDE_OUTER_CAST_EDGE],
);
/// Plants in the cast band and lets the ice bolt do the work. The danger-ring
/// slide is the only time this mage gives ground.
const ICE_PLANT: Decision<Movement> = mode(MotionMode::Hold, 5.0, &[INSIDE_OUTER_CAST_EDGE]);
/// Breaks contact until the caster's held mana break releases. The two
/// casters share the mode but enter it on their own mana rhythms.
const FIRE_RECOVER: Decision<Movement> = mode(
    MotionMode::KeepDistance(RECOVERY_BAND),
    6.8,
    &[HAS_TARGET, RECOVERING],
);
const ICE_RECOVER: Decision<Movement> = mode(
    MotionMode::KeepDistance(RECOVERY_BAND),
    6.6,
    &[HAS_TARGET, RECOVERING],
);

/// Orbits the casting band; the bat breathes innately, no ready weapon needed.
const BAT_STRAFE: Decision<Movement> = mode(
    MotionMode::Strafe(CAST_BAND),
    5.5,
    &[PAST_INNER_CAST_EDGE, INSIDE_OUTER_CAST_EDGE],
);
/// Holds the core so it keeps repositioning toward the middle.
const BAT_KEEP: Decision<Movement> = mode(
    MotionMode::KeepDistance(CAST_CORE_BAND),
    5.0,
    &[INSIDE_OUTER_CAST_EDGE],
);

/// Runs out the break rolled after a swing, then the normal chase/hold
/// decisions bring the warrior back in for the next swing.
const WARRIOR_KITE: Decision<Movement> = maneuver(Maneuver::Dash, &[ATTACK_FINISHED, HAS_TARGET]);
/// A tangent hop after the fire staff finishes a volley.
const FIRE_BURST: Decision<Movement> = maneuver(
    Maneuver::Dash,
    &[ATTACK_FINISHED, HAS_TARGET, NOT_RECOVERING],
);
/// A straight escape when the player crosses the ice mage's danger ring.
const ICE_SLIDE: Decision<Movement> = maneuver(Maneuver::Dash, &[IN_DASH_REACH]);
/// Bite dive while the bat is circling inside its cast range.
const BAT_SWOOP: Decision<Movement> = maneuver(Maneuver::Swoop, &[IN_SWOOP_REACH, CIRCLING]);

static WARRIOR_MOVEMENT: &[Decision<Movement>] = &[
    CALM,
    RETURN_TO_ANCHOR,
    FLEE,
    CHASE_MELEE,
    KEEP_MELEE_RANGE,
    WARRIOR_KITE,
];

static FIRE_MAGE_MOVEMENT: &[Decision<Movement>] = &[
    CALM,
    RETURN_TO_ANCHOR,
    FIRE_RECOVER,
    CHASE_CAST_RANGE,
    FIRE_STRAFE,
    FIRE_BURST,
];

static ICE_MAGE_MOVEMENT: &[Decision<Movement>] = &[
    CALM,
    RETURN_TO_ANCHOR,
    ICE_RECOVER,
    CHASE_CAST_RANGE,
    ICE_PLANT,
    ICE_SLIDE,
];

static BAT_MOVEMENT: &[Decision<Movement>] = &[
    CALM,
    RETURN_TO_ANCHOR,
    CHASE_CAST_RANGE,
    BAT_STRAFE,
    BAT_KEEP,
    BAT_SWOOP,
];

static VILLAGER_MOVEMENT: &[Decision<Movement>] = &[CALM, RETURN_TO_ANCHOR];

static WARRIOR_ACTIONS: &[Decision<Action>] = &[
    action(MAINHAND, &[MAINHAND_READY, IN_MAINHAND_REACH, NOT_DASHING]),
    // Drinks the carried potion only once the flee has opened safe distance.
    action(
        Action::Ability(Ability::Heal),
        &[BADLY_HURT, FAR_FROM_TARGET],
    ),
    action(Action::Ability(Ability::Heal), &[BADLY_HURT, NO_TARGET]),
];

static MAGE_ACTIONS: &[Decision<Action>] = &[action(
    MAINHAND,
    &[
        LINE_OF_SIGHT,
        MAINHAND_READY,
        IN_MAINHAND_REACH,
        CAST_QUALITY,
        NOT_RECOVERING,
    ],
)];

static BAT_ACTIONS: &[Decision<Action>] = &[
    action(
        Action::Ability(Ability::Breathe),
        &[LINE_OF_SIGHT, IN_BREATH_REACH, NOT_SWOOPING],
    ),
    // Bites anything in contact reach, whether it is diving or circling.
    action(Action::Ability(Ability::Bite), &[IN_BITE_REACH]),
];

/// Closes to melee reach, plants to swing, usually runs a break away after a
/// swing, flees when badly hurt, and walks home once disengaged.
pub static WARRIOR: BrainDefinition = BrainDefinition {
    movement: WARRIOR_MOVEMENT,
    actions: WARRIOR_ACTIONS,
    // Matches the hold threshold's exit: a warrior parked by the arrival
    // deadband up to MELEE_REACH can still swing.
    equipment_reach: Some(MELEE_REACH),
};
/// Holds the cast band on a moving orbit, bursts to a fresh tangent after each
/// volley, and backs off only long enough to refill a drained mana pool.
pub static FIRE_MAGE: BrainDefinition = BrainDefinition {
    movement: FIRE_MAGE_MOVEMENT,
    actions: MAGE_ACTIONS,
    equipment_reach: Some(CAST_MAX_RANGE),
};
/// Plants on a firing lane, slides straight out when the player crosses its
/// danger ring, and retreats until its expensive bolts are affordable again.
pub static ICE_MAGE: BrainDefinition = BrainDefinition {
    movement: ICE_MAGE_MOVEMENT,
    actions: MAGE_ACTIONS,
    equipment_reach: Some(CAST_MAX_RANGE),
};
/// Wanders around its anchor and never requests equipment.
pub static VILLAGER: BrainDefinition = BrainDefinition {
    movement: VILLAGER_MOVEMENT,
    actions: &[],
    equipment_reach: None,
};
/// Circles its target, dives to bite, and breathes fireballs; no weapon needed.
pub static BAT: BrainDefinition = BrainDefinition {
    movement: BAT_MOVEMENT,
    actions: BAT_ACTIONS,
    equipment_reach: None,
};

/// Every shipped brain with a name for validation errors.
pub fn all() -> [(&'static str, &'static BrainDefinition); 5] {
    [
        ("warrior", &WARRIOR),
        ("fire mage", &FIRE_MAGE),
        ("ice mage", &ICE_MAGE),
        ("villager", &VILLAGER),
        ("bat", &BAT),
    ]
}
