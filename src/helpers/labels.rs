use avian2d::prelude::PhysicsLayer;

#[derive(PhysicsLayer, Default)]
pub enum GameCollisionLayer {
    #[default]
    Default, // Layer 0 - the default layer that objects are assigned to
    Wall,
    Player,
    Enemy,
    Projectile,
    Portal,
    Npc,
    Interaction, //Used for the player being in an interaction radius
}
