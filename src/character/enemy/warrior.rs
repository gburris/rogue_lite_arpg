use avian2d::prelude::Mass;
use bevy::prelude::*;

use super::{
    Experience, enemy_children, enemy_wander, ground_sight_ray, spawn_enemy_with_equipment,
};
use crate::character::ability::heal;
use crate::character::brain::considerations::{
    ATTACK_FINISHED, HAS_TARGET, IN_MAINHAND_REACH, MAINHAND, NO_TARGET, RETURN_TO_HOME,
    SEEK_LOST_TARGET, WANDER, consideration, flag, motion_running_for, reach_fraction_threshold,
    target_within_distance,
};
use crate::character::brain::{
    Action, Behavior, BrainDefinition, CommitPolicy, Consideration, ConsiderationInput, Curve,
    Interval,
};
use crate::character::motion::{Dash, Motion};
use crate::prelude::*;

const HEAL: Action = Action::Use(ActionKey::Ability(Ability::Heal));

/// Seconds a warrior must stand in melee reach before it swings, so a target that keeps moving can
/// step back out of reach first.
const WARRIOR_SWING_DELAY: f32 = 0.25;
/// Distance from the target at which a wounded warrior may drink its potion.
const HEAL_SAFE_DISTANCE: f32 = 200.0;

/// Zero above half health, one just below it.
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
const STOOD_STILL_IN_REACH: Consideration =
    motion_running_for("stood still in reach", Motion::Idle, WARRIOR_SWING_DELAY);
const FLEEING: Consideration = flag("fleeing", ConsiderationInput::MotionRunning(Motion::Flee));
const FAR_FROM_TARGET: Consideration =
    target_within_distance("near target", HEAL_SAFE_DISTANCE).inverted("far from target");
const HAS_POTION: Consideration = flag("has potion", ConsiderationInput::ActionUsable(HEAL));
const CORNERED: Consideration = flag("cornered", ConsiderationInput::MovementBlocked);

// Reach fractions, so a longer weapon stands the warrior farther back. Both
// enter at `MELEE_ENTER_FRACTION` so one always applies; the wider exits let a
// chase close a little deeper and standing still last until out of reach.
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

/// Breaks away to drink. A warrior with no potion left stays and fights.
const FLEE: Behavior = Behavior::new(
    Some(Motion::Flee),
    None,
    CommitPolicy::ContinuouslyScored,
    7.0,
    &[HAS_TARGET, BADLY_HURT, HAS_POTION],
);
const CHASE_MELEE: Behavior = Behavior::new(
    Some(Motion::Chase),
    None,
    CommitPolicy::ContinuouslyScored,
    4.0,
    &[OUTSIDE_MELEE_REACH],
);

/// Stands still in melee reach instead of orbiting.
const STAND_STILL_IN_MELEE_REACH: Behavior = Behavior::new(
    Some(Motion::Idle),
    None,
    CommitPolicy::ContinuouslyScored,
    5.0,
    &[INSIDE_MELEE_REACH],
);

/// Swings once it has stood still in reach for [`WARRIOR_SWING_DELAY`], standing still
/// throughout.
const WARRIOR_SWING: Behavior = Behavior::new(
    Some(Motion::Idle),
    Some(MAINHAND),
    CommitPolicy::UntilFinished { interrupt_tier: 1 },
    6.5,
    &[IN_MAINHAND_REACH, STOOD_STILL_IN_REACH],
);

/// Runs out the break rolled after a swing.
const WARRIOR_KITE: Behavior = Behavior::new(
    Some(Motion::Dash),
    None,
    CommitPolicy::UntilFinished { interrupt_tier: 1 },
    10.0,
    &[ATTACK_FINISHED, HAS_TARGET],
)
.with_start_chance(0.6, 0.0);

/// Doing nothing on the action slot: the bar a heal must clear to fire.
const WARRIOR_BASELINE_IDLE: Behavior = Behavior::new(
    None,
    Some(Action::Idle),
    CommitPolicy::ContinuouslyScored,
    0.5,
    &[],
);

