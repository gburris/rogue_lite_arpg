use avian2d::prelude::*;
use bevy::prelude::*;
use bevy::sprite::Anchor;

use super::equipment::DEFAULT_EQUIPMENT_POSES;
use crate::interval::Interval;
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
        ~Cooldown::new(SHIELD_COOLDOWN)
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
        let block_direction = vision.aim_direction.try_normalize().unwrap_or(Vec2::X);
        let facing = FacingDirection::from_nearest_axis(block_direction);
        let atlas_index = match facing {
            FacingDirection::Down => 0,
            FacingDirection::Right => 1,
            FacingDirection::Left => 2,
            FacingDirection::Up => 3,
        };
        let z_layer = match facing {
            FacingDirection::Down => ZLayer::AboveSprite,
            FacingDirection::Up | FacingDirection::Left | FacingDirection::Right => {
                ZLayer::BehindSprite
            }
        };

        if let Some(atlas) = &mut shield_sprite.texture_atlas {
            atlas.index = atlas_index;
        }
        *transform = Transform::from_translation(
            (block_direction * RAISED_SHIELD_DISTANCE).extend(z_layer.z()),
        );
    }
}

/// Raises the shield and spawns its blocker for the duration of the use.
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
        PartOfActionUse {
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

/// Seconds a tome waits after a heal before it can heal again.
const TOME_COOLDOWN: f32 = 2.0;
/// Mana each tome heal costs.
const TOME_MANA_COST: f32 = 40.0;

#[derive(Component, Clone)]
struct HealingTome {
    healing: Interval,
}

pub fn tome_of_healing(sprites: &SpriteAssets) -> impl Scene {
    let image = sprites.tome_of_healing.clone();
    bsn! {
        Name::new("Tome Of Healing")
        ~Item::new(355, ItemKind::Tome)
        ~Equippable::new(EquipmentSlot::Offhand, &DEFAULT_EQUIPMENT_POSES)
        ~Cooldown::new(TOME_COOLDOWN)
        ManaCost(TOME_MANA_COST)
        ~{HealingTome {
            healing: Interval::new(25.0, 50.0),
        }}
        Sprite { image }
        on(on_healing_tome_took_effect)
    }
}

fn on_healing_tome_took_effect(
    took_effect: On<ActionTookEffect>,
    mut commands: Commands,
    tome_query: Query<&HealingTome>,
    sprites: Res<SpriteAssets>,
    sprite_layouts: Res<SpriteSheetLayouts>,
) -> Result {
    let tome = tome_query.get(took_effect.source)?;
    let health_to_add = tome.healing.pick_random_value();
    commands.trigger(AttemptHeal {
        entity: took_effect.holder,
        amount: health_to_add,
    });
    let holder = took_effect.holder;
    commands.spawn_scene(bsn! {
        @heal_tome_vfx(&sprites, &sprite_layouts)
        ChildOf(holder)
    });

    Ok(())
}

/// A burst of healing light over the tome's holder.
fn heal_tome_vfx(sprites: &SpriteAssets, sprite_layouts: &SpriteSheetLayouts) -> impl Scene {
    let image = sprites.tome_of_healing_effect.clone();
    bsn! {
        @one_shot_vfx(image, sprite_layouts.spell_effect.clone(), 10, 0.1)
        Anchor(Vec2::new(0.0, 0.10))
    }
}
