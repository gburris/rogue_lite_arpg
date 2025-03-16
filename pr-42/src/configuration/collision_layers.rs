use avian2d::prelude::PhysicsLayer;

#[derive(PhysicsLayer, Default)]
pub enum GameCollisionLayer {
    #[default]
    Default, // Layer 0 - the default layer that objects are assigned to
    Player, // Marker for player, used for interactions and other player-specific stuff
    Enemy,
    Magnet,       // Used to "Magnet" to entities colliding with it
    LowObstacle, // Obstacle that stops ground movement but lets things "fly" over, like projectiles
    HighObstacle, // Obstacle that stops all movement
    Grounded,    // Marks entities that get stopped by all obstacles
    InAir,       // Marks entity as able to go over low obstacle (projectile, )
    Interaction, // Used for the player being in an interaction radius
}

impl GameCollisionLayer {
    pub const LOW_OBSTACLE_FILTERS: [GameCollisionLayer; 1] = [Self::Grounded];

    pub const HIGH_OBSTACLE_FILTERS: [GameCollisionLayer; 2] = [Self::Grounded, Self::InAir];
}
