use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    picking::hover::HoverMap,
    prelude::*,
};

use crate::{
    configuration::assets::GameIcons,
    items::{
        equipment::{Equippable, Equipped},
        inventory::Inventory,
        Item,
    },
    ui::display_case_slot::{spawn_slot, DisplayCaseSlot},
};

use super::{
    constants::DARK_GRAY_ALPHA_COLOR,
    display_case_slot::DisplaySlotContext,
    menu_helpers::{text, TextBuilder},
};

pub const VALUE_WIDTH: f32 = 60.0;
pub const EQUIP_SLOT_WIDTH: f32 = 150.0;

/// Div that wraps all display slots, but not top level component
#[derive(Component)]
#[relationship(relationship_target = DisplayedBy)]
pub struct DisplayCaseOf(Entity);

#[derive(Component)]
#[relationship_target(relationship = DisplayCaseOf)]
pub struct DisplayedBy(Entity);

/// Trigger on entity with Inventory component (i.e. the player entity) to update their associated display case
#[derive(Event)]
pub struct UpdateDisplayCaseEvent;

pub fn display_case(inventory_owner: Entity) -> impl Bundle {
    (
        Node {
            height: Val::Px(800.0),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor::from(DARK_GRAY_ALPHA_COLOR),
        children![
            // inventory header
            (
                Node {
                    width: Val::Px(900.0),
                    height: Val::Px(35.0),
                    border: UiRect::vertical(Val::Px(2.0)),
                    margin: UiRect::top(Val::Px(5.0)),
                    padding: UiRect::all(Val::Px(5.0)),
                    column_gap: Val::Px(5.0),
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderColor::from(Color::WHITE),
                children![
                    Node {
                        width: Val::Px(30.0),
                        ..default()
                    },
                    text("Name", 18.0),
                    Node {
                        flex_grow: 1.0,
                        ..default()
                    },
                    TextBuilder::new("Equip Slot", 18.0)
                        .with_width(EQUIP_SLOT_WIDTH)
                        .build(),
                    TextBuilder::new("Value", 18.0)
                        .with_width(VALUE_WIDTH)
                        .build(),
                ]
            ),
            // Container for items in inventory
            (
                DisplayCaseOf(inventory_owner),
                Node {
                    overflow: Overflow::scroll_y(),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
            )
        ],
    )
}

pub fn on_display_case_updated(
    trigger: Trigger<UpdateDisplayCaseEvent>,
    mut commands: Commands,
    icons: Res<GameIcons>,
    slot_container_query: Query<Option<&Children>, With<DisplayCaseOf>>,
    slots_querys: Query<(Entity, &DisplayCaseSlot)>,
    inventory_query: Query<(&Inventory, &DisplayedBy)>,
    items_query: Query<(&Name, &Item, Option<&Equippable>, Has<Equipped>)>,
) {
    // Get entities inventory
    let (inventory, displayed_by) = inventory_query
        .get(trigger.target())
        .expect("No inventory to update!");

    // Get children entities of DisplayCaseOf which should all have a DisplayCaseSlot
    let display_case_children = slot_container_query
        .get(displayed_by.0)
        .expect("Display case on inventory missing DisplayCaseContainer");

    // Despawn existing slots
    slots_querys
        .iter()
        .filter(|(e, _)| display_case_children.is_some_and(|c| c.contains(e)))
        .for_each(|(e, _)| commands.entity(e).despawn());

    // Get name and entity for each item in inventory
    let items = inventory
        .items
        .iter()
        .enumerate()
        .map(|(index, &e)| (index, e, items_query.get(e).unwrap()))
        .map(
            |(index, _, (name, item, equippable, is_equipped))| DisplaySlotContext {
                index,
                item_name: name,
                item,
                equipment_slot: equippable.map(|e| e.slot),
                is_equipped,
            },
        );

    commands.entity(displayed_by.0).with_children(|builder| {
        for slot_context in items {
            spawn_slot(builder, &icons, slot_context);
        }
    });
}

const LINE_HEIGHT: f32 = 35.;

/// Updates the scroll position of scrollable nodes in response to mouse input
pub fn update_scroll_position(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    hover_map: Res<HoverMap>,
    mut scrolled_node_query: Query<&mut ScrollPosition>,
) {
    for mouse_wheel_event in mouse_wheel_events.read() {
        let dy = match mouse_wheel_event.unit {
            MouseScrollUnit::Line => mouse_wheel_event.y * LINE_HEIGHT,
            MouseScrollUnit::Pixel => mouse_wheel_event.y,
        };

        for (_pointer, pointer_map) in hover_map.iter() {
            for (entity, _hit) in pointer_map.iter() {
                if let Ok(mut scroll_position) = scrolled_node_query.get_mut(*entity) {
                    scroll_position.offset_y -= dy;
                }
            }
        }
    }
}
