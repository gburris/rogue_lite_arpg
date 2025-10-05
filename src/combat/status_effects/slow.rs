use bevy::prelude::*;

use crate::{
    combat::status_effects::{Status, StatusApplied, StatusOf},
    prelude::SimpleMotion,
};

#[derive(Component, Clone)]
#[require(Status)]
pub struct Slowed {
    pub percent: f32,
}

impl Default for Slowed {
    fn default() -> Self {
        Slowed { percent: 0.5 }
    }
}

pub fn apply_slowed(
    mut commands: Commands,
    status_query: Query<(Entity, &StatusOf, &Slowed), Without<StatusApplied>>,
    mut motion_query: Query<&mut SimpleMotion>,
) {
    status_query.iter().for_each(|(status, status_of, slowed)| {
        commands.entity(status).insert(StatusApplied);

        if let Ok(mut motion) = motion_query.get_mut(status_of.0) {
            motion.slow(slowed.percent);
        }
    });
}

pub fn on_slow_removed(
    trigger: On<Remove, Slowed>,
    status_query: Query<&StatusOf, With<Slowed>>,
    mut motion_query: Query<&mut SimpleMotion>,
) {
    let Ok(status_of) = status_query.get(trigger.target()) else {
        return;
    };

    if let Ok(mut motion) = motion_query.get_mut(status_of.0) {
        motion.remove_debuff();
    }
}
