use bevy::prelude::*;

use crate::{
    components::{EffectType, Enemy, Experience, Health, Speed, StatusEffects},
    events::EnemyDefeatedEvent,
};

pub fn handle_status_effects(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut Enemy,
        &mut StatusEffects,
        &mut Health,
        &mut Speed,
        &Transform,
        &Experience,
    )>,
    mut enemy_defeated_events: EventWriter<EnemyDefeatedEvent>,
    asset_server: Res<AssetServer>,
) {
    for (entity, _enemy, mut status, mut health, mut speed, transform, experience) in
        query.iter_mut()
    {
        let mut has_effect = false;

        // Process each active status effect
        status.effects.retain_mut(|effect| {
            effect.duration.tick(time.delta());
            match effect.effect_type {
                EffectType::Burning => {
                    health.hp -= effect.damage_per_second / 60.0;
                    if (health.hp <= 0.0) {
                        if let Some(entity_commands) = commands.get_entity(entity) {
                            commands.entity(entity).despawn();
                            enemy_defeated_events.send(EnemyDefeatedEvent {
                                enemy_entity: entity,
                                enemy_position: transform.translation,
                                exp_value: experience.base_exp,
                            });
                            return false;
                        }
                    }

                    commands
                        .entity(entity)
                        .try_insert(Sprite::from_image(asset_server.load("merman_on_fire.png")));
                }
                EffectType::Slowed => {
                    speed.velocity = speed.velocity * 0.5;
                    commands
                        .entity(entity)
                        .try_insert(Sprite::from_image(asset_server.load("merman_freezing.png")));
                }
                EffectType::Stunned => {
                    speed.velocity = 0.0;
                    // Stunned effect - no movement
                }
            }
            has_effect = true; // Mark that the entity has an active effect
            !effect.duration.finished()
        });

        // If no effects are active, reset speed and sprite
        if !has_effect {
            // Reset speed
            speed.velocity = speed.max_velocity;
            // Reset sprite if it was set
            commands
                .entity(entity)
                .try_insert(Sprite::from_image(asset_server.load("merman.png")));
        }
    }
}
