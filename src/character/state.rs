use avian2d::prelude::LinearVelocity;
use bevy::prelude::*;

use crate::prelude::*;

/// Fraction of its intended speed below which a character counts as blocked.
const BLOCKED_SPEED_FRACTION: f32 = 0.3;

/// A character's intended movement: a heading and a multiple of `max_speed`, with no
/// acceleration. Player input and AI steering write it; [`kinematics_to_velocity`] scales it by
/// the character's [`StatMultipliers`] speed and hands it to physics.
#[derive(Component, Clone)]
#[require(FacingDirection, Attacking, StatMultipliers)]
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

    /// Whether `physics_velocity` falls well short of the velocity intended at `stat_speed`: the
    /// character is pressed against a wall or a crowd. Never true while it stands still or is
    /// stunned.
    pub fn is_blocked_at(&self, physics_velocity: Vec2, stat_speed: f32) -> bool {
        let intended_velocity = self.velocity(stat_speed);
        let intended_speed_squared = intended_velocity.length_squared();
        intended_speed_squared > 1.0
            && physics_velocity.dot(intended_velocity)
                < BLOCKED_SPEED_FRACTION * intended_speed_squared
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

/// Converts [`Kinematics`] into avian's `LinearVelocity`.
pub(super) fn kinematics_to_velocity(
    mut query: Query<(&Kinematics, &StatMultipliers, &mut LinearVelocity)>,
) {
    for (kinematics, stat_multipliers, mut velocity) in &mut query {
        velocity.0 = kinematics.velocity(stat_multipliers.speed);
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
        match vec.normalize() {
            v if v.y > 0.5 => Self::Up,
            v if v.y < -0.5 => Self::Down,
            v if v.x > 0.5 => Self::Right,
            v if v.x < -0.5 => Self::Left,
            _ => self,
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

/// Animation/physics flag mirroring whether an instant-use item is active.
/// Written only by `sync_attacking_from_equipment`.
#[derive(Component, Clone, Copy, Default, PartialEq, Deref, DerefMut)]
pub struct Attacking(pub bool);

/// Syncs whether a holder is attacking; see [`EquipmentSlot::is_instant_item_in_use`].
pub(super) fn sync_attacking_from_equipment(
    mut holders: Query<(&mut Attacking, Option<&Mainhand>, Option<&Offhand>)>,
    in_use: Query<(), With<EquipmentInUse>>,
) {
    for (mut attacking, mainhand, offhand) in &mut holders {
        let using_instant_item = EquipmentSlot::is_instant_item_in_use(mainhand, offhand, &in_use);
        attacking.set_if_neq(Attacking(using_instant_item));
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
