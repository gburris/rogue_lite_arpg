use avian2d::prelude::Collider;
use bevy::prelude::*;

#[derive(Component)]
#[require(Collider)]
pub struct RunStartPortal;
