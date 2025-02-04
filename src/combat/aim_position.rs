use bevy::prelude::*;

#[derive(Component, Default)]
pub struct AimPosition {
    pub position: Vec2, // position where entitiy is aiming, for player this is the cursor
}
