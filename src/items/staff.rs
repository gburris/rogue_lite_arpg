use std::f32::consts::FRAC_PI_8;

use avian2d::prelude::*;
use bevy::{ecs::entity_disabling::Disabled, prelude::*, ui_widgets::observe};

use crate::{
    items::{
        Item,
        equipment::{EquipmentType, Equippable},
        prelude::UseEquipment,
    },
    prelude::*,
};

use super::ItemType;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        tick_and_end_casting.in_set(InGameSystems::DespawnEntities),
    );
}

pub fn fire_staff(sprites: &SpriteAssets, sprite_layouts: &SpriteSheetLayouts) -> impl Bundle {
    (
        Name::new("Staff of Flames"),
        Item::new(1340, ItemType::Staff),
        Equippable {
            equip_type: EquipmentType::Staff,
            ..default()
        },
        ManaCost(6.0),
        Sprite::from_image(sprites.fire_staff.clone()),
        related!(
            Projectiles [
                fireball(sprites, sprite_layouts, -FRAC_PI_8),
                fireball(sprites, sprite_layouts, 0.0),
                fireball(sprites, sprite_layouts, FRAC_PI_8)
            ]
        ),
        observe(on_staff_fired),
    )
}

pub fn ice_staff(sprites: &SpriteAssets, sprite_layouts: &SpriteSheetLayouts) -> impl Bundle {
    (
        Name::new("Staff of Ice"),
        Item::new(2050, ItemType::Staff),
        ManaCost(20.0), // big mana cost
        Equippable {
            use_rate: Timer::from_seconds(0.7, TimerMode::Once),
            equip_type: EquipmentType::Staff,
            ..default()
        },
        Sprite::from_image(sprites.ice_staff.clone()),
        Projectiles::spawn_one(icebolt(sprites, sprite_layouts, 0.0)),
        observe(on_staff_fired),
    )
}

#[derive(Component)]
struct Casting {
    duration: Timer,
}

impl Casting {
    pub fn new(speed: f32) -> Self {
        Self {
            duration: Timer::from_seconds(speed, TimerMode::Once),
        }
    }
}

// "fired" implies this is a projectile weapon
fn on_staff_fired(
    staff_fired: On<UseEquipment>,
    mut commands: Commands,
    staff_query: Query<(&Projectiles, &ItemOf, &Sprite, &Transform)>,
    holder_query: Query<(&Transform, &Vision, Option<&TargetInfo>)>,
    enemy_query: Query<Entity, With<Enemy>>,
    images: Res<Assets<Image>>,
) {
    let Ok((projectiles, item_of, sprite, staff_transform)) = staff_query.get(staff_fired.entity)
    else {
        warn!("Tried to fire staff that is not a projectile staff");
        return;
    };

    let Ok((holder_transform, holder_vision, target_info)) = holder_query.get(item_of.0) else {
        warn!("Tried to fire staff with holder missing aim position or transform");
        return;
    };

    // add casting to staff

    let damage_source = if enemy_query.get(item_of.0).is_ok() {
        DamageSource::Enemy
    } else {
        DamageSource::Player
    };

    // TODO: Move staff "projectile source offset" computation to staff construction time!
    let staff_image = images
        .get(sprite.image.id())
        .expect("Staff should have image");

    // Subtract a little to try to get center of staff "eye" (source of projectile spawn)
    let staff_half_height = (staff_image.height() as f32 / 2.) - 6.;
    // Staff images start straight up (Vec3::Y) so multiply rotation quat by that direction to get direction as a unit vector
    let staff_projectile_source_offset =
        staff_half_height * (staff_transform.rotation * Vec3::Y).truncate();
    let staff_projectile_source_pos =
        staff_transform.translation.truncate() + staff_projectile_source_offset;

    // If no target ditance default to sane value away from holder
    let target_angle = (holder_vision.aim_direction * target_info.map_or(100., |t| t.distance))
        - staff_projectile_source_pos;

    // Staff is child of holder so position is relative, need to add holder transform for global position
    let starting_position = holder_transform.translation.truncate() + staff_projectile_source_pos;

    for projectile_entity in projectiles.iter() {
        commands.trigger(FireProjectile::from((
            projectile_entity,
            damage_source,
            starting_position,
            target_angle,
        )));
    }
}

fn tick_and_end_casting(
    mut commands: Commands,
    casting_query: Query<(Entity, &mut Casting)>,
    time: Res<Time>,
) {
    for (entity, mut casting) in casting_query {
        casting.duration.tick(time.delta());

        if casting.duration.is_finished() {
            commands.entity(entity).remove::<Casting>();
        }
    }
}
