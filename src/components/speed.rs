use bevy::prelude::*;

#[derive(Component)]
pub struct Speed {
    pub velocity: f32,
    pub max_velocity: f32,
    pub acceleration: f32,
}

impl Default for Speed {
    fn default() -> Self {
        Speed {
            velocity: 10.0,
            max_velocity: 10.0, //Not used but I wanna use em for movement
            acceleration: 1.0,  //Not used but I wanna use em for movement
        }
    }
}
