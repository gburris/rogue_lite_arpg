use bevy::prelude::*;

use crate::items::{inventory::Inventory, Item};

use super::pause_menu::{button_interactions::*, inventory_menu::ItemText};

/// Data used to construct the display case. This is not a component just a simple object
pub struct DisplayCaseContext<'a> {
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
            // if let Some(capacity_text) = context.capacity_text {
            //     // Display Case Capacity
            //     parent.spawn((
            //         CapacityText,
            //         Text::new(capacity_text),
            //         TextFont {
            //             font_size: 24.0,
            //             ..default()
            //         },
            //     ));
            // }

            display_case_entity = parent
                .spawn((
                    DisplayCaseContainer,
                    Node {
                        height: Val::Px(1200.0), // flow off the screen, we will scroll this
                        flex_direction: FlexDirection::Column,
                        overflow: Overflow::scroll_y(),
                        ..default()
                    },
                    BackgroundColor::from(Color::srgba(0.1, 0.1, 0.1, 0.95)),
                ))
                .id();
        });

    display_case_entity
}

/// Given an entities inventory, spawn filled slots on top of slot backgrounds
fn spawn_slot(builder: &mut ChildBuilder, index: usize, item_name: &str) {
    builder
        .spawn((
            DisplayCaseSlot { index },
            Node {
                width: Val::Px(900.0),
                justify_content: JustifyContent::Start,
                align_items: AlignItems::Center,
                ..default()
            },
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
    slot_container_query: Query<Option<&Children>, With<DisplayCaseContainer>>,
    slots_querys: Query<(Entity, &DisplayCaseSlot)>,
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

    // Get children entities of DisplayCaseContainer which should all have a DisplayCaseSlot
    let display_case_children = slot_container_query
        .get(display_case)
        .expect("Display case on inventory missing DisplayCaseContainer");

    // Despawn existing slots
    slots_querys
        .iter()
        .filter_map(|(e, _)| {
            if display_case_children
                .map(|c| c.contains(&e))
                .or_else(|| Some(false))
                .unwrap()
            {
                Some(e)
            } else {
                None
            }
        })
        .for_each(|e| commands.entity(e).despawn_recursive());

    // Get name and entity for each item in inventory
    let items = inventory
        .items
        .iter()
        .map(|&e| (e, items_query.get(e).unwrap()))
        .map(|(i1, (i2, i3))| i2); // flatten tuple

    commands.entity(display_case).with_children(|builder| {
        for (index, item_name) in items.enumerate() {
            spawn_slot(builder, index, item_name);
        }
    });
}
