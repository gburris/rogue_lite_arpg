use bevy::prelude::*;
use bevy_behave::prelude::BehaveCtx;

use super::Character;

/// Represents the world coordinate where an entitiy is aiming, for player this is the cursor
#[derive(Component)]
pub struct Vision {
    pub aim_direction: Vec2,
}

impl Default for Vision {
    fn default() -> Self {
        Self {
            aim_direction: Vec2::ZERO,
        }
    }
}

#[derive(Component)]
pub struct Agro {
    pub target: Option<Entity>,
    pub line_of_sight: bool,
    pub target_lock_timer: Option<Timer>,
    lock_duration: f32,
}

impl Default for Agro {
    fn default() -> Self {
        Self {
            target: None,
            line_of_sight: false,
            target_lock_timer: None,
            // enemies chase for 6 seconds when damaged by default
            lock_duration: 6.0,
        }
    }
}

impl Agro {
    pub fn lock_target(&mut self, target: Entity) {
        self.target_lock_timer = Some(Timer::from_seconds(self.lock_duration, TimerMode::Once));
        self.target = Some(target);
    }

    pub fn has_target(&self) -> bool {
        self.target.is_some()
    }
}

/// Add this to a behavior if receiving an agro target can interrupt the behavior
/// Example: Wandering or idling will end if an agro target is acquired
#[derive(Component, Clone, Default)]
pub struct AgroInterrupts;

pub fn check_for_target_interrupt(
    mut commands: Commands,
    behavior_query: Query<&BehaveCtx, With<AgroInterrupts>>,
    target_query: Query<Option<&Agro>, With<Character>>,
) {
    behavior_query.iter().for_each(|ctx| {
        let agro = target_query.get(ctx.target_entity()).unwrap();

        if agro.is_some_and(|a| a.has_target()) {
            info!("Interrupting, target found");
            commands.trigger(ctx.failure());
        }
    });
}
