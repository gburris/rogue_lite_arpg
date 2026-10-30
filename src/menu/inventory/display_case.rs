use bevy::prelude::*;

use super::item_display::{
    DisplaySlotContext, DisplaySlotOf, EQUIP_SLOT_WIDTH, VALUE_WIDTH, display_slot,
};
use crate::prelude::*;
use crate::ui::constants::DARK_GRAY_ALPHA_COLOR;
use crate::ui::primitives::{text, width};

/// Div that wraps all display slots, but not top level component
#[derive(Component)]
#[relationship(relationship_target = DisplayedBy)]
pub struct DisplayCaseOf(Entity);

#[derive(Component)]
#[relationship_target(relationship = DisplayCaseOf)]
pub struct DisplayedBy(Entity);

/// Trigger on entity with Inventory component (i.e. the player entity) to update their associated display case
#[derive(EntityEvent)]
pub(super) struct UpdateDisplayCase {
    pub entity: Entity,
}

pub(super) fn display_case(inventory_owner: Entity) -> impl Bundle {
    (
        Node {
            height: px(800.0),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor::from(DARK_GRAY_ALPHA_COLOR),
        children![
            // inventory header
            (
                Node {
                    width: px(900.0),
                    height: px(35.0),
                    border: px(2.0).vertical(),
                    margin: px(5.0).top(),
                    padding: px(5.0).all(),
                    column_gap: px(5.0),
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

    commands.entity(displayed_by.0).with_children(|parent| {
        for item_entity in items.iter() {
            let (name, item, equippable, is_equipped) = item_query
                .get(item_entity)
                .expect("Inventory items have a name and item");
            parent.spawn(display_slot(
                &icons,
                DisplaySlotContext {
                    item_entity,
                    item_name: name.to_string(),
                    item_type: item.item_type,
                    item_value: item.value,
                    equipment_slot: equippable.map(|e| e.slot),
                    is_equipped,
                },
            ));
        }
    });
}
