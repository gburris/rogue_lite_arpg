use avian2d::prelude::*;
use bevy::prelude::*;

pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        // setup avian physics (used for forces, collision, etc...)
        // length unit here represents "pixels per meter" and is a way to indicate the
        // scale of your world to the physics engine for performance optimizations
        // In this case, our tiles are currently 32 x 32 pixels so we set the scale accordingly
        app.add_plugins(PhysicsPlugins::default().with_length_unit(32.0))
            .insert_resource(Gravity::ZERO); // no gravity since this is top-down game
    }
}

#[derive(PhysicsLayer, Default)]
pub enum GameCollisionLayer {
    #[default]
    Default, // Layer 0 - the default layer that objects are assigned to

    // For dealing and taking damage
    HitBox,
    AllyHurtBox,
    EnemyHurtBox,

    // For physical collisions
    LowObstacle, // Obstacle that stops ground movement but lets things "fly" over, like projectiles
    HighObstacle, // Obstacle that stops all movement
    Grounded,    // Marks entities that get stopped by all obstacles
    InAir,       // Marks entity as able to go over low obstacle (projectile, )
    PlayerCollider,
    EnemyCollider,
    NPCCollider,

    /// Things that interact with player but don't physically collide (NPC dialogue, magnets, etc...)
    PlayerInteractionRadius,
    Interaction,
}

impl GameCollisionLayer {
    pub const PROJECTILE_MEMBERSHIPS: [GameCollisionLayer; 2] = [Self::HitBox, Self::InAir];
    pub const LOW_OBSTACLE_FILTERS: [GameCollisionLayer; 1] = [Self::Grounded];
    pub const HIGH_OBSTACLE_FILTERS: [GameCollisionLayer; 2] = [Self::Grounded, Self::InAir];
}
