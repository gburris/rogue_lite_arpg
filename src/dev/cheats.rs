//! Dev cheat keys, to set up a fight in any map without restarting:
//!
//! - F2 spawns a ring of enemies around the aim reticle, and Shift+F2 picks the next enemy kind.
//! - F10 toggles [`Invincible`] on the player.
//! - F11 defeats every enemy.

use std::f32::consts::TAU;

use bevy::prelude::*;

use crate::prelude::*;

const ENEMIES_PER_SPAWN: usize = 10;

/// Distance from the reticle to each spawned enemy, so they don't start inside each other.
const SPAWN_RING_RADIUS: f32 = 48.0;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            spawn_enemies_at_reticle,
            toggle_invincible,
            defeat_all_enemies,
        )
            .in_set(InGameSystems::PlayerInput),
    );
}

fn spawn_enemies_at_reticle(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    aim: Single<&GlobalTransform, With<PlayerAim>>,
    mut kind_index: Local<usize>,
) {
    if !keyboard_input.just_pressed(KeyCode::F2) {
        return;
    }
    if keyboard_input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
        *kind_index = (*kind_index + 1) % EnemyKind::ALL.len();
        let kind = EnemyKind::ALL[*kind_index];
        info!("F2 spawns {kind:?}");
        return;
    }
    let center = aim.translation().truncate();
    let kind = EnemyKind::ALL[*kind_index];
    let spawns = (0..ENEMIES_PER_SPAWN)
        .map(|index| {
            let angle = index as f32 / ENEMIES_PER_SPAWN as f32 * TAU;
            EnemySpawn {
                position: center + Vec2::from_angle(angle) * SPAWN_RING_RADIUS,
                kind,
            }
        })
        .collect();
    commands.trigger(SpawnEnemies(spawns));
    info!("spawned {ENEMIES_PER_SPAWN} {kind:?} at {center}");
}

fn toggle_invincible(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player: Single<(Entity, Has<Invincible>), With<Player>>,
) {
    if !keyboard_input.just_pressed(KeyCode::F10) {
        return;
    }
    let (player, invincible) = *player;
    if invincible {
        commands.entity(player).remove::<Invincible>();
    } else {
        commands.entity(player).insert(Invincible);
    }
    info!("player invincible: {}", !invincible);
}

fn defeat_all_enemies(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    enemies: Query<(Entity, &Health), With<Enemy>>,
) {
    if !keyboard_input.just_pressed(KeyCode::F11) {
        return;
    }
    // Damage rather than despawn, so defeat runs as it does in a fight.
    for (enemy, health) in &enemies {
        if health.is_dead() {
            continue;
        }
        commands.trigger(AttemptDamage {
            entity: enemy,
            ignore_invulnerable: true,
            damage: Damage::Single(health.hp),
            ..default()
        });
    }
}
