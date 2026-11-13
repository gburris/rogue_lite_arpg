use std::collections::VecDeque;

use avian2d::prelude::{LinearVelocity, RayCaster};
use bevy::diagnostic::FrameCount;
use bevy::ecs::query::QueryData;
use bevy::prelude::*;
use bevy::remote::BrpResult;
use serde::Deserialize;
use serde_json::{Value, json};

use super::json::{debug_name, entity_id, parse_params, position};
use crate::prelude::*;

/// Most hits kept for snapshots to read; older ones are dropped first.
const MAX_RECORDED_HITS: usize = 512;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<RecordedHits>()
        .add_observer(on_damage_dealt_record_hit);
}

/// The latest hits, numbered from 1 in the order they landed.
#[derive(Resource, Debug, Default)]
pub(super) struct RecordedHits {
    hits: VecDeque<RecordedHit>,
    last_sequence: u64,
}

/// One [`DamageDealt`] and when it landed.
#[derive(Clone, Copy, Debug)]
struct RecordedHit {
    sequence: u64,
    time: f32,
    entity: Entity,
    damage: f32,
    attacker: Option<Entity>,
}

fn on_damage_dealt_record_hit(
    damage_dealt: On<DamageDealt>,
    time: Res<Time<Virtual>>,
    mut recorded_hits: ResMut<RecordedHits>,
) {
    if recorded_hits.hits.len() >= MAX_RECORDED_HITS {
        recorded_hits.hits.pop_front();
    }

    recorded_hits.last_sequence += 1;
    let hit = RecordedHit {
        sequence: recorded_hits.last_sequence,
        time: time.elapsed_secs(),
        entity: damage_dealt.entity,
        damage: damage_dealt.damage,
        attacker: damage_dealt.attacker,
    };
    recorded_hits.hits.push_back(hit);
}

#[derive(QueryData)]
pub(super) struct PlayerData {
    entity: Entity,
    transform: &'static Transform,
    velocity: &'static LinearVelocity,
    player: &'static Player,
    purse: &'static Purse,
    vision: &'static Vision,
    health: Option<&'static Health>,
    mana: Option<&'static Mana>,
    is_invincible: Has<Invincible>,
    mainhand: Option<&'static Mainhand>,
    offhand: Option<&'static Offhand>,
    items: Option<&'static Items>,
}

#[derive(QueryData)]
pub(super) struct EnemyData {
    entity: Entity,
    kind: &'static EnemyKind,
    transform: &'static Transform,
    velocity: &'static LinearVelocity,
    kinematics: &'static Kinematics,
    stat_multipliers: &'static StatMultipliers,
    facing_direction: &'static FacingDirection,
    health: &'static Health,
    mana: Option<&'static Mana>,
    brain: Option<&'static Brain>,
    dash: Option<&'static Dash>,
    vision: &'static Vision,
    ray_caster: Option<&'static RayCaster>,
    target_info: &'static TargetInfo,
    has_target: Has<Targeting>,
    has_target_lock: Has<TargetLock>,
    attacking: &'static Attacking,
}

#[derive(QueryData)]
pub(super) struct NpcData {
    entity: Entity,
    transform: &'static Transform,
    name: Option<&'static Name>,
    brain: Option<&'static Brain>,
}

#[derive(QueryData)]
pub(super) struct InventoryItemData {
    item: &'static Item,
    name: Option<&'static Name>,
    equippable: Option<&'static Equippable>,
    is_equipped: Has<Equipped>,
    is_in_use: Has<EquipmentInUse>,
    is_consumable: Has<Consumable>,
}

#[derive(QueryData)]
pub(super) struct GroundItemData {
    entity: Entity,
    transform: &'static Transform,
    name: Option<&'static Name>,
    lifespan: Option<&'static Lifespan>,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct SnapshotParams {
    hits_after: u64,
}

