use bevy::prelude::*;
use bevy::utils::tracing::{info, warn};
use std::net::Ipv4Addr;
use std::net::SocketAddr;

use async_channel::Receiver;
use bevy::{
    ecs::system::SystemState,
    tasks::{IoTaskPool, block_on, poll_once},
};

use super::tcp;
use crate::bevy;
use crate::nc;

#[derive(Resource)]
pub struct NetChannels {
    rx_command: Receiver<nc::Request>,
}

#[derive(Resource, Debug)]
pub struct HostConsoleAddr(pub SocketAddr);
impl Default for HostConsoleAddr {
    fn default() -> Self {
        Self(SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), 9191))
    }
}

pub fn setup_host_console(addr: Res<HostConsoleAddr>, mut commands: Commands) {
    let (tx_command, rx_command) = async_channel::unbounded();
    commands.insert_resource(NetChannels { rx_command });
    let addr = addr.0;
    IoTaskPool::get()
        .spawn(async move { tcp::net_listener(addr, tx_command).await })
        .detach()
}

pub fn update_host_console(world: &mut World, params: &mut SystemState<Res<NetChannels>>) {
    let net_channels = params.get(world);
    let poll = block_on(poll_once(net_channels.rx_command.recv()));
    let (cmd, tx) = match poll {
        Some(Ok(nc::Request {
            request: cmd,
            reply: tx,
        })) => (cmd, tx),
        Some(Err(e)) => return warn!("{}", e),
        None => return,
    };

    info!("Received net command: {cmd:?}");
    let reply = cmd.exec(world);
    let reply = match reply {
        Ok(msg) => nc::Response::Ron(msg),
        Err(e) => {
            warn!("Command error: {e}");
            nc::Response::Reply(e.to_string())
        }
    };
    IoTaskPool::get().spawn(async move { tx.send(reply).await }).detach();
}
