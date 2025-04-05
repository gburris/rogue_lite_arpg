//! 4 basic message types
//!
//! These messages are exchanged between client and server, over network:
//! - [`Request`]
//! - [`Response`]
//!
//! These messages are exchanged between the server and the bevy engine:
//! - [`Command`]
//! - [`CommandResult`]
use std::error::Error;

use crate::bevy;
use anyhow::Result;
use anyhow::anyhow;
use async_channel::Sender;
use bevy::ecs::component::ComponentInfo;
use bevy::prelude::*;
use bevy::ptr::Ptr;
use bevy::reflect::TypeRegistry;
use bevy::reflect::serde::ReflectSerializer;
use bevy::reflect::serde::TypedReflectDeserializer;
use bevy_asset::ron;
use bevy_asset::ron::ser::PrettyConfig;
use bevy_utils::tracing::debug;
use humansize::DECIMAL;
use humansize::format_size;
use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeSeed;

/// A command message sent from a connection handler to the Bevy world.
/// Each message carries its own reply sender.
#[derive(Clone, Debug)]
pub struct Request {
    pub request: Command,
    pub reply: Sender<Response>,
}

/// Messages we send to our netcode task
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Response {
    Ron(CommandResult),
    Reply(String),
    OK,
}
impl From<CommandResult> for Response {
    fn from(value: CommandResult) -> Self {
        Self::Ron(value)
    }
}
impl<T: Error> From<T> for Response {
    fn from(value: T) -> Self {
        Self::Reply(value.to_string())
    }
}

/// The command types available.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Command {
    Get(String),
    Set(String, String),
    DumpResources,
    Archetypes,
    EntityCount,
    Help,
}

impl Command {
    /// Parses an input string into a command.
    /// Expected syntax:
    /// - get [resource]
    /// - resources
    /// - entity_count
    /// - set [value]
    /// - help
    pub fn parse(expr: &str) -> Result<Command> {
        let mut parts = expr.split_whitespace();
        match parts.next() {
            Some("get") => {
                let arg = parts.next().ok_or_else(|| anyhow!("Missing argument for 'get'"))?;
                Ok(Command::Get(arg.to_string()))
            }
            Some("resources") => Ok(Command::DumpResources),
            Some("archetypes") => Ok(Command::Archetypes),
            Some("entity_count") => Ok(Command::EntityCount),
            Some("set") => {
                let ty = parts.next().ok_or_else(|| anyhow!("Missing type for 'set'"))?;
                let value = parts.collect::<Vec<_>>().join(" ");
                Ok(Command::Set(ty.to_string(), value))
            }
            Some("help") => Ok(Command::Help),
            Some(cmd) => Err(anyhow!("Unknown command: {}", cmd)),
            None => Err(anyhow!("Empty input")),
        }
    }

