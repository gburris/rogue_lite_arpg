use avian2d::prelude::*;
use bevy::platform::collections::HashSet;
use bevy::prelude::*;

use crate::prelude::*;

/// Seconds a bite's hitbox stays out: about four frames, one quick snap.
const BITE_HITBOX_SECONDS: f32 = 0.07;

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

impl BiteHitbox {
    /// A bite by `biter` toward `direction`.
    pub(super) fn new(biter: Entity, damage: f32, life_steal: f32, direction: Vec2) -> Self {
        Self {
            biter,
            damage,
            life_steal,
            direction,
            bitten: HashSet::default(),
        }
    }

    /// Snaps this bite as a `radius` circle at `offset` from the biter, hitting `side`'s
    /// opponents.
    pub(super) fn spawn(
        self,
        commands: &mut Commands,
        side: CombatSide,
        offset: Vec2,
        radius: f32,
    ) {
        let biter = self.biter;
        commands.spawn((
            self,
            Collider::circle(radius),
            CollisionLayers::new(GameCollisionLayer::HitBox, side.target_hurtbox()),
            Transform::from_translation(offset.extend(0.0)),
            ChildOf(biter),
        ));
    }
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
