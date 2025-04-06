mod gold;

use bevy::prelude::*;

use crate::labels::sets::InGameSet;

pub struct EconomyPlugin;

/// Shop and Gold Logic
impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            gold::handle_gold_collisions.in_set(InGameSet::Collision),
        )
        .add_observer(gold::on_gold_drop_event);
    }
}

pub use gold::Gold;
pub use gold::GoldDropEvent;
