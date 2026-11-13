use std::fmt::Debug;

use bevy::prelude::*;
use bevy::remote::builtin_methods::parse;
use bevy::remote::{BrpError, BrpResult, error_codes};
use serde::Deserialize;
use serde_json::{Value, json};

/// Reads a method's params; a call without params reads as `{}`, so all-optional params work.
pub(super) fn parse_params<T: for<'de> Deserialize<'de>>(params: Option<Value>) -> BrpResult<T> {
    parse(params.unwrap_or_else(|| json!({})))
}

/// The error for params that parse but name something the game doesn't have.
pub(super) fn invalid_params(message: impl Into<String>) -> BrpError {
    BrpError {
        code: error_codes::INVALID_PARAMS,
        message: message.into(),
        data: None,
    }
}

/// Reads an entity written by [`entity_id`].
pub(super) fn parse_entity(id: u64) -> BrpResult<Entity> {
    Entity::try_from_bits(id).ok_or_else(|| invalid_params(format!("{id} is not an entity")))
}

/// An entity as every `game/…` method writes and reads it.
pub(super) fn entity_id(entity: Entity) -> u64 {
    entity.to_bits()
}

/// A transform's world position as `[x, y]`.
pub(super) fn position(transform: &Transform) -> [f32; 2] {
    transform.translation.truncate().to_array()
}

/// A value's `Debug` text, which for a fieldless enum is its variant name.
pub(super) fn debug_name(value: impl Debug) -> String {
    format!("{value:?}")
}
