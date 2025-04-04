mod event;
mod log;
use anyhow::anyhow;
use anyhow::Result;
use async_channel::{Receiver, Sender};
use baba_yaga::console::NetResponseMsg;
use event::Events;
use event::StreamEvent;
use futures_concurrency::prelude::*;
use futures_lite::{future, StreamExt};
use std::time::Duration;
use std::time::Instant;
use tracing::*;

use baba_yaga::console::NetRequestMsg;
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
    DefaultTerminal, Frame,
};

fn main() -> Result<()> {
    log::init()?;
    future::block_on(_main(async_executor::Executor::new()))
}
pub struct Game(NetRequestMsg);

async fn _main(ex: async_executor::Executor<'_>) -> Result<()> {
    let (tx_command, rx_command) = async_channel::unbounded::<Game>();
    let (tx_update, rx_update) = async_channel::unbounded::<NetResponseMsg>();

    // Spawn a dedicated task loop for network calls
    ex.spawn(async move {
        info!("init");
        loop {
            info!("waiting for app request");
            let Ok(Game(NetRequestMsg { request, reply })) = rx_command.recv().await else {
                return;
            };
            // main TCP listener here
            debug!("{:?}", request);
            match reply.send(baba_yaga::console::NetResponseMsg::OK).await {
                Ok(_) => {}
                Err(e) => error!("{}", e.to_string()),
            };
        }
    })
    .detach();

    // Block main as the UI thread
    future::block_on(ex.run(async {
        let mut terminal = ratatui::init();
        let app_result = App::new(tx_command, tx_update, rx_update).run(&mut terminal).await;
        ratatui::restore();
        app_result
    }))
}

#[derive(Debug)]
pub struct App {
    tx_command: Sender<Game>,
    tx_update: Sender<NetResponseMsg>,
    rx_update: Receiver<NetResponseMsg>,
    buffer: String,
    exit: bool,
}

impl App {
    pub fn new(
        tx_command: Sender<Game>,
        tx_update: Sender<NetResponseMsg>,
        rx_update: Receiver<NetResponseMsg>,
    ) -> Self {
        Self {
            tx_command,
            tx_update,
            rx_update,
            buffer: String::new(),
            exit: false,
        }
    }
    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        let stream = EventStream::new();
        let rx_update = self.rx_update.clone();
        let frame_tick = async_io::Timer::interval_at(Instant::now(), Duration::from_secs_f32(1. / 2.));

        let mut events = Events::new(rx_update, stream, frame_tick);

        // poll, update, render. standard app loop
        while !self.exit {
            match events.next().await {
                Some(StreamEvent::Crossterm(event)) => {
                    debug!("event in app loop: {event:?}");
                    self.handle_events(event)
                }
                Some(StreamEvent::Io(msg)) => {
                    debug!("msg in app loop: {msg:?}");
                    anyhow::Ok(())
                }
                Some(StreamEvent::Tick) => {
                    debug!("t");
                    anyhow::Ok(())
                }
                Some(StreamEvent::Error) => {
                    unimplemented!();
                }
                None => {
                    unimplemented!();
                }
            }?;
            self.update()?;
            terminal.draw(|frame| self.draw(frame))?;
        }
        Ok(())
    }

    fn update(&self) -> Result<()> {
        Ok(())
    }

    //**Handlers**
    /// updates the application's state based on user input
    fn handle_events(&mut self, event: crossterm::event::Event) -> Result<()> {
        match event {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => self.handle_key_event(key_event),
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char(k) => self.add_buffer(k),
            KeyCode::Tab => self.switch(),
            _ => {}
        }
    }

    //**Render**
    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    //**Helpers**
    fn add_buffer(&mut self, k: char) {
        self.buffer.push(k);
    }

    fn switch(&mut self) {
        // TODO:
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Counter App Tutorial ".bold());
        let instructions = Line::from(vec![
            " Decrement ".into(),
            "<Left>".blue().bold(),
            " Increment ".into(),
            "<Right>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let counter_text = Text::from(vec![Line::from(vec!["Value: ".into(), self.buffer.clone().into()])]);

        Paragraph::new(counter_text).centered().block(block).render(area, buf);
    }
}
