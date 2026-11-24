use std::collections::VecDeque;

use avian2d::prelude::LinearVelocity;
use bevy::diagnostic::FrameCount;
use bevy::ecs::query::QueryData;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::remote::BrpResult;
use bevy_firefly::prelude::PointLight2d;
use serde::Deserialize;
use serde_json::{Value, json};

use super::methods::{debug_name, entity_id, invalid_params, parse_params, position};
use crate::prelude::*;

/// Most hits kept for snapshots to read; older ones are dropped first.
const MAX_RECORDED_HITS: usize = 512;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<RecordedHits>()
        .add_observer(on_damage_dealt_record_hit)
        .add_systems(Update, record_snapshot.in_set(InGameSystems::Measure));
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
    vision: &'static Vision,
    sight_ray: Option<&'static SightRay>,
    target_info: &'static TargetInfo,
    has_target: Has<Targeting>,
    has_target_lock: Has<TargetLock>,
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
    is_in_use: Has<ActionSourceInUse>,
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

pub(super) fn take_snapshot(In(params): In<Option<Value>>, game: GameSnapshot) -> BrpResult {
    let SnapshotParams { hits_after } = parse_params(params)?;
    Ok(game.describe_game(hits_after))
}

/// Snapshots taken every `interval` of game time while a script records, each with the hits
/// landed since the one before. See [`record_snapshot`].
#[derive(Resource, Debug)]
pub(super) struct SnapshotRecording {
    interval: Timer,
    last_hit: u64,
    snapshots: Vec<Value>,
    missed_snapshots: u32,
}

#[derive(Deserialize)]
struct StartRecordingParams {
    interval: f32,
}

/// Starts a new [`SnapshotRecording`], replacing any running one. Its first snapshot is taken
/// next frame, and only hits landed from now on count.
pub(super) fn start_recording(
    In(params): In<Option<Value>>,
    mut commands: Commands,
    recorded_hits: Res<RecordedHits>,
) -> BrpResult {
    let StartRecordingParams { interval } = parse_params(params)?;
    if !interval.is_finite() || interval <= 0.0 {
        return Err(invalid_params(format!(
            "interval must be a finite, positive number of seconds, not {interval}"
        )));
    }

    let mut interval = Timer::from_seconds(interval, TimerMode::Repeating);
    interval.almost_finish();
    commands.insert_resource(SnapshotRecording {
        interval,
        last_hit: recorded_hits.last_sequence,
        snapshots: Vec::new(),
        missed_snapshots: 0,
    });
    Ok(Value::Null)
}

/// Ends the [`SnapshotRecording`] and returns its snapshots, with how many intervals passed
/// without one.
pub(super) fn stop_recording(
    In(_): In<Option<Value>>,
    mut commands: Commands,
    recording: Option<ResMut<SnapshotRecording>>,
) -> BrpResult {
    let mut recording = recording.ok_or_else(|| invalid_params("no recording is running"))?;
    commands.remove_resource::<SnapshotRecording>();
    Ok(json!({
        "snapshots": std::mem::take(&mut recording.snapshots),
        "missed_snapshots": recording.missed_snapshots,
    }))
}

/// Adds a snapshot to the [`SnapshotRecording`] each time its interval of game time passes, so
/// a recording samples the game alike at every game speed. It takes at most one a frame, and
/// counts the intervals a long frame skips past as missed.
pub(super) fn record_snapshot(
    time: Res<Time<Virtual>>,
    recording: Option<ResMut<SnapshotRecording>>,
    game: GameSnapshot,
) {
    let Some(mut recording) = recording else {
        return;
    };
    recording.interval.tick(time.delta());
    let intervals_passed = recording.interval.times_finished_this_tick();
    if intervals_passed == 0 {
        return;
    }
    recording.missed_snapshots += intervals_passed - 1;

    let snapshot = game.describe_game(recording.last_hit);
    recording.last_hit = game.recorded_hits.last_sequence;
    recording.snapshots.push(snapshot);
}

