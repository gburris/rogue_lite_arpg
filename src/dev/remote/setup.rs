use bevy::ecs::entity::EntityHashSet;
use bevy::prelude::*;
use bevy::remote::{BrpError, BrpResult};
use serde::Deserialize;
use serde_json::{Value, json};

use super::json::{entity_id, invalid_params, parse_entity, parse_params};
use crate::prelude::*;

#[derive(Deserialize)]
struct TeleportParams {
    position: Vec2,
}

pub(super) fn teleport_player(
    In(params): In<Option<Value>>,
    mut player: Single<&mut Transform, With<Player>>,
) -> BrpResult {
    let TeleportParams { position } = parse_params(params)?;

    player.translation = position.extend(player.translation.z);

    Ok(Value::Null)
}

/// Which way `game/spawn_enemies` turns new enemies, and so where their vision cone points until
/// they move or target something.
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Facing {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Deserialize)]
struct SpawnEnemiesParams {
    kind: EnemyKind,
    positions: Vec<Vec2>,
    facing: Option<Facing>,
    #[serde(default)]
    aggro: bool,
}

pub(super) fn spawn_enemies(In(params): In<Option<Value>>, world: &mut World) -> BrpResult {
    let SpawnEnemiesParams {
        kind,
        positions,
        facing,
        aggro,
    } = parse_params(params)?;

    let enemies_before = all_enemies(world);
    let spawns = positions
        .into_iter()
        .map(|position| EnemySpawn { position, kind })
        .collect();
    world.trigger(SpawnEnemies(spawns));
    world.flush();

    // `SpawnEnemies` doesn't report what it spawned, so the new enemies are the ones that
    // weren't there before.
    let spawned: Vec<Entity> = all_enemies(world)
        .difference(&enemies_before)
        .copied()
        .collect();

    if let Some(facing) = facing {
        turn_enemies(world, &spawned, facing);
    }
    if aggro {
        aggro_enemies_on_player(world, &spawned)?;
    }

    let spawned: Vec<u64> = spawned.into_iter().map(entity_id).collect();
    Ok(json!(spawned))
}

fn all_enemies(world: &mut World) -> EntityHashSet {
    world
        .query_filtered::<Entity, With<Enemy>>()
        .iter(world)
        .collect()
}

fn turn_enemies(world: &mut World, enemies: &[Entity], facing: Facing) {
    let facing = match facing {
        Facing::Up => FacingDirection::Up,
        Facing::Down => FacingDirection::Down,
        Facing::Left => FacingDirection::Left,
        Facing::Right => FacingDirection::Right,
    };

    for &enemy in enemies {
        let mut enemy = world.entity_mut(enemy);
        enemy.insert(facing);

        // Animation re-derives facing from the heading whenever kinematics change.
        if let Some(mut kinematics) = enemy.get_mut::<Kinematics>() {
            kinematics.direction = facing.to_vec2();
        }
    }
}

fn aggro_enemies_on_player(world: &mut World, enemies: &[Entity]) -> BrpResult<()> {
    let player = world
        .query_filtered::<Entity, With<Player>>()
        .single(world)
        .map_err(|error| BrpError::internal(format!("no player to aggro on: {error}")))?;

    let mut commands = world.commands();
    for &enemy in enemies {
        lock_onto_target(&mut commands, enemy, player);
    }
    world.flush();

    Ok(())
}

#[expect(
    clippy::unnecessary_wraps,
    reason = "remote method handlers return `BrpResult`"
)]
pub(super) fn despawn_enemies(
    In(_): In<Option<Value>>,
    mut commands: Commands,
    enemies: Query<Entity, With<Enemy>>,
) -> BrpResult {
    for enemy in &enemies {
        commands.entity(enemy).despawn();
    }

    Ok(json!({ "despawned": enemies.count() }))
}

#[derive(Deserialize)]
struct PlayerHealthParams {
    hp: f32,
    max_hp: f32,
}

