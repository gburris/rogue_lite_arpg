use bevy::prelude::*;

#[derive(Component, Default)]
pub struct AimPosition {
    pub position: Vec2, // position where entitiy is aiming, for player this is the cursor
}

#[derive(Component, PartialEq, Debug, Hash, Eq, Copy, Clone)]
pub enum ActionState {
    Attacking,
    Defeated, //Death Animation
    Movement,
    Idle,
    Casting,
}
