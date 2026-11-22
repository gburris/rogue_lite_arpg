use bevy::color::palettes::tailwind::{BLUE_400, YELLOW_300};
use bevy::ecs::entity_disabling::Disabled;
use bevy::prelude::*;

use super::stat_modifier::{Stat, StatModifier};
use crate::combat::damage::{AttemptDamage, Damage};
use crate::prelude::*;

#[derive(Component, Clone)]
#[relationship(relationship_target = Statuses)]
#[require(Lifespan)]
pub struct StatusOf(pub Entity);

#[derive(Component, Clone)]
#[relationship_target(relationship = StatusOf, linked_spawn)]
pub struct Statuses(Vec<Entity>);

#[derive(Component, Clone)]
pub(super) struct StatusApplied;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum StatusKind {
    Burn,
    Freeze,
    Rally,
    StatRaise { stat: Stat },
    StatLower { stat: Stat },
}

#[derive(Component, FromTemplate, Clone)]
#[relationship(relationship_target = StatusVisualEffects)]
struct StatusVfxOf(pub Entity);

#[derive(Component, Clone)]
#[relationship_target(relationship = StatusVfxOf, linked_spawn)]
struct StatusVisualEffects(Vec<Entity>);

#[derive(Component, Clone)]
#[require(Disabled, Lifespan)]
#[relationship(relationship_target = Effects)]
pub struct EffectOf(Entity);

#[derive(Component, Clone, Debug)]
#[relationship_target(relationship = EffectOf, linked_spawn)]
pub struct Effects(Vec<Entity>);

#[derive(Event)]
pub struct ApplyEffects {
    effects: Vec<Entity>,
    target: Entity,
}

impl ApplyEffects {
    pub fn new(effects: &Effects, target: Entity) -> Self {
        Self {
            effects: effects.0.clone(),
            target,
        }
    }
}

/// Applies each effect to the target as a status, one per [`StatusKind`] with the first in the
/// list winning. A new status replaces the target's existing one of its kind, except that an
/// existing [`StatModifier`] stays when the new one does not [outrank](StatModifier::outranks) it.
pub(super) fn on_apply_effects(
    apply_effects: On<ApplyEffects>,
    mut commands: Commands,
    effect_query: Query<
        (&StatusKind, Option<&StatModifier>, &Lifespan),
        (Without<StatusOf>, Allow<Disabled>),
    >,
    affected_query: Query<&Statuses>,
    status_query: Query<(&StatusKind, Option<&StatModifier>, &Lifespan), With<StatusOf>>,
) {
    let target = apply_effects.target;
    let statuses = affected_query.get(target).ok();
    let mut seen_kinds = Vec::new();
    for &effect in &apply_effects.effects {
        let Ok((kind, modifier, lifespan)) = effect_query.get(effect) else {
            warn!("Entity {effect:?} missing StatusKind component, skipping it");
            continue;
        };
        if seen_kinds.contains(kind) {
            continue;
        }
        seen_kinds.push(*kind);

        let existing = statuses
            .into_iter()
            .flat_map(Statuses::iter)
            .find_map(|status| {
                let (existing_kind, existing_modifier, existing_lifespan) =
                    status_query.get(status).ok()?;
                (existing_kind == kind).then_some((status, existing_modifier, existing_lifespan))
            });
        if let Some((status, existing_modifier, existing_lifespan)) = existing {
            if let (Some(modifier), Some(existing_modifier)) = (modifier, existing_modifier)
                && !modifier.outranks(
                    lifespan.remaining_secs(),
                    *existing_modifier,
                    existing_lifespan.remaining_secs(),
                )
            {
                continue;
            }
            commands.entity(status).despawn();
        }

        debug!("Applying effect {effect:?} to {target:?}");
        commands
            .entity(effect)
            .clone_and_spawn()
            .remove::<(Disabled, EffectOf)>()
            .insert(StatusOf(target));
    }
}

#[derive(Component, Clone)]
#[require(StatusKind::Burn)]
pub struct Burning {
    damage: f32,
    damage_frequency: Timer,
}

impl Default for Burning {
    fn default() -> Self {
        Self {
            damage: 2.0,
            damage_frequency: Timer::from_seconds(0.7, TimerMode::Repeating),
        }
    }
}

pub(super) fn tick_burn(mut burn_query: Query<&mut Burning, With<StatusApplied>>, time: Res<Time>) {
    for mut burn_status in &mut burn_query {
        burn_status.damage_frequency.tick(time.delta());
    }
}

