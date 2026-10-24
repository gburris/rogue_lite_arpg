//! Sense systems: the only code that turns world state into the pure
//! [`DecisionContext`]. They are read-only over the world; each writes a
//! disjoint set of [`Perception`] fields, so every field has exactly one
//! writer.

use bevy::prelude::*;

use super::motion::{ANCHOR_ARRIVAL_RADIUS, ManeuversReadOnly};
use super::{Brain, DecisionContext, ManaRecovery, Maneuver, MotionMode, SelfState, TargetView};
use crate::{
    character::{
        ability::AbilityCapabilities,
        anchor::Anchor,
        vision::{TargetInfo, Targeting},
    },
    prelude::{AttackState, EquipmentRead, EquipmentSlot, Health, Mana},
};

/// Per-tick sensed state for one brain, written by the sense systems and read
/// by `decide_behavior`. Keeps the pure [`DecisionContext`] as the shared
/// staging copy so each sense owns only its fields.
#[derive(Component, Clone, Copy, Debug, Default, Deref, DerefMut)]
pub(super) struct Perception {
    #[deref]
    pub(super) context: DecisionContext,
    /// Attacks finished as of the last decision tick; compared against
    /// `AttackState::attacks_finished` to notice a new finish.
    attacks_seen: u32,
}

/// Builds the pure view of a target from the vision system's observation.
pub(super) fn target_view(has_target: bool, info: Option<&TargetInfo>) -> Option<TargetView> {
    if !has_target {
        return None;
    }
    info.and_then(|info| {
        info.distance.is_finite().then(|| TargetView {
            distance: info.distance,
            line_of_sight: info.line_of_sight,
            bearing: info.direction,
            velocity: info.target_velocity,
            tangential_velocity: info.target_velocity.dot(info.direction.perp()),
        })
    })
}

fn slot_readiness(equipment: &EquipmentRead, entity: Entity) -> [bool; EquipmentSlot::ALL.len()] {
    let mut slot_ready = [false; EquipmentSlot::ALL.len()];
    for slot in EquipmentSlot::ALL {
        slot_ready[slot as usize] = equipment.can_use(entity, slot).is_ok();
    }
    slot_ready
}

fn current_calm_phase(brain: &Brain, now: f32) -> MotionMode {
    let phase = brain.calm_phase.unwrap_or(MotionMode::Idle);
    if brain.mode.is_calm() && brain.committed_until > 0.0 && now >= brain.committed_until {
        brain.mode.next_calm()
    } else {
        phase
    }
}

/// Writes target presence.
pub(super) fn sense_target(
    mut targets: Query<(Has<Targeting>, Option<&TargetInfo>, &mut Perception)>,
) {
    for (has_target, info, mut perception) in &mut targets {
        perception.target = target_view(has_target, info);
    }
}

/// Writes health and mana fractions.
pub(super) fn sense_vitals(
    mut vitals: Query<(Option<&Health>, Option<&Mana>, &mut Perception), With<Brain>>,
) {
    for (health, mana, mut perception) in &mut vitals {
        perception.health_fraction = health.and_then(Health::fraction).unwrap_or(1.0);
        perception.mana_fraction = mana.and_then(Mana::fraction).unwrap_or(1.0);
    }
}

/// Writes equipment readiness plus anchor-relative fields.
pub(super) fn sense_equipment_and_anchor(
    equipment: EquipmentRead,
    mut sensed: Query<(Entity, Option<&Anchor>, &Transform, &mut Perception), With<Brain>>,
) {
    for (entity, anchor, transform, mut perception) in &mut sensed {
        perception.slot_ready = slot_readiness(&equipment, entity);
        perception.anchor_overshoot_fraction =
            anchor.and_then(|anchor| anchor.overshoot_fraction(transform.translation.xy()));
        perception.anchor_arrived = anchor.map(|anchor| {
            anchor.distance_from(transform.translation.xy()) <= ANCHOR_ARRIVAL_RADIUS
        });
    }
}

/// Writes the calm-cycle phase and whether an attack finished since the last
/// decision tick.
pub(super) fn sense_calm_and_attack(
    time: Res<Time<Virtual>>,
    mut sensed: Query<(&Brain, Option<&AttackState>, &mut Perception)>,
) {
    let now = time.elapsed_secs();
    for (brain, attack_state, mut perception) in &mut sensed {
        perception.self_state.calm_phase = current_calm_phase(brain, now);
        let finished = attack_state.map_or(0, |state| state.attacks_finished);
        perception.attack_finished = finished != perception.attacks_seen;
        perception.attacks_seen = finished;
    }
}

/// Writes the rest of [`SelfState`]: the capability components' tuning, which
/// maneuvers are ready, and the held mana break. The calm phase is owned by
/// `sense_calm_and_attack` and left untouched.
pub(super) fn sense_self(
    time: Res<Time<Virtual>>,
    mut brains: Query<(
        &Brain,
        ManeuversReadOnly,
        AbilityCapabilities,
        Option<&ManaRecovery>,
        &mut Perception,
    )>,
) {
    let now = time.elapsed_secs();
    for (brain, maneuvers, abilities, recovery, mut perception) in &mut brains {
        let maneuver_tuning = maneuvers.tuning();
        let maneuver_ready = Maneuver::ALL.map(|maneuver| {
            maneuver_tuning[maneuver as usize].is_some() && brain.maneuver_ready(maneuver, now)
        });
        perception.self_state = SelfState {
            calm_phase: perception.self_state.calm_phase,
            locomotion: brain.mode,
            mana_recovering: recovery.is_some_and(ManaRecovery::recovering),
            active_maneuver: brain.maneuver(),
            maneuver_tuning,
            maneuver_ready,
            ability_tuning: abilities.tuning(),
        };
    }
}