pub(super) fn set_player_health(
    In(params): In<Option<Value>>,
    mut player_health: Single<&mut Health, With<Player>>,
) -> BrpResult {
    let PlayerHealthParams { hp, max_hp } = parse_params(params)?;

    player_health.max_hp = max_hp;
    player_health.hp = hp.min(max_hp);

    Ok(Value::Null)
}

#[derive(Deserialize)]
struct PlayerManaParams {
    mana: f32,
    max_mana: Option<f32>,
}

pub(super) fn set_player_mana(
    In(params): In<Option<Value>>,
    mut player_mana: Single<&mut Mana, With<Player>>,
) -> BrpResult {
    let PlayerManaParams { mana, max_mana } = parse_params(params)?;

    if let Some(max_mana) = max_mana {
        player_mana.max = max_mana;
    }
    player_mana.current = mana.min(player_mana.max);

    Ok(Value::Null)
}

#[derive(Deserialize)]
struct InvincibleParams {
    invincible: bool,
}

pub(super) fn set_invincible(
    In(params): In<Option<Value>>,
    mut commands: Commands,
    player: Single<Entity, With<Player>>,
) -> BrpResult {
    let InvincibleParams { invincible } = parse_params(params)?;

    if invincible {
        commands.entity(*player).insert(Invincible);
    } else {
        commands.entity(*player).remove::<Invincible>();
    }

    Ok(Value::Null)
}

#[derive(Deserialize)]
struct GiveItemParams {
    item: String,
}

pub(super) fn give_item(
    In(params): In<Option<Value>>,
    mut commands: Commands,
    sprites: Res<SpriteAssets>,
    layouts: Res<SpriteSheetLayouts>,
    player: Single<Entity, With<Player>>,
) -> BrpResult {
    let GiveItemParams { item } = parse_params(params)?;

    let item = match item.as_str() {
        "sword" => commands.spawn_scene(sword(&sprites)).id(),
        "axe" => commands.spawn_scene(axe(&sprites)).id(),
        "freeze_axe" => commands.spawn_scene(freeze_axe(&sprites)).id(),
        "fire_staff" => commands.spawn_scene(fire_staff(&sprites, &layouts)).id(),
        "ice_staff" => commands.spawn_scene(ice_staff(&sprites, &layouts)).id(),
        "knight_shield" => commands.spawn_scene(knight_shield(&sprites, &layouts)).id(),
        "magic_shield" => commands.spawn_scene(magic_shield(&sprites, &layouts)).id(),
        "health_potion" => commands.spawn_scene(health_potion(&sprites)).id(),
        "tome_of_healing" => commands.spawn_scene(tome_of_healing(&sprites)).id(),
        unknown => return Err(invalid_params(format!("unknown item {unknown:?}"))),
    };
    commands.entity(item).insert(ItemOf { holder: *player });

    Ok(json!(entity_id(item)))
}

/// What `game/use_item` does with an inventory item.
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ItemAction {
    Equip,
    Unequip,
    Consume,
    Drop,
}

#[derive(Deserialize)]
struct UseItemParams {
    item: u64,
    action: ItemAction,
}

pub(super) fn use_item(
    In(params): In<Option<Value>>,
    mut commands: Commands,
    player: Single<Entity, With<Player>>,
    holders: Query<&ItemOf>,
) -> BrpResult {
    let UseItemParams { item, action } = parse_params(params)?;
    let item = parse_entity(item)?;
    let player = *player;

    let held_by_player = holders
        .get(item)
        .is_ok_and(|item_of| item_of.holder == player);
    if !held_by_player {
        return Err(invalid_params(format!(
            "{item} is not in the player's inventory"
        )));
    }

    match action {
        ItemAction::Equip => commands.trigger(Equip {
            item,
            holder: player,
        }),
        ItemAction::Unequip => commands.trigger(Unequip { item }),
        ItemAction::Consume => commands.trigger(Consume {
            entity: player,
            item,
        }),
        ItemAction::Drop => commands.trigger(DropItem { item }),
    }

    Ok(Value::Null)
}
