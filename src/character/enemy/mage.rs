use bevy::prelude::*;

use super::{
    Experience, enemy_children, enemy_wander, ground_sight_ray, spawn_enemy_with_equipment,
};
use crate::character::brain::considerations::{
    ACTION_BASELINE_IDLE, ATTACK_FINISHED, CALM, CAST_DISTANCE_RANGE, CAST_MAX_RANGE,
    CAST_MIN_RANGE, CHASE_CAST_RANGE, HAS_TARGET, IN_MAINHAND_REACH, INSIDE_OUTER_CAST_EDGE,
    LINE_OF_SIGHT, MAINHAND, RETURN_TO_ANCHOR, consideration, mana_at_or_below, within_reach,
};
use crate::character::brain::{
    Behavior, BrainDefinition, CommitPolicy, Consideration, ConsiderationInput, Curve, Dash,
    Interval, Motion, Orbit,
};
use crate::prelude::*;
use crate::world::MapRng;

/// Distance a caster opens before it stops casting to regenerate mana.
const RECOVERY_DISTANCE_RANGE: Interval = Interval::new(300.0, 340.0);

// The mages cast out to the cast range's edge, which their staffs must still reach.
const _: () = assert!(
    CAST_MAX_RANGE <= STAFF_REACH,
    "staffs must reach the cast range's outer edge"
);

/// The ice mage's danger ring: its escape dash's reach.
const ICE_SLIDE_REACH: f32 = CAST_MIN_RANGE - 30.0;

const IN_DASH_REACH: Consideration = within_reach(
    "in dash reach",
    ConsiderationInput::MotionReachFraction(Motion::Dash),
);

/// Fire mage firing-position quality: zero at the inner cast edge, full 60
/// units out. It stands still through the wind-up, so it orbits outward first.
const FIRE_CAST_QUALITY: Consideration = consideration(
    "fire cast quality",
    ConsiderationInput::TargetDistance,
    Interval::new(CAST_MIN_RANGE, CAST_MIN_RANGE + 60.0),
    Curve::Linear,
);
/// Ice mage firing-position quality: full outside its danger ring, zero 60
/// units inside, where a target reaches it during the wind-up.
const ICE_CAST_QUALITY: Consideration = consideration(
    "ice cast quality",
    ConsiderationInput::TargetDistance,
    Interval::new(ICE_SLIDE_REACH - 60.0, ICE_SLIDE_REACH),
    Curve::Linear,
);

/// Both mages' cast weight. The cast's other two inputs are flags, so it
/// scores `CAST_WEIGHT * quality^(1/3)`.
const CAST_WEIGHT: f32 = 8.0;

/// Sidesteps for a firing lane; each post-volley hop flips the orbit direction.
const FIRE_ORBIT: Behavior = Behavior::new(
    Some(Motion::Orbit(CAST_DISTANCE_RANGE)),
    None,
    CommitPolicy::Momentum,
    5.5,
    &[INSIDE_OUTER_CAST_EDGE],
);
/// Plants and lets the ice bolt do the work.
const ICE_PLANT: Behavior = Behavior::new(
    Some(Motion::Idle),
    None,
    CommitPolicy::Momentum,
    5.0,
    &[INSIDE_OUTER_CAST_EDGE],
);
/// Breaks contact while mana is low; the threshold's exit holds it until the
/// pool refills.
const FIRE_RECOVER: Behavior = Behavior::new(
    Some(Motion::KeepDistance(RECOVERY_DISTANCE_RANGE)),
    None,
    CommitPolicy::Momentum,
    9.0,
    &[HAS_TARGET, mana_at_or_below("fire mana low", 0.2, 0.45)],
);
/// Ice bolts are expensive, so the break enters earlier and lasts longer.
const ICE_RECOVER: Behavior = Behavior::new(
    Some(Motion::KeepDistance(RECOVERY_DISTANCE_RANGE)),
    None,
    CommitPolicy::Momentum,
    9.0,
    &[HAS_TARGET, mana_at_or_below("ice mana low", 0.4, 0.75)],
);

/// A tangent hop after each volley.
const FIRE_HOP: Behavior = Behavior::new(
    Some(Motion::Dash),
    None,
    CommitPolicy::UntilFinished { interrupt_tier: 1 },
    8.5,
    &[ATTACK_FINISHED, HAS_TARGET],
);
/// A straight escape when the player crosses the danger ring. Its higher tier
/// pre-empts a cast, though a bolt already in flight still fires.
const ICE_SLIDE: Behavior = Behavior::new(
    Some(Motion::Dash),
    None,
    CommitPolicy::UntilFinished { interrupt_tier: 2 },
    12.0,
    &[IN_DASH_REACH],
);

/// Stands still through the wind-up to cast.
const FIRE_CAST: Behavior = Behavior::new(
    Some(Motion::Idle),
    Some(MAINHAND),
    CommitPolicy::UntilFinished { interrupt_tier: 1 },
    CAST_WEIGHT,
    &[LINE_OF_SIGHT, IN_MAINHAND_REACH, FIRE_CAST_QUALITY],
);
/// Casts from where the ice mage plants.
const ICE_CAST: Behavior = Behavior::new(
    Some(Motion::Idle),
    Some(MAINHAND),
    CommitPolicy::UntilFinished { interrupt_tier: 1 },
    CAST_WEIGHT,
    &[LINE_OF_SIGHT, IN_MAINHAND_REACH, ICE_CAST_QUALITY],
);

