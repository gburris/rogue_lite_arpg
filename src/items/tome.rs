use bevy::{prelude::*, sprite::Anchor, ui_widgets::observe};
use rand::Rng;

use crate::{
    items::{
        Item, ItemType,
        equipment::{EquipmentSlot, Equippable},
        prelude::UseEquipment,
    },
    prelude::*,
};

#[derive(Component)]
struct HealingTome {
    pub healing: (f32, f32),
}

pub fn tome_of_healing(sprites: &SpriteAssets) -> impl Bundle {
    (
        Name::new("Tome Of Healing"),
        Item::new(355, ItemType::Tome),
        Equippable::new(
            EquipmentSlot::Offhand,
            2.0,
            &DEFAULT_EQUIPMENT_TRANSFORM_MAP,
        ),
        ManaCost(40.0),
        HealingTome {
            healing: (25.0, 50.0),
        },
        Sprite::from_image(sprites.tome_of_healing.clone()),
        observe(on_healing_tome_cast),
    )
}

fn on_healing_tome_cast(
    healing_tome: On<UseEquipment>,
    mut commands: Commands,
    tome_query: Query<(&HealingTome, &ItemOf)>,
    sprites: Res<SpriteAssets>,
    sprite_layouts: Res<SpriteSheetLayouts>,
) -> Result {
    let (tome, item_of) = tome_query.get(healing_tome.entity)?;

    let health_to_add = rand::rng().random_range(tome.healing.0..tome.healing.1);
    commands.trigger(AttemptHeal {
        entity: item_of.0,
        amount: health_to_add,
    });
    commands
        .entity(item_of.0)
        .with_child(heal_tome_vfx(sprites, sprite_layouts));

    Ok(())
}

fn heal_tome_vfx(
    sprites: Res<SpriteAssets>,
    sprite_layouts: Res<SpriteSheetLayouts>,
) -> impl Bundle {
    (
        Sprite {
            image: sprites.tome_of_healing_effect.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: sprite_layouts.spell_effect.clone(),
                index: 0,
            }),
            ..default()
        },
        Anchor(Vec2::new(0.0, 0.10)),
        AnimationIndices::OneShot(0..=9),
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        Lifespan::new(1.0),
    )
}
