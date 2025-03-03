use bevy::prelude::*;

#[derive(Component)]
pub struct PauseBackground;

pub fn spawn_pause_background(mut commands: Commands) {
    trace!("spawn_pause_screen called");
    commands.spawn((
        PauseBackground,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        Visibility::Visible,
        GlobalZIndex(1),
    ));
}