/// Everything a snapshot reports: the session, the player, enemies, villagers, the map's contents
/// and the latest hits.
#[derive(SystemParam)]
pub(super) struct GameSnapshot<'w, 's> {
    frame_count: Res<'w, FrameCount>,
    time: Res<'w, Time<Virtual>>,
    app_state: Res<'w, State<AppState>>,
    playing_state: Option<Res<'w, State<PlayingState>>>,
    menu: Res<'w, State<Menu>>,
    recorded_hits: Res<'w, RecordedHits>,
    map: Option<Single<'w, 's, (Entity, &'static MapBlueprint)>>,
    player: Option<Single<'w, 's, PlayerData, With<Player>>>,
    inventory_items: Query<'w, 's, InventoryItemData>,
    names: Query<'w, 's, &'static Name>,
    enemies: Query<'w, 's, EnemyData, With<Enemy>>,
    actions: ActionStatus<'w, 's>,
    npcs: Query<'w, 's, NpcData, With<Npc>>,
    projectiles: Query<'w, 's, &'static Transform, With<Projectile>>,
    entity_counts: EntityCounts<'w, 's>,
    ground_items: Query<'w, 's, GroundItemData, (With<Item>, Without<ItemOf>)>,
    interaction_zones:
        Query<'w, 's, (&'static ChildOf, &'static GlobalTransform), With<InteractionZone>>,
    portals: Query<'w, 's, &'static Transform, With<Portal>>,
}

impl GameSnapshot<'_, '_> {
    /// The snapshot as JSON, with the hits numbered after `hits_after`.
    fn describe_game(&self, hits_after: u64) -> Value {
        let map = self.map.as_deref().map(|&map| describe_map(map));
        let player = self
            .player
            .as_deref()
            .map(|player| describe_player(player, &self.inventory_items, &self.names));
        let enemies: Vec<Value> = self
            .enemies
            .iter()
            .map(|enemy| describe_enemy(&enemy, self.actions.is_attacking(enemy.entity)))
            .collect();
        let npcs: Vec<Value> = self.npcs.iter().map(|npc| describe_npc(&npc)).collect();
        let projectiles: Vec<[f32; 2]> = self.projectiles.iter().map(position).collect();
        let ground_items: Vec<Value> = self
            .ground_items
            .iter()
            .map(|item| describe_ground_item(&item))
            .collect();
        let interactables: Vec<Value> = self
            .interaction_zones
            .iter()
            .map(|(child_of, transform)| {
                describe_interactable(child_of.parent(), transform, &self.names)
            })
            .collect();
        let portals: Vec<[f32; 2]> = self.portals.iter().map(position).collect();
        let hits: Vec<Value> = self
            .recorded_hits
            .hits
            .iter()
            .filter(|hit| hit.sequence > hits_after)
            .map(describe_hit)
            .collect();

        json!({
            "frame": self.frame_count.0,
            "time": self.time.elapsed_secs(),
            "paused": self.time.is_paused(),
            "state": debug_name(self.app_state.get()),
            "playing_state": self.playing_state.as_ref().map(|state| debug_name(state.get())),
            "menu": debug_name(self.menu.get()),
            "map": map,
            "player": player,
            "enemies": enemies,
            "npcs": npcs,
            "projectiles": projectiles,
            "items_on_ground": ground_items,
            "interactables": interactables,
            "portals": portals,
            "hits": hits,
            "last_hit": self.recorded_hits.last_sequence,
            "entity_counts": {
                "lights": self.entity_counts.lights.count(),
                "status_effects": self.entity_counts.status_effects.count(),
                "hitboxes": self.entity_counts.hitboxes.count(),
                "action_uses": self.entity_counts.action_uses.count(),
            },
        })
    }
}

/// How many of the entities a fight spawns and should clean up exist now. Lights on items don't
/// count, since dropped loot glows until it despawns.
#[derive(SystemParam)]
pub(super) struct EntityCounts<'w, 's> {
    lights: Query<'w, 's, (), (With<PointLight2d>, Without<Item>)>,
    status_effects: Query<'w, 's, (), With<StatusOf>>,
    hitboxes: Query<'w, 's, (), With<HitBox>>,
    action_uses: Query<'w, 's, (), With<ActionInUseOf>>,
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
        ItemKind::Shield => "Shield",
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

fn describe_enemy(enemy: &EnemyDataItem, attacking: bool) -> Value {
    let motion_name = enemy.brain.and_then(Brain::motion).map(MotionKey::name);
    let action = enemy.brain.and_then(Brain::action);
    let behavior_starts: Vec<(&str, f32)> = enemy
        .brain
        .map(|brain| brain.running_behavior_starts().collect())
        .unwrap_or_default();
    let behavior_names: Vec<&str> = behavior_starts.iter().map(|&(name, _)| name).collect();
    let behavior_start_times: serde_json::Map<String, Value> = behavior_starts
        .iter()
        .map(|&(name, started_at)| (name.to_owned(), json!(started_at)))
        .collect();
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
        "behaviors": behavior_names,
        "behavior_start_times": behavior_start_times,
        "has_target": enemy.has_target,
        "target_lock": enemy.has_target_lock,
        "line_of_sight": enemy.target_info.line_of_sight,
        "in_vision_cone": enemy.target_info.in_vision_cone(),
        "sight_range": enemy.sight_ray.map(SightRay::range),
        "target_distance": enemy.target_info.distance,
        "attacking": attacking,
    })
}

fn describe_npc(npc: &NpcDataItem) -> Value {
    let motion = npc.brain.and_then(Brain::motion);

    json!({
        "entity": entity_id(npc.entity),
        "name": npc.name.map(Name::as_str),
        "position": position(npc.transform),
        "motion": motion.map(MotionKey::name),
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
