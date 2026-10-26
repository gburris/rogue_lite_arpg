use super::considerations::{
    ANCHOR_PULL, ATTACK_FINISHED, BADLY_HURT, BEYOND_CAST_DISTANCE_RANGE, BREATH_DISTANCE,
    CAST_QUALITY, CIRCLING, FAR_FROM_TARGET, HAS_TARGET, IN_BITE_REACH, IN_BREATH_REACH,
    IN_DASH_REACH, IN_MAINHAND_REACH, IN_SWOOP_REACH, INSIDE_MELEE_REACH, INSIDE_OUTER_CAST_EDGE,
    LINE_OF_SIGHT, MELEE_REACH, NEEDS_POTION, NO_TARGET, NOT_DASHING, NOT_RECOVERING, NOT_SWOOPING,
    OUTSIDE_MELEE_REACH, PAST_INNER_CAST_EDGE, RECOVERING,
};
use super::model::{
    Ability, Action, Behavior, BrainDefinition, CAST_CORE_DISTANCE_RANGE, CAST_DISTANCE_RANGE,
    CAST_MAX_RANGE, CommitPolicy, Consideration, Decision, MELEE_DISTANCE_RANGE, Motion,
    RECOVERY_DISTANCE_RANGE,
};
use crate::items::prelude::{EquipmentSlot, MELEE_WEAPON_REACH, STAFF_REACH};

/// How long a cast holds its caster still from its start, so a short cast
/// still leaves a readable beat before the next move.
pub const CAST_LOCK_SECONDS: f32 = 0.3;

/// Weight of a locked cast: above the in-range motions it interrupts (even with
/// their momentum), below the slide and hop moves that may break contact.
const CAST_WEIGHT: f32 = 8.0;

/// Weight every locked motion carries: above any continuous motion's weight with momentum, so a
/// locked motion whose considerations all pass always wins the motion slot.
const LOCKED_MOTION_WEIGHT: f32 = 10.0;

/// Builds a decision, rejecting a bad weight at compile time.
const fn decision(
    behavior: Behavior,
    weight: f32,
    considerations: &'static [Consideration],
) -> Decision<Behavior> {
    assert!(
        weight.is_finite() && weight > 0.0,
        "decision weight must be finite and positive"
    );
    Decision {
        value: behavior,
        weight,
        considerations,
    }
}

/// A continuous motion, which keeps momentum while it runs.
const fn momentum_motion(
    motion: Motion,
    weight: f32,
    considerations: &'static [Consideration],
) -> Decision<Behavior> {
    assert!(
        !motion.is_finite(),
        "a momentum motion needs a continuous motion"
    );
    decision(
        Behavior {
            motion: Some(motion),
            action: None,
            commit: CommitPolicy::Momentum,
        },
        weight,
        considerations,
    )
}

// Locked motions carry no distance-range thresholds: one would cancel the move mid-flight
// as it opens or closes range. Their start conditions are scored once; the
// executor reports when they end.
const fn locked_finite_motion(
    motion: Motion,
    weight: f32,
    considerations: &'static [Consideration],
) -> Decision<Behavior> {
    assert!(motion.is_finite(), "a locked motion needs a finite motion");
    decision(
        Behavior {
            motion: Some(motion),
            action: None,
            commit: CommitPolicy::LOCK,
        },
        weight,
        considerations,
    )
}

const fn momentum_action(
    action: Action,
    considerations: &'static [Consideration],
) -> Decision<Behavior> {
    decision(
        Behavior {
            motion: None,
            action: Some(action),
            commit: CommitPolicy::Momentum,
        },
        1.0,
        considerations,
    )
}

/// Claims both slots and holds them until its finite parts are done and
/// `min_seconds` have passed.
const fn locked_motion_and_action(
    motion: Motion,
    action: Action,
    min_seconds: f32,
    weight: f32,
    considerations: &'static [Consideration],
) -> Decision<Behavior> {
    decision(
        Behavior {
            motion: Some(motion),
            action: Some(action),
            commit: CommitPolicy::Lock { min_seconds },
        },
        weight,
        considerations,
    )
}

const MAINHAND: Action = Action::Use(EquipmentSlot::Mainhand);

