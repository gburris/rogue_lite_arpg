use anyhow::anyhow;
use anyhow::Result;
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

enum NetCommand {
    Query(String),
    DumpResources,
    EntityCount,
    Help,
}

/// Messages we send to our netcode task
enum NetControlMsg {
    Reply(String),
}

/// Messages we receive from our netcode task
enum NetUpdateMsg {
    Request(String),
}
#[derive(Resource)]
pub struct NetChannels {
    tx_control: Sender<NetControlMsg>,
    rx_updates: Receiver<NetUpdateMsg>,
}

#[derive(Component)]
pub struct DebugConsole;

fn setup_console(mut commands: Commands) {
    let (tx_control, rx_control) = async_channel::unbounded();
    let (tx_updates, rx_updates) = async_channel::unbounded();
    commands.insert_resource(NetChannels {
        tx_control,
        rx_updates,
    });

    IoTaskPool::get()
        .spawn(async move { net_listener(rx_control, tx_updates).await })
        .detach()
}

async fn net_listener(
    rx_control: async_channel::Receiver<NetControlMsg>,
    tx_updates: async_channel::Sender<NetUpdateMsg>,
) {
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
                if let Err(e) = handle_stream(stream, rx_control.clone(), tx_updates.clone()).await
                {
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

    let Some(Ok(NetUpdateMsg::Request(expr_msg))) =
        block_on(poll_once(net_channels.rx_updates.recv()))
    else {
        return;
    };

    info!("Received net command: {expr_msg}");
    let tx = net_channels.tx_control.clone();

    let reply = parse(&expr_msg).and_then(|cmd| match cmd {
        NetCommand::Query(arg) => cmd_query(world, &arg),
        NetCommand::DumpResources => cmd_resources(world),
        NetCommand::Help => Ok(NetControlMsg::Reply(
            "Available: resources, query [filter], help".into(),
        )),
        NetCommand::EntityCount => todo!(),
    });
    let reply = match reply {
        Ok(r) => r,
        Err(e) => {
            warn!("err: {e}");
            return;
        }
    };

    IoTaskPool::get()
        .spawn(async move { tx.send(reply).await })
        .detach();
}
fn cmd_query(world: &mut World, arg: &str) -> Result<NetControlMsg> {
    Ok(NetControlMsg::Reply("todo".to_string()))
}
fn cmd_resources(world: &mut World) -> Result<NetControlMsg> {
    let info = world
        .iter_resources()
        .map(|(info, _)| (info.name().to_string(), info.layout().size()))
        .collect::<Vec<_>>();
    let ron = ron::ser::to_string_pretty(&info, PrettyConfig::default())?;
    Ok(NetControlMsg::Reply(ron))
}

fn parse(expr: &str) -> Result<NetCommand> {
    let mut parts = expr.split_whitespace();
    match parts.next() {
        Some("query") => {
            let Some(parts) = parts.next() else {
                return Err(anyhow!("missing argument"));
            };
            Ok(NetCommand::Query(parts.to_string()))
        }
        Some("resources") => Ok(NetCommand::DumpResources),
        Some("help") => Ok(NetCommand::Help),
        Some(cmd) => Err(anyhow!("Unknown command: {cmd}")),
        None => Err(anyhow!("Empty input")),
    }
}

async fn handle_stream(
    mut stream: std::net::TcpStream,
    rx_control: async_channel::Receiver<NetControlMsg>,
    tx_updates: async_channel::Sender<NetUpdateMsg>,
) -> anyhow::Result<()> {
    let mut msg_input = String::new();

    let mut reader = std::io::BufReader::new(&mut stream);
    reader.read_line(&mut msg_input)?;
    info!("read input: {msg_input}");
    tx_updates.send(NetUpdateMsg::Request(msg_input)).await?;
    match rx_control.recv().await? {
        NetControlMsg::Reply(result_msg) => stream.write_all(result_msg.as_bytes())?,
    };
    Ok(())
}
