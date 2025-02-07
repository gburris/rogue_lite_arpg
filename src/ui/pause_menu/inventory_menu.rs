use crate::{
    enemy::Enemy,
    items::{equipment::EquipmentSlots, inventory::*, Item},
    npc::NPC,
    player::Player,
    ui::pause_menu::{
        button_interactions::*,
        equipment_menu::{spawn_equipment_slot, EquipmentMenu},
    },
};
use bevy::prelude::*;

#[derive(Component)]
pub struct InventoryMenu;

#[derive(Component)]
pub struct InventorySlot {
    pub item: Option<Entity>,
    pub index: usize,
}

#[derive(Component)]
pub struct InventoryCapacityText;

#[derive(Component)]
pub struct ItemText;

pub fn spawn_inventory_menu(
    mut commands: Commands,
    player: Single<(&Inventory, &EquipmentSlots), (With<Player>, Without<Enemy>, Without<NPC>)>,
) {
    let (inventory, equipment_slots) = player.into_inner();

    debug!("spawn_inventory_menu called");
    commands
        .spawn((
            InventoryMenu,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(40.0), // space between inventory and equipment lists
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            BackgroundColor::from(Color::BLACK.with_alpha(0.9)),
            GlobalZIndex(1),
        ))
        .with_children(|parent| {
            parent
                .spawn(Node {
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                })
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
                        InventoryCapacityText,
                        Text::new("Inventory Capacity"),
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
                            for index in 0..inventory.max_capacity {
                                spawn_empty_slot(slot_parent, index);
                            }
                        });
                });

            parent
                .spawn(Node {
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                })
                .with_children(|parent| {
                    // Title
                    parent.spawn((
                        Text::new("Equipment"),
                        TextFont {
                            font_size: 70.0,
                            ..default()
                        },
                        Node {
                            margin: UiRect::bottom(Val::Px(20.0)),
                            ..default()
                        },
                    ));

                    // Equipment slots container with scrolling
                    parent
                        .spawn((
                            Node {
                                width: Val::Px(600.0),
                                height: Val::Percent(80.0), // Limit height to percentage of screen
                                flex_direction: FlexDirection::Column,
                                padding: UiRect::all(Val::Px(20.0)),
                                overflow: Overflow::scroll_y(), // Enable vertical scrolling
                                ..default()
                            },
                            BackgroundColor::from(Color::srgba(0.1, 0.1, 0.1, 0.95)),
                        ))
                        .with_children(|slot_parent| {
                            spawn_equipment_slot(slot_parent, "Dumb", equipment_slots.mainhand);
                            spawn_equipment_slot(
                                slot_parent,
                                "Head",
                                "Dumb 2",
                                equipment_slots.head,
                            );
                        });
                });
        });
    // We spawned base inventory UI, now lets update it with items
    commands.trigger(UpdateInventoryUIEvent);
}

fn spawn_empty_slot(builder: &mut ChildBuilder, index: usize) {
    builder
        .spawn((
            InventorySlot { index, item: None },
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
        ))
        .observe(on_item_clicked)
        .observe(on_item_hover)
        .observe(on_item_done_hovering)
        .with_children(|parent| {
            parent.spawn((
                ItemText,
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

//Called after dispatching a click event
pub fn handle_inventory_update(
    _: Trigger<UpdateInventoryUIEvent>,
    mut capacity_text: Single<&mut Text, (With<InventoryCapacityText>, Without<ItemText>)>,
    mut item_text_query: Query<(&mut Text, &Parent), With<ItemText>>,
    mut inventory_slot_query: Query<&mut InventorySlot>,
    item_query: Query<&Name, With<Item>>,
    player_inventory: Single<&Inventory, (With<Player>, Without<Enemy>, Without<NPC>)>,
) {
    capacity_text.0 = format!(
        "Capacity: {}/{}",
        player_inventory.items.len(),
        player_inventory.max_capacity
    );

    // Iterate through all ItemText components and fetch the parent InventorySlot component
    for (mut item_text, item_slot) in item_text_query.iter_mut() {
        if let Ok(mut item_slot) = inventory_slot_query.get_mut(item_slot.get()) {
            // If there is an item in the player inventory at this index, set item text and item slot
            if let Some(&item_entity) = player_inventory.items.get(item_slot.index) {
                item_slot.item = Some(item_entity);
                item_text.0 = item_query.get(item_entity).unwrap().to_string();
            } else {
                // Otherwise there is no item, set default stuff
                item_text.0 = "Empty Slot".to_string();
                item_slot.item = None;
            }
        }
    }
}
