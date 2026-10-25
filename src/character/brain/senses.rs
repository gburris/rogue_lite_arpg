//! The sense system: the only code that turns world state into the pure
//! [`DecisionContext`]. It is read-only over the world and rewrites each
//! brain's [`Perception`] once per decision tick.

use bevy::{ecs::query::QueryData, prelude::*};

use super::ability::AbilityCapabilities;
use super::motion::{ANCHOR_ARRIVAL_RADIUS, FiniteMotionsReadOnly};
use super::{Ability, Action, Brain, Cooldown, DecisionContext, ManaRecovery, Motion, TargetView, Tuning};
use crate::{
    character::{
        anchor::Anchor,
        vision::{TargetInfo, Targeting},
    },
    prelude::{AttackState, EquipmentRead, EquipmentSlot, Health, Mana},
};

/// Per-tick sensed state for one brain, written by [`sense`] and read by
/// `decide_behavior`.
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

/// Which finite motions are owned and off cooldown at `now`.
pub(super) fn motion_readiness(
    brain: &Brain,
    motion_tuning: &[Option<Tuning>; Motion::COUNT],
    now: f32,
) -> [bool; Motion::COUNT] {
    let mut motion_ready = [false; Motion::COUNT];
    for motion in Motion::FINITE {
        motion_ready[motion.index()] =
            motion_tuning[motion.index()].is_some() && brain.is_ready(Cooldown::Motion(motion), now);
    }
    motion_ready
}

/// Everything one brain senses.
#[derive(QueryData)]
#[query_data(mutable)]
pub(super) struct Sensed {
    entity: Entity,
    brain: &'static Brain,
    perception: &'static mut Perception,
    transform: &'static Transform,
    has_target: Has<Targeting>,
    target_info: Option<&'static TargetInfo>,
    health: Option<&'static Health>,
    mana: Option<&'static Mana>,
    anchor: Option<&'static Anchor>,
    attack_state: Option<&'static AttackState>,
    recovery: Option<&'static ManaRecovery>,
    finite_motions: FiniteMotionsReadOnly,
    abilities: AbilityCapabilities,
}

/// Writes every brain's [`DecisionContext`] from its world state, its own
/// earlier choices, and the capabilities it carries.
pub(super) fn sense(time: Res<Time<Virtual>>, equipment: EquipmentRead, mut brains: Query<Sensed>) {
    let now = time.elapsed_secs();
    for sensed in &mut brains {
        let SensedItem {
            entity,
            brain,
            mut perception,
            transform,
            has_target,
            target_info,
            health,
            mana,
            anchor,
            attack_state,
            recovery,
            finite_motions,
            abilities,
        } = sensed;
        let position = transform.translation.xy();

        let finished = attack_state.map_or(0, |state| state.attacks_finished);
        let attack_finished = finished != perception.attacks_seen;
        perception.attacks_seen = finished;

        let motion_tuning = finite_motions.tuning();

        // Equipment is always performable (readiness is scored separately) and
        // shares the definition's reach; abilities need their component.
        let mut action_tuning = [None; Action::COUNT];
        for slot in EquipmentSlot::ALL {
            action_tuning[Action::Use(slot).index()] = Some(Tuning {
                reach: brain.definition.equipment_reach,
                ..Tuning::default()
            });
        }
        let ability_tuning = abilities.tuning();
        for ability in Ability::ALL {
            action_tuning[Action::Ability(ability).index()] = ability_tuning[ability as usize];
        }

        perception.context = DecisionContext {
            target: target_view(has_target, target_info),
            health_fraction: health.and_then(Health::fraction).unwrap_or(1.0),
            mana_fraction: mana.and_then(Mana::fraction).unwrap_or(1.0),
            slot_ready: EquipmentSlot::ALL.map(|slot| equipment.can_use(entity, slot).is_ok()),
            anchor_overshoot_fraction: anchor
                .and_then(|anchor| anchor.overshoot_fraction(position)),
            anchor_arrived: anchor
                .map(|anchor| anchor.distance_from(position) <= ANCHOR_ARRIVAL_RADIUS),
            attack_finished,
            motion: brain.motion(),
            mana_recovering: recovery.is_some_and(ManaRecovery::recovering),
            motion_tuning,
            motion_ready: motion_readiness(brain, &motion_tuning, now),
            action_tuning,
        };
    }
}
