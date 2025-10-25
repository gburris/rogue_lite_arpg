use bevy::{prelude::*, sprite::Anchor, ui_widgets::observe};
use rand::Rng;

use crate::{
    combat::{health::AttemptHeal, mana::ManaCost},
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
        Equippable::from(2.0, EquipmentSlot::Offhand),
        ManaCost(40.0),
        HealingTome {
            healing: (25.0, 50.0),
        },
        Sprite::from_image(sprites.tome_of_healing.clone()),
        observe(on_healing_tome_cast),
    )
}

fn on_healing_tome_cast(
    use_healing_tome: On<UseEquipment>,
    mut commands: Commands,
    tome_query: Query<&HealingTome>,
    sprites: Res<SpriteAssets>,
    sprite_layouts: Res<SpriteSheetLayouts>,
) {
    let tome_entity = use_healing_tome.entity;
    let holder_entity = use_healing_tome.holder;

    let Ok(tome) = tome_query.get(tome_entity) else {
        warn!("Tried to use a tome that does not exist");
        return;
    };

    let health_to_add = rand::rng().random_range(tome.healing.0..tome.healing.1);
    commands.trigger(AttemptHeal {
        entity: holder_entity,
        amount: health_to_add,
    });
    commands
        .entity(holder_entity)
        .with_child(heal_tome_vfx(sprites, sprite_layouts));
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
