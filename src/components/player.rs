use crate::components::{Health, Speed};
use avian2d::prelude::Collider;
use bevy::prelude::*;

#[derive(Component)]
#[require(Health, Speed, Collider)]
pub struct Player;
