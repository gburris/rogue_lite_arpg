use crate::{
    configuration::assets::SpriteAssets, labels::states::AppState, player::components::Player,
};
use bevy::prelude::*;

use super::PlayerOverlay; // Assuming Player component is defined here

// Marker components
#[derive(Component)]
struct MainhandActionBox;

#[derive(Component)]
struct ActionImage;

#[derive(Component)]
struct ActionOverlay;

#[derive(Component)]
struct ActionText;

#[derive(Component)]
struct Cooldown(Option<Timer>);

// Event for mainhand activation
#[derive(Event)]
struct MainHandActivated {
    entity: Entity,
    cooldown_duration: f32,
}

// Assumed player equipment component
#[derive(Component)]
struct EquippedItems {
    mainhand: Option<Handle<Item>>,
}

// Assumed item asset
#[derive(Asset, TypePath)]
struct Item {
    ui_sprite: Handle<Image>,
}

// Plugin definition
pub struct ActionBarPlugin;

impl Plugin for ActionBarPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MainHandActivated>()
            .add_systems(OnEnter(AppState::SpawnPlayer), spawn_action_bar)
            .add_systems(Update, (start_cooldown, update_cooldown));
    }
}

// Spawn the action bar
fn spawn_action_bar(mut commands: Commands, sprites: Res<SpriteAssets>) {
    warn!("Spawning overlay");
    let action_bar = commands
        .spawn((
            MainhandActionBox,
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(20.0),
                bottom: Val::Px(20.0),
                width: Val::Px(64.0),
                height: Val::Px(64.0),
                border: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            Cooldown(None),
        ))
        .id();

    commands.entity(action_bar).with_children(|parent| {
        // Action image (sprite)
        parent.spawn((
            ActionImage,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                border: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            Sprite {
                image: sprites.axe.clone(),
                ..default()
            },
        ));

        // Cooldown overlay
        parent.spawn((
            ActionOverlay,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
            Visibility::Hidden,
        ));

        // Cooldown text
        parent.spawn((
            ActionText,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            // Text::from_section(
            //     "",
            //     TextStyle {
            //         font_size: 24.0,
            //         color: Color::WHITE,
            //         ..default()
            //     },
            // ),
        ));
    });
}

// Update the action sprite when equipment changes
// fn update_action_sprite(
//     player_query: Query<&EquippedItems, (With<Player>, Changed<EquippedItems>)>,
//     items: Res<Assets<Item>>,
//     mut image_query: Query<&mut UiImage, With<ActionImage>>,
//     asset_server: Res<AssetServer>,
// ) {
//     if let Ok(equipped) = player_query.get_single() {
//         if let Ok(mut image) = image_query.get_single_mut() {
//             if let Some(mainhand_handle) = &equipped.mainhand {
//                 if let Some(item) = items.get(mainhand_handle) {
//                     image.texture = item.ui_sprite.clone();
//                 }
//             } else {
//                 image.texture = asset_server.load("textures/default_item.png");
//             }
//         }
//     }
// }

// Start the cooldown when the event fires
fn start_cooldown(
    mut events: EventReader<MainHandActivated>,
    mut cooldown_query: Query<&mut Cooldown, With<MainhandActionBox>>,
) {
    for event in events.read() {
        if let Ok(mut cooldown) = cooldown_query.get_single_mut() {
            cooldown.0 = Some(Timer::from_seconds(
                event.cooldown_duration,
                TimerMode::Once,
            ));
        }
    }
}

// Update the cooldown visualization
fn update_cooldown(
    time: Res<Time>,
    mut cooldown_query: Query<&mut Cooldown, With<MainhandActionBox>>,
    mut overlay_query: Query<&mut Visibility, With<ActionOverlay>>,
    mut text_query: Query<&mut Text, With<ActionText>>,
) {
    if let Ok(mut cooldown) = cooldown_query.get_single_mut() {
        if let Some(timer) = cooldown.0.as_mut() {
            timer.tick(time.delta());
            if timer.finished() {
                cooldown.0 = None;
                if let Ok(mut visibility) = overlay_query.get_single_mut() {
                    *visibility = Visibility::Hidden;
                }
                if let Ok(mut text) = text_query.get_single_mut() {
                    // text.sections[0].value = "".to_string();
                }
            } else {
                if let Ok(mut visibility) = overlay_query.get_single_mut() {
                    *visibility = Visibility::Visible;
                }
                if let Ok(mut text) = text_query.get_single_mut() {
                    let remaining = timer.duration().as_secs_f32() - timer.elapsed().as_secs_f32();
                    // text.sections[0].value = format!("{:.1}", remaining);
                }
            }
        }
    }
}
