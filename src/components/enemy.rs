use crate::components::{Collider, Health, Speed};

use bevy::prelude::*;

//favoring #[require] as a default approach is generally recommended.
#[derive(Component)]
#[require(Health)]
#[require(Collider)]
#[require(Speed)]
pub struct Enemy;
