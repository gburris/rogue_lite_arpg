use std::collections::BTreeMap;

use bevy::prelude::*;
use rand::RngExt;
use rand_chacha::ChaCha8Rng;
use serde::Deserialize;

use super::blueprint::PlacementKind;
use super::build::{MapResult, Stage};
use super::placement::{Anchor, CountRange, DecileRange, PlaceRule, Side};
use super::structures::{FeatureShape, FeatureSize};
use super::tiles::TileKind;
use crate::prelude::*;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RecipeFile {
    /// Named spawn lists that recipes include with `presets: ["Name"]`.
    #[serde(default)]
    pub(super) spawn_presets: BTreeMap<String, Vec<SpawnEntry>>,
    pub(super) defaults: RecipeDefaults,
    pub(super) structures: BTreeMap<String, StructureDefinition>,
    pub(super) hub: RecipeEntry,
    #[serde(default)]
    pub(super) dungeons: Vec<RecipeEntry>,
    #[serde(default)]
    pub(super) arenas: Vec<RecipeEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct StructureDefinition {
    pub(super) floor: TileKind,
    /// One row per entry, top row first; joined with `'\n'` when the catalog loads.
    pub(super) glyphs: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RecipeDefaults {
    pub(super) size_x_range: CountRange,
    pub(super) size_y_range: CountRange,
    pub(super) floor: TileKind,
    pub(super) structures: Vec<StructureEntry>,
    /// Radius around the player start, in tiles, that no enemy may occupy.
    pub(super) start_clearance: u32,
    #[serde(default)]
    pub(super) presets: Vec<String>,
    pub(super) spawns: Vec<SpawnEntry>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RecipeEntry {
    pub(super) name: Option<String>,
    pub(super) weight: Option<u32>,
    /// Shorthand for a square map: `size: 60` sets both size ranges to `(60, 60)`.
    pub(super) size: Option<u32>,
    pub(super) size_x_range: Option<CountRange>,
    pub(super) size_y_range: Option<CountRange>,
    pub(super) floor: Option<TileKind>,
    pub(super) structures: Option<Vec<StructureEntry>>,
    pub(super) start_clearance: Option<u32>,
    /// Replaces the default presets; their spawns come before the recipe's own.
    pub(super) presets: Option<Vec<String>>,
    /// Replaces the default spawns entirely, like `structures`.
    pub(super) spawns: Option<Vec<SpawnEntry>>,
}

/// Resolved map generation settings loaded from a recipe definition.
#[derive(Clone, Debug)]
pub struct MapRecipe {
    pub(super) name: String,
    pub(super) weight: u32,
    pub(super) size_x_range: CountRange,
    pub(super) size_y_range: CountRange,
    pub(super) floor: TileKind,
    pub(super) structures: Vec<StructureEntry>,
    pub(super) start_clearance: u32,
    /// Preset spawns followed by the recipe's own.
    pub(super) spawns: Vec<SpawnEntry>,
}

#[cfg(feature = "dev_tools")]
impl MapRecipe {
    /// The recipe's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Sets every enemy spawn to exactly `count_of(kind)` and drops those with a count of zero.
    pub fn with_enemy_counts(mut self, count_of: impl Fn(EnemyKind) -> u32) -> Self {
        self.spawns.retain_mut(|spawn| {
            let SpawnKind::Enemy(kind) = spawn.what else {
                return true;
            };
            let count = count_of(kind);
            spawn.count = CountRange::exactly(count);
            count > 0
        });
        self
    }
}

impl RecipeEntry {
    pub(super) fn resolve(
        self,
        name: String,
        weight: u32,
        defaults: &RecipeDefaults,
        spawn_presets: &BTreeMap<String, Vec<SpawnEntry>>,
    ) -> Result<MapRecipe> {
        if self.size.is_some() && (self.size_x_range.is_some() || self.size_y_range.is_some()) {
            return Err(format!(
                "recipe {name} sets size together with size_x_range or size_y_range"
            )
            .into());
        }
        let square = self.size.map(CountRange::exactly);
        let mut spawns = Vec::new();
        for preset in self.presets.as_ref().unwrap_or(&defaults.presets) {
            let preset_spawns = spawn_presets
                .get(preset)
                .ok_or_else(|| format!("recipe {name} uses unknown spawn preset {preset}"))?;
            spawns.extend(preset_spawns.iter().cloned());
        }
        spawns.extend(self.spawns.unwrap_or_else(|| defaults.spawns.clone()));
        Ok(MapRecipe {
            name,
            weight,
            size_x_range: self
                .size_x_range
                .or(square)
                .unwrap_or(defaults.size_x_range),
            size_y_range: self
                .size_y_range
                .or(square)
                .unwrap_or(defaults.size_y_range),
            floor: self.floor.unwrap_or(defaults.floor),
            structures: expand_counts(
                self.structures
                    .unwrap_or_else(|| defaults.structures.clone()),
            ),
            start_clearance: self.start_clearance.unwrap_or(defaults.start_clearance),
            spawns,
        })
    }
}

/// Replaces each entry with `count` single copies, as if the line were repeated.
pub(super) fn expand_counts(entries: Vec<StructureEntry>) -> Vec<StructureEntry> {
    entries
        .into_iter()
        .flat_map(|entry| {
            let copies = entry.count as usize;
            std::iter::repeat_n(StructureEntry { count: 1, ..entry }, copies)
        })
        .collect()
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct StructureEntry {
    pub(super) kind: StructureKind,
    #[serde(default)]
    pub(super) name: String,
    /// Where the structure goes, from the same options as spawns; `Random()` by default.
    #[serde(default = "random_location")]
    pub(super) at: Location,
    /// Resolved names stay `{kind}_{index}`, so a named entry must keep `count: 1`.
    #[serde(default = "one")]
    pub(super) count: u32,
}

pub(super) fn one() -> u32 {
    1
}

/// What a structure entry places: a glyph structure from the file's `structures` table,
/// or a builtin procedural feature. Each feature takes only the fields it uses.
///
/// To add a feature: add a variant with its fields, write its builder from
/// `claim_feature_footprint` in structures.rs (see rubble for a one-tile feature), and
/// handle the variant in the exhaustive `place_structures` match.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) enum StructureKind {
    /// A named glyph structure: `Structure("Temple")`.
    Structure(String),
    /// Walled dead-zone square, a 3–10 tile square by default: `EmptySquare()`.
    EmptySquare {
        #[serde(default)]
        size: Option<FeatureSize>,
    },
    /// Water that blocks ground movement and spawns, a 3–8 tile square by default:
    /// `Pond(size: Exact((14, 9)), shape: Blob)`.
    Pond {
        #[serde(default)]
        size: Option<FeatureSize>,
        #[serde(default)]
        shape: FeatureShape,
    },
    /// Wall rubble, a single tile by default: `Rubble()`.
    Rubble {
        #[serde(default)]
        size: Option<FeatureSize>,
    },
}

impl StructureKind {
    /// Name used for `{kind}_{index}` structure names and in messages.
    pub(super) fn label(&self) -> &str {
        match self {
            Self::Structure(name) => name,
            Self::EmptySquare { .. } => "EmptySquare",
            Self::Pond { .. } => "Pond",
            Self::Rubble { .. } => "Rubble",
        }
    }

    pub(super) fn feature_size(&self) -> Option<FeatureSize> {
        match self {
            Self::Structure(_) => None,
            Self::EmptySquare { size } | Self::Pond { size, .. } | Self::Rubble { size } => *size,
        }
    }
}

fn random_location() -> Location {
    Location::Random {
        x_deciles: interior_deciles(),
    }
}

pub(super) fn resolved_name(kind: &str, index: usize, explicit: &str) -> String {
    if explicit.is_empty() {
        format!("{kind}_{index}")
    } else {
        explicit.to_owned()
    }
}

/// One line of a recipe's `spawns`: what to place, where, and how many.
///
/// ```ron
/// (what: PlayerStart, at: Edge(Left)),
/// (what: Enemy(FireMage), at: MapMin((5, 5))),
/// (what: RandomEnemy, count: (10, 15), at: Random(x_deciles: (2, 8)), separation: 5),
/// (what: Chest, count: (3, 5), at: Near(structure: "Temple", radius: 6)),
/// ```
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SpawnEntry {
    pub(super) what: SpawnKind,
    /// How many to place; a bare number or an inclusive `(min, max)`. Defaults to 1.
    #[serde(default = "one_spawn")]
    pub(super) count: CountRange,
    pub(super) at: Location,
    /// Minimum tile distance from every point placed so far (`Random` and `Near` only).
    #[serde(default)]
    pub(super) separation: u32,
}

fn one_spawn() -> CountRange {
    CountRange::exactly(1)
}

/// What a spawn entry places.
#[derive(Clone, Copy, Debug, Deserialize)]
pub(super) enum SpawnKind {
    PlayerStart,
    Exit,
    Chest,
    Enemy(EnemyKind),
    /// An enemy picked uniformly from `EnemyKind::ALL`, rolled per spawn.
    RandomEnemy,
    Npc(NpcKind),
}

impl SpawnKind {
    /// Build stage this kind's sampled (non-fixed) spawns run in.
    pub(super) fn stage(self) -> Stage {
        match self {
            Self::PlayerStart => Stage::PlayerStart,
            Self::Exit => Stage::Exits,
            Self::Enemy(_) | Self::RandomEnemy => Stage::Enemies,
            Self::Chest => Stage::Chests,
            Self::Npc(_) => Stage::Npcs,
        }
    }

    /// Resolves to a concrete placement, rolling `RandomEnemy`.
    pub(super) fn placement(self, random_generator: &mut ChaCha8Rng) -> PlacementKind {
        match self {
            Self::PlayerStart => PlacementKind::PlayerStart,
            Self::Exit => PlacementKind::Exit,
            Self::Chest => PlacementKind::Chest,
            Self::Enemy(kind) => PlacementKind::Enemy(kind),
            Self::RandomEnemy => {
                let index = random_generator.random_range(0..EnemyKind::ALL.len());
                PlacementKind::Enemy(EnemyKind::ALL[index])
            }
            Self::Npc(kind) => PlacementKind::Npc(kind),
        }
    }
}

/// Where a spawn or structure goes. `Random`, `Edge`, `Near` and `Choose` sample a spot;
/// the rest are fixed anchors, which place exactly one. Positions are measured to the
/// center of what is placed, except the `Map*` anchors and `Edge`, which place its
/// bottom-left corner.
#[derive(Clone, Debug, Deserialize)]
pub(super) enum Location {
    /// Anywhere inside the half-open x-decile band, `(1, 9)` by default: `Random()`.
    Random {
        #[serde(default = "interior_deciles")]
        x_deciles: DecileRange,
    },
    /// One tile inside a wall; several spawns spread along it: `Edge(Left)`.
    Edge(Side),
    /// Within `radius` tiles (a square) of a named structure's center plus `offset`.
    Near {
        structure: String,
        #[serde(default)]
        offset: IVec2,
        radius: u32,
    },
    /// One of these, picked per map.
    Choose(Vec<Self>),
    Center,
    CenterOffset(IVec2),
    MapMin(IVec2),
    /// Offset from the top-right tile: `MapMax((-6, -6))`.
    MapMax(IVec2),
    /// An earlier structure's center plus `offset`.
    Structure {
        structure: String,
        #[serde(default)]
        offset: IVec2,
    },
}

fn interior_deciles() -> DecileRange {
    DecileRange { min: 1, max: 9 }
}

impl Location {
    /// Fixed anchors place exactly one and run before every sampled spawn.
    pub(super) fn is_fixed(&self) -> bool {
        !matches!(
            self,
            Self::Random { .. } | Self::Edge(_) | Self::Near { .. } | Self::Choose(_)
        )
    }

    /// Builds the placement rule for `count` spawns, picking one option of a `Choose`.
    pub(super) fn rule(
        &self,
        count: u32,
        separation: u32,
        random_generator: &mut ChaCha8Rng,
    ) -> MapResult<PlaceRule> {
        let count_range = CountRange::exactly(count);
        Ok(match self {
            Self::Random { x_deciles } => PlaceRule::Random {
                count: count_range,
                x_deciles: *x_deciles,
                separation,
            },
            Self::Edge(side) => PlaceRule::Edge {
                count: count_range,
                side: *side,
            },
            Self::Near {
                structure,
                offset,
                radius,
            } => PlaceRule::NearStructure {
                count: count_range,
                structure: structure.clone(),
                offset: *offset,
                radius: *radius,
                separation,
            },
            Self::Choose(options) => {
                if options.is_empty() {
                    return Err("spawn choice list is empty".into());
                }
                let index = random_generator.random_range(0..options.len());
                return options[index].rule(count, separation, random_generator);
            }
            Self::Center => PlaceRule::Fixed(Anchor::Center),
            Self::CenterOffset(offset) => PlaceRule::Fixed(Anchor::CenterOffset(*offset)),
            Self::MapMin(offset) => PlaceRule::Fixed(Anchor::MapMin(*offset)),
            Self::MapMax(offset) => PlaceRule::Fixed(Anchor::MapMax(*offset)),
            Self::Structure { structure, offset } => PlaceRule::Fixed(Anchor::Structure {
                name: structure.clone(),
                offset: *offset,
            }),
        })
    }
}
