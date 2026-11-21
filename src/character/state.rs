use std::time::Duration;
use avian2d::prelude::LinearVelocity;
use bevy::ecs::query::QueryData;
use bevy::prelude::*;
use crate::prelude::*;
use bevy::ecs::system::SystemParam;
use avian2d::prelude::*;

/// Fraction of its intended speed below which a character counts as blocked.
const BLOCKED_SPEED_FRACTION: f32 = 0.3;
/// Seconds a knockback shove lasts, its speed fading linearly to zero.
const KNOCKBACK_SECONDS: f32 = 0.2;

/// A shove from a hit, added on top of the character's own movement until it fades out.
#[derive(Component, Clone, Debug)]
pub struct KnockedBack {
    initial_velocity: Vec2,
    timer: Timer,
}

impl KnockedBack {
    /// Shoves `distance` world units along `direction` over [`KNOCKBACK_SECONDS`].
    pub fn new(direction: Vec2, distance: f32) -> Self {
        // A linear fade covers half of the starting speed times the duration.
        let initial_speed = 2.0 * distance / KNOCKBACK_SECONDS;
        Self {
            initial_velocity: direction.normalize_or_zero() * initial_speed,
            timer: Timer::from_seconds(KNOCKBACK_SECONDS, TimerMode::Once),
        }
    }

    /// Advances the shove by `delta` and returns its velocity, or `None` once it has faded.
    fn tick_and_compute_velocity(&mut self, delta: Duration) -> Option<Vec2> {
        self.timer.tick(delta);
        if self.timer.is_finished() {
            return None;
        }
        Some(self.initial_velocity * self.timer.fraction_remaining())
    }
}

/// Whether a character is interrupted: knocked back, stunned or defeated. An interrupted character
/// can't start using an item or ability, and loses any use or finite motion in progress.
#[derive(QueryData)]
pub struct InterruptionState {
    health: &'static Health,
    stat_multipliers: &'static StatMultipliers,
    knocked_back: Has<KnockedBack>,
}

impl InterruptionStateItem<'_, '_> {
    /// See [`InterruptionState`].
    pub fn is_interrupted(&self) -> bool {
        self.knocked_back || self.health.is_dead() || self.stat_multipliers.is_stunned()
    }
}

/// Whether a character is blocked: its physics velocity is well below the velocity its
/// [`Kinematics`] intend. Never true while it stands still or is stunned.
#[derive(QueryData)]
pub struct BlockedState {
    kinematics: &'static Kinematics,
    stat_multipliers: &'static StatMultipliers,
    linear_velocity: &'static LinearVelocity,
}

impl BlockedStateItem<'_, '_> {
    /// See [`BlockedState`].
    pub fn is_blocked(&self) -> bool {
        let intended_velocity = self.kinematics.velocity(self.stat_multipliers.speed);
        let intended_speed_squared = intended_velocity.length_squared();
        intended_speed_squared > 1.0
            && self.linear_velocity.dot(intended_velocity)
                < BLOCKED_SPEED_FRACTION * intended_speed_squared
    }
}

/// A character's intended movement: a heading and a multiple of `max_speed`, with no
/// acceleration. Player input and AI steering write it; [`kinematics_to_velocity`] scales it by
/// the character's [`StatMultipliers`] speed and passes it to physics.
#[derive(Component, Clone)]
#[require(FacingDirection, StatMultipliers)]
pub struct Kinematics {
    pub direction: Vec2,
    pub max_speed: f32,
    speed_multiplier: f32,
}

impl Kinematics {
    pub fn new(max_speed: f32) -> Self {
        Self {
            max_speed,
            speed_multiplier: 0.0,
            direction: Vec2::ZERO,
        }
    }

    /// Moves along `direction` at `speed_multiplier` times max speed: below
    /// `1.0` for unhurried movement like wandering, above it for bursts.
    pub fn move_in_direction(&mut self, direction: Vec2, speed_multiplier: f32) {
        self.direction = direction;
        self.speed_multiplier = speed_multiplier.max(0.0);
    }

    /// Stops in place. `direction` keeps the last heading, which steering
    /// reads to continue the way the character was going.
    pub fn stop_moving(&mut self) {
        self.speed_multiplier = 0.0;
    }

    /// Already fully stopped: lets per-frame drivers skip a no-op write that
    /// would mark `Changed<Kinematics>` and defeat downstream filters.
    pub fn is_idle(&self) -> bool {
        self.speed_multiplier == 0.0
    }

    /// Velocity at `stat_speed` times this movement, so a stunned character reports zero. An
    /// over-long `direction` counts as unit length.
    pub fn velocity(&self, stat_speed: f32) -> Vec2 {
        self.direction.clamp_length_max(1.0) * self.max_speed * self.speed_multiplier * stat_speed
    }
}

impl Default for Kinematics {
    fn default() -> Self {
        Self::new(10.0)
    }
}

