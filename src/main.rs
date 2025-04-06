use bevy::prelude::*;

use baba_yaga::configuration::RuntimePlugin;

fn main() {
    App::new().add_plugins(RuntimePlugin).run();
}
