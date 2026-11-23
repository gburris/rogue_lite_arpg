use avian2d::prelude::{Collider, CollisionLayers, Mass, SpatialQueryFilter};
use bevy::prelude::*;

use super::{Experience, enemy_motion_sources, sight_ray};
use crate::character::ability::{Bite, bite, breath};
use crate::character::animation::AnimationSheet;
use crate::character::brain::{
    Action, Behavior, BrainDefinition, CAST_DISTANCE_RANGE, CAST_HYSTERESIS, CAST_MAX_RANGE,
    CAST_MIN_RANGE, CHASE_CAST_RANGE, CommitPolicy, Consideration, ConsiderationInput, Curve,
    INSIDE_OUTER_CAST_EDGE, LINE_OF_SIGHT, RETURN_TO_HOME, SEEK_LOST_TARGET, WANDER, consideration,
    distance_threshold, flag, flag_kept_while_current, reach_fraction_threshold, within_reach,
};
use crate::character::motion::{KeepDistance, MotionKey, MotionSources, Orbit, Swoop, TurnRate};
use crate::interval::Interval;
use crate::prelude::*;

const BREATH: Action = Action::Use(ActionKey::Ability(Ability::Breath));
const BITE: Action = Action::Use(ActionKey::Ability(Ability::Bite));

/// Core of the cast distance range, which a bat backs out to after a swoop.
const CAST_CORE_DISTANCE_RANGE: Interval =
    Interval::new(CAST_MIN_RANGE + 20.0, CAST_MAX_RANGE - 20.0);

/// Seconds before a bat can swoop again after a swoop ends or misses its start roll.
const SWOOP_COOLDOWN: f32 = 2.0;

