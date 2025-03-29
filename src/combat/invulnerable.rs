use std::time::Duration;

use bevy::prelude::*;

/// Component to mark whether an entity has iframes when hit
#[derive(Component)]
pub struct IFrames {
    // time to be invulnerable when hit
    pub is_invulnerable: bool,
    invulnerable_timer: Timer,
    flash_timer: Option<Timer>,
}

impl Default for IFrames {
    fn default() -> Self {
        Self {
            is_invulnerable: false,
            invulnerable_timer: Timer::new(Duration::from_millis(800), TimerMode::Once),
            flash_timer: Some(Timer::new(Duration::from_millis(100), TimerMode::Repeating)),
        }
    }
}

impl IFrames {
    fn reset(&mut self) {
        self.is_invulnerable = false;
        self.invulnerable_timer.reset();

        if let Some(flash) = &mut self.flash_timer {
            flash.reset();
        };
    }
}

// System to handle invulnerability duration and flashing
pub fn handle_invulnerability(time: Res<Time>, mut query: Query<(&mut IFrames, &mut Sprite)>) {
    for (mut iframes, mut sprite) in query.iter_mut() {
        if iframes.is_invulnerable {
            iframes.invulnerable_timer.tick(time.delta());

            if iframes.invulnerable_timer.finished() {
                iframes.reset();
                sprite.color.set_alpha(1.0);
            } else if let Some(flash) = &mut iframes.flash_timer {
                flash.tick(time.delta());

                if flash.just_finished() {
                    let current_alpha = sprite.color.alpha();
                    sprite.color.set_alpha(1.1 - current_alpha);
                }
            }
        }
    }
}
