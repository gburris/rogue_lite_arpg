use crate::components::{Collider, Health, Speed};
use bevy::prelude::*;

#[derive(Component)]
#[require(Health)]
#[require(Collider)]
#[require(Speed)]
pub struct Player;
