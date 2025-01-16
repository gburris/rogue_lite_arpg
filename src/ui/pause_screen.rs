use bevy::prelude::*;

#[derive(Component)]
pub struct PauseScreen;

pub fn spawn_pause_screen(mut commands: Commands) {
    warn!("spawn_pause_screen called");
    commands.spawn((
        PauseScreen,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor::from(Color::BLACK.with_alpha(0.9)),
        Visibility::Visible,
        GlobalZIndex(1),
    ));
}

pub fn despawn_pause_screen(
    mut commands: Commands,
    pause_menu_background_query: Query<Entity, With<PauseScreen>>,
) {
    warn!("despawn_pause_screen called");
    for entity in pause_menu_background_query.iter() {
        commands.entity(entity).despawn();
    }
}
