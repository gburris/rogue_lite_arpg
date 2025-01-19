use crate::{items::ItemName, player::Inventory};
use bevy::prelude::*;

use super::button_interactions::InventoryUpdatedEvent;

#[derive(Component)]
pub struct InventoryMenu;

#[derive(Component)]
pub struct InventoryMenuButton;

#[derive(Component)]
pub struct InventoryDisplay;

pub fn spawn_inventory_menu(
    mut commands: Commands,
    item_query: Query<&ItemName>,
    player_inventory: Query<&Inventory>,
) {
    debug!("spawn_inventory_menu called");

    if let Ok(inventory) = player_inventory.get_single() {
        commands
            .spawn((
                InventoryMenu,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(20.0)),
                    ..default()
                },
                BackgroundColor::from(Color::BLACK.with_alpha(0.9)),
                Visibility::Visible,
                GlobalZIndex(1),
            ))
            .with_children(|parent| {
                // Title
                parent.spawn((
                    Text::new("Inventory"),
                    TextFont {
                        font_size: 70.0,
                        ..default()
                    },
                    Node {
                        margin: UiRect::bottom(Val::Px(20.0)),
                        ..default()
                    },
                ));

                // Inventory capacity display
                parent.spawn((
                    Text::new(format!(
                        "Capacity: {}/{}",
                        inventory.items.len(),
                        inventory.max_capacity
                    )),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    Node {
                        margin: UiRect::bottom(Val::Px(20.0)),
                        ..default()
                    },
                ));

                // Inventory items container with scrolling
                parent
                    .spawn((
                        InventoryDisplay,
                        Node {
                            width: Val::Px(600.0),
                            height: Val::Percent(80.0),
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::all(Val::Px(20.0)),
                            overflow: Overflow::scroll_y(),
                            ..default()
                        },
                        BackgroundColor::from(Color::srgba(0.1, 0.1, 0.1, 0.95)),
                    ))
                    .with_children(|slot_parent| {
                        // Display all inventory items
                        for item_entity in inventory.items.values() {
                            let item_name = item_query.get(*item_entity).unwrap();
                            spawn_inventory_item(slot_parent, item_name.0.clone(), *item_entity);
                        }

                        // Display empty slots
                        let empty_slots = inventory.max_capacity - inventory.items.len();
                        for _ in 0..empty_slots {
                            spawn_empty_slot(slot_parent);
                        }
                    });
            });
    }
}

// Add these new components
#[derive(Component)]
pub struct InventoryItemButton {
    pub item_entity: Option<Entity>, // None for empty slots
}

// Add these new components
#[derive(Component)]
pub struct InventoryItemName;

// Modified spawn_inventory_item function
fn spawn_inventory_item(builder: &mut ChildBuilder, item_name: String, item_entity: Entity) {
    builder
        .spawn((
            InventoryItemButton {
                item_entity: Some(item_entity),
            },
            Button,
            Interaction::default(),
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
            // Add border
            BorderColor::from(Color::NONE),
            //Border::all(Val::Px(2.0)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(item_name),
                TextColor::default(),
                InventoryItemName,
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                Node::default(),
            ));
        });
}

// Modified spawn_empty_slot function
fn spawn_empty_slot(builder: &mut ChildBuilder) {
    builder
        .spawn((
            InventoryItemButton { item_entity: None },
            Button,
            Interaction::default(),
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(60.0),
                padding: UiRect::all(Val::Px(10.0)),
                margin: UiRect::bottom(Val::Px(5.0)),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor::from(Color::srgba(0.2, 0.2, 0.2, 0.5)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Empty Slot"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                Node {
                    margin: UiRect::left(Val::Px(10.0)),
                    ..default()
                },
            ));
        });
}

pub fn despawn_inventory_menu(
    mut commands: Commands,
    inventory_menu_query: Query<Entity, With<InventoryMenu>>,
) {
    debug!("despawn_inventory_menu called");
    for entity in inventory_menu_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

//Called after dispatching a click event
pub fn handle_inventory_update(
    _: Trigger<InventoryUpdatedEvent>,
    mut commands: Commands,
    inventory_menu_query: Query<Entity, With<InventoryMenu>>,
    item_query: Query<&ItemName>,
    player_inventory: Query<&Inventory>,
) {
    // Despawn the existing inventory menu
    for entity in inventory_menu_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    // Respawn the inventory menu
    spawn_inventory_menu(commands, item_query, player_inventory);
}
