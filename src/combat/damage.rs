use avian2d::prelude::*;
use bevy::prelude::*;
use rand::RngExt;

use super::health::Health;
use super::invulnerable::IFrames;
use super::status_effects::{ApplyEffects, Effects};
use crate::prelude::{GameCollisionLayer, Player};

/// Which side of combat a character fights on, required by every character marker.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
#[component(immutable)]
pub enum CombatSide {
    Ally,
    Enemy,
}

impl CombatSide {
    /// The layer my hurtbox is on.
    pub fn hurtbox_layer(self) -> GameCollisionLayer {
        match self {
            Self::Ally => GameCollisionLayer::AllyHurtBox,
            Self::Enemy => GameCollisionLayer::EnemyHurtBox,
        }
    }

    /// Layer my attacks collide with (the enemy's hurtbox layer).
    pub fn target_hurtbox(self) -> GameCollisionLayer {
        self.opposite().hurtbox_layer()
    }

    pub fn opposite(self) -> Self {
        match self {
            Self::Ally => Self::Enemy,
            Self::Enemy => Self::Ally,
        }
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
    hurtbox_at(size, membership, HURTBOX_OFFSET)
}

/// Hurtbox with a custom vertical centre, for sheets whose body does not sit
/// at the humanoid offset.
pub fn hurtbox_at(size: Vec2, membership: GameCollisionLayer, y_offset: f32) -> impl Bundle {
    (
        HurtBox,
        Collider::rectangle(size.x, size.y),
        Transform::from_xyz(0.0, y_offset, 0.0),
        CollisionLayers::new(membership, [GameCollisionLayer::HitBox]),
    )
}

/// Default hurtbox centre relative to the character transform.
const HURTBOX_OFFSET: f32 = -8.0;

#[derive(EntityEvent)]
pub struct AttemptDamage {
    pub entity: Entity,
    /// Not all damage gets blocked by invulnerable (ex: burn from status effect)
    pub ignore_invulnerable: bool,
    /// We treat damage as a range with RNG determining which value is dealt
    pub damage: Damage,
    /// Weapon or projectile entity used for `Effects` and `Knockback`.
    /// Not all damage has one, like environmental damage or damage-over-time effects.
    pub source_item: Option<Entity>,
    /// Character responsible for aggro or credit. Melee sets the weapon holder,
    /// projectiles carry the attacker stamped at fire time, reflection transfers
    /// it to the blocker, environmental damage has none.
    pub attacker: Option<Entity>,
    /// damage direction, ex. velocity direction of projectile or character position for melee
    pub direction: Option<Vec2>,
}

impl Default for AttemptDamage {
    fn default() -> Self {
        Self {
            entity: Entity::PLACEHOLDER,
            ignore_invulnerable: false,
            damage: Damage::Single(1.0),
            source_item: None,
            attacker: None,
            direction: None,
        }
    }
}

/// While `AttemptDamage` is sent any time a damage source interacts with an entity,
/// this event represents when that damage attempt succeeds
#[derive(EntityEvent)]
pub struct DamageDealt {
    pub entity: Entity,
    pub damage: f32,
    /// Weapon or projectile entity used for `Effects` and `Knockback`.
    pub source_item: Option<Entity>,
    /// Character responsible for aggro or credit (see `AttemptDamage::attacker`).
    pub attacker: Option<Entity>,
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
        // Damage that ignores invulnerability (ex: burn ticks) shouldn't grant iframes either,
        // otherwise damage-over-time effects would shield the player from regular hits
        if let Some(mut iframes) = has_iframes
            && !attempt_damage.ignore_invulnerable
        {
            if iframes.is_invulnerable {
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

        // Because AttemptDamage may not result in damage being applied (invulnerable or entity without health)
        // we send this event for guranteed "X damage has been done". Proper change detection added to bevy would mean this isn't needed
        commands.trigger(DamageDealt {
            entity: damaged_entity,
            damage,
            source_item: attempt_damage.source_item,
            attacker: attempt_damage.attacker,
            direction: attempt_damage.direction,
        });

        if health.hp == 0.0 {
            commands.trigger(Defeated {
                entity: damaged_entity,
            });
        } else if let Some(source_entity) = attempt_damage.source_item {
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
            // Only reset color the frame the flash ends, otherwise we overwrite sprite color every frame
            if flash.0.tick(time.delta()).just_finished() {
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
    if let Some(source_item) = damage_dealt.source_item
        && let Ok(knockback) = knockback_query.get(source_item)
        && let Some(damage_direction) = damage_dealt.direction
    {
        forces
            .get_mut(damage_dealt.entity)?
            .apply_force(damage_direction * knockback.0 * 1_000_000.0);
    }
    Ok(())
}
