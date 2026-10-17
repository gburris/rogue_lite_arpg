mod empty_square;
mod temple;

use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub use empty_square::build_empty_square;
pub use temple::build_temple;

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]

pub enum PrefabType {
    #[default]
    HubTown,
    Temple,
    EmptySquare,
}

impl FromStr for PrefabType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "HubTown" => Ok(PrefabType::HubTown),
            "Temple" => Ok(PrefabType::Temple),
            "EmptySquare" => Ok(PrefabType::EmptySquare),
            _ => Err(format!("Unknown prefab type: {s}")),
        }
    }
}
