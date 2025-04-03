mod log;
use anyhow::anyhow;
use anyhow::Result;
use async_channel::{Receiver, Sender};
use baba_yaga::console::NetResponseMsg;
use futures_concurrency::prelude::*;
use futures_lite::{future, FutureExt, StreamExt};
use std::io;
use std::time::Duration;
use std::time::Instant;
use tracing::*;
use tracing_subscriber::field::debug;

use baba_yaga::console::{NetCommand, NetRequestMsg};
use crossterm::event::{self, Event, EventStream, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
    DefaultTerminal, Frame,
};

#[derive(Debug)]
pub enum Io {
    Game(NetRequestMsg),
}

fn main() -> Result<()> {
    log::init()?;
    future::block_on(_main(async_executor::Executor::new()))
}

async fn _main(ex: async_executor::Executor<'_>) -> Result<()> {
    let (tx_command, rx_command) = async_channel::unbounded::<Io>();
    let (tx_update, rx_update) = async_channel::unbounded::<NetResponseMsg>();
    // Spawn a dedicated task loop for network calls
    ex.spawn(async move {
        info!("init");
        loop {
            info!("waiting for app request");
            let Ok(Io::Game(NetRequestMsg { request, reply })) = rx_command.recv().await else {
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

    future::block_on(ex.run(async {
        let mut terminal = ratatui::init();
        let app_result = App::new(tx_command, tx_update, rx_update).run(&mut terminal).await;
        ratatui::restore();
        app_result
    }))
}

#[derive(Debug)]
pub struct App {
    tx_command: Sender<Io>,
    tx_update: Sender<NetResponseMsg>,
    rx_update: Receiver<NetResponseMsg>,
    counter: usize,
    exit: bool,
}

impl App {
    pub fn new(tx_command: Sender<Io>, tx_update: Sender<NetResponseMsg>, rx_update: Receiver<NetResponseMsg>) -> Self {
        Self {
            tx_command,
            tx_update,
            rx_update,
            counter: 0,
            exit: false,
        }
    }
    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        let mut events = EventStream::new();
        let rx_update = self.rx_update.clone();
        let mut tick = async_io::Timer::interval_at(Instant::now(), Duration::from_secs_f32(1. / 2.));

        while !self.exit {
            (
                async {
                    let Some(event) = events.next().await.transpose()? else {
                        return Err(anyhow!("event stream was closed"));
                    };
                    debug!("event in app loop: {event:?}");
                    self.handle_events(event)
                },
                async {
                    let msg = rx_update.recv().await?;
                    debug!("msg in app loop: {msg:?}");
                    anyhow::Ok(())
                },
                async {
                    let _ = tick.next().await;
                    debug!("t");
                    anyhow::Ok(())
                },
            )
                .race()
                .await?;
            terminal.draw(|frame| self.draw(frame))?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

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
            KeyCode::Char('q') => self.exit(),
            KeyCode::Left => self.decrement_counter(),
            KeyCode::Right => self.increment_counter(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn increment_counter(&mut self) {
        self.counter += 1;
    }

    fn decrement_counter(&mut self) {
        self.counter -= 1;
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

        let counter_text = Text::from(vec![Line::from(vec![
            "Value: ".into(),
            self.counter.to_string().yellow(),
        ])]);

        Paragraph::new(counter_text).centered().block(block).render(area, buf);
    }
}
