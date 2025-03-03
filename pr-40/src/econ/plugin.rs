use bevy::prelude::*;

use crate::labels::sets::InGameSet;

use super::{currency::handle_currency_collisions, gold_drop::on_gold_drop_event};

pub struct EconPlugin;

//Shop / Coin Logic
impl Plugin for EconPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            handle_currency_collisions.in_set(InGameSet::Simulation),
        )
        .add_observer(on_gold_drop_event);
    }
}
