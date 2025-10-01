use bevy::prelude::*;

use baba_yaga::configuration::plugins::NativePlugins;

fn main() {
    App::new().add_plugins(NativePlugins).run();
}
