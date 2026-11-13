//! Innate abilities. Each is a capability component on its own entity, linked to its character
//! by [`AbilityOf`], plus an applier that re-checks the live world for the brain's held action
//! and reports [`ActionFinished`](crate::character::brain::ActionFinished). A defeated character
//! loses its brain, so appliers never see a dead caller.

use bevy::prelude::*;

use crate::character::brain::{Action, Allies, Brain, CapabilityProfile, TargetInfo, Targeting};
use crate::prelude::*;

/// An innate move a brain can ask its character for, carried out by a capability
/// component in this module on one of the character's [`Abilities`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ability {
    /// Drink a carried potion.
    Heal,
    /// Loose an innate projectile with no weapon.
    Breathe,
    /// Damage a target in contact reach.
    Bite,
    /// Raise the stats of every ally in reach with a war cry.
    Rally,
}

impl Ability {
    /// Every ability, in array order.
    pub(in crate::character::brain) const ALL: [Self; 4] =
        [Self::Heal, Self::Breathe, Self::Bite, Self::Rally];
}

/// Links an ability entity to the character that uses it.
#[derive(Component, Clone)]
#[relationship(relationship_target = Abilities)]
pub struct AbilityOf {
    pub holder: Entity,
}

/// A character's ability entities, each carrying one capability component and whatever it
/// applies or fires.
#[derive(Component, Clone)]
#[relationship_target(relationship = AbilityOf, linked_spawn)]
pub struct Abilities(Vec<Entity>);

/// Drinking carried potions to recover health, paced by `cooldown`.
#[derive(Component, Clone, Copy, Debug)]
pub struct Heal {
    cooldown: f32,
}

impl Heal {
    /// Drinks at most one potion per `cooldown` seconds.
    pub fn new(cooldown: f32) -> Self {
        Self { cooldown }
    }

    /// What a decision reads of this capability.
    pub(in crate::character::brain) fn capability_profile(self) -> CapabilityProfile {
        CapabilityProfile {
            cooldown: self.cooldown,
            ..CapabilityProfile::default()
        }
    }
}

/// An innate projectile attack fired from its ability entity's [`Projectiles`] templates.
#[derive(Component, Clone, Copy, Debug)]
pub struct Breath {
    cooldown: f32,
    reach: f32,
}

impl Breath {
    /// Breathes at most once per `period` seconds at targets within `reach`.
    pub fn new(period: f32, reach: f32) -> Self {
        Self {
            cooldown: period,
            reach,
        }
    }

    /// What a decision reads of this capability.
    pub(in crate::character::brain) fn capability_profile(self) -> CapabilityProfile {
        CapabilityProfile {
            reach: Some(self.reach),
            cooldown: self.cooldown,
            ..CapabilityProfile::default()
        }
    }
}

/// Contact damage on a target within reach.
#[derive(Component, Clone, Copy, Debug)]
pub struct Bite {
    reach: f32,
    damage: f32,
    life_steal: f32,
    cooldown: f32,
}

impl Bite {
    /// Bites for `damage` inside `reach`, at most once per `cooldown` seconds.
    pub fn new(reach: f32, damage: f32, cooldown: f32) -> Self {
        Self {
            reach,
            damage,
            life_steal: 0.0,
            cooldown,
        }
    }

    /// Heals the biter by `fraction` of the damage each landed bite deals.
    pub fn with_life_steal(self, fraction: f32) -> Self {
        Self {
            life_steal: fraction,
            ..self
        }
    }

    /// What a decision reads of this capability.
    pub(in crate::character::brain) fn capability_profile(self) -> CapabilityProfile {
        CapabilityProfile {
            reach: Some(self.reach),
            cooldown: self.cooldown,
            ..CapabilityProfile::default()
        }
    }
}

/// A war cry that gives every other ally within `reach` its ability entity's [`Effects`], at
/// most once per `cooldown` seconds.
#[derive(Component, Clone, Copy, Debug)]
pub struct Rally {
    reach: f32,
    cooldown: f32,
}

impl Rally {
    /// Rallies allies within `reach`, at most once per `cooldown` seconds.
    pub fn new(reach: f32, cooldown: f32) -> Self {
        Self { reach, cooldown }
    }

    /// What a decision reads of this capability.
    pub(in crate::character::brain) fn capability_profile(self) -> CapabilityProfile {
        CapabilityProfile {
            reach: Some(self.reach),
            cooldown: self.cooldown,
            ..CapabilityProfile::default()
        }
    }
}

