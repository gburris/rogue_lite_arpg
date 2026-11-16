use bevy::prelude::*;

use super::{Ability, AbilityCooldown, AbilityOf, AbilityTookEffect};
use crate::character::Allies;
use crate::prelude::*;

/// Scale of the shockwave sheet, whose ring spans 174 px at its widest.
const RALLY_SHOUT_SCALE: f32 = 2.0;

/// A war cry that gives every other ally within `reach` its ability entity's [`Effects`].
#[derive(Component, Clone, Copy, Debug)]
#[require(Ability::Rally, AbilityCooldown)]
pub struct Rally {
    reach: f32,
}

impl Rally {
    /// Rallies allies within `reach`.
    pub fn new(reach: f32) -> Self {
        Self { reach }
    }

    pub(super) fn reach(self) -> f32 {
        self.reach
    }
}

/// A [`Rally`] ability whose war cry raises the speed and damage of allies within `reach` by
/// `raise` for `seconds`.
pub fn speed_and_damage_rally(reach: f32, raise: f32, seconds: f32) -> impl Scene {
    bsn! {
        ~Rally::new(reach)
        on(on_rally_took_effect)
        Effects [
            Rallied
            Lifespan::new(seconds)
            --
            @stat_modifier(Stat::Speed, raise)
            Lifespan::new(seconds)
            --
            @stat_modifier(Stat::Damage, raise)
            Lifespan::new(seconds)
        ]
    }
}

/// Gives every other ally in reach the rally's [`Effects`] and sends a shockwave out from the
/// holder's feet.
fn on_rally_took_effect(
    took_effect: On<AbilityTookEffect>,
    mut commands: Commands,
    sprites: Res<SpriteAssets>,
    sprite_layouts: Res<SpriteSheetLayouts>,
    rallies: Query<(&Rally, &Effects, &AbilityOf)>,
    allies: Allies,
) {
    let Ok((rally, effects, ability_of)) = rallies.get(took_effect.entity) else {
        return;
    };
    let holder = ability_of.holder;
    for ally in allies.within(holder, rally.reach) {
        commands.trigger(ApplyEffects::new(effects, ally.entity));
    }
    commands.spawn_scene(bsn! {
        @rally_shout_vfx(&sprites, &sprite_layouts)
        ChildOf(holder)
    });
}

/// A one-shot shockwave ring racing out along the ground from the holder's feet.
fn rally_shout_vfx(sprites: &SpriteAssets, sprite_layouts: &SpriteSheetLayouts) -> impl Scene {
    let sprite = Sprite::from_atlas_image(
        sprites.rally_shout.clone(),
        TextureAtlas {
            layout: sprite_layouts.rally_shout.clone(),
            index: 0,
        },
    );
    bsn! {
        template(move |_| Ok(sprite.clone()))
        ~{AnimationIndices::OneShot(0..=7)}
        AnimationTimer(Timer::from_seconds(0.071, TimerMode::Repeating))
        Lifespan::new(0.57)
        Transform {
            translation: Vec3::new(0.0, CHARACTER_FEET_POS_OFFSET, {ZLayer::BehindSprite.z()}),
            scale: Vec3::new(RALLY_SHOUT_SCALE, RALLY_SHOUT_SCALE, 1.0),
        }
    }
}
