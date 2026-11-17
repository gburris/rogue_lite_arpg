use avian2d::prelude::*;
use bevy::platform::collections::HashSet;
use bevy::prelude::*;

use crate::prelude::*;

/// A sensor that damages each opponent it touches once, on behalf of `attacker`. Its entity is
/// the hit's `source_item`, so the [`Knockback`], [`Effects`] and [`LifeSteal`] on it apply.
#[derive(Component, Clone, Debug)]
#[require(Sensor, CollidingEntities)]
pub struct HitBox {
    attacker: Entity,
    damage: Damage,
    direction: Vec2,
    already_hit: HashSet<Entity>,
}

impl HitBox {
    /// Deals `damage` along `direction` to each opponent it touches, credited to `attacker`.
    pub fn new(attacker: Entity, damage: Damage, direction: Vec2) -> Self {
        Self {
            attacker,
            damage,
            direction,
            already_hit: HashSet::default(),
        }
    }
}

#[derive(Component, Default, Clone)]
#[require(Sensor)]
pub struct HurtBox;

pub fn hurtbox(size: Vec2, membership: GameCollisionLayer) -> impl Scene {
    hurtbox_at(size, membership, HURTBOX_OFFSET)
}

/// Hurtbox with a custom vertical centre, for sheets whose body does not sit
/// at the humanoid offset.
pub fn hurtbox_at(size: Vec2, membership: GameCollisionLayer, y_offset: f32) -> impl Scene {
    bsn! {
        HurtBox
        ~Collider::rectangle(size.x, size.y)
        Transform::from_xyz(0.0, y_offset, 0.0)
        ~CollisionLayers::new(membership, [GameCollisionLayer::HitBox])
    }
}

/// Default hurtbox centre relative to the character transform.
const HURTBOX_OFFSET: f32 = -8.0;

/// Heals the attacker by this fraction of the damage each hit from this source deals.
#[derive(Component, Clone, Copy, Debug, Default, Deref)]
pub struct LifeSteal(pub f32);

pub(super) fn damage_hitbox_contacts(
    mut commands: Commands,
    mut hitboxes: Query<(Entity, &mut HitBox, &CollidingEntities)>,
) {
    for (hitbox_entity, mut hitbox, colliding_entities) in &mut hitboxes {
        for &touched in colliding_entities.iter() {
            if !hitbox.already_hit.insert(touched) {
                continue;
            }
            commands.trigger(AttemptDamage {
                entity: touched,
                damage: hitbox.damage,
                source_item: Some(hitbox_entity),
                attacker: Some(hitbox.attacker),
                direction: Some(hitbox.direction),
                ..default()
            });
        }
    }
}

pub(super) fn on_damage_dealt_life_steal(
    dealt: On<DamageDealt>,
    mut commands: Commands,
    life_steals: Query<&LifeSteal>,
) {
    let (Some(source_item), Some(attacker)) = (dealt.source_item, dealt.attacker) else {
        return;
    };
    let Ok(&LifeSteal(fraction)) = life_steals.get(source_item) else {
        return;
    };
    commands.trigger(AttemptHeal {
        entity: attacker,
        amount: dealt.damage * fraction,
    });
}
