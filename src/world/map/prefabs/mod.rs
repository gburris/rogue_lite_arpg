mod empty_square;
mod hub;
mod temple;

use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub use empty_square::build_empty_square;
pub use hub::build_hub;
pub use temple::build_temple;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]

pub enum PrefabType {
    NpcHub,
    Temple,
    EmptySquare,
}

impl FromStr for PrefabType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "NpcHub" => Ok(PrefabType::NpcHub),
            "Temple" => Ok(PrefabType::Temple),
            "EmptySquare" => Ok(PrefabType::EmptySquare),
            _ => Err(format!("Unknown prefab type: {s}")),
        }
    }
}
