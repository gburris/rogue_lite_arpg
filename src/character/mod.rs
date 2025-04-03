use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    ai::state::ActionState, animation::AnimationTimer, configuration::YSort,
    player::player_data::CHARACTER_FEET_POS_OFFSET,
};

#[derive(Component, Default)]
#[require(RigidBody,
    LockedAxes(|| LockedAxes::new().lock_rotation()),
    TransformInterpolation,
    // Set stable mass for characters so speed can be compared numerically
    Mass(|| Mass(50.0)),
    NoAutoMass,
    ActionState,
    AnimationTimer,
    YSort(|| YSort::from_offset(CHARACTER_FEET_POS_OFFSET)))]
pub struct Character;
