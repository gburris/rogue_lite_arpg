use bevy::prelude::*;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum MainSet {
    InGame,
    Menu,
    Shared,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum InGameSet {
    DespawnEntities, // Despawn entities only! MUST happen before simulation of this new frame we are in!
    PlayerInput,
    Simulation, // Most game logic (queries modifying components)
    HudOverlay, // Render UI overlay based on simulation
    Physics,    // Apply forces/velocity using avian based on simulation
    Collision,  // Finally detect collisions using avian based on velocity changed
}
