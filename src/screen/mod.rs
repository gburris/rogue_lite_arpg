mod game_over;
mod loading;
mod splash;

use bevy::app::App;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((game_over::plugin, loading::plugin, splash::plugin));
}
