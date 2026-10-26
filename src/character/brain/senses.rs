//! Observing the world: the only code that turns world state into the facts
//! one decision reads. It is read-only and never looks at [`Brain`]'s state.

use bevy::{ecs::query::QueryData, ecs::system::SystemParam, prelude::*};

use super::ability::find_carried_potion;
use super::capability::{Capabilities, Tuning};
use super::motion::ANCHOR_ARRIVAL_RADIUS;
use super::{
    Ability, Action, AnchorView, ManaRecovery, Motion, MotionActionTable, SelfView, TargetView,
};
use crate::{
    character::{
        anchor::Anchor,
        state::SimpleMotion,
        vision::{TargetInfo, Targeting},
    },
    prelude::{AttackState, Consumable, EquipmentRead, EquipmentSlot, Health, Items, Mana},
};

/// World facts for one decision, built without reading `Brain`'s mutable state.
#[derive(Clone, Copy, Debug)]
pub(super) struct Observation {
    pub me: SelfView,
    pub target: Option<TargetView>,
    pub in_reach: MotionActionTable<bool>,
    /// Whether the world lets each motion or action run now (equipment ready,
    /// potion carried).
    pub world_allows_use: MotionActionTable<bool>,
    /// `AttackState::attacks_finished` as read this decision.
    pub attacks_finished: u32,
}

/// Builds the pure view of a target from the vision system's observation.
pub(super) fn build_target_view(has_target: bool, info: Option<&TargetInfo>) -> Option<TargetView> {
    if !has_target {
        return None;
    }
    info.and_then(|info| {
        info.distance.is_finite().then(|| TargetView {
            distance: info.distance,
            line_of_sight: info.line_of_sight,
            bearing: info.direction,
            velocity: info.target_velocity,
        })
    })
}

/// Whether the target is inside each owned capability's reach.
fn build_in_reach(
    target: Option<TargetView>,
    capabilities: &Capabilities,
) -> MotionActionTable<bool> {
    let mut in_reach = MotionActionTable::default();
    let Some(target) = target else {
        return in_reach;
    };
    let within_reach = |tuning: Option<Tuning>| {
        tuning
            .and_then(|tuning| tuning.reach)
            .is_some_and(|reach| target.distance <= reach)
    };
    for motion in Motion::FINITE {
        in_reach[motion] = within_reach(capabilities[motion]);
    }
    for slot in EquipmentSlot::ALL {
        in_reach[Action::Use(slot)] = within_reach(capabilities[Action::Use(slot)]);
    }
    for ability in Ability::ALL {
        in_reach[ability] = within_reach(capabilities[ability]);
    }
    in_reach
}

/// The queries behind each capability's world-allows-use check, shared by
/// `observe_world_facts` and the appliers.
#[derive(SystemParam)]
pub(super) struct WorldUseChecks<'w, 's> {
    pub equipment: EquipmentRead<'w, 's>,
    pub consumables: Query<'w, 's, (), With<Consumable>>,
}

/// Whether the world lets each motion or action run now: equipment that can be
/// used, and a carried potion for `Heal`. Everything else is always allowed.
fn build_world_allows_use(
    entity: Entity,
    items: Option<&Items>,
    use_checks: &WorldUseChecks,
) -> MotionActionTable<bool> {
    let mut world_allows_use = MotionActionTable::splat(true);
    for slot in EquipmentSlot::ALL {
        world_allows_use[Action::Use(slot)] = use_checks.equipment.can_use(entity, slot).is_ok();
    }
    world_allows_use[Ability::Heal] = find_carried_potion(items, &use_checks.consumables).is_some();
    world_allows_use
}

/// The world-fact components one decision reads.
#[derive(QueryData)]
pub(super) struct ObservedComponents {
    entity: Entity,
    transform: &'static Transform,
    has_target: Has<Targeting>,
    target_info: Option<&'static TargetInfo>,
    health: Option<&'static Health>,
    mana: Option<&'static Mana>,
    anchor: Option<&'static Anchor>,
    attack_state: Option<&'static AttackState>,
    movement: Option<&'static SimpleMotion>,
    recovery: Option<&'static ManaRecovery>,
    items: Option<&'static Items>,
}

/// Reads the world facts one decision needs from the entity's components and
/// the capabilities it carries.
pub(super) fn observe_world_facts(
    components: &ObservedComponentsItem,
    capabilities: &Capabilities,
    use_checks: &WorldUseChecks,
) -> Observation {
    let position = components.transform.translation.xy();
    let target = build_target_view(components.has_target, components.target_info);
    let anchor = components.anchor.and_then(|anchor| {
        Some(AnchorView {
            overshoot_fraction: anchor.overshoot_fraction(position)?,
            arrived: anchor.distance_from(position) <= ANCHOR_ARRIVAL_RADIUS,
        })
    });
    Observation {
        me: SelfView {
            health_fraction: components.health.and_then(Health::fraction).unwrap_or(1.0),
            mana_fraction: components.mana.and_then(Mana::fraction).unwrap_or(1.0),
            mana_recovering: components.recovery.is_some_and(ManaRecovery::recovering),
            stunned: components.movement.is_some_and(SimpleMotion::is_stunned),
            anchor,
        },
        target,
        in_reach: build_in_reach(target, capabilities),
        world_allows_use: build_world_allows_use(components.entity, components.items, use_checks),
        attacks_finished: components
            .attack_state
            .map_or(0, |state| state.attacks_finished),
    }
}
