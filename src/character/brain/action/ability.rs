//! Innate abilities. Each is an opt-in capability component plus an applier that re-checks the
//! live world for the brain's held action and reports
//! [`ActionFinished`](crate::character::brain::ActionFinished). A defeated character loses its
//! brain, so appliers never see the dead.

use bevy::prelude::*;

use crate::character::brain::{Action, Brain, CapabilityProfile, TargetInfo, Targeting};
use crate::prelude::*;

/// An innate move a brain can ask its character for, carried out by a capability
/// component in this module.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ability {
    /// Drink a carried potion.
    Heal,
    /// Loose an innate projectile with no weapon.
    Breathe,
    /// Damage a target in contact reach.
    Bite,
}

impl Ability {
    /// Every ability, in array order.
    pub(in crate::character::brain) const ALL: [Self; 3] = [Self::Heal, Self::Breathe, Self::Bite];
}

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

/// An innate projectile attack fired from the entity's `Projectiles` template.
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
    drinkers: Query<(Entity, &Brain, Option<&Items>), With<Heal>>,
    consumables: Query<(), With<Consumable>>,
) {
    let now = time.elapsed_secs();
    for (entity, brain, items) in &drinkers {
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
    breathers: Query<(
        Entity,
        &Brain,
        Has<Targeting>,
        &TargetInfo,
        &Transform,
        &CombatSide,
        &Projectiles,
    )>,
) {
    let now = time.elapsed_secs();
    for (entity, brain, has_target, target_info, transform, side, templates) in &breathers {
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
    biters: Query<(
        Entity,
        &Brain,
        &Bite,
        &Targeting,
        &TargetInfo,
        Option<&Health>,
    )>,
) {
    let now = time.elapsed_secs();
    for (entity, brain, bite, targeting, target_info, health) in &biters {
        let Some(biting) = brain.held_action(Action::Ability(Ability::Bite)) else {
            continue;
        };
        if !(brain.is_cooldown_ready(Ability::Bite, now) && target_info.is_within(bite.reach)) {
            biting.report_not_started(&mut commands, entity);
            continue;
        }
        commands.trigger(AttemptDamage {
            entity: targeting.0,
            damage: Damage::Single(bite.damage),
            source_item: None,
            attacker: Some(entity),
            direction: Some(target_info.direction),
            ..default()
        });
        // Skip the heal at full health so the overlay does not pop a zero.
        if bite.life_steal > 0.0 && health.is_some_and(|health| health.hp < health.max_hp) {
            commands.trigger(AttemptHeal {
                entity,
                amount: bite.damage * bite.life_steal,
            });
        }
        biting.report_completed(&mut commands, entity);
    }
}
