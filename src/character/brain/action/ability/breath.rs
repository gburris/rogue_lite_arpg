use bevy::prelude::*;

use crate::character::brain::Capability;
use crate::{
    character::{
        brain::{Ability, Action, Brain, Tuning, perception::Targeting},
        vision::TargetInfo,
    },
    prelude::*,
};

/// An innate projectile attack fired from the entity's `Projectiles` template.
#[derive(Component, Debug)]
pub struct Breath {
    pub cooldown: f32,
    pub reach: f32,
}

impl Breath {
    /// Breathes at most once per `period` seconds at targets within `reach`.
    pub fn new(period: f32, reach: f32) -> Self {
        Self {
            cooldown: period,
            reach,
        }
    }
}

impl Capability for Breath {
    type Key = Ability;

    fn key(&self) -> Ability {
        Ability::Breathe
    }

    fn tuning(&self) -> Tuning {
        Tuning {
            reach: Some(self.reach),
            cooldown: self.cooldown,
            ..Tuning::default()
        }
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
        // Sitting on the target leaves a zero bearing to fire along, so wait.
        let aim_direction = target_info.direction;
        let ready = brain.is_cooldown_ready(Ability::Breathe, now);
        if !(ready
            && has_target
            && target_info.line_of_sight
            && aim_direction.length_squared() > f32::EPSILON)
        {
            breathing.report_not_started(&mut commands, entity);
            continue;
        }

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
