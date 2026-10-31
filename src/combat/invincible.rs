use bevy::prelude::*;

/// Takes no damage from any source, including damage that ignores invulnerability.
/// Used by the stress arena so a fight can run as long as a capture needs.
#[derive(Component, Clone, Copy, Debug)]
pub struct Invincible;
