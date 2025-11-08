pub mod constants;
mod damage_overlay;
pub mod primitives;

use bevy::app::App;

pub(super) fn plugin(app: &mut App) {
    // Heal and damage overlays
    app.add_observer(damage_overlay::on_damage_overlay_amount)
        .add_observer(damage_overlay::on_healing_overlay_amount);
}