/// `game/snapshot {hits_after?}`: the frame number, app and menu states, game time, the current
/// map (its entity changes with every build), the player and their inventory, every enemy's
/// brain, movement and perception, villagers, projectiles, items on the ground, interactables,
/// portals and the latest hits. Each hit has a `sequence`; pass a snapshot's `last_hit` back as
/// `hits_after` to get only the hits landed since that snapshot.
pub(super) fn take_snapshot(
    In(params): In<Option<Value>>,
    frame_count: Res<FrameCount>,
    time: Res<Time<Virtual>>,
    app_state: Res<State<AppState>>,
    playing_state: Option<Res<State<PlayingState>>>,
    menu: Res<State<Menu>>,
    recorded_hits: Res<RecordedHits>,
    map: Option<Single<(Entity, &MapBlueprint)>>,
    player: Option<Single<PlayerData, With<Player>>>,
    inventory_items: Query<InventoryItemData>,
    names: Query<&Name>,
    enemies: Query<EnemyData, With<Enemy>>,
    npcs: Query<NpcData, With<Npc>>,
    projectiles: Query<&Transform, With<Projectile>>,
    ground_items: Query<GroundItemData, (With<Item>, Without<ItemOf>)>,
    interaction_zones: Query<(&ChildOf, &GlobalTransform), With<InteractionZone>>,
    portals: Query<&Transform, With<Portal>>,
) -> BrpResult {
    let SnapshotParams { hits_after } = parse_params(params)?;

    let map = map.map(|map| describe_map(map.into_inner()));
    let player = player.map(|player| describe_player(&player, &inventory_items, &names));
    let enemies: Vec<Value> = enemies.iter().map(|enemy| describe_enemy(&enemy)).collect();
    let npcs: Vec<Value> = npcs.iter().map(|npc| describe_npc(&npc)).collect();
    let projectiles: Vec<[f32; 2]> = projectiles.iter().map(position).collect();
    let ground_items: Vec<Value> = ground_items
        .iter()
        .map(|item| describe_ground_item(&item))
        .collect();
    let interactables: Vec<Value> = interaction_zones
        .iter()
        .map(|(child_of, transform)| describe_interactable(child_of.parent(), transform, &names))
        .collect();
    let portals: Vec<[f32; 2]> = portals.iter().map(position).collect();
    let hits: Vec<Value> = recorded_hits
        .hits
        .iter()
        .filter(|hit| hit.sequence > hits_after)
        .map(describe_hit)
        .collect();

    Ok(json!({
        "frame": frame_count.0,
        "time": time.elapsed_secs(),
        "paused": time.is_paused(),
        "state": debug_name(app_state.get()),
        "playing_state": playing_state.map(|state| debug_name(state.get())),
        "menu": debug_name(menu.get()),
        "map": map,
        "player": player,
        "enemies": enemies,
        "npcs": npcs,
        "projectiles": projectiles,
        "items_on_ground": ground_items,
        "interactables": interactables,
        "portals": portals,
        "hits": hits,
        "last_hit": recorded_hits.last_sequence,
    }))
}

fn describe_map((entity, blueprint): (Entity, &MapBlueprint)) -> Value {
    json!({
        "entity": entity_id(entity),
        "recipe": blueprint.recipe,
        "seed": blueprint.seed,
    })
}

fn describe_player(
    player: &PlayerDataItem,
    inventory_items: &Query<InventoryItemData>,
    names: &Query<&Name>,
) -> Value {
    let name_of = |entity: Entity| names.get(entity).ok().map(Name::as_str);

    let mut inventory = Vec::new();
    for item_entity in player.items.into_iter().flat_map(Items::iter) {
        if let Ok(item) = inventory_items.get(item_entity) {
            inventory.push(describe_inventory_item(item_entity, &item));
        }
    }

    json!({
        "entity": entity_id(player.entity),
        "position": position(player.transform),
        "velocity": player.velocity.0.to_array(),
        "aim": player.vision.aim_direction.to_array(),
        "hp": player.health.map(|health| health.hp),
        "max_hp": player.health.map(|health| health.max_hp),
        "mana": player.mana.map(|mana| mana.current),
        "max_mana": player.mana.map(|mana| mana.max),
        "invincible": player.is_invincible,
        "gold": player.purse.amount,
        "level": player.player.level(),
        "mainhand": player.mainhand.and_then(|mainhand| name_of(mainhand.get())),
        "offhand": player.offhand.and_then(|offhand| name_of(offhand.get())),
        "inventory": inventory,
    })
}

