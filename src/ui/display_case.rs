use bevy::prelude::*;

use crate::{
    configuration::assets::GameIcons,
    items::{
        equipment::{EquipmentSlot, Equippable, Equipped},
        inventory::Inventory,
        Item, ItemType,
    },
};

use super::pause_menu::button_interactions::*;

const VALUE_WIDTH: Val = Val::Px(60.0);
const EQUIP_SLOT_WIDTH: Val = Val::Px(150.0);

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
    builder
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
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(900.0),
                        height: Val::Px(35.0),
                        border: UiRect::new(Val::ZERO, Val::ZERO, Val::Px(2.0), Val::Px(2.0)),
                        margin: UiRect::top(Val::Px(5.0)),
                        padding: UiRect::all(Val::Px(5.0)),
                        column_gap: Val::Px(5.0),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor::from(Color::WHITE),
                ))
                .with_children(|parent| {
                    parent.spawn((Node {
                        width: Val::Px(30.0),
                        ..default()
                    },));

                    parent.spawn((
                        Text::new("Name"),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                    ));

                    parent.spawn((Node {
                        flex_grow: 1.0,
                        ..default()
                    },));

                    parent.spawn((
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

                    parent.spawn((
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
        })
        .id()
}

/// Internal struct to make building display slots easier
struct DisplaySlotContext<'a> {
    index: usize,
    item_name: &'a str,
    item: &'a Item,
    equipment_slot: Option<EquipmentSlot>,
    is_equipped: bool,
}

/// Spawns a given "slot" in a display case representing a single item in the inventory
fn spawn_slot(builder: &mut ChildBuilder, icons: &GameIcons, context: &DisplaySlotContext) {
    builder
        .spawn((
            DisplayCaseSlot {
                index: context.index,
            },
            Node {
                width: Val::Px(900.0),
                height: Val::Px(32.0),
                padding: UiRect::all(Val::Px(5.0)),
                column_gap: Val::Px(5.0),
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_children(|parent| {
            let item_icon = match context.item.item_type {
                ItemType::Melee => icons.melee_icon.clone(),
                ItemType::Staff => icons.staff_icon.clone(),
                ItemType::Potion => icons.potion_icon.clone(),
                ItemType::Tome => icons.spell_book_icon.clone(),
            };

            parent.spawn((
                ImageNode {
                    image: item_icon,
                    ..default()
                },
                Node {
                    width: Val::Px(30.0),
                    height: Val::Px(30.0),
                    ..default()
                },
            ));

            parent.spawn((
                Text::new(context.item_name),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
            ));

            if context.is_equipped {
                parent.spawn((
                    ImageNode {
                        image: icons.equip_icon.clone(),
                        ..default()
                    },
                    Node {
                        height: Val::Px(16.0),
                        width: Val::Px(16.0),
                        ..default()
                    },
                ));
            }

            parent.spawn((Node {
                flex_grow: 1.0,
                ..default()
            },));

            let slot_string: String = context
                .equipment_slot
                .map(|slot| slot.to_string())
                .unwrap_or("-".to_string());
            parent.spawn((
                Text::new(slot_string),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                Node {
                    width: EQUIP_SLOT_WIDTH,
                    ..default()
                },
            ));

            parent.spawn((
                Text::new(context.item.value.to_string()),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                Node {
                    width: VALUE_WIDTH,
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
    icons: Res<GameIcons>,
    slot_container_query: Query<Option<&Children>, With<DisplayCaseContainer>>,
    slots_querys: Query<(Entity, &DisplayCaseSlot)>,
    inventory_query: Query<&Inventory>,
    items_query: Query<(&Name, &Item, Option<&Equippable>, Has<Equipped>)>,
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
        ); // flatten tuple

    commands.entity(display_case).with_children(|builder| {
        for slot_context in items {
            spawn_slot(builder, &icons, &slot_context);
        }
    });
}
