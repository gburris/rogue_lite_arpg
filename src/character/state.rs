use avian2d::prelude::LinearVelocity;
use bevy::prelude::*;

use crate::prelude::{EquipmentInUse, EquipmentSlot, Mainhand, Offhand};

/// A character's intended movement: a heading and a multiple of `max_speed`,
/// reduced by any slow, with no acceleration. Player input and AI steering
/// write it; [`kinematics_to_velocity`] hands it to physics.
#[derive(Component, Clone)]
#[require(FacingDirection, AttackState)]
pub struct Kinematics {
    pub direction: Vec2,
    pub max_speed: f32,
    speed_multiplier: f32,
    slowed_percentage: f32,
}

impl Kinematics {
    pub fn new(max_speed: f32) -> Self {
        Kinematics {
            max_speed,
            speed_multiplier: 0.0,
            direction: Vec2::ZERO,
            slowed_percentage: 0.0,
        }
    }

    /// Sets the fraction of speed removed, where `1.0` is a stun.
    pub fn slow(&mut self, percentage: f32) {
        assert!(
            (0.0..=1.0).contains(&percentage),
            "slow percentage is a fraction of speed removed, got {percentage}"
        );

        self.slowed_percentage = percentage;
    }

    pub fn slowed_percentage(&self) -> f32 {
        self.slowed_percentage
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

    pub fn is_stunned(&self) -> bool {
        self.slowed_percentage >= 1.0
    }

    /// Fraction of movement speed retained after the current slow: `1.0` is
    /// full speed and `0.0` is stunned.
    pub fn speed_scale(&self) -> f32 {
        (1.0 - self.slowed_percentage).clamp(0.0, 1.0)
    }

    pub fn is_moving(&self) -> bool {
        self.speed_multiplier > 0.0 && !self.is_stunned()
    }

    /// Already fully stopped: lets per-frame drivers skip a no-op write that
    /// would mark `Changed<Kinematics>` and defeat downstream filters.
    pub fn is_idle(&self) -> bool {
        self.speed_multiplier == 0.0
    }

    /// Velocity after any slow, so a stunned character reports zero. An
    /// over-long `direction` counts as unit length.
    pub fn velocity(&self) -> Vec2 {
        self.direction.clamp_length_max(1.0)
            * self.max_speed
            * self.speed_multiplier
            * self.speed_scale()
    }
}

impl Default for Kinematics {
    fn default() -> Self {
        Kinematics::new(10.0)
    }
}

/// Converts [`Kinematics`] into avian's `LinearVelocity`.
pub(super) fn kinematics_to_velocity(mut query: Query<(&Kinematics, &mut LinearVelocity)>) {
    for (kinematics, mut velocity) in &mut query {
        velocity.0 = kinematics.velocity();
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
            FacingDirection::Up => Vec2::new(0.0, 1.0),
            FacingDirection::Down => Vec2::new(0.0, -1.0),
            FacingDirection::Left => Vec2::new(-1.0, 0.0),
            FacingDirection::Right => Vec2::new(1.0, 0.0),
        }
    }
}

/// Animation/physics flag mirroring whether an instant-use item is active.
/// Written only by `sync_attack_state_from_equipment`.
#[derive(Component, Clone, Copy, Default, PartialEq)]
pub struct AttackState {
    pub is_attacking: bool,
}

/// Syncs whether a holder is attacking from `EquipmentInUse` items. Held slots
/// (shield blocking) do not count as attacking.
pub(super) fn sync_attack_state_from_equipment(
    mut holders: Query<(&mut AttackState, Option<&Mainhand>, Option<&Offhand>)>,
    in_use: Query<(), With<EquipmentInUse>>,
) {
    for (mut state, mainhand, offhand) in &mut holders {
        let attacking = EquipmentSlot::ALL
            .into_iter()
            .filter(|equipment| equipment.is_instant_use())
            .filter_map(|equipment| equipment.equipped(mainhand, offhand))
            .any(|item| in_use.contains(item));
        state.set_if_neq(AttackState {
            is_attacking: attacking,
        });
    }
}
