use bevy::prelude::*;

use baba_yaga::configuration::AppRuntimePlugin;

fn main() {
    App::new().add_plugins(AppRuntimePlugin).run();
}
