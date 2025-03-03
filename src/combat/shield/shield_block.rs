use crate::items::Shield;
use bevy::prelude::*;

pub fn start_shield_block(
    commands: &mut Commands,
    shield_entity: Entity,
    shield: &Shield,
    block_angle: f32,
) {
    warn!("blocking started");
    //TODO: Transform the shield
    //Use on of the four sprites based on aimposition
    //Add a hitbox to it
    //If a melee attack collides with the shield, destroy it?
}
