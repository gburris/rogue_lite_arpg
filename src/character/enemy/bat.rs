use avian2d::prelude::{Collider, CollisionLayers, SpatialQueryFilter};
use bevy::prelude::*;

use super::{Experience, enemy_wander, sight_ray};
use crate::character::animation::AnimationSheet;
use crate::character::brain::considerations::{
    CAST_DISTANCE_RANGE, CAST_HYSTERESIS, CAST_MAX_RANGE, CAST_MIN_RANGE, CHASE_CAST_RANGE,
    INSIDE_OUTER_CAST_EDGE, LINE_OF_SIGHT, RETURN_TO_HOME, WANDER, consideration,
    distance_threshold, flag, reach_fraction_threshold, within_reach,
};
use crate::character::brain::{
    Ability, Action, Behavior, Bite, BrainDefinition, Breath, CommitPolicy, Consideration,
    ConsiderationInput, Curve, Interval, Motion, Orbit, Swoop, TurnRate,
};
use crate::prelude::*;

/// Core of the cast distance range where shot quality stays full.
const CAST_CORE_DISTANCE_RANGE: Interval =
    Interval::new(CAST_MIN_RANGE + 20.0, CAST_MAX_RANGE - 20.0);

const NOT_SWOOPING: Consideration =
    flag("swooping", ConsiderationInput::MotionRunning(Motion::Swoop)).inverted("not swooping");
const CIRCLING: Consideration = flag("circling", ConsiderationInput::Circling);
const IN_SWOOP_REACH: Consideration = within_reach(
    "in swoop reach",
    ConsiderationInput::MotionReachFraction(Motion::Swoop),
);
const IN_BREATH_REACH: Consideration = within_reach(
    "in breath reach",
    ConsiderationInput::ActionReachFraction(Action::Ability(Ability::Breathe)),
);
/// Distance out to which the decision arms a bite: 40 units past contact.
/// Decisions sample at ~10 Hz, so a dive crossing the reach could otherwise
/// fall between two samples; the applier still checks the true reach.
const BITE_ARMING_DISTANCE: f32 = BITE_REACH + 40.0;
const BITE_ARMING_REACH_FRACTION: f32 = BITE_ARMING_DISTANCE / BITE_REACH;
const NEAR_BITE_REACH: Consideration = reach_fraction_threshold(
    "near bite reach",
    ConsiderationInput::ActionReachFraction(Action::Ability(Ability::Bite)),
    BITE_ARMING_REACH_FRACTION,
    BITE_ARMING_REACH_FRACTION,
);
/// Zero where a bite arms, so breath and bite never compete; full at the
/// inner cast edge.
const BREATH_DISTANCE: Consideration = consideration(
    "breath distance",
    ConsiderationInput::TargetDistance,
    Interval::new(BITE_ARMING_DISTANCE, CAST_MIN_RANGE),
    Curve::Linear,
);
/// Zero for the first second after spotting a target, full a second later.
const BREATH_WARMED_UP: Consideration = consideration(
    "breath warmed up",
    ConsiderationInput::SecondsSinceTargeted,
    Interval::new(1.0, 2.0),
    Curve::Linear,
);
const PAST_INNER_CAST_EDGE: Consideration = distance_threshold(
    "past inner cast edge",
    CAST_MIN_RANGE + CAST_HYSTERESIS,
    CAST_MIN_RANGE - CAST_HYSTERESIS,
);

/// Orbits the cast distance range.
const BAT_ORBIT: Behavior = Behavior::new(
    Some(Motion::Orbit(CAST_DISTANCE_RANGE)),
    None,
    CommitPolicy::ContinuouslyScored,
    5.5,
    &[PAST_INNER_CAST_EDGE, INSIDE_OUTER_CAST_EDGE],
);
/// Holds the core so it keeps repositioning toward the middle.
const BAT_KEEP: Behavior = Behavior::new(
    Some(Motion::KeepDistance(CAST_CORE_DISTANCE_RANGE)),
    None,
    CommitPolicy::ContinuouslyScored,
    5.0,
    &[INSIDE_OUTER_CAST_EDGE],
);
/// Bite dive while circling.
const BAT_SWOOP: Behavior = Behavior::new(
    Some(Motion::Swoop),
    None,
    CommitPolicy::UntilFinished { interrupt_tier: 1 },
    10.0,
    &[IN_SWOOP_REACH, CIRCLING],
);

/// Doing nothing on the action slot: the bar a breath must clear to fire.
const BAT_BASELINE_IDLE: Behavior = Behavior::new(
    None,
    Some(Action::Idle),
    CommitPolicy::ContinuouslyScored,
    0.5,
    &[],
);
/// Fireball from range, never mid-dive.
const BAT_BREATHE: Behavior = Behavior::new(
    None,
    Some(Action::Ability(Ability::Breathe)),
    CommitPolicy::ContinuouslyScored,
    1.0,
    &[
        LINE_OF_SIGHT,
        IN_BREATH_REACH,
        BREATH_DISTANCE,
        BREATH_WARMED_UP,
        NOT_SWOOPING,
    ],
);
/// Arms a bite near contact reach, whether diving or circling.
const BAT_BITE: Behavior = Behavior::new(
    None,
    Some(Action::Ability(Ability::Bite)),
    CommitPolicy::ContinuouslyScored,
    1.5,
    &[NEAR_BITE_REACH],
);

