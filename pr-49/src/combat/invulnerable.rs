use std::time::Duration;

use bevy::prelude::*;

/// Component to mark whether an entity has iframes when hit
#[derive(Component)]
pub struct HasIFrames {
    // time to be invulnerable when hit
    pub duration: Duration,
}

// Component to track invulnerability state and timer
#[derive(Component)]
pub struct Invulnerable {
    pub total_time: Timer,
    pub flash_timer: Timer,
}

impl Invulnerable {
    pub fn new(iframes: &HasIFrames) -> Self {
        Self {
            total_time: Timer::new(iframes.duration, TimerMode::Once),
            ..default()
        }
    }

    pub fn death() -> Self {
        Self {
            total_time: Timer::new(Duration::from_secs(4), TimerMode::Once),
            flash_timer: Timer::new(Duration::from_millis(5000), TimerMode::Repeating), //Don't flash
        }
    }
}

impl Default for Invulnerable {
    fn default() -> Self {
        Self {
            total_time: Timer::new(Duration::from_secs(2), TimerMode::Once),
            flash_timer: Timer::new(Duration::from_millis(200), TimerMode::Repeating),
        }
    }
}

// System to handle invulnerability duration and flashing
pub fn handle_invulnerability(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Invulnerable, &mut Sprite)>,
) {
    for (entity, mut invulnerable, mut sprite) in query.iter_mut() {
        invulnerable.total_time.tick(time.delta());
        invulnerable.flash_timer.tick(time.delta());

        //  Alternate sprite alpha between 1.0 and 0.1 on flash timer interval
        if invulnerable.flash_timer.just_finished() {
            let current_alpha = sprite.color.alpha();
            sprite.color.set_alpha(1.1 - current_alpha);
        }

        // Remove invulnerability when timer is finished and ensure sprite is visible
        if invulnerable.total_time.finished() {
            sprite.color.set_alpha(1.0);
            commands.entity(entity).remove::<Invulnerable>();
        }
    }
}