// Brains hold distances their weapons must still reach: the warrior parks up
// to its melee hold exit and the mages cast out to the cast range's edge.
const _: () = assert!(
    MELEE_REACH <= MELEE_WEAPON_REACH,
    "melee weapons must reach the warrior's melee hold exit"
);
const _: () = assert!(
    CAST_MAX_RANGE <= STAFF_REACH,
    "staffs must reach the cast range's outer edge"
);

/// Disengaged ambling; each body's `Wander` component sets the rhythm.
const CALM: Decision<Behavior> = momentum_motion(Motion::Calm, 1.0, &[NO_TARGET]);
const RETURN_TO_ANCHOR: Decision<Behavior> =
    momentum_motion(Motion::ReturnToAnchor, 6.0, &[NO_TARGET, ANCHOR_PULL]);
const FLEE: Decision<Behavior> = momentum_motion(Motion::Flee, 7.0, &[HAS_TARGET, BADLY_HURT]);
const CHASE_MELEE: Decision<Behavior> = momentum_motion(Motion::Chase, 4.0, &[OUTSIDE_MELEE_REACH]);

/// Holds melee reach instead of orbiting; sword readiness only gates the swing.
const KEEP_MELEE_RANGE: Decision<Behavior> = momentum_motion(
    Motion::KeepDistance(MELEE_DISTANCE_RANGE),
    5.0,
    &[INSIDE_MELEE_REACH],
);
const CHASE_CAST_RANGE: Decision<Behavior> =
    momentum_motion(Motion::Chase, 4.0, &[BEYOND_CAST_DISTANCE_RANGE]);
/// Sidesteps for a good firing lane. Each hop after a volley flips the orbit
/// direction, so the fire mage's turns come from its own volleys instead of
/// the player's movement.
const FIRE_ORBIT: Decision<Behavior> = momentum_motion(
    Motion::Orbit(CAST_DISTANCE_RANGE),
    5.5,
    &[INSIDE_OUTER_CAST_EDGE],
);
/// Plants in the cast distance range and lets the ice bolt do the work. The danger-ring
/// slide is the only time this mage gives ground.
const ICE_PLANT: Decision<Behavior> = momentum_motion(Motion::Idle, 5.0, &[INSIDE_OUTER_CAST_EDGE]);
/// Breaks contact until the caster's held mana break releases. The two
/// casters share the mode but enter it on their own mana rhythms.
const FIRE_RECOVER: Decision<Behavior> = momentum_motion(
    Motion::KeepDistance(RECOVERY_DISTANCE_RANGE),
    6.8,
    &[HAS_TARGET, RECOVERING],
);
const ICE_RECOVER: Decision<Behavior> = momentum_motion(
    Motion::KeepDistance(RECOVERY_DISTANCE_RANGE),
    6.6,
    &[HAS_TARGET, RECOVERING],
);

/// Orbits the casting distance range; the bat breathes innately, no ready weapon needed.
const BAT_ORBIT: Decision<Behavior> = momentum_motion(
    Motion::Orbit(CAST_DISTANCE_RANGE),
    5.5,
    &[PAST_INNER_CAST_EDGE, INSIDE_OUTER_CAST_EDGE],
);
/// Holds the core so it keeps repositioning toward the middle.
const BAT_KEEP: Decision<Behavior> = momentum_motion(
    Motion::KeepDistance(CAST_CORE_DISTANCE_RANGE),
    5.0,
    &[INSIDE_OUTER_CAST_EDGE],
);

/// Runs out the break rolled after a swing, then the normal chase/hold
/// decisions bring the warrior back in for the next swing.
const WARRIOR_KITE: Decision<Behavior> = locked_finite_motion(
    Motion::Dash,
    LOCKED_MOTION_WEIGHT,
    &[ATTACK_FINISHED, HAS_TARGET],
);
/// A tangent hop after the fire staff finishes a volley.
const FIRE_HOP: Decision<Behavior> = locked_finite_motion(
    Motion::Dash,
    LOCKED_MOTION_WEIGHT,
    &[ATTACK_FINISHED, HAS_TARGET, NOT_RECOVERING],
);
/// A straight escape when the player crosses the ice mage's danger ring.
const ICE_SLIDE: Decision<Behavior> =
    locked_finite_motion(Motion::Dash, LOCKED_MOTION_WEIGHT, &[IN_DASH_REACH]);
