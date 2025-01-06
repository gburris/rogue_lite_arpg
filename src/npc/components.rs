use crate::components::{Health, Speed};
use avian2d::prelude::{Collider, CollidingEntities};
use bevy::prelude::*;

#[derive(Component)]
#[require(Health, Speed, Collider)]
pub struct NPC;

//Marker component for the circular collider  around an NPC
// that a player must be in to interact with them
#[derive(Component)]
#[require(Collider)]
pub struct NPCInteractionRadius;
