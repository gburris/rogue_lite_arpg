use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    picking::hover::HoverMap,
    prelude::*,
};

use crate::{
    character::Purse,
    menu::inventory::item_display::{
        DisplaySlotContext, DisplaySlotOf, EQUIP_SLOT_WIDTH, VALUE_WIDTH, display_slot,
    },
    prelude::*,
    ui::{
        constants::{color, val},
        element::{Element, node},
        primitives::{menu_header, text, width},
    },
};

mod item_display;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Inventory), spawn_inventory_menu);

    app.add_systems(
        Update,
        update_scroll_position
            .run_if(in_state(Menu::Inventory))
            .in_set(MainSystems::Menu),
    );

    app.add_observer(on_display_case_updated);
}

#[derive(Component)]
struct InventoryMenu;

fn spawn_inventory_menu(
    mut commands: Commands,
    player: Single<(Entity, &Purse), (With<Player>, Without<Enemy>, Without<NPC>)>,
) {
    commands.spawn((
        InventoryMenu,
        DespawnOnExit(Menu::Inventory),
        Element::builder(
            node()
                .width(percent(100.0))
                .height(percent(100.0))
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::SpaceBetween)
                .flex_direction(FlexDirection::Column)
                .row_gap(px(20.0))
                .build(),
        )
        .global_z_index(2)
        .build(),
        children![
            menu_header("INVENTORY"),
            display_case(player.0),
            (
                node()
                    .width(percent(100.0))
                    .height(val::FOOTER_HEIGHT)
                    .flex_direction(FlexDirection::Row)
                    .justify_content(JustifyContent::SpaceBetween)
                    .align_items(AlignItems::Center)
                    .padding(px(40.0).horizontal())
                    .column_gap(px(20.0))
                    .build(),
                BackgroundColor::from(color::DARK_GRAY),
                children![
                    text("Left click to equip/consume", 24.0),
                    text("Right click to drop", 24.0),
                    text(format!("Total coins: {:.1}", player.1.amount), 24.0),
                    // Spacer between left and right side of footer
                    node()
                        .flex_grow(1.0)
                        .build(),
                    text("Press ESC to unpause", 24.0)
                ],
            )
        ],
    ));

    // We spawned base inventory UI, now lets update it with items
    commands.trigger(UpdateDisplayCase { entity: player.0 });
}

/// Div that wraps all display slots, but not top level component
#[derive(Component)]
#[relationship(relationship_target = DisplayedBy)]
pub struct DisplayCaseOf(Entity);

#[derive(Component)]
#[relationship_target(relationship = DisplayCaseOf)]
pub struct DisplayedBy(Entity);

/// Trigger on entity with Inventory component (i.e. the player entity) to update their associated display case
#[derive(EntityEvent)]
struct UpdateDisplayCase {
    pub entity: Entity,
}

fn display_case(inventory_owner: Entity) -> impl Bundle {
    (
        node()
            .height(px(800.0))
            .flex_direction(FlexDirection::Column)
            .build(),
        BackgroundColor::from(color::DARK_GRAY_ALPHA),
        children![
            // inventory header
            (
                node()
                    .width(px(900.0))
                    .height(px(35.0))
                    .border(px(2.0).vertical())
                    .margin(px(5.0).top())
                    .padding(px(5.0).all())
                    .column_gap(px(5.0))
                    .align_items(AlignItems::Center)
                    .build(),
                BorderColor::from(color::WHITE),
                children![
                    width(30.0),
                    text("Name", 18.0),
                    node()
                        .flex_grow(1.0)
                        .build(),
                    (text("Equip Slot", 18.0), width(EQUIP_SLOT_WIDTH)),
                    (text("Value", 18.0), width(VALUE_WIDTH)),
                ]
            ),
            // Container for items in inventory
            (
                DisplayCaseOf(inventory_owner),
                node()
                    .overflow(Overflow::scroll_y())
                    .flex_direction(FlexDirection::Column)
                    .build(),
            )
        ],
    )
}

fn on_display_case_updated(
    update_display_case: On<UpdateDisplayCase>,
    mut commands: Commands,
    icons: Res<GameIcons>,
    slot_container_query: Query<Option<&Children>, With<DisplayCaseOf>>,
    slots_querys: Query<(Entity, &DisplaySlotOf)>,
    items_query: Query<(Option<&Items>, &DisplayedBy)>,
    item_query: Query<(&Name, &Item, Option<&Equippable>, Has<Equipped>)>,
) -> Result {
    // Get entities inventory
    let (items, displayed_by) = items_query
        .get(update_display_case.entity)
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
        return Ok(());
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
    Ok(())
}

const LINE_HEIGHT: f32 = 35.;

/// Updates the scroll position of scrollable nodes in response to mouse input
fn update_scroll_position(
    mut mouse_wheel_messages: MessageReader<MouseWheel>,
    hover_map: Res<HoverMap>,
    mut scrolled_node_query: Query<&mut ScrollPosition>,
) {
    for mouse_wheel_message in mouse_wheel_messages.read() {
        let dy = match mouse_wheel_message.unit {
            MouseScrollUnit::Line => mouse_wheel_message.y * LINE_HEIGHT,
            MouseScrollUnit::Pixel => mouse_wheel_message.y,
        };

        for (_pointer, pointer_map) in hover_map {
            for (entity, _hit) in pointer_map {
                if let Ok(mut scroll_position) = scrolled_node_query.get_mut(*entity) {
                    scroll_position.y -= dy;
                }
            }
        }
    }
}