// Motion priority: recover (9, or 11.25 once running), hop (8.5), cast (8 at
// full quality), return (6), orbit (5.5), chase (4), calm (1).
// - Recover beats the hop on the tick it starts, so a volley that drains mana
//   starts the break instead of a hop.
// - The hop beats a running orbit (6.875).
// - The cast beats a running orbit from about 238 units away, and a newly
//   started one from about 220. A mage crowded closer orbits outward until it
//   is about 238 away, then casts again.
static FIRE_MAGE_BEHAVIORS: &[Behavior] = &[
    CALM,
    ACTION_BASELINE_IDLE,
    RETURN_TO_ANCHOR,
    FIRE_RECOVER,
    CHASE_CAST_RANGE,
    FIRE_ORBIT,
    FIRE_HOP,
    FIRE_CAST,
];

// Motion priority: slide (12), recover (9, or 11.25 once running), cast (8 at
// full quality), return (6), plant (5), chase (4), calm (1).
// - The slide beats a running recover and interrupts a cast.
// - Cast quality is full from the danger ring out to the staff's reach. Inside
//   the ring (while the slide recharges), the cast beats a running plant (6.25)
//   from about 139 units away, and a newly started one from about 125.
static ICE_MAGE_BEHAVIORS: &[Behavior] = &[
    CALM,
    ACTION_BASELINE_IDLE,
    RETURN_TO_ANCHOR,
    ICE_RECOVER,
    CHASE_CAST_RANGE,
    ICE_PLANT,
    ICE_SLIDE,
    ICE_CAST,
];

/// Holds the cast distance range on a moving orbit, hops to a fresh tangent after each
/// volley, and backs off only long enough to refill a drained mana pool.
pub(super) static FIRE_MAGE: BrainDefinition = BrainDefinition {
    behaviors: FIRE_MAGE_BEHAVIORS,
};
/// Plants on a firing lane, slides straight out when the player crosses its
/// danger ring, and retreats until its expensive bolts are affordable again.
pub(super) static ICE_MAGE: BrainDefinition = BrainDefinition {
    behaviors: ICE_MAGE_BEHAVIORS,
};

/// Spawns an ice-staff mage on top of the shared enemy `base` bundle.
pub(super) fn spawn_ice_mage(
    commands: &mut Commands,
    base: impl Bundle,
    sprites: &SpriteAssets,
    sprite_layouts: &SpriteSheetLayouts,
    shadows: &Shadows,
) {
    spawn_enemy_with_equipment(
        commands,
        (
            ice_mage(sprites, sprite_layouts),
            base,
            enemy_children(shadows),
        ),
        ice_staff(sprites, sprite_layouts),
    );
}

/// Spawns a fire mage on top of the shared enemy `base` bundle, carrying one of
/// two flame staves picked from the seeded map stream.
pub(super) fn spawn_fire_mage(
    commands: &mut Commands,
    base: impl Bundle,
    sprites: &SpriteAssets,
    sprite_layouts: &SpriteSheetLayouts,
    shadows: &Shadows,
    map_rng: &mut MapRng,
) {
    let enemy = (
        fire_mage(sprites, sprite_layouts),
        base,
        enemy_children(shadows),
    );

    if map_rng.flip() {
        spawn_enemy_with_equipment(
            commands,
            enemy,
            mage_fire_staff_single_bolt(sprites, sprite_layouts),
        );
    } else {
        spawn_enemy_with_equipment(
            commands,
            enemy,
            mage_fire_staff_fan(sprites, sprite_layouts),
        );
    }
}

fn ice_mage(sprites: &SpriteAssets, sprite_layouts: &SpriteSheetLayouts) -> impl Bundle {
    (
        (SimpleMotion::new(100.0), Health::new(20.0)),
        Experience { base_exp: 12.0 },
        ground_sight_ray(),
        YSort::from_offset(CHARACTER_FEET_POS_OFFSET),
        Mana::new(100.0, 10.0),
        // The cooldown is the player's window to close in again.
        Dash::slide(
            3.0 * TILE_SIZE as f32,
            4.0 * TILE_SIZE as f32,
            2.2,
            ICE_SLIDE_REACH,
            1.2,
        ),
        enemy_wander(),
        Sprite::from_atlas_image(
            sprites.ice_mage_enemy_sprite_sheet.clone(),
            TextureAtlas {
                layout: sprite_layouts.enemy_atlas_layout.clone(),
                ..default()
            },
        ),
        related!(Items[health_potion(sprites)]),
    )
}

fn fire_mage(sprites: &SpriteAssets, sprite_layouts: &SpriteSheetLayouts) -> impl Bundle {
    (
        (SimpleMotion::new(150.0), Health::new(20.0)),
        Experience { base_exp: 15.0 },
        ground_sight_ray(),
        YSort::from_offset(CHARACTER_FEET_POS_OFFSET),
        Mana::new(100.0, 10.0),
        // A still target inside the close range gets a step-in hop instead.
        Dash::hop(
            2.0 * TILE_SIZE as f32,
            3.0 * TILE_SIZE as f32,
            2.2,
            CAST_MIN_RANGE - 10.0,
        ),
        Orbit::new(),
        enemy_wander(),
        Sprite::from_atlas_image(
            sprites.fire_mage_enemy_sprite_sheet.clone(),
            TextureAtlas {
                layout: sprite_layouts.enemy_atlas_layout.clone(),
                ..default()
            },
        ),
        related!(Items[health_potion(sprites)]),
    )
}