    pub fn exec(&self, world: &mut World) -> Result<CommandResult> {
        match self {
            Command::Get(arg) => cmd_get(world, arg),
            Command::DumpResources => cmd_resources(world),
            Command::EntityCount => cmd_entity_count(world),
            Command::Set(ty, value) => cmd_set(world, ty, value),
            Command::Help => Ok(CommandResult::Help(
                "Available commands: resources, get [resource], entity_count, set [value], help".into(),
            )),
            Command::Archetypes => cmd_archetypes(world),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CommandResult {
    Get(String),
    EntityCount(usize),
    Resources(Vec<(String, String, usize)>),
    Archetypes(Vec<(usize, Vec<(String, usize)>, usize, usize, String)>),
    Help(String),
    OK,
}

/// Retrieves a resource by name using Bevy’s reflection system.
/// The unsafe block is justified because we know that the resource data is valid for the lifetime
/// of the call and Bevy’s API ensures that the reflection is sound.
fn cmd_get(world: &mut World, ty: &str) -> Result<CommandResult> {
    let registry = world.resource::<AppTypeRegistry>().read();

    let registration = registry
        .get_with_short_type_path(ty)
        .ok_or_else(|| anyhow!("Type '{}' not found in registry", ty))?;

    let reflect_data = registration.data::<ReflectResource>().ok_or_else(|| {
        anyhow!(
            "ReflectResource missing for type '{}'",
            registration.type_info().type_path()
        )
    })?;

    let value = reflect_data.reflect(world).ok_or_else(|| {
        anyhow!(
            "Resource data not found for type '{}'",
            registration.type_info().type_path()
        )
    })?;

    let refser = ReflectSerializer::new(value, &registry);
    let ron = ron::ser::to_string_pretty(&refser, PrettyConfig::new())?;

    Ok(CommandResult::Get(ron))
}

fn cmd_set(world: &mut World, ty: &str, args: &str) -> Result<CommandResult> {
    world.resource_scope(
        |world: &mut World, registry: Mut<AppTypeRegistry>| -> Result<CommandResult> {
            let registry = registry.read();
            let registration = registry
                .get_with_short_type_path(ty)
                .ok_or_else(|| anyhow!("Type '{}' not found in registry", ty))?;

            let reflect_data = registration.data::<ReflectResource>().ok_or_else(|| {
                anyhow!(
                    "ReflectResource missing for type '{}'",
                    registration.type_info().type_path()
                )
            })?;

            let mut value = reflect_data.reflect_mut(world).ok_or_else(|| {
                anyhow!(
                    "Resource data not found for type '{}'",
                    registration.type_info().type_path()
                )
            })?;
            let mut deserializer = ron::Deserializer::from_str(args)?;
            let reflect_deserializer = TypedReflectDeserializer::new(registration, &registry);
            let result = reflect_deserializer.deserialize(&mut deserializer)?;
            value.apply(&*result);
            Ok(CommandResult::OK)
        },
    )
}

/// Dumps a list of resources, including their short type paths, names, and sizes.
fn cmd_resources(world: &mut World) -> Result<CommandResult> {
    fn process_resource(
        (info, _data): (&ComponentInfo, Ptr<'_>),
        registry: &TypeRegistry,
    ) -> Option<(String, String, usize)> {
        info.type_id().and_then(|i| registry.get_type_info(i)).map(|tinfo| {
            (
                tinfo.type_path_table().short_path().to_string(),
                format_size(info.layout().size(), DECIMAL),
                info.layout().size(),
            )
        })
    }

    let registry = world.resource::<AppTypeRegistry>().read();
    let mut info = world
        .iter_resources()
        .filter_map(|resource| process_resource(resource, &registry))
        .collect::<Vec<_>>();
    info.sort_by_key(|(_, _, key)| *key);
    Ok(CommandResult::Resources(info))
}

/// Counts the number of entities in the world.
fn cmd_entity_count(world: &mut World) -> Result<CommandResult> {
    let count = world.iter_entities().count();
    Ok(CommandResult::EntityCount(count))
}

fn cmd_archetypes(world: &mut World) -> Result<CommandResult> {
    let registry = world.resource::<AppTypeRegistry>().read();
    let components = world.components();
    let mut archetypes = world.archetypes().iter().collect::<Vec<_>>();
    archetypes.sort_by_key(|a| a.len());

    let mut archetype_info = vec![];
    for a in archetypes {
        let component_info = a
            .components()
            .filter_map(|c| components.get_info(c))
            .map(|info| {
                let name = info
                    .type_id()
                    .and_then(|id| registry.get_type_info(id))
                    .map_or(info.name(), |ti| ti.type_path_table().short_path());
                let size_bytes = info.layout().size();
                (name.to_string(), size_bytes)
            })
            .collect::<Vec<_>>();
        let ron = ron::ser::to_string_pretty(&component_info, PrettyConfig::default())?;
        debug!("{:?} = {}\n{}", a.id(), a.len(), ron);
        let sum_archetype: usize = component_info.iter().map(|(_, size)| size).sum();
        archetype_info.push((
            a.id().index(),
            component_info,
            sum_archetype,
            a.len(),
            format_size(sum_archetype * a.len(), DECIMAL),
        ));
    }
    Ok(CommandResult::Archetypes(archetype_info))
}
