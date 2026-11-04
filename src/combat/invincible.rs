use bevy::prelude::*;

/// Takes no damage from any source, including damage that ignores invulnerability.
/// Used by the stress arena, so a fight lasts as long as a capture needs, and by the F10 dev key.
#[derive(Component, Clone, Copy, Debug)]
pub struct Invincible;
