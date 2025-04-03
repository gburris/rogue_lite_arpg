mod net;
mod plugin;
use anyhow::anyhow;
use anyhow::Result;
use net::NetCommand;
use net::NetCommandResult;
pub use plugin::ConsolePlugin;
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
struct NetRequestMsg {
    request: NetCommand,
    reply: Sender<NetResponseMsg>,
}

/// Messages we send to our netcode task
pub enum NetResponseMsg {
    Ron(NetCommandResult),
    Reply(String),
    OK,
}

#[derive(Resource)]
pub struct NetChannels {
    rx_command: Receiver<NetRequestMsg>,
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

async fn net_listener(tx_command: async_channel::Sender<NetRequestMsg>) {
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
    let Some(Ok(NetRequestMsg {
        request: cmd,
        reply: tx,
    })) = block_on(poll_once(net_channels.rx_command.recv()))
    else {
        return;
    };
    info!("Received net command: {cmd:?}");

    let reply = cmd.exec(world);
    let reply = match reply {
        Ok(msg) => NetResponseMsg::Ron(msg),
        Err(e) => {
            warn!("Command error: {e}");
            NetResponseMsg::Reply(e.to_string())
        }
    };
    IoTaskPool::get().spawn(async move { tx.send(reply).await }).detach();
}

async fn handle_stream(
    mut stream: std::net::TcpStream,
    tx_command: async_channel::Sender<NetRequestMsg>,
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
        let cmd = match NetCommand::parse(&msg_input) {
            Ok(cmd) => cmd,
            Err(e) => {
                let err_msg = format!("Error parsing command: {e}\n");
                stream.write_all(err_msg.as_bytes())?;
                return Err(anyhow!("{err_msg}"));
            }
        };

        // Send the command to the Bevy system.
        let net_msg = NetRequestMsg {
            request: cmd,
            reply: reply_tx,
        };
        tx_command.send(net_msg).await?;
        match reply_rx.recv().await? {
            NetResponseMsg::Reply(result_msg) => stream.write_all(result_msg.as_bytes())?,
            NetResponseMsg::OK => stream.write_all(b"OK")?,
            NetResponseMsg::Ron(ron_msg) => ron::ser::to_writer_pretty(&stream, &ron_msg, PrettyConfig::default())?,
        };
    }
}
