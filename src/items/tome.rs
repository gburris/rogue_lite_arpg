use bevy::prelude::*;
use bevy::sprite::Anchor;
use rand::RngExt;

use super::equipment::prelude::DEFAULT_EQUIPMENT_POSES;
use super::{Item, ItemKind};
use crate::prelude::*;

/// Seconds a tome waits after a heal before it can heal again.
const TOME_COOLDOWN: f32 = 2.0;
/// Mana each tome heal costs.
const TOME_MANA_COST: f32 = 40.0;

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
        ~Equippable::new(EquipmentSlot::Offhand, &DEFAULT_EQUIPMENT_POSES)
        ~ActionCooldown::new(TOME_COOLDOWN)
        ManaCost(TOME_MANA_COST)
        ~{HealingTome {
            min_healing: 25.0,
            max_healing: 50.0,
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
    let health_to_add = rand::rng().random_range(tome.min_healing..tome.max_healing);
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
