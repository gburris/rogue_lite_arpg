use bevy::prelude::*;
use rand::RngExt;

use crate::prelude::*;

/// Names an [`ActionSource`] of a character: the item equipped in a slot, or one of its abilities
/// by kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ActionKey {
    Equipped(EquipmentSlot),
    Ability(Ability),
}

impl ActionKey {
    /// Every key, equipment first.
    pub fn all() -> impl Iterator<Item = Self> {
        let equipment = EquipmentSlot::ALL.into_iter().map(Self::Equipped);
        equipment.chain(Ability::ALL.into_iter().map(Self::Ability))
    }
}

/// The farthest target distance at which an AI holder uses this action source.
#[derive(Component, Clone, Copy, Debug, Deref)]
pub struct ActionReach(pub f32);

/// Marks an action source whose use is an attack, like a swing, a cast or a bite.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Attack;

/// A kind of item the holder must carry to use this action source, such as a heal's potion.
#[derive(Component, Clone, Copy, Debug, Deref)]
pub struct RequiredItem(pub ItemKind);

/// Seconds an action source waits after a use interrupted during its wind-up before it can be used
/// again.
const INTERRUPTED_LOCKOUT_SECONDS: f32 = 1.5;

/// How long an action source waits after a use takes effect before it can be used again.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct ActionCooldown {
    seconds: f32,
    jitter: f32,
    ready_at: f32,
}

impl ActionCooldown {
    /// Waits `seconds` after each use takes effect.
    pub fn new(seconds: f32) -> Self {
        Self {
            seconds,
            ..default()
        }
    }

    /// Varies each wait by up to `jitter` seconds either way, so characters that start using the
    /// same thing together drift out of step.
    pub fn with_jitter(self, jitter: f32) -> Self {
        Self { jitter, ..self }
    }

    /// Seconds left at virtual time `now` before the next use can start.
    pub fn remaining_seconds(&self, now: f32) -> f32 {
        (self.ready_at - now).max(0.0)
    }

    pub(super) fn is_ready(&self, now: f32) -> bool {
        now >= self.ready_at
    }

    pub(super) fn start(&mut self, now: f32) {
        let offset = if self.jitter > 0.0 {
            rand::rng().random_range(-self.jitter..=self.jitter)
        } else {
            0.0
        };
        self.ready_at = now + (self.seconds + offset).max(0.0);
    }

    pub(super) fn start_interrupted_lockout(&mut self, now: f32) {
        self.ready_at = self.ready_at.max(now + INTERRUPTED_LOCKOUT_SECONDS);
    }
}

/// How a use ends once it takes effect.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub enum ActionUseStyle {
    /// Ends the moment it takes effect, like a heal or a tome's spell.
    #[default]
    Instant,
    /// Plays out for `seconds` after it takes effect, then ends, like a sword swing or a staff
    /// cast.
    Timed { seconds: f32 },
    /// Lasts until it is stopped, like a shield block.
    Sustained,
}

impl ActionUseStyle {
    /// Whether a use in this style ends without being stopped.
    pub fn ends_on_its_own(self) -> bool {
        match self {
            Self::Instant | Self::Timed { .. } => true,
            Self::Sustained => false,
        }
    }
}

/// How long each use winds up before it takes effect, a window for a hit to interrupt it. Without
/// it, a use takes effect on the frame it starts.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct ActionWindup {
    pub seconds: f32,
}

/// Lights the holder in this color while a use winds up, a stand-in until uses have animations.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct ActionGlow(pub Color);
