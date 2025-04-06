use avian2d::prelude::*;

use bevy::prelude::*;
use rand::Rng;

use crate::{
    combat::{
        health::Health,
        invulnerable::IFrames,
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
    /// Not all damage gets blocked by invulnerable (ex: burn from status effect)
    pub ignore_invulnerable: bool,
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
    hurt_box_query: Query<&ChildOf, With<HurtBox>>,
    mut damaged_query: Query<(&mut Health, Option<&mut IFrames>)>,
    source_query: Query<&EffectsList>,
) {
    // Damage can be applied to an entities hurtbox, or to the entity directly
    let damaged_entity = if let Ok(child_of) = hurt_box_query.get(damage_trigger.target()) {
        child_of.parent
    } else if damaged_query.contains(damage_trigger.target()) {
        damage_trigger.target()
    } else {
        return;
    };

    if let Ok((mut health, has_iframes)) = damaged_query.get_mut(damaged_entity) {
        // Entities have to "opt-in" to having iframes. Right now that is only the player
        if let Some(mut iframes) = has_iframes {
            if iframes.is_invulnerable && !damage_trigger.ignore_invulnerable {
                return;
            }

            iframes.is_invulnerable = true;
        }

        // Convert `Damage` to raw damage amount
        let damage = damage_trigger.damage.to_float();
        health.take_damage(damage);

        // Because AttemptDamageEvent may not result in damage being applied (invulnerable or entity without health)
        // we send this event for guranteed "X damage has been done". Proper change detection added to bevy would mean this isn't needed
        commands.trigger_targets(DamageDealtEvent { damage }, damaged_entity);

        if health.hp == 0.0 {
            commands.trigger_targets(DefeatedEvent, damaged_entity);
        } else if let Some(source_entity) = damage_trigger.damage_source {
            // If entity is still alive and damage source exists and has effects list, we apply status effects
            if let Ok(effects_list) = source_query.get(source_entity) {
                commands.trigger_targets(
                    ApplyEffect {
                        effect: effects_list.effects.clone(),
                    },
                    damaged_entity,
                );
            }
        }
    }
}
