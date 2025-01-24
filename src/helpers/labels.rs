use avian2d::prelude::PhysicsLayer;

#[derive(PhysicsLayer, Default)]
pub enum GameCollisionLayer {
    #[default]
    Default, // Layer 0 - the default layer that objects are assigned to
    Wall,
    Water, //Water is honestly a bit diff than wall
    Player,
    Chest,
    Enemy,
    Projectile,
    Portal,
    Npc,
    Interaction, //Used for the player being in an interaction radius
}
