use tracing::level_filters::LevelFilter;
use tracing_subscriber::{self, layer::SubscriberExt, util::SubscriberInitExt, Layer};

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
