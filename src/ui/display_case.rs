use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    picking::hover::HoverMap,
    prelude::*,
};

use crate::{
    configuration::assets::GameIcons,
    items::{
        Item, Items,
        equipment::{Equippable, Equipped},
    },
    ui::display_case_slot::{DisplaySlotOf, display_slot},
};

use super::{
    constants::DARK_GRAY_ALPHA_COLOR,
    display_case_slot::DisplaySlotContext,
    primitives::{text, width},
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
#[derive(EntityEvent)]
pub struct UpdateDisplayCaseEvent {
    pub entity: Entity,
}

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
                    width(30.0),
                    text("Name", 18.0),
                    Node {
                        flex_grow: 1.0,
                        ..default()
                    },
                    (text("Equip Slot", 18.0), width(EQUIP_SLOT_WIDTH)),
                    (text("Value", 18.0), width(VALUE_WIDTH)),
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
    trigger: On<UpdateDisplayCaseEvent>,
    mut commands: Commands,
    icons: Res<GameIcons>,
    slot_container_query: Query<Option<&Children>, With<DisplayCaseOf>>,
    slots_querys: Query<(Entity, &DisplaySlotOf)>,
    items_query: Query<(Option<&Items>, &DisplayedBy)>,
    item_query: Query<(&Name, &Item, Option<&Equippable>, Has<Equipped>)>,
) {
    // Get entities inventory
    let (items, displayed_by) = items_query
        .get(trigger.target())
        .expect("Item holder is not displayed by anything");

    // Get children entities of DisplayCaseOf which should all have a DisplayCaseSlot
    let display_case_children = slot_container_query
        .get(displayed_by.0)
        .expect("Display case on inventory missing DisplayCaseContainer");

    // Despawn existing slots
    slots_querys
        .iter()
        .filter(|(e, _)| display_case_children.is_some_and(|c| c.contains(e)))
        .for_each(|(e, _)| commands.entity(e).despawn());

    let Some(items) = items else {
        return;
    };

    // Get name and entity for each item in inventory
    let items = items.iter().map(|e| (e, item_query.get(e).unwrap())).map(
        |(item_entity, (name, item, equippable, is_equipped))| DisplaySlotContext {
            item_entity,
            item_name: name.to_string(),
            item_type: item.item_type,
            item_value: item.value,
            equipment_slot: equippable.map(|e| e.slot),
            is_equipped,
        },
    );

    commands.entity(displayed_by.0).with_children(|parent| {
        for slot_context in items {
            parent.spawn(display_slot(&icons, slot_context));
        }
    });
}

const LINE_HEIGHT: f32 = 35.;

/// Updates the scroll position of scrollable nodes in response to mouse input
pub fn update_scroll_position(
    mut mouse_wheel_messages: MessageReader<MouseWheel>,
    hover_map: Res<HoverMap>,
    mut scrolled_node_query: Query<&mut ScrollPosition>,
) {
    for mouse_wheel_message in mouse_wheel_messages.read() {
        let dy = match mouse_wheel_message.unit {
            MouseScrollUnit::Line => mouse_wheel_message.y * LINE_HEIGHT,
            MouseScrollUnit::Pixel => mouse_wheel_message.y,
        };

        for (_pointer, pointer_map) in hover_map.iter() {
            for (entity, _hit) in pointer_map.iter() {
                if let Ok(mut scroll_position) = scrolled_node_query.get_mut(*entity) {
                    scroll_position.y -= dy;
                }
            }
        }
    }
}
