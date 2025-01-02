use crate::components::{Health, Speed};

use avian2d::prelude::Collider;
use bevy::prelude::*;

//favoring #[require] as a default approach is generally recommended.
#[derive(Component)]
#[require(Health, Speed, Collider, Experience)]
pub struct Enemy;

//Experience granted by the enemy when player defeats it
#[derive(Component)]
pub struct Experience {
    pub base_exp: u32,
}

impl Default for Experience {
    fn default() -> Self {
        Experience { base_exp: 10 }
    }
}
