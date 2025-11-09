use bevy::prelude::*;

use crate::{
    prelude::*,
    ui::{
        constants::{color, font_size},
        element::Element,
    },
};

use super::primitives::text;

#[derive(Component)]
pub struct GameOverScreen;

#[derive(Component)]
pub struct RestartButton;

pub fn spawn(mut commands: Commands) {
    commands.spawn((
        GameOverScreen,
        DespawnOnExit(AppState::GameOver),
        Element::new()
            .width(percent(100.0))
            .height(percent(100.0))
            .flex_direction(FlexDirection::Column)
            .align_items(AlignItems::Center)
            .padding(px(200.0).top())
            .row_gap(px(20.0))
            .background_color(Color::BLACK)
            .global_z_index(1),
        children![
            text("Game Over!", font_size::TITLE),
            (
                Button,
                RestartButton,
                Element::new()
                    .width(px(150.0))
                    .height(px(65.0))
                    .border(px(5.0).all())
                    .justify_content(JustifyContent::Center)
                    .align_items(AlignItems::Center)
                    .border_color(BorderColor::all(Color::BLACK))
                    .border_radius(BorderRadius::MAX)
                    .background_color(color::BUTTON_BACKGROUND),
                children![Text::new("Restart"), Observer::new(on_restart_clicked)]
            )
        ],
    ));
}

/// Passes players current level to the next instance of the game, despawns everything and starts again
fn on_restart_clicked(_: On<Pointer<Click>>, mut commands: Commands, player: Single<&Player>) {
    commands.trigger(RestartEvent {
        player_level: player.get_level(),
    });
}