// Motion priority: kite (10), flee (7), seek (7), swing (6.5), return (6),
// stand still in melee reach (5), chase (4), wander (1).
// - Kite beats a running flee (7 x 1.25 = 8.75).
// - Seek beats a running stand still (6.25), so a warrior walks onto the spot it last saw its
//   target instead of stopping a sword's length short of it.
// - The swing beats the running stand still (6.25) it waits in, taking the action slot with it. It
//   needs that idle motion, so a chasing, dashing or fleeing warrior never swings.
//
// Action priority: swing (6.5, see above), heal (5, scaled by need), idle (0.5).
// Each beats the next even while that one holds the slot. The swing and heals never compete,
// since every heal needs distance, no target, or a flee.
// - Heal pairs `NEEDS_POTION` with a yes/no input, so its score is the square root of the need. It
//   beats idle from about 43% missing health, before the warrior starts fleeing at about 52%
//   (health never recovers during a flee).
// - The cornered heal has two yes/no inputs, so it scores the cube root of the need, which beats
//   idle at any need a fleeing warrior has.
// - Every heal is committed, so the action slot holds the drink until it ends.
static WARRIOR_BEHAVIORS: &[Behavior] = &[
    WANDER,
    WARRIOR_BASELINE_IDLE,
    RETURN_TO_HOME,
    FLEE,
    SEEK_LOST_TARGET,
    CHASE_MELEE,
    STAND_STILL_IN_MELEE_REACH,
    WARRIOR_KITE,
    WARRIOR_SWING,
    // The three heals: drink at a safe distance, with no target, or when a flee hits a wall.
    Behavior::new(
        None,
        Some(HEAL),
        CommitPolicy::UntilFinished { interrupt_tier: 1 },
        5.0,
        &[NEEDS_POTION, FAR_FROM_TARGET],
    ),
    Behavior::new(
        None,
        Some(HEAL),
        CommitPolicy::UntilFinished { interrupt_tier: 1 },
        5.0,
        &[NEEDS_POTION, NO_TARGET],
    ),
    Behavior::new(
        None,
        Some(HEAL),
        CommitPolicy::UntilFinished { interrupt_tier: 1 },
        5.0,
        &[NEEDS_POTION, FLEEING, CORNERED],
    ),
];

/// Closes to melee reach, plants to swing, often runs a short break away after a
/// swing, flees to drink its potion when badly hurt, and walks home once disengaged.
pub(super) static WARRIOR: BrainDefinition = BrainDefinition {
    behaviors: WARRIOR_BEHAVIORS,
};

/// Seconds between potions, so a hurt warrior drinks one at a time.
const HEAL_COOLDOWN: f32 = 1.0;
/// Seconds a drink takes before the potion heals.
const HEAL_WINDUP_SECONDS: f32 = 0.6;
const HEAL_GLOW_COLOR: Color = Color::srgb(0.3, 1.0, 0.4);

/// Spawns a sword-carrying warrior on top of the shared enemy `base` scene.
pub(super) fn spawn_warrior(
    commands: &mut Commands,
    base: impl Scene,
    sprites: &SpriteAssets,
    sprite_layouts: &SpriteSheetLayouts,
    shadows: &Shadows,
) {
    spawn_enemy_with_equipment(
        commands,
        bsn! {
            @warrior(sprites, sprite_layouts)
            @{base}
            @enemy_children(shadows)
        },
        sword(sprites),
    );
}

fn warrior(sprites: &SpriteAssets, sprite_layouts: &SpriteSheetLayouts) -> impl Scene {
    let sprite = Sprite::from_atlas_image(
        sprites.warrior_enemy_sprite_sheet.clone(),
        TextureAtlas {
            layout: sprite_layouts.enemy.clone(),
            ..default()
        },
    );
    bsn! {
        ~Kinematics::new(200.0)
        ~Health::new(40.0)
        Mass(80.0)
        Experience(20.0)
        ground_sight_ray()
        YSort::from_offset(CHARACTER_FEET_POS_OFFSET)
        ~Dash::kite(Interval::new(3.0 * TILE_SIZE as f32, 5.0 * TILE_SIZE as f32))
        Abilities [
            @heal()
            ~ActionCooldown::new(HEAL_COOLDOWN)
            ActionWindup { seconds: HEAL_WINDUP_SECONDS }
            ActionGlow(HEAL_GLOW_COLOR)
        ]
        enemy_wander()
        template(move |_| Ok(sprite.clone()))
        Items [
            @health_potion(sprites)
        ]
    }
}
