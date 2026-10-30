use serde::Deserialize;

/// Recipes name these directly, e.g. `(what: Npc(Shopkeeper), ...)` in `map_recipes.ron`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum NpcKind {
    Helper,
    Shopkeeper,
    StatTrainer,
}
