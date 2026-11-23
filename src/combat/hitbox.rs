use avian2d::prelude::*;
use bevy::platform::collections::HashSet;
use bevy::prelude::*;

use crate::prelude::*;

/// A sensor that damages each opponent it touches once, credited to its [`CreditedAttackOf`]
/// attacker. Its hits name its [`HitBoxOf`] item as their source, or the hitbox itself without
/// one, so the [`Knockback`], [`Effects`] and [`LifeSteal`] on that entity apply.
#[derive(Component, Clone, Debug)]
#[require(Sensor, CollidingEntities)]
pub struct HitBox {
    damage: Damage,
    direction: Vec2,
    already_hit: HashSet<Entity>,
}

impl HitBox {
    /// Deals `damage` along `direction` to each opponent it touches.
    pub fn new(damage: Damage, direction: Vec2) -> Self {
        Self {
            damage,
            direction,
            already_hit: HashSet::default(),
        }
    }
}

/// Names `item`, such as a weapon, as the source of this hitbox's hits. Despawning the item
/// despawns its hitboxes.
#[derive(Component, Clone, Copy, Debug)]
#[relationship(relationship_target = HitBoxes)]
pub struct HitBoxOf {
    pub item: Entity,
}

/// The hitboxes whose hits name this item as their source; see [`HitBoxOf`].
#[derive(Component, Debug)]
#[relationship_target(relationship = HitBoxOf, linked_spawn)]
pub struct HitBoxes(Vec<Entity>);

#[derive(Component, Default, Clone)]
#[require(Sensor)]
pub(super) struct HurtBox;

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
    mut hitboxes: Query<(
        Entity,
        &mut HitBox,
        &CollidingEntities,
        Option<&HitBoxOf>,
        Option<&CreditedAttackOf>,
    )>,
) {
    for (hitbox_entity, mut hitbox, colliding_entities, hitbox_of, credited_attack_of) in
        &mut hitboxes
    {
        let source_item = hitbox_of.map_or(hitbox_entity, |hitbox_of| hitbox_of.item);
        let attacker = credited_attack_of.map(|credited_attack_of| credited_attack_of.attacker);
        for &touched in colliding_entities.iter() {
            if !hitbox.already_hit.insert(touched) {
                continue;
            }
            commands.trigger(AttemptDamage {
                entity: touched,
                damage: hitbox.damage,
                source_item: Some(source_item),
                attacker,
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