/// One during the swoop's cooldown or start roll retry delay; zero while a swoop is in progress or
/// can start.
const SWOOP_COOLING_DOWN: Consideration = flag(
    "swoop ready",
    ConsiderationInput::MotionReady(MotionKey::Swoop),
)
.inverted("swoop cooling down");
const IN_SWOOP_REACH: Consideration = within_reach(
    "in swoop reach",
    ConsiderationInput::MotionReachFraction(MotionKey::Swoop),
);
const IN_BREATH_REACH: Consideration = within_reach(
    "in breath reach",
    ConsiderationInput::ActionReachFraction(BREATH),
);
/// Distance out to which the decision arms a bite: 40 units past contact. Decisions are
/// sampled, so a swoop crossing the reach could otherwise fall between two samples; the bite's
/// jaws stay open long enough to close the gap.
const BITE_ARMING_DISTANCE: f32 = BITE_REACH + 40.0;
const BITE_ARMING_REACH_FRACTION: f32 = BITE_ARMING_DISTANCE / BITE_REACH;
const NEAR_BITE_REACH: Consideration = reach_fraction_threshold(
    "near bite reach",
    ConsiderationInput::ActionReachFraction(BITE),
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
// Both enter at one shared distance, so a bat with no incumbent always has one of the two.
const PAST_INNER_CAST_EDGE: Consideration = distance_threshold(
    "past inner cast edge",
    CAST_MIN_RANGE + CAST_HYSTERESIS,
    CAST_MIN_RANGE - CAST_HYSTERESIS,
);
const INSIDE_INNER_CAST_EDGE: Consideration = distance_threshold(
    "inside inner cast edge",
    CAST_MIN_RANGE + CAST_HYSTERESIS,
    CAST_MIN_RANGE + 2.0 * CAST_HYSTERESIS,
);

/// Orbits the cast distance range.
const BAT_ORBIT: Behavior = Behavior::new(
    Some(MotionKey::Orbit),
    None,
    CommitPolicy::ContinuouslyScored,
    5.5,
    &[PAST_INNER_CAST_EDGE, INSIDE_OUTER_CAST_EDGE],
);
/// Backs out of the target's face toward the core, then hands back to the orbit.
const BAT_KEEP: Behavior = Behavior::new(
    Some(MotionKey::KeepDistance),
    None,
    CommitPolicy::ContinuouslyScored,
    5.0,
    &[INSIDE_INNER_CAST_EDGE],
);
/// Blocked while moving; see [`flag_kept_while_current`].
const CORNERED: Consideration =
    flag_kept_while_current("cornered", ConsiderationInput::MovementBlocked);

/// Slides along whatever pins a bat backing away, circling the target until the bat is back
/// past the inner cast edge.
const BAT_SLIP_AWAY: Behavior = Behavior::new(
    Some(MotionKey::Orbit),
    None,
    CommitPolicy::ContinuouslyScored,
    6.5,
    &[CORNERED, INSIDE_INNER_CAST_EDGE],
);
/// Swoops in to bite.
const BAT_SWOOP: Behavior = Behavior::new(
    Some(MotionKey::Swoop),
    None,
    CommitPolicy::UntilFinished { interrupt_tier: 1 },
    10.0,
    &[IN_SWOOP_REACH],
)
.with_start_chance(0.35, SWOOP_COOLDOWN);

/// Doing nothing on the action slot: the bar a breath must clear to fire.
const BAT_BASELINE_IDLE: Behavior = Behavior::new(
    None,
    Some(Action::Idle),
    CommitPolicy::ContinuouslyScored,
    0.5,
    &[],
);
/// Fireball from range, only between swoops.
const BAT_BREATH: Behavior = Behavior::new(
    None,
    Some(BREATH),
    CommitPolicy::ContinuouslyScored,
    1.0,
    &[
        LINE_OF_SIGHT,
        IN_BREATH_REACH,
        BREATH_DISTANCE,
        BREATH_WARMED_UP,
        SWOOP_COOLING_DOWN,
    ],
);
const SWOOPING: Consideration = flag(
    "swooping",
    ConsiderationInput::MotionRunning(MotionKey::Swoop),
);

/// Arms a bite near contact reach, only while swooping.
const BAT_BITE: Behavior = Behavior::new(
    None,
    Some(BITE),
    CommitPolicy::ContinuouslyScored,
    1.5,
    &[LINE_OF_SIGHT, NEAR_BITE_REACH, SWOOPING],
);

// Motion priority: swoop (10), seek (7), slip away (6.5), return (6), orbit (5.5), keep (5),
// chase (4), wander (1).
// - Slipping away beats a running keep (6.25), so a bat pinned against a wall circles out instead
//   of pushing into it. It holds the slot past the orbit's entry edge, where the orbit continues
//   the same circling.
// - A swoop beats an orbit that is already running (5.5 x 1.25 = 6.875).
// - Keep and orbit never score together: keep holds the slot out to 10 units past the inner cast
//   edge, where the orbit takes over, and the orbit holds it down to 5 units inside that edge.
//
// Action priority: bite (1.5), breathe (up to 1), idle (0.5).
// - Idle is the bar an action must beat to fire: 0.5, or 0.625 while it holds the slot.
// - The breath has three yes/no inputs and two graded ones (distance, warm-up). Its score is the
//   fifth root of their product, which flattens both ramps. It beats a running idle from about 82
//   units away once warmed up, and from about 1.1 s after spotting the target (1.4 s at 100 units).
static BAT_BEHAVIORS: &[Behavior] = &[
    WANDER,
    BAT_BASELINE_IDLE,
    RETURN_TO_HOME,
    SEEK_LOST_TARGET,
    CHASE_CAST_RANGE,
    BAT_ORBIT,
    BAT_KEEP,
    BAT_SLIP_AWAY,
    BAT_SWOOP,
    BAT_BREATH,
    BAT_BITE,
];

/// Circles its target, swoops in to bite, and breathes fireballs.
pub(super) static BAT: BrainDefinition = BrainDefinition {
    behaviors: BAT_BEHAVIORS,
};

const BAT_SIGHT_RANGE: f32 = 550.0;
const BAT_BREATH_PERIOD: f32 = 1.1;
/// Seconds either way each wait between a bat's breaths may vary.
const BAT_BREATH_JITTER: f32 = 0.3;
const BAT_TURN_RATE: f32 = 6.0;
/// Half-width of a bat's rerolled orbit-radius offset from the band's centre.
const BAT_ORBIT_MAX_DISTANCE_OFFSET: f32 = 50.0;
/// Contact range for a bite in world units, where its hitbox's front edge reaches. Short enough
/// that a sidestep as the swoop starts usually gets clear.
const BITE_REACH: f32 = 24.0;
const BITE_DAMAGE: f32 = 6.0;
/// Radius of the bite's hitbox: about the bat's own size.
const BAT_BITE_RADIUS: f32 = 12.0;
/// Seconds between bites; longer than a swoop's time in reach, so a swoop bites once.
const BITE_COOLDOWN: f32 = 1.0;
/// Bat sprites sort and clip at their own 32x32 frame edge.
const BAT_FEET_POS_OFFSET: f32 = -16.0;
const BAT_SHADOW_DROP: f32 = 4.0;
const BAT_BODY_OFFSET: f32 = BAT_FEET_POS_OFFSET + 8.0;
const BAT_HURTBOX_OFFSET: f32 = BAT_FEET_POS_OFFSET + 12.0;

/// Spawns a weaponless bat on top of the shared enemy `base` scene.
pub(super) fn spawn_bat(
    commands: &mut Commands,
    base: impl Scene,
    sprites: &SpriteAssets,
    sprite_layouts: &SpriteSheetLayouts,
    shadows: &Shadows,
) {
    commands.spawn_scene(bsn! {
        @bat(sprites, sprite_layouts)
        @{base}
        @bat_children(shadows)
    });
}

fn bat(sprites: &SpriteAssets, sprite_layouts: &SpriteSheetLayouts) -> impl Scene {
    let sprite = Sprite::from_atlas_image(
        sprites.bat_enemy_sprite_sheet.clone(),
        TextureAtlas {
            layout: sprite_layouts.bat.clone(),
            index: 0, // Down row; FacingDirection defaults to Down.
        },
    );
    bsn! {
        ~Kinematics::new(170.0)
        ~Health::new(15.0)
        Mass(25.0)
        Experience(6.0)
        // Bats fly high: longer reach, and only the high obstacles that also stop
        // their flight block the ray.
        sight_ray(
            BAT_SIGHT_RANGE,
            BAT_BODY_OFFSET,
            SpatialQueryFilter::from_mask([
                GameCollisionLayer::AllyHurtBox,
                GameCollisionLayer::HighObstacle,
            ]),
        )
        YSort::from_offset(BAT_FEET_POS_OFFSET)
        TurnRate(BAT_TURN_RATE)
        MotionSources [
            {enemy_motion_sources()}
            --
            ~Orbit::new(CAST_DISTANCE_RANGE).with_distance_offset_spread(BAT_ORBIT_MAX_DISTANCE_OFFSET)
            --
            ~KeepDistance::new(CAST_CORE_DISTANCE_RANGE)
            --
            Swoop
            ~{Reach(CAST_MAX_RANGE)}
            ~Cooldown::new(SWOOP_COOLDOWN)
        ]
        template(move |_| Ok(sprite.clone()))
        ~AnimationSheet::bat()
        Abilities [
            @bite(
                BITE_REACH,
                Bite::new(BITE_DAMAGE, BAT_BITE_RADIUS)
                    .with_life_steal(1.0)
                    .with_mouth_height(BAT_HURTBOX_OFFSET)
            )
            ~Cooldown::new(BITE_COOLDOWN)
            --
            @breath(CAST_MAX_RANGE)
            ~Cooldown::new(BAT_BREATH_PERIOD).with_jitter(BAT_BREATH_JITTER)
            Projectiles [
                @fireball(sprites, sprite_layouts, 0.0)
            ]
        ]
    }
}

/// The shadow, airborne body collider and hurtbox of a bat.
fn bat_children(shadows: &Shadows) -> impl Scene {
    bsn! {
        Children [
            @shadow(shadows, BAT_FEET_POS_OFFSET - BAT_SHADOW_DROP)
            --
            Transform::from_xyz(0.0, BAT_BODY_OFFSET, 0.0)
            ~Collider::circle(8.0)
            ~CollisionLayers::new([GameCollisionLayer::InAir], [GameCollisionLayer::HighObstacle])
            --
            @hurtbox_at(Vec2::new(24.0, 24.0), GameCollisionLayer::EnemyHurtBox, BAT_HURTBOX_OFFSET)
        ]
    }
}