/// Converts [`Kinematics`], plus any [`KnockedBack`] shove, into avian's `LinearVelocity`.
pub(super) fn kinematics_to_velocity(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &Kinematics,
        &StatMultipliers,
        &mut LinearVelocity,
        Option<&mut KnockedBack>,
    )>,
) {
    for (entity, kinematics, stat_multipliers, mut velocity, knocked_back) in &mut query {
        let mut shove_velocity = Vec2::ZERO;
        if let Some(mut knocked_back) = knocked_back {
            match knocked_back.tick_and_compute_velocity(time.delta()) {
                Some(velocity) => shove_velocity = velocity,
                None => {
                    commands.entity(entity).try_remove::<KnockedBack>();
                }
            }
        }
        velocity.0 = kinematics.velocity(stat_multipliers.speed) + shove_velocity;
    }
}

/// The direction a character faces and aims are not tied to each other in this game.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum FacingDirection {
    Up,
    #[default]
    Down,
    Left,
    Right,
}

impl FacingDirection {
    pub fn turned_toward(self, vec: Vec2) -> Self {
        match vec.try_normalize() {
            Some(v) if v.y > 0.5 => Self::Up,
            Some(v) if v.y < -0.5 => Self::Down,
            Some(v) if v.x > 0.5 => Self::Right,
            Some(v) if v.x < -0.5 => Self::Left,
            _ => self,
        }
    }

    /// The direction whose quarter of the circle `direction` falls in; [`Self::Down`] for zero.
    pub fn from_nearest_axis(direction: Vec2) -> Self {
        if direction.x.abs() > direction.y.abs() {
            if direction.x > 0.0 {
                Self::Right
            } else {
                Self::Left
            }
        } else if direction.y > 0.0 {
            Self::Up
        } else {
            Self::Down
        }
    }

    pub fn to_vec2(self) -> Vec2 {
        match self {
            Self::Up => Vec2::new(0.0, 1.0),
            Self::Down => Vec2::new(0.0, -1.0),
            Self::Left => Vec2::new(-1.0, 0.0),
            Self::Right => Vec2::new(1.0, 0.0),
        }
    }
}

/// Where an entity aims: a direction and how far along it (the cursor for the player, the target
/// for AI). This is decoupled from movement-facing direction.
#[derive(Component)]
pub struct Vision {
    pub aim_direction: Vec2,
    pub aim_distance: f32,
}

impl Default for Vision {
    fn default() -> Self {
        Self {
            aim_direction: Vec2::ZERO,
            // A holder that never aims still fires a sane distance away from itself.
            aim_distance: 100.0,
        }
    }
}

/// Finds a character's allies: the other living characters on its side, gathered on each call.
#[derive(SystemParam)]
pub(super) struct Allies<'w, 's> {
    characters: Query<
        'w,
        's,
        (
            Entity,
            &'static Transform,
            &'static CombatSide,
            &'static Health,
        ),
    >,
}

/// An ally found by [`Allies`], and its distance from the character that asked.
#[derive(Clone, Copy, Debug)]
pub(super) struct Ally {
    pub entity: Entity,
    pub distance: f32,
}

impl Allies<'_, '_> {
    /// The closest ally of `entity`, if any is alive.
    pub(super) fn closest(&self, entity: Entity) -> Option<Ally> {
        self.allies_of(entity)
            .min_by(|ally, other| ally.distance.total_cmp(&other.distance))
    }

    /// The allies of `entity` at most `reach` away, in no particular order.
    pub(super) fn within(&self, entity: Entity, reach: f32) -> impl Iterator<Item = Ally> {
        self.allies_of(entity)
            .filter(move |ally| ally.distance <= reach)
    }

    fn allies_of(&self, entity: Entity) -> impl Iterator<Item = Ally> {
        let caller = self.characters.get(entity).ok();
        caller
            .into_iter()
            .flat_map(move |(_, caller_transform, caller_side, _)| {
                let position = caller_transform.translation.xy();
                self.characters
                    .iter()
                    .filter(move |(ally, _, side, health)| {
                        *ally != entity && side == &caller_side && !health.is_dead()
                    })
                    .map(move |(ally, transform, ..)| Ally {
                        entity: ally,
                        distance: transform.translation.xy().distance(position),
                    })
            })
    }
}

/// Circle collider at the character's feet that collides with the ground and obstacles.
pub fn physical_collider() -> impl Scene {
    bsn! {
        Transform::from_xyz(0.0, CHARACTER_FEET_POS_OFFSET, 0.0)
        ~Collider::circle(10.0)
        ~CollisionLayers::new(
            [GameCollisionLayer::Grounded],
            [
                GameCollisionLayer::Grounded,
                GameCollisionLayer::HighObstacle,
                GameCollisionLayer::LowObstacle,
            ],
        )
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct Purse {
    pub amount: u32,
}

impl Purse {
    pub fn add(&mut self, amount: u32) {
        self.amount += amount;
    }
}
