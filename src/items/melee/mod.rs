use avian2d::prelude::*;
use bevy::{platform::collections::HashSet, prelude::*, ui_widgets::observe};

use crate::{
    combat::{
        damage::{AttemptDamage, Damage, DamageSource, Knockback},
        status_effects::{Effects, Frozen},
    },
    configuration::GameCollisionLayer,
    items::{Item, ItemType, equipment::Equippable, prelude::UseEquipment},
    prelude::*,
};

mod swing;

pub use swing::{MeleeSwingType, process_melee_attacks};

pub mod prelude {
    pub use super::swing::*;
    pub use super::{ActiveMeleeAttack, MeleeWeapon, axe, freeze_axe, sword};
}

/// Our pixel weapons all face upwards currently, so we must rotate them 90 degrees for attacks to
/// occur in the direction we expect. This value will need to be updated if our assets change
pub const MELEE_WEAPON_ROTATION: f32 = std::f32::consts::FRAC_PI_2;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (process_melee_attacks, end_melee_attacks).in_set(InGameSystems::Simulation),
    );

    app.add_systems(
        Update,
        handle_melee_collisions.in_set(InGameSystems::Collision),
    );
}

pub fn sword(sprites: &SpriteAssets) -> impl Bundle {
    (
        MeleeWeapon {
            damage: (1.0, 6.0),
            hitbox: Collider::rectangle(10.0, 40.0),
            attack_type: MeleeSwingType::STAB,
            attack_time: 0.2,
            hold_distance: 15.0,
        },
        Name::new("Sword"),
        Knockback(10.0),
        Equippable::default(),
        Item::new(120, ItemType::Melee),
        Sprite::from_image(sprites.sword.clone()),
        observe(on_weapon_melee),
    )
}

pub fn axe(sprites: &SpriteAssets) -> impl Bundle {
    (
        MeleeWeapon {
            damage: (2.0, 12.0),
            hitbox: Collider::rectangle(10.0, 40.0),
            attack_type: MeleeSwingType::SLASH,
            attack_time: 0.3,
            hold_distance: 30.0,
        },
        Name::new("Axe"),
        Knockback(20.0),
        Equippable::default(),
        Item::new(220, ItemType::Melee),
        Sprite::from_image(sprites.axe.clone()),
        observe(on_weapon_melee),
    )
}

pub fn freeze_axe(sprites: &SpriteAssets) -> impl Bundle {
    (
        MeleeWeapon {
            damage: (2.0, 12.0),
            hitbox: Collider::rectangle(10.0, 40.0),
            attack_type: MeleeSwingType::SLASH,
            attack_time: 0.3,
            hold_distance: 30.0,
        },
        Name::new("Freeze Axe"),
        Knockback(2.0),
        Equippable::default(),
        Item::new(220, ItemType::Melee),
        Sprite::from_image(sprites.axe.clone()),
        related!(Effects[(Frozen, Lifespan::new(2.0))]),
        observe(on_weapon_melee),
    )
}

//Repesent a melee weapon
#[derive(Component, Clone, Debug)]
pub struct MeleeWeapon {
    // Time it takes (seconds) to complete the attack, smaller = faster
    pub attack_time: f32,
    pub damage: (f32, f32),
    pub hitbox: Collider,
    pub attack_type: swing::MeleeSwingType,
    pub hold_distance: f32,
}

impl MeleeWeapon {
    /// Gets collision layers for melee weapon based on source of damage
    /// It can either target allies or enemies
    pub fn collision_layers(damage_source: DamageSource) -> CollisionLayers {
        CollisionLayers::new(GameCollisionLayer::HitBox, LayerMask::from(damage_source))
    }
}

fn handle_melee_collisions(
    mut commands: Commands,
    mut melee_query: Query<(
        Entity,
        &MeleeWeapon,
        &mut ActiveMeleeAttack,
        &CollidingEntities,
    )>,
) {
    for (weapon_entity, melee_weapon, mut active_melee_attack, colliding_entities) in
        melee_query.iter_mut()
    {
        for &colliding_entity in colliding_entities.iter() {
            // We only hit a given entity once per attack
            if !active_melee_attack
                .entities_damaged
                .contains(&colliding_entity)
            {
                commands.trigger(AttemptDamage {
                    entity: colliding_entity,
                    ignore_invulnerable: false,
                    damage: Damage::Range(melee_weapon.damage),
                    damage_source: Some(weapon_entity),
                    direction: Some(Vec2::from_angle(active_melee_attack.initial_angle)),
                });
                active_melee_attack
                    .entities_damaged
                    .insert(colliding_entity);
            }
        }
    }
}

#[derive(Component)]
#[require(CollidingEntities, Sensor)]
pub struct ActiveMeleeAttack {
    /// Comes from the direction the entity holding the weapon is aiming
    initial_angle: f32,
    /// Comes from "attack_speed" defined on MeleeWeapon
    duration: Timer,
    entities_damaged: HashSet<Entity>,
}

impl ActiveMeleeAttack {
    pub fn new(initial_angle: f32, speed: f32) -> Self {
        Self {
            initial_angle,
            duration: Timer::from_seconds(speed, TimerMode::Once),
            entities_damaged: HashSet::default(),
        }
    }
}

fn on_weapon_melee(
    melee_weapon_used: On<UseEquipment>,
    mut commands: Commands,
    mut weapon_query: Query<(Entity, &mut MeleeWeapon)>,
    mut attack_state_query: Query<&mut AttackState>,
    holder_query: Query<&Vision>,
) {
    let Ok((weapon_entity, mut melee_weapon)) = weapon_query.get_mut(melee_weapon_used.entity)
    else {
        warn!("Tried to melee attack with invalid weapon");
        return;
    };

    let Ok(vision) = holder_query.get(melee_weapon_used.holder) else {
        warn!("Holder missing required components");
        return;
    };

    let attack_angle = vision.aim_direction.to_angle();

    start_melee_attack(
        &mut commands,
        weapon_entity,
        &mut melee_weapon,
        attack_angle,
    );

    if let Ok(mut attack_state) = attack_state_query.get_mut(melee_weapon_used.holder) {
        attack_state.is_attacking = true;
    }
}

fn start_melee_attack(
    commands: &mut Commands,
    weapon_entity: Entity,
    melee_weapon: &mut MeleeWeapon,
    attack_angle: f32,
) {
    commands
        .entity(weapon_entity)
        .insert(ActiveMeleeAttack::new(
            attack_angle,
            melee_weapon.attack_time,
        ));
}

fn end_melee_attacks(
    mut commands: Commands,
    mut query: Query<(Entity, &ChildOf, &ActiveMeleeAttack)>,
    mut attack_state_query: Query<&mut AttackState>,
) {
    for (entity, child_of, attack) in query.iter_mut() {
        if attack.duration.just_finished() {
            if let Ok(mut attack_state) = attack_state_query.get_mut(child_of.parent()) {
                attack_state.is_attacking = false;
            }
            commands.entity(entity).remove::<ActiveMeleeAttack>();
        }
    }
}
