mod aim;
mod death;
mod input;
mod interact;
mod level;
mod movement;
mod overlay;
mod progression;
mod spawn;
mod stats;

pub mod prelude {
    pub use super::aim::PlayerAim;
    pub use super::interact::{InteractionZone, PlayerInteraction, PlayerInteractionRadius};
    pub use super::progression::GameProgress;
    pub use super::{DisplayableStatType, Player, PlayerStats};
}

use avian2d::prelude::*;
use bevy::prelude::*;

pub use self::stats::{DisplayableStatType, PlayerStats};
use crate::character::Character;
use crate::lifespan::despawn_all;
use crate::prelude::*;

/// How much more experience is required (as a multiplier) after each level up.
const PLAYER_LEVEL_REQUIREMENT_MULTIPLIER: f32 = 2.0;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        aim::plugin,
        death::plugin,
        input::plugin,
        interact::plugin,
        level::plugin,
        movement::plugin,
        overlay::plugin,
        progression::plugin,
    ));

    // Player spawn
    app.add_systems(
        OnEnter(AppState::SpawnPlayer),
        (
            spawn::spawn_player,
            overlay::spawn_player_overlay,
            spawn::transition_to_create_hub,
        )
            .chain(),
    );

    // Player despawn
    app.add_observer(despawn_all::<RestartEvent, Player>);
}

#[derive(Component)]
#[require(
    Character,
    CombatSide = CombatSide::Ally,
    ItemCapacity(50),
    Health::new(100.0),
    Kinematics::new(250.0),
    // Double the mass of npcs/enemies so the player can push them around more
    Mass(100.0),
    IFrames,
    Purse
)]
pub struct Player {
    current_level: u32,
    // Outside systems may give the player experience, like when an enemy dies
    pub current_experience: f32,
    next_level_experience_req: f32,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            current_level: 1,
            current_experience: 0.0,
            next_level_experience_req: 20.0,
        }
    }
}

impl Player {
    /// Attempts to increase player level based on current experience and level requirement, and
    /// then sets the new level requirement based on `PLAYER_LEVEL_REQUIREMENT_MULTIPLIER`
    ///
    /// returns whether the player leveled up.
    pub fn attempt_level_up(&mut self) -> bool {
        if self.current_experience >= self.next_level_experience_req {
            self.current_experience -= self.next_level_experience_req;
            self.next_level_experience_req *= PLAYER_LEVEL_REQUIREMENT_MULTIPLIER;
            self.current_level += 1;
            return true;
        }

        false
    }

    pub fn progress_to_next_level(&self) -> f32 {
        self.current_experience / self.next_level_experience_req
    }

    pub fn level(&self) -> u32 {
        self.current_level
    }
}
