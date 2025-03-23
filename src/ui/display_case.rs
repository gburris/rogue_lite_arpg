use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    picking::focus::HoverMap,
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

use super::{constants::DARK_GRAY_ALPHA_COLOR, display_case_slot::DisplaySlotContext};

pub const VALUE_WIDTH: Val = Val::Px(60.0);
pub const EQUIP_SLOT_WIDTH: Val = Val::Px(150.0);

/// Trigger on entity with Inventory component (i.e. the player entity) to update their associated display case
#[derive(Event)]
pub struct UpdateDisplayCaseEvent;

/// Div that wraps all display slots, but not top level component
#[derive(Component)]
pub struct DisplayCaseContainer;

pub fn spawn_display_case(builder: &mut ChildBuilder) -> Entity {
    let mut scroll_container = Entity::PLACEHOLDER;

    builder
        .spawn((
            Node {
                height: Val::Px(800.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor::from(DARK_GRAY_ALPHA_COLOR),
        ))
        .with_children(|ChildOf| {
            ChildOf
                .spawn((
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
                ))
                .with_children(|ChildOf| {
                    ChildOf.spawn((Node {
                        width: Val::Px(30.0),
                        ..default()
                    },));

                    ChildOf.spawn((
                        Text::new("Name"),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                    ));

                    ChildOf.spawn((Node {
                        flex_grow: 1.0,
                        ..default()
                    },));

                    ChildOf.spawn((
                        Text::new("Equip Slot"),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        Node {
                            width: EQUIP_SLOT_WIDTH,
                            ..default()
                        },
                    ));

                    ChildOf.spawn((
                        Text::new("Value"),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        Node {
                            width: VALUE_WIDTH,
                            ..default()
                        },
                    ));
                });

            scroll_container = ChildOf
                .spawn((
                    DisplayCaseContainer,
                    Node {
                        overflow: Overflow::scroll_y(),
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                ))
                .id();
        });

    scroll_container
}

pub fn on_display_case_updated(
    trigger: Trigger<UpdateDisplayCaseEvent>,
    mut commands: Commands,
    icons: Res<GameIcons>,
    slot_container_query: Query<Option<&Children>, With<DisplayCaseContainer>>,
    slots_querys: Query<(Entity, &DisplayCaseSlot)>,
    inventory_query: Query<&Inventory>,
    items_query: Query<(&Name, &Item, Option<&Equippable>, Has<Equipped>)>,
) {
    // Get entities inventory
    let inventory = inventory_query
        .get(trigger.target())
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

    commands.entity(display_case).with_children(|builder| {
        for slot_context in items {
            spawn_slot(builder, &icons, &slot_context);
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
