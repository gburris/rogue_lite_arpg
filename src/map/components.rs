use avian2d::prelude::Collider;
use bevy::prelude::*;

/**
 *
 */
#[derive(Component)]
#[require(Collider)]

pub enum Portal {
    StartingPortal,
    WarpZone,
}
