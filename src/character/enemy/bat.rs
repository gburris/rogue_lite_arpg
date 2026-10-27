use avian2d::prelude::{Collider, CollisionLayers, SpatialQueryFilter};
use bevy::prelude::*;

use super::{Experience, enemy_wander, sight_ray};
use crate::character::animation::AnimationSheet;
use crate::character::brain::considerations::{
    CALM, CAST_DISTANCE_RANGE, CAST_HYSTERESIS, CAST_MAX_RANGE, CAST_MIN_RANGE, CHASE_CAST_RANGE,
    INSIDE_OUTER_CAST_EDGE, LINE_OF_SIGHT, RETURN_TO_ANCHOR, consideration, distance_threshold,
    flag, reach_fraction_threshold, within_reach,
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
/// How much the target distance calls for breath: zero where a bite arms, so
/// the bat only breathes once biting stops being an option, rising to full at
/// the inner cast edge, where it circles.
const BREATH_DISTANCE: Consideration = consideration(
    "breath distance",
    ConsiderationInput::TargetDistance,
    Interval::new(BITE_ARMING_DISTANCE, CAST_MIN_RANGE),
    Curve::Linear,
);
/// How settled the engagement is: zero for the first second after spotting a
/// target, full a second later, so a fresh target gets a moment before the
/// first fireball.
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

/// Orbits the casting distance range; the bat breathes innately, no ready weapon needed.
const BAT_ORBIT: Behavior = Behavior::new(
    Some(Motion::Orbit(CAST_DISTANCE_RANGE)),
    None,
    CommitPolicy::Momentum,
    5.5,
    &[PAST_INNER_CAST_EDGE, INSIDE_OUTER_CAST_EDGE],
);
/// Holds the core so it keeps repositioning toward the middle.
const BAT_KEEP: Behavior = Behavior::new(
    Some(Motion::KeepDistance(CAST_CORE_DISTANCE_RANGE)),
    None,
    CommitPolicy::Momentum,
    5.0,
    &[INSIDE_OUTER_CAST_EDGE],
);
/// Bite dive while the bat is circling inside its cast range.
const BAT_SWOOP: Behavior = Behavior::new(
    Some(Motion::Swoop),
    None,
    CommitPolicy::UntilFinished { interrupt_tier: 1 },
    10.0,
    &[IN_SWOOP_REACH, CIRCLING],
);

/// Doing nothing on the action slot: the bar a breath must clear to fire.
const BAT_BASELINE_IDLE: Behavior =
    Behavior::new(None, Some(Action::Idle), CommitPolicy::Momentum, 0.5, &[]);
/// Fireball from range, never mid-dive.
const BAT_BREATHE: Behavior = Behavior::new(
    None,
    Some(Action::Ability(Ability::Breathe)),
    CommitPolicy::Momentum,
    1.0,
    &[
        LINE_OF_SIGHT,
        IN_BREATH_REACH,
        BREATH_DISTANCE,
        BREATH_WARMED_UP,
        NOT_SWOOPING,
    ],
);
/// Arms a bite near contact reach, whether diving or circling; the applier
/// bites only in contact.
const BAT_BITE: Behavior = Behavior::new(
    None,
    Some(Action::Ability(Ability::Bite)),
    CommitPolicy::Momentum,
    1.5,
    &[NEAR_BITE_REACH],
);

// Motion priority: swoop (10), return (6), orbit (5.5), keep (5), chase (4),
// calm (1). A swoop beats an orbit that is already running (5.5 x 1.25 = 6.875).
//
// Action priority: bite (1.5), breathe (up to 1), idle (0.5).
// - Idle is the bar an action must beat to fire: 0.5, or 0.625 while it holds
//   the slot.
// - A bite never competes with the breath. The breath's distance input is zero
//   wherever a bite arms.
// - The breath has three yes/no inputs and two graded ones (distance, warm-up).
//   Its score is the fifth root of their product, which flattens both ramps. It
//   beats a running idle from about 82 units away once warmed up, and from
//   about 1.1 s after spotting the target (1.4 s at 100 units).
static BAT_BEHAVIORS: &[Behavior] = &[
    CALM,
    BAT_BASELINE_IDLE,
    RETURN_TO_ANCHOR,
    CHASE_CAST_RANGE,
    BAT_ORBIT,
    BAT_KEEP,
    BAT_SWOOP,
    BAT_BREATHE,
    BAT_BITE,
];

/// Circles its target, dives to bite, and breathes fireballs; no weapon needed.
pub(super) static BAT: BrainDefinition = BrainDefinition {
    behaviors: BAT_BEHAVIORS,
};

const BAT_SIGHT_RANGE: f32 = 550.0;
const BAT_BREATH_PERIOD: f32 = 1.1;
const BAT_TURN_RATE: f32 = 6.0;
/// Half-width of a bat's rerolled orbit-radius offset from the band's centre.
const BAT_ORBIT_STANDOFF: f32 = 50.0;
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
    // Bats carry no staff: the breath template rides on the bat
    // itself and the `Breath` applier launches it from brain intent.
    commands.spawn((bat(sprites, sprite_layouts), base, bat_children(shadows)));
}

fn bat(sprites: &SpriteAssets, sprite_layouts: &SpriteSheetLayouts) -> impl Bundle {
    (
        (SimpleMotion::new(170.0), Health::new(15.0)),
        Experience { base_exp: 6.0 },
        // Bats fly high: longer reach, and high obstacles never block the
        // ray, so a bat holds its target through cover.
        sight_ray(
            BAT_SIGHT_RANGE,
            SpatialQueryFilter::from_mask([GameCollisionLayer::AllyHurtBox]),
        ),
        YSort::from_offset(BAT_FEET_POS_OFFSET),
        TurnRate(BAT_TURN_RATE),
        Orbit::with_standoff(BAT_ORBIT_STANDOFF),
        enemy_wander(),
        // 35% chance per roll within cast range; after a dive ends, wait 2 s
        // before the next roll. The dive is a twice-as-fast pass that flies
        // through the target and back out to roughly the circling radius.
        Swoop::new(0.35, CAST_MAX_RANGE, 2.0),
        // The bite is its own ability: it lands whenever a target is in reach,
        // and a landed bite feeds the bat. The cooldown spans a whole pass so
        // one dive bites once.
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
