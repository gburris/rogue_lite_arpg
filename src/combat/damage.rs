use avian2d::prelude::*;

use bevy::{ecs::entity_disabling::Disabled, prelude::*};
use rand::Rng;

use crate::{
    combat::{
        health::Health,
        invulnerable::IFrames,
        status_effects::{EffectOf, Effects, StatusOf},
    },
    configuration::GameCollisionLayer,
};

#[derive(PartialEq, Clone, Copy)]
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
    Single(f32),
    Range((f32, f32)),
}

impl Damage {
    fn to_float(self) -> f32 {
        match self {
            Damage::Range((min, max)) => rand::rng().random_range(min..max),
            Damage::Single(amount) => amount,
        }
    }
}

impl Default for Damage {
    fn default() -> Self {
        Self::Single(0.0)
    }
}

#[derive(Component)]
#[require(Sensor)]
pub struct HurtBox;

pub fn hurtbox(size: Vec2, membership: GameCollisionLayer) -> impl Bundle {
    (
        HurtBox,
        Collider::rectangle(size.x, size.y),
        Transform::from_xyz(0.0, -8.0, 0.0),
        CollisionLayers::new(membership, [GameCollisionLayer::HitBox]),
    )
}

#[derive(EntityEvent)]
pub struct AttemptDamage {
    pub entity: Entity,
    /// Not all damage gets blocked by invulnerable (ex: burn from status effect)
    pub ignore_invulnerable: bool,
    /// We treat damage as a range with RNG determining which value is dealt
    pub damage: Damage,
    /// Not all damage has a "Source" entity, like environmental damage or damage-over-time effects
    pub damage_source: Option<Entity>,
}

/// While AttemptDamageEvent is sent any time a damage source interacts with an entity,
///this event represents when that damage attempt succeeds
#[derive(EntityEvent)]
pub struct DamageDealt {
    pub entity: Entity,
    pub damage: f32,
    pub damage_source: Option<Entity>,
}

/// This is the character holding the weapon that dealt damage
#[derive(Component)]
pub struct Damager(pub Entity);

#[derive(EntityEvent)]
pub struct Defeated {
    pub entity: Entity,
}

pub fn on_damage_event(
    attempt_damage: On<AttemptDamage>,
    mut commands: Commands,
    hurt_box_query: Query<&ChildOf, With<HurtBox>>,
    mut damaged_query: Query<(&mut Health, Option<&mut IFrames>)>,
    source_query: Query<&Effects>,
) {
    // Damage can be applied to an entities hurtbox, or to the entity directly
    let damaged_entity = if let Ok(child_of) = hurt_box_query.get(attempt_damage.entity) {
        child_of.parent()
    } else if damaged_query.contains(attempt_damage.entity) {
        attempt_damage.entity
    } else {
        return;
    };

    if let Ok((mut health, has_iframes)) = damaged_query.get_mut(damaged_entity) {
        // Entities have to "opt-in" to having iframes. Right now that is only the player
        if let Some(mut iframes) = has_iframes {
            if iframes.is_invulnerable && !attempt_damage.ignore_invulnerable {
                return;
            }

            iframes.is_invulnerable = true;
        }

        // Convert `Damage` to raw damage amount
        let damage = attempt_damage.damage.to_float();
        health.take_damage(damage);

        // Because AttemptDamageEvent may not result in damage being applied (invulnerable or entity without health)
        // we send this event for guranteed "X damage has been done". Proper change detection added to bevy would mean this isn't needed
        commands.trigger(DamageDealt {
            entity: damaged_entity,
            damage,
            damage_source: attempt_damage.damage_source,
        });

        if health.hp == 0.0 {
            commands.trigger(Defeated {
                entity: damaged_entity,
            });
        } else if let Some(source_entity) = attempt_damage.damage_source {
            // If entity is still alive and damage source exists and has effects list, we apply status effects
            if let Ok(effects) = source_query.get(source_entity) {
                trace!("Applying effects: {:?}", effects);
                effects.iter().for_each(|e| {
                    commands
                        .entity(e)
                        .clone_and_spawn()
                        .remove::<(Disabled, EffectOf)>()
                        .insert(StatusOf(damaged_entity));
                });
            }
        }
    }
}
