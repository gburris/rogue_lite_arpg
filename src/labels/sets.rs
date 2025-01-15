use bevy::prelude::*;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum MainSet {
    GamePlay,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum GamePlaySet {
    DespawnEntities, // Despawn entities only! MUST happen before simulation of this new frame we are in!
    PlayerInput,
    UI,
    Simulation, // Most game logic (queries modifying components)
    Physics,    // Apply forces using rapier based on simulation
    Collision,  // Finally detect collisions using avian based on velocity changed
}
