use bevy::prelude::*;
use std::time::Duration;

use crate::{collision::EnemyCollidesWithPlayer, components::Health, player::Player};

// Component to track invulnerability state and timer
#[derive(Component)]
pub struct InvulnerableFromDamage {
    pub timer: Timer,
    pub flash_timer: Timer,
    pub is_transparent: bool,
}

impl Default for InvulnerableFromDamage {
    fn default() -> Self {
        Self {
            timer: Timer::new(Duration::from_secs(2), TimerMode::Once),
            flash_timer: Timer::new(Duration::from_millis(200), TimerMode::Repeating),
            is_transparent: false,
        }
    }
}

// System to handle enemy collision and damage
pub fn handle_enemy_collision(
    collision_event: Trigger<EnemyCollidesWithPlayer>,
    mut commands: Commands,
    mut player_query: Query<(Entity, &mut Health), With<Player>>,
    player_invulnerability_query: Query<(Entity, &InvulnerableFromDamage)>, // New query to check invulnerability
) {
    warn!("EnemyCollidesWithPlayer Observer triggered");
    if let Ok((player_entity, mut health)) = player_query.get_single_mut() {
        warn!("Player damage system start");

        // Check if player is invulnerable using the helper function
        if !is_player_invulnerable(player_entity, &player_invulnerability_query) {
            // Apply damage only if player is not invulnerable
            health.hp -= collision_event.collision_damage.damage;

            // Add invulnerability component
            commands
                .entity(player_entity)
                .insert(InvulnerableFromDamage::default());

            warn!(
                "Player took {} damage, HP: {}/{}",
                collision_event.collision_damage.damage, health.hp, health.max_hp
            );
        }
    }
}

fn is_player_invulnerable(
    player_entity: Entity,
    player_query: &Query<(Entity, &InvulnerableFromDamage)>,
) -> bool {
    player_query.get(player_entity).is_ok()
}

// System to handle invulnerability duration and flashing
pub fn handle_invulnerability(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut InvulnerableFromDamage, &mut Sprite)>,
) {
    for (entity, mut invulnerable, mut sprite) in query.iter_mut() {
        // Update main invulnerability timer
        invulnerable.timer.tick(time.delta());

        // Update flash timer
        invulnerable.flash_timer.tick(time.delta());

        // Handle flashing
        if invulnerable.flash_timer.just_finished() {
            invulnerable.is_transparent = !invulnerable.is_transparent;
            sprite.color.set_alpha(if invulnerable.is_transparent {
                0.1
            } else {
                1.0
            });
        }

        // Remove invulnerability when timer is finished
        if invulnerable.timer.finished() {
            commands.entity(entity).remove::<InvulnerableFromDamage>();
            sprite.color.set_alpha(1.0); // Ensure sprite is fully visible when done
        }
    }
}
