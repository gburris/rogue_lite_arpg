use std::io::BufRead;
use std::io::Write;

use anyhow::Result;
use anyhow::anyhow;
use bevy::utils::tracing::info;
use bevy::utils::tracing::warn;
use bevy_asset::ron;
use bevy_asset::ron::ser::PrettyConfig;
use std::net::TcpListener;
use std::net::ToSocketAddrs;

use crate::bevy;
use crate::nc;

pub async fn net_listener(addr: impl ToSocketAddrs, tx_command: async_channel::Sender<nc::Request>) {
    info!("setting up net listener");
    let listener = match TcpListener::bind(addr) {
        Ok(listener) => listener,
        Err(e) => {
            warn!("unable to setup listener: {e}");
            return;
        }
    };
    for client in listener.incoming() {
        info!("got stream");
        let client = match client {
            Ok(s) => s,
            Err(e) => {
                warn!("unable to setup stream: {e}");
                continue;
            }
        };
        let handler = handle_client(client, tx_command.clone());
        if let Err(e) = handler.await {
            warn!("stream error: {e}");
        }
    }
}

pub async fn handle_client(
    mut stream: std::net::TcpStream,
    tx_command: async_channel::Sender<nc::Request>,
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
        let cmd = match nc::Command::parse(&msg_input) {
            Ok(cmd) => cmd,
            Err(e) => {
                let err_msg = format!("Error parsing command: {e}\n");
                stream.write_all(err_msg.as_bytes())?;
                return Err(anyhow!("{err_msg}"));
            }
        };

        // Send the command to the Bevy system.
        let net_msg = nc::Request {
            request: cmd,
            reply: reply_tx,
        };
        tx_command.send(net_msg).await?;
        match reply_rx.recv().await? {
            nc::Response::Reply(result_msg) => stream.write_all(result_msg.as_bytes())?,
            nc::Response::OK => stream.write_all(b"OK")?,
            nc::Response::Ron(ron_msg) => ron::ser::to_writer_pretty(&mut stream, &ron_msg, PrettyConfig::default())?,
        };
    }
}
