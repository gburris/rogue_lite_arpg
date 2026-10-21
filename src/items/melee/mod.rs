mod swing;

pub mod prelude {
    pub use super::{ActiveMeleeAttack, axe, freeze_axe, sword};
}

use std::sync::LazyLock;

use avian2d::prelude::*;
use bevy::{
    platform::collections::{HashMap, HashSet},
    prelude::*,
    ui_widgets::observe,
};
use bevy_enhanced_input::prelude::*;

use crate::{equipment_transforms, items::melee::swing::MeleeSwingType, prelude::*};

/// Our pixel weapons all face upwards currently, so we must rotate them 90 degrees for attacks to
/// occur in the direction we expect. This value will need to be updated if our assets change
pub const MELEE_WEAPON_ROTATION: f32 = std::f32::consts::FRAC_PI_2;

pub(super) fn plugin(app: &mut App) {
    app.add_input_context::<MeleeWeapon>();

    app.add_systems(
        Update,
        (
            swing::process_melee_attacks.before(crate::character::AiSet::Perception),
            end_melee_attacks,
        )
            .chain()
            .in_set(SimulationSystems::Logic),
    );

    app.add_systems(
        Update,
        handle_melee_collisions.in_set(InGameSystems::Collision),
    );
}

pub fn sword(sprites: &SpriteAssets) -> impl Bundle {
    (
        Name::new("Sword"),
        MeleeWeapon {
            damage: (1.0, 6.0),
            hitbox: Collider::rectangle(10.0, 40.0),
            attack_type: MeleeSwingType::STAB,
            attack_time: 0.2,
            hold_distance: 15.0,
        },
        Knockback(10.0),
        Equippable::new(EquipmentSlot::Mainhand, 0.4, &MELEE_EQUIPMENT_TRANSFORMS),
        Item::new(120, ItemType::Melee),
        Sprite::from_image(sprites.sword.clone()),
        observe(on_stop_melee),
        observe(on_start_melee),
    )
}

pub fn axe(sprites: &SpriteAssets) -> impl Bundle {
    (
        Name::new("Axe"),
        MeleeWeapon {
            damage: (2.0, 12.0),
            hitbox: Collider::rectangle(10.0, 40.0),
            attack_type: MeleeSwingType::SLASH,
            attack_time: 0.3,
            hold_distance: 30.0,
        },
        Knockback(20.0),
        Equippable::new(EquipmentSlot::Mainhand, 0.4, &MELEE_EQUIPMENT_TRANSFORMS),
        Sprite::from_image(sprites.axe.clone()),
        Item::new(220, ItemType::Melee),
        observe(on_stop_melee),
        observe(on_start_melee),
    )
}

pub fn freeze_axe(sprites: &SpriteAssets) -> impl Bundle {
    (
        Name::new("Freeze Axe"),
        MeleeWeapon {
            damage: (2.0, 12.0),
            hitbox: Collider::rectangle(10.0, 40.0),
            attack_type: MeleeSwingType::SLASH,
            attack_time: 0.3,
            hold_distance: 30.0,
        },
        Knockback(2.0),
        Equippable::new(EquipmentSlot::Mainhand, 0.4, &MELEE_EQUIPMENT_TRANSFORMS),
        Item::new(220, ItemType::Melee),
        Sprite::from_image(sprites.axe.clone()),
        related!(Effects[(Frozen, Lifespan::new(2.0))]),
        observe(on_stop_melee),
        observe(on_start_melee),
    )
}

//Repesent a melee weapon
#[derive(Component, Clone, Debug)]
#[require(ContextPriority::<MeleeWeapon>::new(1))]
struct MeleeWeapon {
    // Time it takes (seconds) to complete the attack, smaller = faster
    attack_time: f32,
    damage: (f32, f32),
    hitbox: Collider,
    attack_type: swing::MeleeSwingType,
    hold_distance: f32,
}

impl MeleeWeapon {
    /// Gets collision layers for melee weapon based on the holder's side.
    /// It can either target allies or enemies.
    pub fn collision_layers(side: CombatSide) -> CollisionLayers {
        CollisionLayers::new(GameCollisionLayer::HitBox, side.target_hurtbox())
    }
}

use ZLayer::BehindSprite;
static MELEE_EQUIPMENT_TRANSFORMS: LazyLock<HashMap<FacingDirection, Transform>> =
    equipment_transforms!([
        (Up, ((0.0, -8.0), 30.0)),
        (Down, ((0.0, 8.0), -30.0, BehindSprite)),
        (Left, ((-8.0, -15.0), 90.0, BehindSprite)),
        (Right, ((8.0, -15.0), -90.0)),
    ]);

fn handle_melee_collisions(
    mut commands: Commands,
    mut melee_query: Query<(
        Entity,
        &MeleeWeapon,
        &mut ActiveMeleeAttack,
        &CollidingEntities,
        &ItemOf,
    )>,
) {
    for (weapon_entity, melee_weapon, mut active_melee_attack, colliding_entities, item_of) in
        &mut melee_query
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
                    source_item: Some(weapon_entity),
                    attacker: Some(item_of.holder),
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
    /// Comes from "`attack_speed`" defined on `MeleeWeapon`
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

/// Start observer: spawns the transient swing hitbox removed by `on_stop_melee`.
fn on_start_melee(
    melee: On<StartUsingEquipment>,
    mut commands: Commands,
    weapon_query: Query<&MeleeWeapon>,
    holder_query: Query<(&Vision, &CombatSide)>,
) {
    let Ok(melee_weapon) = weapon_query.get(melee.item) else {
        warn!("Tried to melee attack with invalid weapon");
        return;
    };

    let Ok((vision, side)) = holder_query.get(melee.holder) else {
        warn!("Holder missing vision");
        return;
    };

    let attack_angle = vision.aim_direction.to_angle();

    // Hitbox and layers only exist for the swing; the stop observer removes them.
    commands.entity(melee.item).insert((
        EquipmentInUse,
        ActiveMeleeAttack::new(attack_angle, melee_weapon.attack_time),
        melee_weapon.hitbox.clone(),
        MeleeWeapon::collision_layers(*side),
    ));
}

/// End polling: queues `StopUsingEquipment` once each swing timer finishes.
fn end_melee_attacks(mut commands: Commands, swings: Query<(Entity, &ItemOf, &ActiveMeleeAttack)>) {
    for (item, item_of, attack) in &swings {
        if attack.duration.just_finished() {
            commands.trigger(StopUsingEquipment {
                item,
                holder: item_of.holder,
            });
        }
    }
}

/// Stop observer: removes the transient swing state queued by `end_melee_attacks`.
fn on_stop_melee(stop: On<StopUsingEquipment>, mut commands: Commands) {
    commands
        .entity(stop.item)
        .remove::<(EquipmentInUse, ActiveMeleeAttack, Collider, CollisionLayers)>();
}
