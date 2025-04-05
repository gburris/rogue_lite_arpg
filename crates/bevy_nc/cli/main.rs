mod app;

use anyhow::Result;
use app::App;
use bevy_nc::nc;
use futures_lite::future;
use tracing::*;

fn main() -> Result<()> {
    log::init()?;
    future::block_on(_main(async_executor::Executor::new()))
}

async fn _main(ex: async_executor::Executor<'_>) -> Result<()> {
    let (tx_command, rx_command) = async_channel::unbounded::<app::ClientMsg>();
    let (tx_update, rx_update) = async_channel::unbounded::<nc::Response>();

    // Spawn a dedicated task loop for network calls
    ex.spawn(async move {
        info!("init");
        loop {
            info!("waiting for app request");
            let Ok(app::ClientMsg(nc::Request { request, reply })) = rx_command.recv().await else {
                return;
            };
            // main TCP listener here
            debug!("{:?}", request);
            match reply.send(nc::Response::OK).await {
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

mod log {
    use tracing::level_filters::LevelFilter;
    use tracing_subscriber::{self, Layer, layer::SubscriberExt, util::SubscriberInitExt};

    pub fn init() -> anyhow::Result<()> {
        let log_file = std::fs::OpenOptions::new().create(true).append(true).open("cli.log")?;
        let file_subscriber = tracing_subscriber::fmt::layer()
            .with_file(true)
            .with_line_number(true)
            .with_writer(log_file)
            .with_target(false)
            .with_ansi(false)
            .with_filter(
                tracing_subscriber::filter::EnvFilter::builder()
                    .with_default_directive(LevelFilter::DEBUG.into())
                    .from_env_lossy(),
            );
        tracing_subscriber::registry().with(file_subscriber).init();
        Ok(())
    }
}
