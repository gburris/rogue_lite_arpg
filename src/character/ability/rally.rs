use bevy::prelude::*;

use super::Ability;
use crate::character::Allies;
use crate::prelude::*;

/// Radius of the shockwave sheet's ring at its widest, in sheet pixels.
const RALLY_SHOUT_RING_RADIUS: f32 = 87.0;

/// A war cry that gives every other ally within the ability's [`ActionReach`] its ability entity's
/// [`Effects`].
#[derive(Component, Clone, Copy, Debug, Default)]
#[require(Ability::Rally)]
pub struct Rally;

/// A [`Rally`] ability whose war cry raises the speed and damage of allies within `reach` by
/// `raise` for `seconds`.
pub fn speed_and_damage_rally(reach: f32, raise: f32, seconds: f32) -> impl Scene {
    bsn! {
        Rally
        ~{ActionReach(reach)}
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
    took_effect: On<ActionTookEffect>,
    mut commands: Commands,
    sprites: Res<SpriteAssets>,
    sprite_layouts: Res<SpriteSheetLayouts>,
    rallies: Query<(&Effects, &ActionReach), With<Rally>>,
    allies: Allies,
) -> Result {
    let (effects, &ActionReach(reach)) = rallies.get(took_effect.source)?;
    let holder = took_effect.holder;
    for ally in allies.within(holder, reach) {
        commands.trigger(ApplyEffects::new(effects, ally.entity));
    }
    commands.spawn_scene(bsn! {
        @rally_shout_vfx(&sprites, &sprite_layouts, reach)
        ChildOf(holder)
    });
    Ok(())
}

/// A one-shot shockwave ring racing out along the ground from the holder's feet to `reach`.
fn rally_shout_vfx(
    sprites: &SpriteAssets,
    sprite_layouts: &SpriteSheetLayouts,
    reach: f32,
) -> impl Scene {
    let scale = reach / RALLY_SHOUT_RING_RADIUS;
    bsn! {
        @one_shot_vfx(sprites.rally_shout.clone(), sprite_layouts.rally_shout.clone(), 8, 0.071)
        Transform {
            translation: Vec3::new(0.0, CHARACTER_FEET_POS_OFFSET, {ZLayer::BehindSprite.z()}),
            scale: Vec3::new(scale, scale, 1.0),
        }
    }
}
