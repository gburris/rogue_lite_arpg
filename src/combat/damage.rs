use avian2d::prelude::*;

use bevy::prelude::*;
use rand::Rng;

use crate::{
    combat::{
        health::Health,
        invulnerable::{HasIFrames, Invulnerable},
        status_effects::{components::EffectsList, events::ApplyEffect},
    },
    configuration::GameCollisionLayer,
};

#[derive(PartialEq)]
pub enum DamageSource {
    Player,
    Enemy,
    NPC,
    Environment,
}

impl From<DamageSource> for LayerMask {
    fn from(source: DamageSource) -> Self {
        match source {
            DamageSource::Player => GameCollisionLayer::EnemyHurtBox.to_bits(),
            DamageSource::NPC => GameCollisionLayer::EnemyHurtBox.to_bits(),
            DamageSource::Enemy => GameCollisionLayer::AllyHurtBox.to_bits(),
            DamageSource::Environment => {
                // Environment can affect all characters
                GameCollisionLayer::AllyHurtBox.to_bits()
                    | GameCollisionLayer::EnemyHurtBox.to_bits()
            }
        }
        .into()
    }
}

#[derive(Copy, Clone)]
pub enum Damage {
    Range((f32, f32)),
    Single(f32),
}

impl Damage {
    fn to_float(self) -> f32 {
        match self {
            Damage::Range((min, max)) => rand::thread_rng().gen_range(min..max),
            Damage::Single(amount) => amount,
        }
    }
}

#[derive(Component)]
pub struct HurtBox;

#[derive(Event)]
pub struct AttemptDamageEvent {
    /// We treat damage as a range with RNG determining which value is dealt
    pub damage: Damage,
    /// Not all damage has a "Source" entity, like environmental damage or damage-over-time effects
    pub damage_source: Option<Entity>,
}

/// While AttemptDamageEvent is sent any time a damage source interacts with an entity,
///this event represents when that damage attempt succeeds
#[derive(Event)]
pub struct DamageDealtEvent {
    pub damage: f32,
}

#[derive(Event)]
pub struct DefeatedEvent;

pub fn on_damage_event(
    damage_trigger: Trigger<AttemptDamageEvent>,
    mut commands: Commands,
    mut damaged_query: Query<(&mut Health, Option<&HasIFrames>), Without<Invulnerable>>,
    source_query: Query<&EffectsList>,
) {
    if let Ok((mut health, has_iframes)) = damaged_query.get_mut(damage_trigger.entity()) {
        // Convert `Damage` to raw damage amount
        let damage = damage_trigger.damage.to_float();
        health.take_damage(damage);

        // Because AttemptDamageEvent may not result in damage being applied (invulnerable or entity without health)
        // we send this event for guranteed "X damage has been done". Proper change detection added to bevy would mean this isn't needed
        commands.trigger_targets(DamageDealtEvent { damage }, damage_trigger.entity());

        // Entities have to "opt-in" to having iframes. Right now that is only the player
        if let Some(iframes) = has_iframes {
            commands
                .entity(damage_trigger.entity())
                .insert(Invulnerable::new(iframes));
        }

        if health.hp == 0.0 {
            commands.trigger_targets(DefeatedEvent, damage_trigger.entity());
        } else if let Some(source_entity) = damage_trigger.damage_source {
            // If entity is still alive and damage source exists and has effects list, we apply status effects
            if let Ok(effects_list) = source_query.get(source_entity) {
                commands.trigger_targets(
                    ApplyEffect {
                        effect: effects_list.effects.clone(),
                    },
                    damage_trigger.entity(),
                );
            }
        }
    }
}