// TODO: Modify this to be a "DamagePerSecond" component + system since it isn't specific to burning
pub(super) fn while_burning(
    mut commands: Commands,
    status_query: Query<(&Burning, &StatusOf), With<StatusApplied>>,
    mut health_query: Query<Entity, With<Health>>,
) {
    for (burn, status_of) in &status_query {
        if let Ok(entity) = health_query.get_mut(status_of.0)
            && burn.damage_frequency.just_finished()
        {
            commands.trigger(AttemptDamage {
                entity,
                ignore_invulnerable: true,
                damage: Damage::Single(burn.damage),
                ..default()
            });
        }
    }
}

pub(super) fn apply_burning(
    mut commands: Commands,
    status_query: Query<(Entity, &StatusOf), (With<Burning>, Without<StatusApplied>)>,
    sprites: Res<SpriteAssets>,
    sprite_layouts: Res<SpriteSheetLayouts>,
) {
    for (status, status_of) in &status_query {
        commands.entity(status).insert(StatusApplied);

        let target = status_of.0;
        commands.spawn_scene(bsn! {
            @burn_vfx(&sprites, &sprite_layouts, status)
            ChildOf(target)
        });
    }
}

/// Flames licking up around the burning character.
fn burn_vfx(
    sprites: &SpriteAssets,
    sprite_layouts: &SpriteSheetLayouts,
    status: Entity,
) -> impl Scene {
    bsn! {
        @looping_vfx(sprites.flame.clone(), sprite_layouts.flame_vfx.clone(), 8, 0.1)
        StatusVfxOf(status)
        Transform {
            translation: Vec3::new(0.0, {CHARACTER_FEET_POS_OFFSET + 8.0}, {ZLayer::SpriteForeground.z()}),
            scale: Vec3::new(1.2, 1.2, 1.0),
        }
        @point_light(Color::from(YELLOW_300), 2.0, 150.0, 10.0)
    }
}

/// Ice under a frozen character's feet. Only the look: the effect list that freezes carries its
/// speed [`StatModifier`].
#[derive(Component, Clone, Default)]
#[require(StatusKind::Freeze)]
pub struct Frozen;

pub(super) fn apply_frozen(
    mut commands: Commands,
    status_query: Query<(Entity, &StatusOf), (With<Frozen>, Without<StatusApplied>)>,
    sprites: Res<SpriteAssets>,
) {
    for (status, status_of) in &status_query {
        commands.entity(status).insert(StatusApplied);
        let target = status_of.0;
        commands.spawn_scene(bsn! {
            @grounded_ice_vfx(&sprites, status)
            ChildOf(target)
        });
    }
}

/// A sheet of ice under the frozen character's feet.
fn grounded_ice_vfx(sprites: &SpriteAssets, status: Entity) -> impl Scene {
    let image = sprites.grounded_ice.clone();
    bsn! {
        Sprite { image }
        StatusVfxOf(status)
        @point_light(Color::from(BLUE_400), 1.4, 80.0, 5.0)
        Transform {
            translation: Vec3::new(0.0, {CHARACTER_FEET_POS_OFFSET + 16.0}, {ZLayer::SpriteForeground.z()}),
            scale: Vec3::new(1.4, 1.4, 1.0),
        }
    }
}

/// Rallied by a war cry. Only the look: the effect list that rallies carries its
/// [`StatModifier`]s.
#[derive(Component, Clone, Copy, Debug, Default)]
#[require(StatusKind::Rally)]
pub struct Rallied;

/// Marks a rallied character with rising arrows at its feet for as long as the status lasts.
pub(super) fn apply_rallied(
    mut commands: Commands,
    status_query: Query<(Entity, &StatusOf), (With<Rallied>, Without<StatusApplied>)>,
    sprites: Res<SpriteAssets>,
    sprite_layouts: Res<SpriteSheetLayouts>,
) {
    for (status, status_of) in &status_query {
        commands.entity(status).insert(StatusApplied);
        let target = status_of.0;
        commands.spawn_scene(bsn! {
            @rallied_aura_vfx(&sprites, &sprite_layouts, status)
            ChildOf(target)
        });
    }
}

fn rallied_aura_vfx(
    sprites: &SpriteAssets,
    sprite_layouts: &SpriteSheetLayouts,
    status: Entity,
) -> impl Scene {
    bsn! {
        @looping_vfx(sprites.rallied_aura.clone(), sprite_layouts.rallied_aura.clone(), 12, 0.083)
        StatusVfxOf(status)
        // The sheet's feet sit 14 px below its centre.
        Transform::from_xyz(0.0, {CHARACTER_FEET_POS_OFFSET + 14.0}, {ZLayer::SpriteForeground.z()})
    }
}
