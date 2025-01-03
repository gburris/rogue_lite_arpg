use bevy::prelude::*;

use crate::{
    components::{EffectType, Enemy, Experience, Health, Speed, StatusEffects},
    enemy::EnemyDamageEvent,
    resources::assets::SpriteAssets,
};

pub fn handle_status_effects(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut Enemy,
        &mut StatusEffects,
        &mut Speed,
    )>,
    mut enemy_damaged_events: EventWriter<EnemyDamageEvent>,
    sprites: Res<SpriteAssets>,
) {
    for (entity, _enemy, mut status, mut speed) in
        query.iter_mut()
    {
        let mut has_effect = false;

        // Process each active status effect
        status.effects.retain_mut(|effect| {
            effect.duration.tick(time.delta());
            match effect.effect_type {
                EffectType::Burning => {
                    enemy_damaged_events.send(EnemyDamageEvent {
                        enemy_entity: entity,
                        damage_source: None,
                        damage: effect.damage_per_second / 60.0, //This assumes 60 FPS and ticks too "quickly", make it tick every 1/2 second instead
                    });
                    commands
                        .entity(entity)
                        .try_insert(Sprite::from_image(sprites.merman_on_fire.clone()));
                }
                EffectType::Slowed => {
                    speed.velocity = speed.velocity * 0.5;
                    commands
                        .entity(entity)
                        .try_insert(Sprite::from_image(sprites.merman_freezing.clone()));
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
                .try_insert(Sprite::from_image(sprites.merman_enemy.clone()));
        }
    }
}
