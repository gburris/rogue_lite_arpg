use anyhow::anyhow;
use anyhow::Result;
use bevy::reflect::serde::ReflectSerializer;
use bevy::reflect::ReflectFromPtr;
use humansize::format_size;
use humansize::DECIMAL;
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

pub struct ConsolePlugin;

impl Plugin for ConsolePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_console)
            .add_systems(Update, update_console);
    }
}

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
        NetCommand::Set(var, value) => cmd_set(world, &var, &value),
        NetCommand::Help => Ok(NetReplyMsg::Reply(
            "Available commands: resources, get [resource], entity_count, set [var] [value], help".into(),
        )),
    };

    let reply = match reply {
        Ok(msg) => msg,
        Err(e) => {
            warn!("err: {e}");
            NetReplyMsg::Reply(e.to_string())
        }
    };
    IoTaskPool::get().spawn(async move { tx.send(reply).await }).detach();
}

/// Retrieves a resource by name using Bevy’s reflection system.
/// The unsafe block is justified because we know that the resource data is valid for the lifetime
/// of the call and Bevy’s API ensures that the reflection is sound.
fn cmd_get(world: &mut World, arg: &str) -> Result<NetReplyMsg> {
    let type_registry = world.resource::<AppTypeRegistry>().read();
    let components = world.components();

    let type_data = type_registry
        .get_with_short_type_path(arg)
        .ok_or_else(|| anyhow!("Type '{}' not found in registry", arg))?;

    let type_info = type_data.type_info();
    let cid = components
        .get_resource_id(type_info.type_id())
        .ok_or_else(|| anyhow!("No resource ID found for type '{}'", type_info.type_path()))?;

    let resource_data = world
        .get_resource_by_id(cid)
        .ok_or_else(|| anyhow!("Resource data not found for type '{}'", type_info.type_path()))?;

    let reflect_data = type_data
        .data::<ReflectFromPtr>()
        .ok_or_else(|| anyhow!("ReflectFromPtr missing for type '{}'", type_info.type_path()))?;

    // SAFETY: We rely on Bevy’s guarantees that the resource’s lifetime is managed and valid.
    let value = unsafe { reflect_data.as_reflect(resource_data) };

    let refser = ReflectSerializer::new(value, &type_registry);
    let ron = ron::ser::to_string_pretty(&refser, PrettyConfig::new())?;

    Ok(NetReplyMsg::Reply(ron))
}
fn cmd_set(world: &mut World, var: &str, value: &str) -> Result<NetReplyMsg> {
    Ok(NetReplyMsg::Reply(format!("Set command received: {} = {}", var, value)))
}

/// Dumps a list of resources, including their short type paths, names, and sizes.
fn cmd_resources(world: &mut World) -> Result<NetReplyMsg> {
    let registry = world.resource::<AppTypeRegistry>().read();
    let info = world
        .iter_resources()
        .filter_map(|(info, _data)| {
            info.type_id().and_then(|i| registry.get_type_info(i)).map(|tinfo| {
                (
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
    Ok(NetReplyMsg::Reply(format!("Entity count: {}", count)))
}

/// Parses an input string into a command.
/// Expected syntax:
/// - get [resource]
/// - resources
/// - entity_count
/// - set [variable] [value]
/// - help
fn parse(expr: &str) -> Result<NetCommand> {
    let mut parts = expr.split_whitespace();
    match parts.next() {
        Some("get") => {
            let arg = parts.next().ok_or_else(|| anyhow!("missing argument for 'get'"))?;
            Ok(NetCommand::Get(arg.to_string()))
        }
        Some("resources") => Ok(NetCommand::DumpResources),
        Some("entity_count") => Ok(NetCommand::EntityCount),
        Some("set") => {
            let var = parts.next().ok_or_else(|| anyhow!("missing variable for 'set'"))?;
            let value = parts.next().ok_or_else(|| anyhow!("missing value for 'set'"))?;
            Ok(NetCommand::Set(var.to_string(), value.to_string()))
        }
        Some("help") => Ok(NetCommand::Help),
        Some(cmd) => Err(anyhow!("Unknown command: {}", cmd)),
        None => Err(anyhow!("Empty input")),
    }
}

async fn handle_stream(
    mut stream: std::net::TcpStream,
    tx_command: async_channel::Sender<NetCommandMsg>,
) -> anyhow::Result<()> {
    let mut msg_input = String::new();

    let mut reader = std::io::BufReader::new(&mut stream);
    reader.read_line(&mut msg_input)?;
    info!("read input: {msg_input}");
    // Create a one-shot channel for the reply.
    let (reply_tx, reply_rx) = async_channel::bounded(1);
    let cmd = match parse(&msg_input) {
        Ok(cmd) => cmd,
        Err(e) => {
            let err_msg = format!("error parsing command: {e}\n");
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
    Ok(())
}
