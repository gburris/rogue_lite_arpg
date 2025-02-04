use bevy::prelude::*;

#[derive(Component, Default)]
pub struct AimPosition {
    pub position: Vec2, // position where entitiy is aiming, for player this is the cursor
}

//This is used to decide if we should update the item transform to keep up with the player moving
//Or leave it alone during an attack animation
#[derive(Component, Debug, Hash, Eq, PartialEq, Clone)]
pub enum ActionState {
    Attacking, //Sword is swinging
    None,      //Moving
}
