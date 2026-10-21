use avian2d::prelude::LinearVelocity;
use bevy::prelude::*;

use crate::prelude::{EquipmentInUse, EquipmentSlot, Mainhand, Offhand};

/// Simple motion has no acceleration and assumes all entities move at max speed unless altered by `slowed_percentage`
/// by Movement
#[derive(Component, Clone)]
#[require(FacingDirection, AttackState)]
pub struct SimpleMotion {
    pub direction: Vec2,
    pub max_speed: f32,
    current_speed: f32,
    /// Applied on top of `max_speed`, `slowed_percentage` of 1.0 represents being "stunned"
    slowed_percentage: f32,
}

impl SimpleMotion {
    pub fn new(max_speed: f32) -> Self {
        SimpleMotion {
            max_speed,
            current_speed: 0.0,
            direction: Vec2::ZERO,
            slowed_percentage: 0.0,
        }
    }

    pub fn stun(&mut self) {
        self.slowed_percentage = 1.0;
    }

    pub fn slow(&mut self, percentage: f32) {
        assert!((0.0..=1.0).contains(&percentage));

        self.slowed_percentage = percentage;
    }

    pub fn remove_debuff(&mut self) {
        self.slowed_percentage = 0.0;
    }

    pub fn start_moving(&mut self, direction: Vec2) {
        self.current_speed = self.max_speed;
        self.direction = direction;
    }
    /// Starts moving at a fraction of max speed, for gaits like wandering
    /// that should read as unhurried next to full-speed combat movement.
    pub fn start_moving_scaled(&mut self, direction: Vec2, scale: f32) {
        self.current_speed = self.max_speed * scale.clamp(0.0, 1.0);
        self.direction = direction;
    }

    pub fn stop_moving(&mut self) {
        self.current_speed = 0.0;
        self.direction = Vec2::ZERO;
    }

    pub fn is_stunned(&self) -> bool {
        self.slowed_percentage >= 1.0
    }

    pub fn is_moving(&self) -> bool {
        self.current_speed > 0.0 && !self.is_stunned()
    }

    pub fn get_velocity(&self) -> Vec2 {
        (self.direction * self.current_speed).clamp_length_max(self.max_speed)
    }
}

impl Default for SimpleMotion {
    fn default() -> Self {
        SimpleMotion::new(10.0)
    }
}

/// Converts simulation motion into physics "real" motion (using avian linear velocity)
pub(super) fn motion_to_velocity(mut query: Query<(&SimpleMotion, &mut LinearVelocity)>) {
    for (motion, mut velocity) in &mut query {
        if motion.is_moving() {
            let temp_vel = motion.get_velocity();
            velocity.x = temp_vel.x;
            velocity.y = temp_vel.y;
        } else {
            velocity.x = 0.0;
            velocity.y = 0.0;
        }
    }
}

/// The direction a character faces and aims are not tied to each other in this game
#[derive(Component, Default, Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub enum FacingDirection {
    Up,
    #[default]
    Down,
    Left,
    Right,
}

impl FacingDirection {
    pub fn from_vec2(&self, vec: Vec2) -> Self {
        match vec.normalize() {
            v if v.y > 0.5 => Self::Up,
            v if v.y < -0.5 => Self::Down,
            v if v.x > 0.5 => Self::Right,
            v if v.x < -0.5 => Self::Left,
            _ => *self,
        }
    }

    pub fn to_vec2(&self) -> Vec2 {
        match self {
            FacingDirection::Up => Vec2::new(0.0, 1.0),
            FacingDirection::Down => Vec2::new(0.0, -1.0),
            FacingDirection::Left => Vec2::new(-1.0, 0.0),
            FacingDirection::Right => Vec2::new(1.0, 0.0),
        }
    }
}

/// Animation/physics flag mirroring whether an instant-use item is active.
/// Never written by AI; only synced from `EquipmentInUse` by
/// `sync_attack_state_from_equipment`.
#[derive(Component, Clone, Default, PartialEq)]
pub struct AttackState {
    pub is_attacking: bool,
}

/// Syncs whether a holder is attacking from `EquipmentInUse` items. Items decide
/// whether something started; AI only requests use and never writes this.
/// Held slots (shield blocking) do not count as attacking for animation.
pub(super) fn sync_attack_state_from_equipment(
    mut holders: Query<(&mut AttackState, Option<&Mainhand>, Option<&Offhand>)>,
    in_use: Query<(), With<EquipmentInUse>>,
) {
    for (mut state, mainhand, offhand) in &mut holders {
        let attacking = EquipmentSlot::ALL
            .into_iter()
            .filter(|equipment| equipment.is_instant_use())
            .filter_map(|equipment| equipment.get_equipped(mainhand, offhand))
            .any(|item| in_use.contains(item));
        state.set_if_neq(AttackState {
            is_attacking: attacking,
        });
    }
}
