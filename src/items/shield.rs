use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};

use avian2d::prelude::*;
use bevy::prelude::*;

use super::equipment::prelude::DEFAULT_EQUIPMENT_POSES;
use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, orient_active_shields.in_set(SimulationSystems::Act));
}

pub fn magic_shield(sprites: &SpriteAssets, sprite_layouts: &SpriteSheetLayouts) -> impl Scene {
    bsn! {
        @shield(sprites.magic_shield.clone(), sprite_layouts)
        Name::new("Magic Shield")
        ManaCost(5.0)
        ManaDrainRate(20.0)
        ProjectileReflection
    }
}

pub fn knight_shield(sprites: &SpriteAssets, sprite_layouts: &SpriteSheetLayouts) -> impl Scene {
    bsn! {
        @shield(sprites.knight_shield.clone(), sprite_layouts)
        Name::new("Knight Shield")
        ManaCost(25.0)
        ManaDrainRate(25.0)
    }
}

/// What every shield shares: an offhand item with a square block hitbox, shown with `image`.
fn shield(image: Handle<Image>, sprite_layouts: &SpriteSheetLayouts) -> impl Scene {
    let sprite = Sprite::from_atlas_image(
        image,
        TextureAtlas {
            layout: sprite_layouts.shield.clone(),
            index: 0,
        },
    );
    bsn! {
        ~Item::new(355, ItemKind::Shield)
        ~Equippable::new(EquipmentSlot::Offhand, &DEFAULT_EQUIPMENT_POSES)
        ~ActionCooldown::new(0.5)
        ~{ActionUseStyle::Sustained}
        ~{Shield {
            hitbox: Collider::rectangle(25.0, 25.0),
        }}
        template(move |_| Ok(sprite.clone()))
        on(on_shield_took_effect)
        on(on_stop_shield)
    }
}

#[derive(Component, Clone)]
pub struct Shield {
    pub hitbox: Collider,
}

#[derive(Component)]
struct ActiveShield;

/// Turns each active shield's sprite and offset toward its holder's aim.
fn orient_active_shields(
    mut commands: Commands,
    mut active_shield_query: Query<
        (Entity, &ItemOf, &mut Sprite),
        (With<ActiveShield>, With<Equipped>),
    >,
    holder_query: Query<&Vision>,
) -> Result {
    for (shield_entity, item_of, mut shield_sprite) in &mut active_shield_query {
        let vision = holder_query.get(item_of.holder)?;

        let block_angle = vision.aim_direction.y.atan2(vision.aim_direction.x) + FRAC_PI_2;

        let normalized_angle = if block_angle < -PI {
            block_angle + 2.0 * PI
        } else if block_angle > PI {
            block_angle - 2.0 * PI
        } else {
            block_angle
        };

        let atlas_index = if normalized_angle > -FRAC_PI_4 && normalized_angle < FRAC_PI_4 {
            0
        } else if (-3.0 * FRAC_PI_4..=-FRAC_PI_4).contains(&normalized_angle) {
            2
        } else if (normalized_angle <= -3.0 * FRAC_PI_4) || (normalized_angle >= 3.0 * FRAC_PI_4) {
            3
        } else {
            1
        };

        let offset_distance = 40.0;
        let position_offset = Vec3::new(
            offset_distance * normalized_angle.sin(),
            -offset_distance * normalized_angle.cos(),
            if atlas_index == 0 {
                ZLayer::AboveSprite.z()
            } else {
                ZLayer::BehindSprite.z()
            },
        );

        if let Some(atlas) = &mut shield_sprite.texture_atlas {
            atlas.index = atlas_index;
        }

        commands.entity(shield_entity).insert(Transform::from_xyz(
            position_offset.x,
            position_offset.y,
            position_offset.z,
        ));
    }
    Ok(())
}

/// Raises the shield and begins blocking.
fn on_shield_took_effect(
    used_shield: On<ActionTookEffect>,
    mut commands: Commands,
    mut shield_query: Query<&Shield>,
) {
    let Ok(shield) = shield_query.get_mut(used_shield.entity) else {
        warn!("Tried to block with invalid shield");
        return;
    };

    commands.entity(used_shield.entity).insert((
        ActiveShield,
        shield.hitbox.clone(),
        CollisionLayers::new(GameCollisionLayer::HighObstacle, GameCollisionLayer::InAir),
    ));
}

/// Lowers the shield and restores its resting transform.
fn on_stop_shield(
    shield: On<StopAction>,
    mut commands: Commands,
    holder_query: Query<&FacingDirection>,
    mut shield_query: Query<
        (&mut Sprite, &ItemOf, &Equippable),
        (With<Shield>, With<ActiveShield>),
    >,
) {
    // A shield lowered earlier in the frame, say by an interruption and a drained pool at once,
    // has nothing left to stop.
    let Ok((mut shield_sprite, item_of, equippable)) = shield_query.get_mut(shield.entity) else {
        return;
    };

    let Ok(facing_direction) = holder_query.get(item_of.holder) else {
        warn!("Tried to stop blocking but entity no facing direction");
        return;
    };

    commands
        .entity(shield.entity)
        .remove::<(ActiveShield, Collider, CollisionLayers)>()
        .insert(equippable.poses.transform(*facing_direction));

    if let Some(atlas) = &mut shield_sprite.texture_atlas {
        atlas.index = 0;
    }
}
