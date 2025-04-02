use bevy::prelude::*;

use baba_yaga::configuration::plugins::NativePlugins;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    App::new().add_plugins(NativePlugins).run();
    Ok(())
}
