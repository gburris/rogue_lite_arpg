use crate::{
    enemy::Enemy,
    items::{equipment::EquipmentSlots, inventory::*, Item},
    npc::NPC,
    player::Player,
    ui::{
        display_case::{self, DisplayCaseContext},
        pause_menu::{
            button_interactions::*,
            equipment_menu::{spawn_equipment_slot, EquipmentMenu},
        },
    },
};
use bevy::prelude::*;

#[derive(Component)]
pub struct InventoryMenu;

#[derive(Component)]
pub struct InventoryCapacityText;

#[derive(Component)]
pub struct ItemText;

pub fn spawn_inventory_menu(
    mut commands: Commands,
    player: Single<
        (Entity, &mut Inventory, &EquipmentSlots),
        (With<Player>, Without<Enemy>, Without<NPC>),
    >,
) {
    let (player, mut inventory, equipment_slots) = player.into_inner();

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
            let inventory_context = DisplayCaseContext {
                title: "Player Inventory",
                capacity: inventory.max_capacity,
                capacity_text: Some("Item Capacity"),
            };

            inventory.display_case =
                Some(display_case::spawn_display_case(parent, &inventory_context));

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
                            // spawn_equipment_slot(slot_parent, "Dumb", equipment_slots.mainhand);
                            // spawn_equipment_slot(
                            //     slot_parent,
                            //     "Head",
                            //     "Dumb 2",
                            //     equipment_slots.head,
                            // );
                        });
                });
        });
    // We spawned base inventory UI, now lets update it with items
    commands.trigger_targets(UpdateInventoryUIEvent, player);
}