/// A rally ability whose war cry raises the speed and damage of allies in `reach` by `raise` for
/// `seconds`, at most once per `cooldown` seconds.
pub fn speed_and_damage_rally(reach: f32, cooldown: f32, raise: f32, seconds: f32) -> impl Scene {
    bsn! {
        ~Rally::new(reach, cooldown)
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

/// The first consumable the entity carries, the one check both the decision
/// and `apply_heal` use for whether a potion can be drunk.
pub(in crate::character::brain) fn find_carried_potion(
    items: Option<&Items>,
    consumables: &Query<(), With<Consumable>>,
) -> Option<Entity> {
    items
        .into_iter()
        .flat_map(Items::iter)
        .find(|item| consumables.contains(*item))
}

/// Drinks a carried potion straight from the inventory. The action refreshes
/// at the decision rate but this runs every frame, so the cooldown is what
/// keeps one tick from draining the whole stack.
pub(super) fn apply_heal(
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    heals: Query<&AbilityOf, With<Heal>>,
    drinkers: Query<(&Brain, Option<&Items>)>,
    consumables: Query<(), With<Consumable>>,
) {
    let now = time.elapsed_secs();
    for heal in &heals {
        let entity = heal.holder;
        let Ok((brain, items)) = drinkers.get(entity) else {
            continue;
        };
        let Some(drinking) = brain.held_action(Action::Ability(Ability::Heal)) else {
            continue;
        };
        let potion = find_carried_potion(items, &consumables)
            .filter(|_| brain.is_cooldown_ready(Ability::Heal, now));
        let Some(potion) = potion else {
            drinking.report_not_started(&mut commands, entity);
            continue;
        };
        commands.trigger(Consume {
            entity,
            item: potion,
        });
        drinking.report_completed(&mut commands, entity);
    }
}

/// Fires at a sighted target. `reach` only shapes the decision; a projectile
/// may fly past it.
pub(super) fn apply_breath(
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    breaths: Query<(&AbilityOf, &Projectiles), With<Breath>>,
    breathers: Query<(&Brain, Has<Targeting>, &TargetInfo, &Transform, &CombatSide)>,
) {
    let now = time.elapsed_secs();
    for (breath, templates) in &breaths {
        let entity = breath.holder;
        let Ok((brain, has_target, target_info, transform, side)) = breathers.get(entity) else {
            continue;
        };
        let Some(breathing) = brain.held_action(Action::Ability(Ability::Breathe)) else {
            continue;
        };
        let ready = brain.is_cooldown_ready(Ability::Breathe, now);
        let Some(aim_direction) = target_info
            .nonzero_direction()
            .filter(|_| ready && has_target && target_info.line_of_sight)
        else {
            breathing.report_not_started(&mut commands, entity);
            continue;
        };

        for template in templates.iter() {
            commands.trigger(FireProjectile {
                item: template,
                side: *side,
                holder: Some(entity),
                position: transform.translation.xy(),
                aim_direction,
            });
        }
        breathing.report_completed(&mut commands, entity);
    }
}

/// Bites a target within the true reach; contact needs no line of sight.
pub(super) fn apply_bite(
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    bites: Query<(&Bite, &AbilityOf)>,
    biting_characters: Query<(
        &Brain,
        &Targeting,
        &TargetInfo,
        &StatMultipliers,
        Option<&Health>,
    )>,
) {
    let now = time.elapsed_secs();
    for (bite, ability_of) in &bites {
        let entity = ability_of.holder;
        let Ok((brain, targeting, target_info, stat_multipliers, health)) =
            biting_characters.get(entity)
        else {
            continue;
        };
        let Some(biting) = brain.held_action(Action::Ability(Ability::Bite)) else {
            continue;
        };
        if !(brain.is_cooldown_ready(Ability::Bite, now) && target_info.is_within(bite.reach)) {
            biting.report_not_started(&mut commands, entity);
            continue;
        }
        let damage = bite.damage * stat_multipliers.damage;
        commands.trigger(AttemptDamage {
            entity: targeting.0,
            damage: Damage::Single(damage),
            source_item: None,
            attacker: Some(entity),
            direction: Some(target_info.direction),
            ..default()
        });
        // Skip the heal at full health so the overlay does not pop a zero.
        if bite.life_steal > 0.0 && health.is_some_and(|health| health.hp < health.max_hp) {
            commands.trigger(AttemptHeal {
                entity,
                amount: damage * bite.life_steal,
            });
        }
        biting.report_completed(&mut commands, entity);
    }
}

/// Gives every other ally in reach the rally's [`Effects`] and sends a shockwave out from the
/// caller's feet.
pub(super) fn apply_rally(
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    sprites: Res<SpriteAssets>,
    sprite_layouts: Res<SpriteSheetLayouts>,
    rallies: Query<(&Rally, &Effects, &AbilityOf)>,
    rallyers: Query<&Brain>,
    allies: Allies,
) {
    let now = time.elapsed_secs();
    for (rally, effects, ability_of) in &rallies {
        let entity = ability_of.holder;
        let Ok(brain) = rallyers.get(entity) else {
            continue;
        };
        let Some(rallying) = brain.held_action(Action::Ability(Ability::Rally)) else {
            continue;
        };
        if !brain.is_cooldown_ready(Ability::Rally, now) {
            rallying.report_not_started(&mut commands, entity);
            continue;
        }
        for ally in allies.within(entity, rally.reach) {
            commands.trigger(ApplyEffects::new(effects, ally.entity));
        }
        commands.spawn_scene(bsn! {
            @rally_shout_vfx(&sprites, &sprite_layouts)
            ChildOf(entity)
        });
        rallying.report_completed(&mut commands, entity);
    }
}

/// Scale of the shockwave sheet, whose ring spans 174 px at its widest.
const RALLY_SHOUT_SCALE: f32 = 2.0;

/// A one-shot shockwave ring racing out along the ground from the caller's feet.
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