/// Bite dive while the bat is circling inside its cast range.
const BAT_SWOOP: Decision<Behavior> = locked_finite_motion(
    Motion::Swoop,
    LOCKED_MOTION_WEIGHT,
    &[IN_SWOOP_REACH, CIRCLING],
);

/// Stands still to cast, so mages stand through the wind-up. Their only
/// mainhand use.
const MAGE_CAST: Decision<Behavior> = locked_motion_and_action(
    Motion::Idle,
    MAINHAND,
    CAST_LOCK_SECONDS,
    CAST_WEIGHT,
    &[
        LINE_OF_SIGHT,
        IN_MAINHAND_REACH,
        CAST_QUALITY,
        NOT_RECOVERING,
    ],
);

static WARRIOR_BEHAVIORS: &[Decision<Behavior>] = &[
    CALM,
    RETURN_TO_ANCHOR,
    FLEE,
    CHASE_MELEE,
    KEEP_MELEE_RANGE,
    WARRIOR_KITE,
    momentum_action(MAINHAND, &[IN_MAINHAND_REACH, NOT_DASHING]),
    // Drinks the carried potion only once the flee has opened safe distance.
    momentum_action(
        Action::Ability(Ability::Heal),
        &[NEEDS_POTION, FAR_FROM_TARGET],
    ),
    momentum_action(Action::Ability(Ability::Heal), &[NEEDS_POTION, NO_TARGET]),
];

static FIRE_MAGE_BEHAVIORS: &[Decision<Behavior>] = &[
    CALM,
    RETURN_TO_ANCHOR,
    FIRE_RECOVER,
    CHASE_CAST_RANGE,
    FIRE_ORBIT,
    FIRE_HOP,
    MAGE_CAST,
];

static ICE_MAGE_BEHAVIORS: &[Decision<Behavior>] = &[
    CALM,
    RETURN_TO_ANCHOR,
    ICE_RECOVER,
    CHASE_CAST_RANGE,
    ICE_PLANT,
    ICE_SLIDE,
    MAGE_CAST,
];

static BAT_BEHAVIORS: &[Decision<Behavior>] = &[
    CALM,
    RETURN_TO_ANCHOR,
    CHASE_CAST_RANGE,
    BAT_ORBIT,
    BAT_KEEP,
    BAT_SWOOP,
    momentum_action(
        Action::Ability(Ability::Breathe),
        &[
            LINE_OF_SIGHT,
            IN_BREATH_REACH,
            BREATH_DISTANCE,
            NOT_SWOOPING,
        ],
    ),
    // Bites anything in contact reach, whether it is diving or circling.
    momentum_action(Action::Ability(Ability::Bite), &[IN_BITE_REACH]),
];

static VILLAGER_BEHAVIORS: &[Decision<Behavior>] = &[CALM, RETURN_TO_ANCHOR];

/// Closes to melee reach, plants to swing, usually runs a break away after a
/// swing, flees when badly hurt, and walks home once disengaged.
pub static WARRIOR: BrainDefinition = BrainDefinition {
    behaviors: WARRIOR_BEHAVIORS,
};
/// Holds the cast distance range on a moving orbit, hops to a fresh tangent after each
/// volley, and backs off only long enough to refill a drained mana pool.
pub static FIRE_MAGE: BrainDefinition = BrainDefinition {
    behaviors: FIRE_MAGE_BEHAVIORS,
};
/// Plants on a firing lane, slides straight out when the player crosses its
/// danger ring, and retreats until its expensive bolts are affordable again.
pub static ICE_MAGE: BrainDefinition = BrainDefinition {
    behaviors: ICE_MAGE_BEHAVIORS,
};
/// Wanders around its anchor and never requests equipment.
pub static VILLAGER: BrainDefinition = BrainDefinition {
    behaviors: VILLAGER_BEHAVIORS,
};
/// Circles its target, dives to bite, and breathes fireballs; no weapon needed.
pub static BAT: BrainDefinition = BrainDefinition {
    behaviors: BAT_BEHAVIORS,
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
