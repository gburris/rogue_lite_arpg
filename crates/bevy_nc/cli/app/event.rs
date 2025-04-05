use std::pin::Pin;

use async_channel::Receiver;
use bevy_nc::nc;
use crossterm::event::{Event as CrosstermEvent, *};
use futures_concurrency::stream::{IntoStream, Merge};
use futures_lite::{Stream, StreamExt};

pub struct Events(Pin<Box<dyn Stream<Item = StreamEvent>>>);

#[derive(Clone, Debug)]
pub enum StreamEvent {
    Io(nc::Response),
    Crossterm(CrosstermEvent),
    Tick,
    Error,
}
impl Events {
    pub fn new(rx_command: Receiver<nc::Response>, crossterm: EventStream, tick: async_io::Timer) -> Self {
        let streams = [
            netcmd_stream(rx_command),
            crossterm_stream(crossterm),
            tick_stream(tick),
        ]
        .merge();
        Self(Box::pin(streams))
    }

    pub async fn next(&mut self) -> Option<StreamEvent> {
        self.0.next().await
    }
}
fn netcmd_stream(rx_command: Receiver<nc::Response>) -> Pin<Box<dyn Stream<Item = StreamEvent>>> {
    Box::pin(rx_command.into_stream().map(StreamEvent::Io))
}

fn crossterm_stream(crossterm: EventStream) -> Pin<Box<dyn Stream<Item = StreamEvent>>> {
    Box::pin(crossterm.fuse().filter_map(|event| match event {
        // Ignore key release / repeat events
        Ok(CrosstermEvent::Key(key)) if key.kind == KeyEventKind::Release => None,
        Ok(event) => Some(StreamEvent::Crossterm(event)),
        Err(_) => Some(StreamEvent::Error),
    }))
}
fn tick_stream(tick: async_io::Timer) -> Pin<Box<dyn Stream<Item = StreamEvent>>> {
    Box::pin(tick.into_stream().map(|_| StreamEvent::Tick))
}
