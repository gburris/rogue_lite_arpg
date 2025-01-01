use crate::components::{Health, Speed};

use avian2d::prelude::Collider;
use bevy::prelude::*;

//favoring #[require] as a default approach is generally recommended.
#[derive(Component)]
#[require(Health, Speed, Collider)]
pub struct Enemy;
