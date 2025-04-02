use std::{
    io::{Read, Write},
    net::TcpListener,
};

use async_channel::{Receiver, Sender};
use bevy::{
    prelude::*,
    scene::ron::{self, ser::PrettyConfig},
    tasks::{block_on, poll_once, IoTaskPool},
};

use crate::player::player_data::PlayerData;

pub struct ConsolePlugin;

impl Plugin for ConsolePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_console)
            .add_systems(Update, update_console);
    }
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
    // let engine = rhai::Engine::new();
    // commands.insert_resource(RhaiEngine(engine));

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

async fn handle_stream(
    mut stream: std::net::TcpStream,
    rx_control: async_channel::Receiver<NetControlMsg>,
    tx_updates: async_channel::Sender<NetUpdateMsg>,
) -> anyhow::Result<()> {
    let mut msg_input = String::new();
    stream.read_to_string(&mut msg_input)?;
    info!("read input: {msg_input}");
    tx_updates.send(NetUpdateMsg::Request(msg_input)).await?;
    match rx_control.recv().await? {
        NetControlMsg::Reply(result_msg) => stream.write_all(result_msg.as_bytes())?,
    };
    Ok(())
}
fn update_console(
    mut commands: Commands,
    cfgs: ResMut<PlayerData>,
    // engine: Res<RhaiEngine>,
    net_channels: Res<NetChannels>,
) {
    let Some(Ok(NetUpdateMsg::Request(expr_msg))) =
        block_on(poll_once(net_channels.rx_updates.recv()))
    else {
        return;
    };
    info!("Received net command: {expr_msg}");
    let result = eval(&*cfgs, &expr_msg);
    let tx = net_channels.tx_control.clone();

    IoTaskPool::get()
        .spawn(async move { tx.send(NetControlMsg::Reply(result)).await })
        .detach();
}

fn eval(cfgs: &PlayerData, expr: &str) -> String {
    ron::ser::to_string_pretty(cfgs, PrettyConfig::default()).unwrap_or_else(|e| e.to_string())
}