// Motion priority: swoop (10), return (6), orbit (5.5), keep (5), chase (4),
// wander (1). A swoop beats an orbit that is already running (5.5 x 1.25 = 6.875).
//
// Action priority: bite (1.5), breathe (up to 1), idle (0.5).
// - Idle is the bar an action must beat to fire: 0.5, or 0.625 while it holds
//   the slot.
// - The breath has three yes/no inputs and two graded ones (distance, warm-up).
//   Its score is the fifth root of their product, which flattens both ramps. It
//   beats a running idle from about 82 units away once warmed up, and from
//   about 1.1 s after spotting the target (1.4 s at 100 units).
static BAT_BEHAVIORS: &[Behavior] = &[
    WANDER,
    BAT_BASELINE_IDLE,
    RETURN_TO_HOME,
    CHASE_CAST_RANGE,
    BAT_ORBIT,
    BAT_KEEP,
    BAT_SWOOP,
    BAT_BREATHE,
    BAT_BITE,
];

/// Circles its target, dives to bite, and breathes fireballs.
pub(super) static BAT: BrainDefinition = BrainDefinition {
    behaviors: BAT_BEHAVIORS,
};

const BAT_SIGHT_RANGE: f32 = 550.0;
const BAT_BREATH_PERIOD: f32 = 1.1;
const BAT_TURN_RATE: f32 = 6.0;
/// Half-width of a bat's rerolled orbit-radius offset from the band's centre.
const BAT_ORBIT_MAX_DISTANCE_OFFSET: f32 = 50.0;
/// Contact range for a bite in world units. Anatomy, not tuning.
const BITE_REACH: f32 = 30.0;
const BITE_DAMAGE: f32 = 6.0;
/// Seconds between bites; longer than a dive's time in reach, so a pass bites once.
const BITE_COOLDOWN: f32 = 1.0;
/// Bat sprites sort and clip at their own 32x32 frame edge.
const BAT_FEET_POS_OFFSET: f32 = -16.0;
const BAT_SHADOW_DROP: f32 = 4.0;
const BAT_BODY_OFFSET: f32 = BAT_FEET_POS_OFFSET + 8.0;
const BAT_HURTBOX_OFFSET: f32 = BAT_FEET_POS_OFFSET + 12.0;

/// Spawns a weaponless bat on top of the shared enemy `base` bundle.
pub(super) fn spawn_bat(
    commands: &mut Commands,
    base: impl Bundle,
    sprites: &SpriteAssets,
    sprite_layouts: &SpriteSheetLayouts,
    shadows: &Shadows,
) {
    commands.spawn((bat(sprites, sprite_layouts), base, bat_children(shadows)));
}

fn bat(sprites: &SpriteAssets, sprite_layouts: &SpriteSheetLayouts) -> impl Bundle {
    (
        (Kinematics::new(170.0), Health::new(15.0)),
        Experience { base_exp: 6.0 },
        // Bats fly high: longer reach, and high obstacles never block the
        // ray, so a bat holds its target through cover.
        sight_ray(
            BAT_SIGHT_RANGE,
            SpatialQueryFilter::from_mask([GameCollisionLayer::AllyHurtBox]),
        ),
        YSort::from_offset(BAT_FEET_POS_OFFSET),
        TurnRate(BAT_TURN_RATE),
        Orbit::new().with_max_distance_offset(BAT_ORBIT_MAX_DISTANCE_OFFSET),
        enemy_wander(),
        Swoop::new(0.35, CAST_MAX_RANGE, 2.0),
        Bite::new(BITE_REACH, BITE_DAMAGE, BITE_COOLDOWN).with_life_steal(1.0),
        Sprite::from_atlas_image(
            sprites.bat_enemy_sprite_sheet.clone(),
            TextureAtlas {
                layout: sprite_layouts.bat_enemy_layout.clone(),
                index: 0, // Down row; FacingDirection defaults to Down.
            },
        ),
        AnimationSheet::bat(),
        Breath::new(BAT_BREATH_PERIOD, CAST_MAX_RANGE),
        related!(Projectiles[fireball(sprites, sprite_layouts, 0.0)]),
    )
}

fn bat_children(shadows: &Shadows) -> impl Bundle {
    children![
        shadow(shadows, BAT_FEET_POS_OFFSET - BAT_SHADOW_DROP),
        bat_collider(),
        hurtbox_at(
            Vec2::new(24.0, 24.0),
            GameCollisionLayer::EnemyHurtBox,
            BAT_HURTBOX_OFFSET,
        ),
    ]
}

fn bat_collider() -> impl Bundle {
    (
        Transform::from_xyz(0.0, BAT_BODY_OFFSET, 0.0),
        Collider::circle(8.0),
        CollisionLayers::new(
            [GameCollisionLayer::InAir],
            [GameCollisionLayer::HighObstacle],
        ),
    )
}
