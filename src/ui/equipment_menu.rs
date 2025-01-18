use crate::items::ItemName;
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
    item_query: Query<&ItemName>,
    player_equipment: Query<&PlayerEquipmentSlots>,
) {
    debug!("spawn_equipment_menu called");

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
                        spawn_equipment_slot(
                            item_query,
                            slot_parent,
                            "Mainhand",
                            &equipment.mainhand,
                        );
                    });
            });
    }
}

fn spawn_equipment_slot(
    item_query: Query<&ItemName>,
    builder: &mut ChildBuilder,
    slot_name: &str,
    slot_entity: &Option<Entity>,
) {
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
            if let Some(slot_entity) = slot_entity {
                // Get equipment name here.
                // Can do this by passing in item query, and finding that item in the slot
                // resolve text from slot entity
                // Display all inventory items
                if let Ok(item_name) = item_query.get(*slot_entity) {
                    parent.spawn((
                        Text::new(item_name.0.clone()),
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
    debug!("despawn_equipment_menu called");
    for entity in equipment_menu_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
