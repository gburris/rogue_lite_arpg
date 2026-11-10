use bevy::prelude::*;
use bevy::ui_widgets::ScrollArea;

use super::item_display::{
    DisplaySlotContext, DisplaySlotOf, EQUIP_SLOT_WIDTH, VALUE_WIDTH, display_slot,
};
use crate::prelude::*;

/// Div that wraps all display slots, but not top level component.
#[derive(Component, FromTemplate, Clone)]
#[relationship(relationship_target = DisplayedBy)]
pub struct DisplayCaseOf(Entity);

#[derive(Component)]
#[relationship_target(relationship = DisplayCaseOf)]
pub struct DisplayedBy(Entity);

/// Trigger on entity with Inventory component (i.e. the player entity) to update their associated
/// display case.
#[derive(EntityEvent)]
pub(super) struct UpdateDisplayCase {
    pub entity: Entity,
}

pub(super) fn display_case(inventory_owner: Entity) -> impl Scene {
    bsn! {
        Node {
            height: px(800),
            flex_direction: FlexDirection::Column,
        }
        BackgroundColor(DARK_GRAY_ALPHA_COLOR)
        Children [
            // Column headings
            Node {
                width: px(900),
                height: px(35),
                border: {px(2).vertical()},
                margin: {px(5).top()},
                padding: px(5),
                column_gap: px(5),
                align_items: AlignItems::Center,
            }
            BorderColor::all(Color::WHITE)
            Children [
                @width(30.0)
                --
                @text("Name", SMALL_FONT_SIZE)
                --
                @spacer()
                --
                @text("Equip Slot", SMALL_FONT_SIZE)
                @width(EQUIP_SLOT_WIDTH)
                --
                @text("Value", SMALL_FONT_SIZE)
                @width(VALUE_WIDTH)
            ]
            --
            // One slot per item, scrolled with the mouse wheel.
            DisplayCaseOf(inventory_owner)
            ScrollArea
            Node {
                flex_grow: 1.0,
                overflow: Overflow::scroll_y(),
                flex_direction: FlexDirection::Column,
            }
        ]
    }
}

pub(super) fn on_display_case_updated(
    update_display_case: On<UpdateDisplayCase>,
    mut commands: Commands,
    icons: Res<GameIcons>,
    slot_container_query: Query<Option<&Children>, With<DisplayCaseOf>>,
    slots_querys: Query<(Entity, &DisplaySlotOf)>,
    items_query: Query<(Option<&Items>, &DisplayedBy)>,
    item_query: Query<(&Name, &Item, Option<&Equippable>, Has<Equipped>)>,
) {
    // Get entities inventory
    let (items, displayed_by) = items_query
        .get(update_display_case.entity)
        .expect("Item holder is not displayed by anything");

    // Get children entities of DisplayCaseOf which should all have a DisplayCaseSlot
    let display_case_children = slot_container_query
        .get(displayed_by.0)
        .expect("Display case on inventory missing DisplayCaseContainer");

    // Despawn existing slots
    for (slot_entity, _) in &slots_querys {
        if display_case_children.is_some_and(|children| children.contains(&slot_entity)) {
            commands.entity(slot_entity).despawn();
        }
    }

    let Some(items) = items else {
        return;
    };

    let case = displayed_by.0;
    for item_entity in items.iter() {
        let (name, item, equippable, equipped) = item_query
            .get(item_entity)
            .expect("Inventory items have a name and item");
        let slot = display_slot(
            &icons,
            DisplaySlotContext {
                item: item_entity,
                name: name.to_string(),
                kind: item.kind,
                value: item.value,
                equipment_slot: equippable.map(|e| e.slot),
                equipped,
            },
        );
        commands.spawn_scene(bsn! { @{slot} ChildOf(case) });
    }
}
