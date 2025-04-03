mod plugin;
use anyhow::anyhow;
use anyhow::Result;
use bevy::reflect::serde::ReflectDeserializer;
use bevy::reflect::serde::ReflectSerializer;
use bevy::reflect::serde::TypedReflectDeserializer;
use bevy::reflect::DynamicStruct;
use bevy::reflect::ReflectFromPtr;
use bevy::reflect::TypeData;
use bevy::reflect::TypeInfo;
use humansize::format_size;
use humansize::DECIMAL;
pub use plugin::ConsolePlugin;
use serde::de::DeserializeSeed;
use serde::Deserialize;
use serde::Serialize;
use std::{
    io::{BufRead, Write},
    net::TcpListener,
};

use async_channel::{Receiver, Sender};
use bevy::{
    ecs::system::SystemState,
    prelude::*,
    scene::ron::{self, ser::PrettyConfig},
    tasks::{block_on, poll_once, IoTaskPool},
};

/// A command message sent from a connection handler to the Bevy world.
/// Each message carries its own reply sender.
struct NetCommandMsg {
    request: NetCommand,
    reply: Sender<NetReplyMsg>,
}

/// The command types available.
#[derive(Debug)]
enum NetCommand {
    Get(String),
    Set(String, String),
    DumpResources,
    EntityCount,
    Help,
}

/// Messages we send to our netcode task
enum NetReplyMsg {
    Reply(String),
    OK,
}

#[derive(Resource)]
pub struct NetChannels {
    rx_command: Receiver<NetCommandMsg>,
}

#[derive(Component)]
pub struct DebugConsole;

fn setup_console(mut commands: Commands) {
    let (tx_command, rx_command) = async_channel::unbounded();
    commands.insert_resource(NetChannels { rx_command });

    IoTaskPool::get()
        .spawn(async move { net_listener(tx_command).await })
        .detach()
}

async fn net_listener(tx_command: async_channel::Sender<NetCommandMsg>) {
    info!("setting up net listener");
    let listener = match TcpListener::bind("127.0.0.1:8080") {
        Ok(listener) => listener,
        Err(e) => {
            warn!("unable to setup listener: {e}");
            return;
        }
    };
    for stream in listener.incoming() {
        info!("got stream");
        match stream {
            Ok(stream) => {
                if let Err(e) = handle_stream(stream, tx_command.clone()).await {
                    warn!("stream error: {e}");
                }
            }
            Err(e) => {
                warn!("unable to setup stream: {e}");
            }
        };
    }
}

fn update_console(world: &mut World, params: &mut SystemState<Res<NetChannels>>) {
    let net_channels = params.get(world);
    let Some(Ok(NetCommandMsg {
        request: cmd,
        reply: tx,
    })) = block_on(poll_once(net_channels.rx_command.recv()))
    else {
        return;
    };
    info!("Received net command: {cmd:?}");

    let reply = match cmd {
        NetCommand::Get(arg) => cmd_get(world, &arg),
        NetCommand::DumpResources => cmd_resources(world),
        NetCommand::EntityCount => cmd_entity_count(world),
        NetCommand::Set(ty, value) => cmd_set(world, &ty, &value),
        NetCommand::Help => Ok(NetReplyMsg::Reply(
            "Available commands: resources, get [resource], entity_count, set [value], help".into(),
        )),
    };

    let reply = match reply {
        Ok(msg) => msg,
        Err(e) => {
            warn!("Command error: {e}");
            NetReplyMsg::Reply(e.to_string())
        }
    };
    IoTaskPool::get().spawn(async move { tx.send(reply).await }).detach();
}

/// Retrieves a resource by name using Bevy’s reflection system.
/// The unsafe block is justified because we know that the resource data is valid for the lifetime
/// of the call and Bevy’s API ensures that the reflection is sound.
fn cmd_get(world: &mut World, ty: &str) -> Result<NetReplyMsg> {
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

    Ok(NetReplyMsg::Reply(ron))
}

// TODO: quick and dirty
#[derive(Debug, Serialize, Deserialize)]
struct KVPair(String, String);

