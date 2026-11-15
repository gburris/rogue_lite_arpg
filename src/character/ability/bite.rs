use avian2d::prelude::*;
use bevy::platform::collections::HashSet;
use bevy::prelude::*;

use super::{Ability, AbilityCooldown, AbilityOf, AbilityReleased, AbilitySteps};
use crate::prelude::*;

/// Seconds a bite's hitbox stays out: about four frames, one quick snap.
const BITE_HITBOX_SECONDS: f32 = 0.07;

/// A snap of the jaws: a brief round hitbox in front of the biter, along its aim.
#[derive(Component, Clone, Copy, Debug)]
#[require(Ability::Bite, AbilityCooldown, AbilitySteps)]
pub struct Bite {
    reach: f32,
    damage: f32,
    life_steal: f32,
    hitbox_radius: f32,
    mouth_height: f32,
}

impl Bite {
    /// Bites for `damage` with a hitbox whose front edge reaches `reach`.
    pub fn new(reach: f32, damage: f32) -> Self {
        Self {
            reach,
            damage,
            life_steal: 0.0,
            hitbox_radius: reach / 2.0,
            mouth_height: 0.0,
        }
    }

    /// Sizes the hitbox at `radius`, centered `mouth_height` above the biter's transform before
    /// it is pushed along the aim.
    pub fn with_hitbox(self, radius: f32, mouth_height: f32) -> Self {
        Self {
            hitbox_radius: radius,
            mouth_height,
            ..self
        }
    }

    /// Heals the biter by `fraction` of the damage each landed bite deals.
    pub fn with_life_steal(self, fraction: f32) -> Self {
        Self {
            life_steal: fraction,
            ..self
        }
    }

    pub(super) fn reach(&self) -> f32 {
        self.reach
    }
}

/// The short-lived hitbox of one bite, a child of the biter. It damages each opponent it touches
/// once, and heals the biter by `life_steal` of every bite that lands.
#[derive(Component, Clone, Debug)]
#[require(Sensor, CollidingEntities, Lifespan::new(BITE_HITBOX_SECONDS))]
pub(super) struct BiteHitbox {
    biter: Entity,
    damage: f32,
    life_steal: f32,
    direction: Vec2,
    bitten: HashSet<Entity>,
}

/// Snaps a bite along the holder's aim. The hitbox decides whether it lands.
pub(super) fn on_bite_released(
    released: On<AbilityReleased>,
    mut commands: Commands,
    bites: Query<(&Bite, &AbilityOf)>,
    biting_characters: Query<(&Vision, &StatMultipliers, &CombatSide)>,
) {
    let Ok((bite, ability_of)) = bites.get(released.entity) else {
        return;
    };
    let holder = ability_of.holder;
    let Ok((vision, stat_multipliers, side)) = biting_characters.get(holder) else {
        return;
    };
    let Some(direction) = vision.aim_direction.try_normalize() else {
        return;
    };
    let mouth_offset = Vec2::Y * bite.mouth_height + direction * (bite.reach - bite.hitbox_radius);
    commands.spawn((
        BiteHitbox {
            biter: holder,
            damage: bite.damage * stat_multipliers.damage,
            life_steal: bite.life_steal,
            direction,
            bitten: HashSet::default(),
        },
        Collider::circle(bite.hitbox_radius),
        CollisionLayers::new(GameCollisionLayer::HitBox, side.target_hurtbox()),
        Transform::from_translation(mouth_offset.extend(0.0)),
        ChildOf(holder),
    ));
}

/// Damages each opponent a bite hitbox touches, once per bite.
pub(super) fn damage_bite_hits(
    mut commands: Commands,
    mut hitboxes: Query<(&mut BiteHitbox, &CollidingEntities)>,
    biters: Query<&Health>,
) {
    for (mut hitbox, colliding_entities) in &mut hitboxes {
        for &bitten in colliding_entities.iter() {
            if !hitbox.bitten.insert(bitten) {
                continue;
            }
            commands.trigger(AttemptDamage {
                entity: bitten,
                damage: Damage::Single(hitbox.damage),
                source_item: None,
                attacker: Some(hitbox.biter),
                direction: Some(hitbox.direction),
                ..default()
            });

            // Skip the heal at full health so the overlay does not pop a zero.
            let is_hurt = biters
                .get(hitbox.biter)
                .is_ok_and(|health| health.hp < health.max_hp);
            if hitbox.life_steal > 0.0 && is_hurt {
                commands.trigger(AttemptHeal {
                    entity: hitbox.biter,
                    amount: hitbox.damage * hitbox.life_steal,
                });
            }
        }
    }
}
