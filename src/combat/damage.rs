use avian2d::prelude::*;
use bevy::prelude::*;

use super::health::{Health, IFrames};
use super::hitbox::HurtBox;
use super::status_effects::prelude::{ApplyEffects, Effects};
use crate::interval::Interval;
use crate::prelude::*;

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

    /// Collision layers for a hitbox that hits this side's opponents.
    pub fn hitbox_layers(self) -> CollisionLayers {
        CollisionLayers::new(GameCollisionLayer::HitBox, self.target_hurtbox())
    }

    /// Collision layers for a projectile that hits this side's opponents and stops at walls.
    pub fn projectile_layers(self) -> CollisionLayers {
        CollisionLayers::new(
            GameCollisionLayer::PROJECTILE_MEMBERSHIPS,
            LayerMask::from(self.target_hurtbox()) | GameCollisionLayer::HighObstacle,
        )
    }

    pub fn opposite(self) -> Self {
        match self {
            Self::Ally => Self::Enemy,
            Self::Enemy => Self::Ally,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Damage {
    Single(f32),
    Range { amount: Interval },
}

impl Damage {
    /// This damage multiplied by `factor`, such as its attacker's [`StatMultipliers`] damage.
    pub fn scaled(self, factor: f32) -> Self {
        match self {
            Self::Range { amount } => Self::Range {
                amount: amount.scaled(factor),
            },
            Self::Single(amount) => Self::Single(amount * factor),
        }
    }

    /// Picks the amount of damage dealt, rolling within a range.
    fn roll(self) -> f32 {
        match self {
            Self::Range { amount } => amount.pick_random_value(),
            Self::Single(amount) => amount,
        }
    }
}

impl Default for Damage {
    fn default() -> Self {
        Self::Single(0.0)
    }
}

/// Bright white damage shader at home: linear channels far above 1.0 overexpose the sprite
/// so a hit flashes it white.
const DAMAGE_FLASH_COLOR: Color = Color::linear_rgba(255., 255., 255., 0.9);

#[derive(EntityEvent)]
pub struct AttemptDamage {
    pub entity: Entity,
    /// Not all damage gets blocked by invulnerable (ex: burn from status effect).
    pub ignore_invulnerable: bool,
    /// We treat damage as a range with RNG determining which value is dealt.
    pub damage: Damage,
    /// Weapon or projectile entity used for `Effects` and `Knockback`.
    /// Not all damage has one, like environmental damage or damage-over-time effects.
    pub source_item: Option<Entity>,
    /// Character responsible for aggro or credit. Melee sets the weapon holder,
    /// projectiles carry the attacker stamped at fire time, reflection transfers
    /// it to the blocker, environmental damage has none.
    pub attacker: Option<Entity>,
    /// Damage direction, ex. velocity direction of projectile or character position for melee.
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
/// this event represents when that damage attempt succeeds.
#[derive(EntityEvent)]
pub struct DamageDealt {
    pub entity: Entity,
    pub damage: f32,
    /// Weapon or projectile entity used for `Effects` and `Knockback`.
    pub source_item: Option<Entity>,
    /// Character responsible for aggro or credit (see `AttemptDamage::attacker`).
    pub attacker: Option<Entity>,
    /// Damage direction, ex. velocity direction of projectile or character position for melee.
    pub direction: Option<Vec2>,
}

#[derive(EntityEvent)]
pub struct Defeated {
    pub entity: Entity,
}

pub(super) fn on_attempt_damage(
    attempt_damage: On<AttemptDamage>,
    mut commands: Commands,
    hurt_box_query: Query<&ChildOf, With<HurtBox>>,
    mut damaged_query: Query<(&mut Health, Option<&mut IFrames>)>,
    source_query: Query<&Effects>,
    #[cfg(feature = "dev_tools")] invincible_query: Query<(), With<Invincible>>,
) {
    // Damage can be applied to an entities hurtbox, or to the entity directly
    let damaged_entity = if let Ok(child_of) = hurt_box_query.get(attempt_damage.entity) {
        child_of.parent()
    } else if damaged_query.contains(attempt_damage.entity) {
        attempt_damage.entity
    } else {
        return;
    };

    #[cfg(feature = "dev_tools")]
    if invincible_query.contains(damaged_entity) {
        return;
    }

    if let Ok((mut health, has_iframes)) = damaged_query.get_mut(damaged_entity) {
        // Entities have to "opt-in" to having iframes. Right now that is only the player
        // Damage that ignores invulnerability (ex: burn ticks) shouldn't grant iframes either,
        // otherwise damage-over-time effects would shield the player from regular hits
        if let Some(mut iframes) = has_iframes
            && !attempt_damage.ignore_invulnerable
        {
            if iframes.invulnerable {
                return;
            }

            iframes.invulnerable = true;
        }

        if health.is_dead() {
            return;
        }

        // Convert `Damage` to raw damage amount
        let damage = attempt_damage.damage.roll();
        health.take_damage(damage);

        // Because AttemptDamage may not result in damage being applied (invulnerable or entity
        // without health) we send this event for guranteed "X damage has been done". Proper
        // change detection added to bevy would mean this isn't needed
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
            // If entity is still alive and damage source exists and has effects list, we apply
            // status effects
            if let Ok(effects) = source_query.get(source_entity) {
                commands.trigger(ApplyEffects::new(effects, damaged_entity));
            }
        }
    }
}

#[derive(Component, Deref, DerefMut)]
pub(super) struct DamageFlash(Timer);

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
        sprite.color = DAMAGE_FLASH_COLOR;
    }
}

pub(super) fn tick_and_remove_damage_flash(
    mut damage_flash_query: Query<(&mut DamageFlash, &mut Sprite)>,
    time: Res<Time>,
) {
    damage_flash_query
        .par_iter_mut()
        .for_each(|(mut flash, mut sprite)| {
            // Only reset color the frame the flash ends, otherwise we overwrite sprite color every
            // frame
            if flash.tick(time.delta()).just_finished() {
                sprite.color = Color::WHITE;
            }
        });
}

/// The impulse a weapon or projectile's hit deals: it shoves a character `impulse / mass` world
/// units; see [`KnockedBack`].
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Knockback(pub f32);

pub(super) fn on_damage_dealt_knockback(
    damage_dealt: On<DamageDealt>,
    mut commands: Commands,
    knockbacks: Query<&Knockback>,
    masses: Query<&Mass, With<Kinematics>>,
) {
    if let Some(source_item) = damage_dealt.source_item
        && let Ok(knockback) = knockbacks.get(source_item)
        && let Some(damage_direction) = damage_dealt.direction
        && let Ok(mass) = masses.get(damage_dealt.entity)
    {
        commands
            .entity(damage_dealt.entity)
            .try_insert(KnockedBack::new(damage_direction, knockback.0 / mass.0));
    }
}

/// Takes no damage from any source, including damage that ignores invulnerability.
/// Used by the stress arena, so a fight lasts as long as a capture needs, and by the F10 dev key.
#[cfg(feature = "dev_tools")]
#[derive(Component, Clone, Copy, Debug)]
pub struct Invincible;