fn describe_inventory_item(entity: Entity, item: &InventoryItemDataItem) -> Value {
    let kind = match item.item.kind {
        ItemKind::Melee => "Melee",
        ItemKind::Staff => "Staff",
        ItemKind::Potion => "Potion",
        ItemKind::Tome => "Tome",
    };

    json!({
        "entity": entity_id(entity),
        "name": item.name.map(Name::as_str),
        "kind": kind,
        "slot": item.equippable.map(|equippable| debug_name(equippable.slot)),
        "equipped": item.is_equipped,
        "in_use": item.is_in_use,
        "consumable": item.is_consumable,
        "value": item.item.value,
    })
}

fn describe_enemy(enemy: &EnemyDataItem) -> Value {
    let motion = enemy.brain.and_then(Brain::motion);
    let motion_name = motion.map(|motion| match (motion, enemy.dash) {
        (Motion::Dash, Some(dash)) => dash.name(),
        _ => motion.name(),
    });
    let action = enemy.brain.and_then(Brain::action);
    let intended_velocity = enemy.kinematics.velocity(enemy.stat_multipliers.speed);

    json!({
        "entity": entity_id(enemy.entity),
        "kind": debug_name(enemy.kind),
        "position": position(enemy.transform),
        "intended_velocity": intended_velocity.to_array(),
        "velocity": enemy.velocity.0.to_array(),
        "facing": debug_name(enemy.facing_direction),
        "aim": enemy.vision.aim_direction.to_array(),
        "stunned": enemy.stat_multipliers.is_stunned(),
        "hp": enemy.health.hp,
        "max_hp": enemy.health.max_hp,
        "dead": enemy.health.is_dead(),
        "mana": enemy.mana.map(|mana| mana.current),
        "motion": motion_name,
        "action": action.map(debug_name),
        "has_target": enemy.has_target,
        "target_lock": enemy.has_target_lock,
        "line_of_sight": enemy.target_info.line_of_sight,
        "in_vision_cone": enemy.target_info.in_vision_cone(),
        "sight_range": enemy.ray_caster.map(|ray_caster| ray_caster.max_distance),
        "target_distance": enemy.target_info.distance,
        "attacking": **enemy.attacking,
    })
}

fn describe_npc(npc: &NpcDataItem) -> Value {
    let motion = npc.brain.and_then(Brain::motion);

    json!({
        "entity": entity_id(npc.entity),
        "name": npc.name.map(Name::as_str),
        "position": position(npc.transform),
        "motion": motion.map(Motion::name),
    })
}

fn describe_ground_item(item: &GroundItemDataItem) -> Value {
    json!({
        "entity": entity_id(item.entity),
        "name": item.name.map(Name::as_str),
        "position": position(item.transform),
        "seconds_left": item.lifespan.map(|lifespan| lifespan.remaining_secs()),
    })
}

/// An interaction zone, described by the entity it belongs to and where the zone itself is.
fn describe_interactable(
    owner: Entity,
    zone_transform: &GlobalTransform,
    names: &Query<&Name>,
) -> Value {
    json!({
        "entity": entity_id(owner),
        "name": names.get(owner).ok().map(Name::as_str),
        "position": zone_transform.translation().truncate().to_array(),
    })
}

fn describe_hit(hit: &RecordedHit) -> Value {
    json!({
        "sequence": hit.sequence,
        "time": hit.time,
        "entity": entity_id(hit.entity),
        "damage": hit.damage,
        "attacker": hit.attacker.map(entity_id),
    })
}
