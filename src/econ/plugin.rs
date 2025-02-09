use bevy::prelude::*;

use super::handle_gold_drop_event::on_gold_drop_event;

pub struct EconPlugin;

//Shop / Coin Logic
impl Plugin for EconPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_gold_drop_event);
    }
}
