use bevy::prelude::*;

use crate::despawn::components::LiveDuration;

#[derive(Component, Default)]
#[require(LiveDuration)]
pub struct Projectile;
