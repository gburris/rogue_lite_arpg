//! Starts the game in the stress arena when `STRESS_ARENA` is set.
//!
//! `STRESS_ARENA=warrior=250,ice_mage=250,fire_mage=250,bat=250` replaces the hub request with
//! the `StressArena` recipe, spawns that many of each listed kind (unlisted kinds spawn none),
//! and makes the player [`Invincible`].

use bevy::prelude::*;

use super::build::MapRequest;
use super::placement::CountRange;
use super::recipe::{MapRecipe, SpawnKind};
use crate::prelude::*;

/// Name of the recipe `STRESS_ARENA` loads in place of the hub.
const STRESS_ARENA_RECIPE: &str = "StressArena";

/// How many enemies of one kind the stress arena spawns.
#[derive(Clone, Copy, Debug)]
struct KindCount {
    kind: EnemyKind,
    count: u32,
}

/// Enemy counts parsed from `STRESS_ARENA`. Present only when the variable is set.
#[derive(Resource, Debug)]
pub(super) struct StressArena {
    kind_counts: Vec<KindCount>,
}

pub(super) fn plugin(app: &mut App) {
    let Ok(value) = std::env::var("STRESS_ARENA") else {
        return;
    };
    let kind_counts = parse_kind_counts(&value)
        .unwrap_or_else(|error| panic!("STRESS_ARENA=`{value}` is invalid: {error}"));
    info!("Stress arena enemy counts: {kind_counts:?}");
    app.insert_resource(StressArena { kind_counts });
    app.add_observer(on_player_added);
}

/// Sends the game's first map request, the hub, to the stress arena.
pub(super) fn redirect_hub(request: MapRequest) -> MapRequest {
    match request {
        MapRequest::Hub => MapRequest::Named {
            recipe: STRESS_ARENA_RECIPE.to_owned(),
            seed: None,
        },
        MapRequest::Weighted | MapRequest::Named { .. } => request,
    }
}

impl StressArena {
    /// Returns the stress arena recipe with its enemy counts replaced, or `None` for any
    /// other recipe.
    pub(super) fn override_counts(&self, recipe: &MapRecipe) -> Option<MapRecipe> {
        if recipe.name != STRESS_ARENA_RECIPE {
            return None;
        }
        let mut recipe = recipe.clone();
        recipe.spawns.retain_mut(|spawn| {
            let SpawnKind::Enemy(kind) = spawn.what else {
                return true;
            };
            let count = self
                .kind_counts
                .iter()
                .rev()
                .find(|kind_count| kind_count.kind == kind)
                .map_or(0, |kind_count| kind_count.count);
            spawn.count = CountRange::exactly(count);
            count > 0
        });
        Some(recipe)
    }
}

fn on_player_added(player_added: On<Add, Player>, mut commands: Commands) {
    commands.entity(player_added.entity).insert(Invincible);
}

/// Parses `kind=count,kind=count`. Kind names ignore case and underscores (`ice_mage`).
fn parse_kind_counts(value: &str) -> Result<Vec<KindCount>, String> {
    let mut kind_counts = Vec::new();
    for entry in value
        .split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
    {
        let (name, count) = entry
            .split_once('=')
            .ok_or_else(|| format!("`{entry}` is not kind=count"))?;
        let kind = enemy_kind_named(name.trim())?;
        let count = count
            .trim()
            .parse::<u32>()
            .map_err(|error| format!("count in `{entry}`: {error}"))?;
        kind_counts.push(KindCount { kind, count });
    }
    Ok(kind_counts)
}

fn enemy_kind_named(name: &str) -> Result<EnemyKind, String> {
    let normalize = |text: &str| text.replace('_', "").to_lowercase();
    EnemyKind::ALL
        .iter()
        .copied()
        .find(|kind| normalize(&format!("{kind:?}")) == normalize(name))
        .ok_or_else(|| {
            let valid: Vec<String> = EnemyKind::ALL
                .iter()
                .map(|kind| format!("{kind:?}"))
                .collect();
            format!("unknown enemy kind `{name}`, expected one of {valid:?}")
        })
}
