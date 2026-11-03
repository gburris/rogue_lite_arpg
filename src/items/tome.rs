use bevy::prelude::*;
use bevy::sprite::Anchor;
use rand::RngExt;

use super::equipment::prelude::DEFAULT_EQUIPMENT_POSES;
use super::{Item, ItemKind};
use crate::prelude::*;

#[derive(Component, Clone)]
struct HealingTome {
    min_healing: f32,
    max_healing: f32,
}

pub fn tome_of_healing(sprites: &SpriteAssets) -> impl Scene {
    let image = sprites.tome_of_healing.clone();
    bsn! {
        Name::new("Tome Of Healing")
        ~Item::new(355, ItemKind::Tome)
        ~Equippable::new(EquipmentSlot::Offhand, 2.0, &DEFAULT_EQUIPMENT_POSES)
        ManaCost(40.0)
        ~{HealingTome {
            min_healing: 25.0,
            max_healing: 50.0,
        }}
        ~Sprite::from_image(image)
        on(on_healing_tome_cast)
    }
}

fn on_healing_tome_cast(
    healing_tome: On<StartUsingEquipment>,
    mut commands: Commands,
    tome_query: Query<(&HealingTome, &ItemOf)>,
    sprites: Res<SpriteAssets>,
    sprite_layouts: Res<SpriteSheetLayouts>,
) -> Result {
    let (tome, _) = tome_query.get(healing_tome.item)?;
    let health_to_add = rand::rng().random_range(tome.min_healing..tome.max_healing);
    commands.trigger(AttemptHeal {
        entity: healing_tome.holder,
        amount: health_to_add,
    });
    let holder = healing_tome.holder;
    commands.spawn_scene(bsn! {
        @heal_tome_vfx(&sprites, &sprite_layouts)
        ChildOf(holder)
    });

    Ok(())
}

/// A burst of healing light over the tome's holder.
fn heal_tome_vfx(sprites: &SpriteAssets, sprite_layouts: &SpriteSheetLayouts) -> impl Scene {
    let sprite = Sprite::from_atlas_image(
        sprites.tome_of_healing_effect.clone(),
        TextureAtlas {
            layout: sprite_layouts.spell_effect.clone(),
            index: 0,
        },
    );
    bsn! {
        sprite
        Anchor(Vec2::new(0.0, 0.10))
        ~{AnimationIndices::OneShot(0..=9)}
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating))
        Lifespan::new(1.0)
    }
}
