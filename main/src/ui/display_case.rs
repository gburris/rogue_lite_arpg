use bevy::prelude::*;

use crate::items::{inventory::Inventory, Item};

use super::pause_menu::{button_interactions::*, inventory_menu::ItemText};

/// Data used to construct the display case. This is not a component just a simple object
pub struct DisplayCaseContext<'a> {
    pub title: &'a str,
    /// If there is a max capacity AND we want to display capacity text
    pub capacity: usize,
    pub capacity_text: Option<&'a str>,
}

#[derive(Component)]
pub struct CapacityText;

/// Div that wraps all display slots, but not top level component
#[derive(Component)]
pub struct DisplayCaseContainer;

/// We sort display case slots by index, magic!!!
#[derive(Component, Copy, Clone, Deref, PartialEq, Eq, PartialOrd, Ord)]
pub struct DisplayCaseSlot {
    /// Index in the display case correspoding to index in actual entities inventory
    pub index: usize,
}

#[derive(Component)]
pub struct FilledDisplaySlot {
    pub item: Entity,
}

pub fn spawn_display_case(builder: &mut ChildBuilder, context: &DisplayCaseContext) -> Entity {
    let mut display_case_entity = Entity::PLACEHOLDER;

    builder
        .spawn(Node {
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Start,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(20.0),
            ..default()
        })
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new(context.title),
                TextFont {
                    font_size: 60.0,
                    ..default()
                },
            ));

            if let Some(capacity_text) = context.capacity_text {
                // Display Case Capacity
                parent.spawn((
                    CapacityText,
                    Text::new(capacity_text),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                ));
            }

            display_case_entity = parent
                .spawn((
                    DisplayCaseContainer,
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(5.0), // Space items in display case
                        padding: UiRect::all(Val::Px(5.0)),
                        overflow: Overflow::scroll_y(),
                        ..default()
                    },
                    BackgroundColor::from(Color::srgba(0.1, 0.1, 0.1, 0.95)),
                ))
                .with_children(|slot_parent| {
                    for index in 0..context.capacity {
                        spawn_slot_background(slot_parent, index);
                    }
                })
                .id();
        });

    display_case_entity
}

/// For each possible slot in a display case we spawn a background that holds the slots index
fn spawn_slot_background(builder: &mut ChildBuilder, index: usize) {
    builder.spawn((
        DisplayCaseSlot { index },
        Node {
            padding: UiRect::all(Val::Px(5.0)),
            min_height: Val::Px(40.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor::from(Color::BLACK),
    ));
}

/// Given an entities inventory, spawn filled slots on top of slot backgrounds
fn spawn_filled_slot(builder: &mut ChildBuilder, item: Entity, item_name: &str) {
    builder
        .spawn((
            FilledDisplaySlot { item },
            Node {
                width: Val::Px(500.0), // Make width % of parent slot
                justify_content: JustifyContent::Start,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor::from(Color::srgba(0.2, 0.2, 0.2, 0.5)),
        ))
        .with_children(|parent| {
            parent.spawn((
                ItemText,
                Text::new(item_name),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                Node {
                    margin: UiRect::all(Val::Px(5.0)),
                    ..default()
                },
            ));
        })
        .observe(on_item_clicked)
        .observe(on_item_hover)
        .observe(on_item_done_hovering);
}

pub fn on_display_case_updated(
    trigger: Trigger<UpdateInventoryUIEvent>,
    mut commands: Commands,
    slot_container_query: Query<&Children, With<DisplayCaseContainer>>,
    empty_slots_query: Query<(Entity, &DisplayCaseSlot)>,
    filled_slots_query: Query<Entity, With<FilledDisplaySlot>>,
    inventory_query: Query<&Inventory>,
    items_query: Query<(&Name, &Item)>,
) {
    // Get entities inventory
    let inventory = inventory_query
        .get(trigger.entity())
        .expect("No inventory to update!");

    let Some(display_case) = inventory.display_case else {
        warn!("No display case attached to updated inventory");
        return;
    };

    let items = inventory
        .items
        .iter()
        .map(|&e| (e, items_query.get(e).unwrap()))
        .map(|(i1, (i2, i3))| (i1, i2, i3)); // flatten tuple

    // Get children entities of DisplayCaseContainer which should all have a DisplayCaseSlot
    let display_case_children = slot_container_query
        .get(display_case)
        .expect("Display case on inventory missing DisplayCaseContainer");

    // Despawn existing filled slots
    // TODO: Make sure this are only this cases items
    for filled_slot in filled_slots_query.iter() {
        commands.entity(filled_slot).despawn_recursive();
    }

    let empty_slots = empty_slots_query
        .iter()
        .sort::<&DisplayCaseSlot>()
        .filter_map(|(e, _)| {
            if display_case_children.contains(&e) {
                Some(e)
            } else {
                None
            }
        });

    for ((item_entity, item_name, _), slot_entity) in items.zip(empty_slots) {
        commands
            .entity(slot_entity)
            .with_children(|builder| spawn_filled_slot(builder, item_entity, item_name));
    }
}