fn cmd_set(world: &mut World, ty: &str, args: &str) -> Result<NetReplyMsg> {
    world.resource_scope(|w: &mut World, registry: Mut<AppTypeRegistry>| -> Result<()> {
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

        let mut value = reflect_data.reflect_mut(w).ok_or_else(|| {
            anyhow!(
                "Resource data not found for type '{}'",
                registration.type_info().type_path()
            )
        })?;
        let mut deserializer = ron::Deserializer::from_str(args)?;
        let reflect_deserializer = TypedReflectDeserializer::new(registration, &registry);
        let result = reflect_deserializer.deserialize(&mut deserializer)?;
        value.apply(&*result);
        Ok(())
    })?;
    Ok(NetReplyMsg::OK)
}

/// Dumps a list of resources, including their short type paths, names, and sizes.
fn cmd_resources(world: &mut World) -> Result<NetReplyMsg> {
    let registry = world.resource::<AppTypeRegistry>().read();
    let info = world
        .iter_resources()
        .inspect(|(ci, _)| {
            info!(
                "{:?}: {:?}",
                ci.name(),
                ci.type_id().and_then(|i| registry.get_type_info(i))
            )
        })
        .filter_map(|(info, _data)| {
            info.type_id().and_then(|i| registry.get_type_info(i)).map(|tinfo| {
                (
                    tinfo.type_path_table().path(),
                    tinfo.type_path_table().short_path(),
                    format_size(info.layout().size(), DECIMAL),
                )
            })
        })
        .collect::<Vec<_>>();
    let ron = ron::ser::to_string_pretty(&info, PrettyConfig::default())?;
    Ok(NetReplyMsg::Reply(ron))
}

/// Counts the number of entities in the world.
fn cmd_entity_count(world: &mut World) -> Result<NetReplyMsg> {
    let count = world.iter_entities().count();
    Ok(NetReplyMsg::Reply(format!("entity count: {}", count)))
}

async fn handle_stream(
    mut stream: std::net::TcpStream,
    tx_command: async_channel::Sender<NetCommandMsg>,
) -> Result<()> {
    let mut rdr = stream.try_clone()?;
    let mut reader = std::io::BufReader::new(&mut rdr);

    loop {
        let mut msg_input = String::new();
        let bytes_read = reader.read_line(&mut msg_input)?;
        if bytes_read == 0 {
            // End-of-stream, connection closed.
            return Ok(());
        }
        info!("received input: {msg_input}");

        // Create a one-shot channel for the reply.
        let (reply_tx, reply_rx) = async_channel::bounded(1);
        let cmd = match parse(&msg_input) {
            Ok(cmd) => cmd,
            Err(e) => {
                let err_msg = format!("Error parsing command: {e}\n");
                stream.write_all(err_msg.as_bytes())?;
                return Err(anyhow!("{err_msg}"));
            }
        };

        // Send the command to the Bevy system.
        let net_msg = NetCommandMsg {
            request: cmd,
            reply: reply_tx,
        };
        tx_command.send(net_msg).await?;
        match reply_rx.recv().await? {
            NetReplyMsg::Reply(result_msg) => stream.write_all(result_msg.as_bytes())?,
            NetReplyMsg::OK => stream.write_all(b"OK")?,
        };
    }
}

/// Parses an input string into a command.
/// Expected syntax:
/// - get [resource]
/// - resources
/// - entity_count
/// - set [value]
/// - help
fn parse(expr: &str) -> Result<NetCommand> {
    let mut parts = expr.split_whitespace();
    match parts.next() {
        Some("get") => {
            let arg = parts.next().ok_or_else(|| anyhow!("Missing argument for 'get'"))?;
            Ok(NetCommand::Get(arg.to_string()))
        }
        Some("resources") => Ok(NetCommand::DumpResources),
        Some("entity_count") => Ok(NetCommand::EntityCount),
        Some("set") => {
            let ty = parts.next().ok_or_else(|| anyhow!("Missing type for 'set'"))?;
            let value = parts.collect::<Vec<_>>().join(" ");
            Ok(NetCommand::Set(ty.to_string(), value))
        }
        Some("help") => Ok(NetCommand::Help),
        Some(cmd) => Err(anyhow!("Unknown command: {}", cmd)),
        None => Err(anyhow!("Empty input")),
    }
}
