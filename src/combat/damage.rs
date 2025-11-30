use avian2d::prelude::*;

use bevy::prelude::*;
use rand::Rng;

use crate::{
    combat::{
        health::Health,
        invulnerable::IFrames,
        status_effects::{ApplyEffects, Effects},
    },
    prelude::{GameCollisionLayer, Player},
};

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum DamageSource {
    Player,
    Enemy,
    NPC,
    Environment,
}

impl From<bool> for DamageSource {
    fn from(is_enemy: bool) -> Self {
        if is_enemy {
            DamageSource::Enemy
        } else {
            DamageSource::Player
        }
    }
}

impl From<DamageSource> for LayerMask {
    fn from(source: DamageSource) -> Self {
        match source {
            DamageSource::Player | DamageSource::NPC => GameCollisionLayer::EnemyHurtBox.to_bits(),
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
    /// damage direction, ex. velocity direction of projectile or character position for melee
    pub direction: Option<Vec2>,
}

impl Default for AttemptDamage {
    fn default() -> Self {
        Self {
            entity: Entity::PLACEHOLDER,
            ignore_invulnerable: false,
            damage: Damage::Single(1.0),
            damage_source: None,
            direction: None,
        }
    }
}

/// While `AttemptDamageEvent` is sent any time a damage source interacts with an entity,
///this event represents when that damage attempt succeeds
#[derive(EntityEvent)]
pub struct DamageDealt {
    pub entity: Entity,
    pub damage: f32,
    pub damage_source: Option<Entity>,
    /// damage direction, ex. velocity direction of projectile or character position for melee
    pub direction: Option<Vec2>,
}

#[derive(EntityEvent)]
pub struct Defeated {
    pub entity: Entity,
}

pub(super) fn on_damage_event(
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

        if health.is_dead() {
            return;
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
            direction: attempt_damage.direction,
        });

        if health.hp == 0.0 {
            commands.trigger(Defeated {
                entity: damaged_entity,
            });
        } else if let Some(source_entity) = attempt_damage.damage_source {
            // If entity is still alive and damage source exists and has effects list, we apply status effects
            if let Ok(effects) = source_query.get(source_entity) {
                commands.trigger(ApplyEffects::new(effects, damaged_entity));
            }
        }
    }
}

#[derive(Component)]
pub struct DamageFlash(Timer);

impl Default for DamageFlash {
    fn default() -> Self {
        Self(Timer::from_seconds(0.05, TimerMode::Once))
    }
}

pub(super) fn on_damage_dealt_flash(
    damage_dealt: On<DamageDealt>,
    mut commands: Commands,
    mut sprite_query: Query<&mut Sprite>,
    player: Single<Entity, With<Player>>,
) {
    // Player has iframe animation different from damage flash
    if damage_dealt.entity == *player {
        return;
    }

    if let Ok(mut sprite) = sprite_query.get_mut(damage_dealt.entity) {
        commands
            .entity(damage_dealt.entity)
            .insert(DamageFlash::default());
        sprite.color = Color::linear_rgba(255., 255., 255., 0.9);
    }
}

pub(super) fn tick_and_remove_damage_flash(
    mut damage_flash_query: Query<(&mut DamageFlash, &mut Sprite)>,
    time: Res<Time>,
) {
    damage_flash_query
        .par_iter_mut()
        .for_each(|(mut flash, mut sprite)| {
            if flash.0.tick(time.delta()).is_finished() {
                sprite.color = Color::WHITE;
            }
        });
}

#[derive(Component, Clone)]
pub struct Knockback(pub f32);

pub(super) fn on_damage_dealt_knockback(
    damage_dealt: On<DamageDealt>,
    knockback_query: Query<&Knockback>,
    mut forces: Query<Forces>,
) -> Result {
    if let Some(damage_source) = damage_dealt.damage_source
        && let Ok(knockback) = knockback_query.get(damage_source)
        && let Some(damage_direction) = damage_dealt.direction
    {
        forces
            .get_mut(damage_dealt.entity)?
            .apply_force(damage_direction * knockback.0 * 1_000_000.0);
    }
    Ok(())
}
