use super::{game_overlay::GameOverlay, pause_menu::PauseScreen};
use crate::items::{Item, StatType};
use crate::labels::states::{GameState, PausedState};
use crate::player::PlayerEquipmentSlots;
use bevy::prelude::*;

#[derive(Component)]
pub struct EquipmentMenu;

#[derive(Component)]
pub struct EquipmentMenuButton;

#[derive(Component)]
pub struct EquipmentSlotDisplay;

pub fn spawn_equipment_menu(
    mut commands: Commands,
    player_equipment: Query<&PlayerEquipmentSlots>,
) {
    warn!("spawn_equipment_menu called");

    if let Ok(equipment) = player_equipment.get_single() {
        commands
            .spawn((
                EquipmentMenu,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(20.0)), // Add padding to prevent edge touching
                    ..default()
                },
                BackgroundColor::from(Color::BLACK.with_alpha(0.9)),
                Visibility::Visible,
                GlobalZIndex(1),
            ))
            .with_children(|parent| {
                // Title
                parent.spawn((
                    Text::new("Equipment"),
                    TextFont {
                        font_size: 70.0,
                        ..default()
                    },
                    Node {
                        margin: UiRect::bottom(Val::Px(20.0)),
                        ..default()
                    },
                ));

                // Equipment slots container with scrolling
                parent
                    .spawn((
                        EquipmentSlotDisplay,
                        Node {
                            width: Val::Px(600.0),
                            height: Val::Percent(80.0), // Limit height to percentage of screen
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::all(Val::Px(20.0)),
                            overflow: Overflow::scroll_y(), // Enable vertical scrolling
                            ..default()
                        },
                        BackgroundColor::from(Color::srgba(0.1, 0.1, 0.1, 0.95)),
                    ))
                    .with_children(|slot_parent| {
                        spawn_equipment_slot(slot_parent, "Mainhand", &equipment.mainhand);
                        spawn_equipment_slot(slot_parent, "Offhand", &equipment.offhand);
                        spawn_equipment_slot(slot_parent, "Head", &equipment.head);
                        spawn_equipment_slot(slot_parent, "Chest", &equipment.chest);
                        spawn_equipment_slot(slot_parent, "Legs", &equipment.legs);
                        spawn_equipment_slot(slot_parent, "Feet", &equipment.feet);
                        spawn_equipment_slot(slot_parent, "Hands", &equipment.hands);
                        spawn_equipment_slot(slot_parent, "Shoulders", &equipment.shoulders);
                        spawn_equipment_slot(slot_parent, "Neck", &equipment.neck);
                        spawn_equipment_slot(slot_parent, "Ring 1", &equipment.ring);
                        spawn_equipment_slot(slot_parent, "Trinket", &equipment.trinket);
                    });
            });
    }
}

fn spawn_equipment_slot(builder: &mut ChildBuilder, slot_name: &str, item: &Option<Item>) {
    builder
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(60.0),
                padding: UiRect::all(Val::Px(10.0)),
                margin: UiRect::bottom(Val::Px(5.0)),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor::from(Color::srgba(0.2, 0.2, 0.2, 0.5)),
        ))
        .with_children(|parent| {
            // Slot name
            parent.spawn((
                Text::new(slot_name),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                Node::default(),
            ));

            // Item details
            if let Some(item) = item {
                let mut stats_text = format!("{}", item.name);
                for (stat_type, value) in &item.stats {
                    stats_text.push_str(&format!(" | {}: {}", stat_type, value));
                }

                parent.spawn((
                    Text::new(stats_text),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    Node {
                        margin: UiRect::left(Val::Px(10.0)),
                        ..default()
                    },
                ));
            } else {
                parent.spawn((
                    Text::new("Empty"),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    Node {
                        margin: UiRect::left(Val::Px(10.0)),
                        ..default()
                    },
                ));
            }
        });
}

pub fn despawn_equipment_menu(
    mut commands: Commands,
    equipment_menu_query: Query<Entity, With<EquipmentMenu>>,
) {
    warn!("despawn_equipment_menu called");
    for entity in equipment_menu_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
