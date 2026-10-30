use bevy::prelude::*;

use super::{
    Experience, enemy_children, enemy_wander, ground_sight_ray, spawn_enemy_with_equipment,
};
use crate::character::brain::considerations::{
    ATTACK_FINISHED, HAS_TARGET, IN_MAINHAND_REACH, MAINHAND, NO_TARGET, RETURN_TO_HOME, WANDER,
    consideration, flag, reach_fraction_threshold, target_within_distance,
};
use crate::character::brain::{
    Ability, Action, Behavior, BrainDefinition, CommitPolicy, Consideration, ConsiderationInput,
    Curve, Dash, Heal, Interval, Motion,
};
use crate::prelude::*;

/// Distance from the target at which a wounded warrior may drink its potion.
const HEAL_SAFE_DISTANCE: f32 = 200.0;

/// Empty above half health, full just past it: warriors break at ~50% hp.
const BADLY_HURT: Consideration = consideration(
    "badly hurt",
    ConsiderationInput::MissingHealth,
    Interval::new(0.45, 0.55),
    Curve::Linear,
);
/// Potion need: none above 58% health, full at 18%. One potion and no
/// regeneration, so it is saved for a real wound.
const NEEDS_POTION: Consideration = consideration(
    "needs potion",
    ConsiderationInput::MissingHealth,
    Interval::new(0.42, 0.82),
    Curve::Linear,
);
const NOT_DASHING: Consideration =
    flag("dashing", ConsiderationInput::MotionRunning(Motion::Dash)).inverted("not dashing");
const FLEEING: Consideration = flag("fleeing", ConsiderationInput::MotionRunning(Motion::Flee));
const FAR_FROM_TARGET: Consideration =
    target_within_distance("near target", HEAL_SAFE_DISTANCE).inverted("far from target");

// Reach fractions, so a longer weapon stands the warrior farther back. Both
// enter at `MELEE_ENTER_FRACTION` so one always applies; the wider exits let a
// chase close a little deeper and a hold stay until out of reach.
const MELEE_ENTER_FRACTION: f32 = 0.97;

const OUTSIDE_MELEE_REACH: Consideration = reach_fraction_threshold(
    "outside melee reach",
    ConsiderationInput::ActionReachFraction(MAINHAND),
    MELEE_ENTER_FRACTION,
    0.94,
);
const INSIDE_MELEE_REACH: Consideration = reach_fraction_threshold(
    "inside melee reach",
    ConsiderationInput::ActionReachFraction(MAINHAND),
    MELEE_ENTER_FRACTION,
    1.0,
);

const FLEE: Behavior = Behavior::new(
    Some(Motion::Flee),
    None,
    CommitPolicy::ContinuouslyScored,
    7.0,
    &[HAS_TARGET, BADLY_HURT],
);
const CHASE_MELEE: Behavior = Behavior::new(
    Some(Motion::Chase),
    None,
    CommitPolicy::ContinuouslyScored,
    4.0,
    &[OUTSIDE_MELEE_REACH],
);

/// Stands still in melee reach instead of orbiting.
const HOLD_IN_MELEE_REACH: Behavior = Behavior::new(
    Some(Motion::Idle),
    None,
    CommitPolicy::ContinuouslyScored,
    5.0,
    &[INSIDE_MELEE_REACH],
);

/// Runs out the break rolled after a swing.
const WARRIOR_KITE: Behavior = Behavior::new(
    Some(Motion::Dash),
    None,
    CommitPolicy::UntilFinished { interrupt_tier: 1 },
    10.0,
    &[ATTACK_FINISHED, HAS_TARGET],
);

/// Doing nothing on the action slot: the bar a heal must clear to fire.
const WARRIOR_BASELINE_IDLE: Behavior = Behavior::new(
    None,
    Some(Action::Idle),
    CommitPolicy::ContinuouslyScored,
    0.5,
    &[],
);

/// A fleeing warrior stops swinging until the flee ends. Flee is read from the
/// last decision, so the tick a flee starts can still swing.
const HOLD_FIRE_WHILE_FLEEING: Behavior = Behavior::new(
    None,
    Some(Action::Idle),
    CommitPolicy::ContinuouslyScored,
    1.5,
    &[FLEEING],
);

// Motion priority: kite (10), flee (7), return (6), hold melee (5), chase (4),
// wander (1). Kite beats a running flee (7 x 1.25 = 8.75).
//
// Action priority: heal (5, scaled by need), hold fire (1.5), swing (1),
// idle (0.5). Each beats the next even while that one holds the slot.
// - Heal pairs `NEEDS_POTION` with a yes/no input, so its score is the square root of the need. It
//   beats idle from about 43% missing health and hold fire from about 48%, before the warrior
//   starts fleeing at about 50% (health never recovers during a flee).
static WARRIOR_BEHAVIORS: &[Behavior] = &[
    WANDER,
    WARRIOR_BASELINE_IDLE,
    RETURN_TO_HOME,
    FLEE,
    CHASE_MELEE,
    HOLD_IN_MELEE_REACH,
    WARRIOR_KITE,
    HOLD_FIRE_WHILE_FLEEING,
    Behavior::new(
        None,
        Some(MAINHAND),
        CommitPolicy::ContinuouslyScored,
        1.0,
        &[IN_MAINHAND_REACH, NOT_DASHING],
    ),
    // The two heals: drink only at a safe distance, or with no target.
    Behavior::new(
        None,
        Some(Action::Ability(Ability::Heal)),
        CommitPolicy::ContinuouslyScored,
        5.0,
        &[NEEDS_POTION, FAR_FROM_TARGET],
    ),
    Behavior::new(
        None,
        Some(Action::Ability(Ability::Heal)),
        CommitPolicy::ContinuouslyScored,
        5.0,
        &[NEEDS_POTION, NO_TARGET],
    ),
];

/// Closes to melee reach, plants to swing, usually runs a break away after a
/// swing, flees when badly hurt, and walks home once disengaged.
pub(super) static WARRIOR: BrainDefinition = BrainDefinition {
    behaviors: WARRIOR_BEHAVIORS,
};

/// Seconds between potions, so a hurt warrior drinks one at a time.
const HEAL_COOLDOWN: f32 = 1.0;

/// Spawns a sword-carrying warrior on top of the shared enemy `base` bundle.
pub(super) fn spawn_warrior(
    commands: &mut Commands,
    base: impl Bundle,
    sprites: &SpriteAssets,
    sprite_layouts: &SpriteSheetLayouts,
    shadows: &Shadows,
) {
    spawn_enemy_with_equipment(
        commands,
        (
            warrior(sprites, sprite_layouts),
            base,
            enemy_children(shadows),
        ),
        sword(sprites),
    );
}

fn warrior(sprites: &SpriteAssets, sprite_layouts: &SpriteSheetLayouts) -> impl Bundle {
    (
        (Kinematics::new(200.0), Health::new(40.0)),
        Experience { base_exp: 20.0 },
        ground_sight_ray(),
        YSort::from_offset(CHARACTER_FEET_POS_OFFSET),
        // 80% of swings end in a 6-8 tile run inside a 90° cone away from the target.
        Dash::kite(
            0.8,
            Interval::new(6.0 * TILE_SIZE as f32, 8.0 * TILE_SIZE as f32),
        ),
        Heal::new(HEAL_COOLDOWN),
        enemy_wander(),
        Sprite::from_atlas_image(
            sprites.warrior_enemy_sprite_sheet.clone(),
            TextureAtlas {
                layout: sprite_layouts.enemy_atlas_layout.clone(),
                ..default()
            },
        ),
        related!(Items[health_potion(sprites)]),
    )
}
