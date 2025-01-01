use bevy::prelude::*;

#[derive(Component)]
pub struct Collider {
    pub size: Vec2,
}

impl Default for Collider {
    fn default() -> Self {
        Collider { size: Vec2::new(0., 0.) }
    }
}
