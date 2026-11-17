use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};

use avian2d::prelude::*;
use bevy::prelude::*;

use super::equipment::prelude::DEFAULT_EQUIPMENT_POSES;
use crate::prelude::*;

/// Distance from its holder at which a raised shield sits.
const RAISED_SHIELD_DISTANCE: f32 = 40.0;
/// Seconds a shield waits after it is raised before it can be raised again.
const SHIELD_COOLDOWN: f32 = 0.5;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        orient_raised_shields.in_set(SimulationSystems::Derive),
    );
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
    bsn! {
        @atlas_sprite(image, sprite_layouts.shield.clone())
        ~Item::new(355, ItemKind::Shield)
        ~Equippable::new(EquipmentSlot::Offhand, &DEFAULT_EQUIPMENT_POSES)
        ~ActionCooldown::new(SHIELD_COOLDOWN)
        ~{ActionUseStyle::Sustained}
        ~{Shield {
            hitbox: Collider::rectangle(25.0, 25.0),
        }}
        on(on_shield_took_effect)
        on(on_shield_finished)
    }
}

/// An offhand item that blocks with `hitbox` while raised.
#[derive(Component, Clone)]
pub struct Shield {
    pub hitbox: Collider,
}

/// Turns each raised shield's sprite and offset toward its holder's aim.
fn orient_raised_shields(
    mut raised_shield_query: Query<
        (&ItemOf, &mut Sprite, &mut Transform),
        (With<Shield>, With<ActionInProgress>),
    >,
    holder_query: Query<&Vision>,
) {
    for (item_of, mut shield_sprite, mut transform) in &mut raised_shield_query {
        let Ok(vision) = holder_query.get(item_of.holder) else {
            continue;
        };
        // Measured from straight down, wrapped into -PI..PI.
        let block_angle =
            (vision.aim_direction.to_angle() + FRAC_PI_2 + PI).rem_euclid(2.0 * PI) - PI;

        let atlas_index = if block_angle > -FRAC_PI_4 && block_angle < FRAC_PI_4 {
            0
        } else if (-3.0 * FRAC_PI_4..=-FRAC_PI_4).contains(&block_angle) {
            2
        } else if (block_angle <= -3.0 * FRAC_PI_4) || (block_angle >= 3.0 * FRAC_PI_4) {
            3
        } else {
            1
        };
        let z = if atlas_index == 0 {
            ZLayer::AboveSprite.z()
        } else {
            ZLayer::BehindSprite.z()
        };

        if let Some(atlas) = &mut shield_sprite.texture_atlas {
            atlas.index = atlas_index;
        }
        *transform = Transform::from_xyz(
            RAISED_SHIELD_DISTANCE * block_angle.sin(),
            -RAISED_SHIELD_DISTANCE * block_angle.cos(),
            z,
        );
    }
}

/// Raises the shield and begins blocking with a blocker spawned on it for the length of the use.
fn on_shield_took_effect(
    took_effect: On<ActionTookEffect>,
    mut commands: Commands,
    shield_query: Query<(&Shield, Has<ProjectileReflection>)>,
) -> Result {
    let shield_entity = took_effect.source;
    let (shield, reflects_projectiles) = shield_query.get(shield_entity)?;
    let mut blocker = commands.spawn((
        shield.hitbox.clone(),
        CollisionLayers::new(GameCollisionLayer::HighObstacle, GameCollisionLayer::InAir),
        Transform::default(),
        ChildOf(shield_entity),
        SpawnedForUse {
            source: shield_entity,
        },
    ));
    if reflects_projectiles {
        blocker.insert(ProjectileReflection::default());
    }
    Ok(())
}

/// Lowers the shield. Its resting pose comes back with the rest of the holder's equipment once
/// the use ends.
fn on_shield_finished(
    finished: On<ActionFinished>,
    mut shield_query: Query<&mut Sprite, With<Shield>>,
) {
    let Ok(mut shield_sprite) = shield_query.get_mut(finished.source) else {
        return;
    };
    if let Some(atlas) = &mut shield_sprite.texture_atlas {
        atlas.index = 0;
    }
}
