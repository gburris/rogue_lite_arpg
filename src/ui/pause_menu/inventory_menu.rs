use crate::{
    enemy::Enemy,
    items::inventory::*,
    npc::NPC,
    player::Player,
    ui::{
        display_case::{self, DisplayCaseContext},
        pause_menu::button_interactions::*,
    },
};
use bevy::prelude::*;

#[derive(Component)]
pub struct InventoryMenu;

#[derive(Component)]
pub struct ItemText;

pub fn spawn_inventory_menu(
    mut commands: Commands,
    player: Single<(Entity, &mut Inventory), (With<Player>, Without<Enemy>, Without<NPC>)>,
) {
    let (player, mut inventory) = player.into_inner();

    debug!("spawn_inventory_menu called");
    commands
        .spawn((
            InventoryMenu,
            Node {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0), // space between header and item list
                ..default()
            },
            BackgroundColor::from(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            GlobalZIndex(1),
        ))
        .with_children(|parent| {
            // Header Section
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(120.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor::from(Color::srgb(0.1, 0.1, 0.1)),
                ))
                .with_children(|header| {
                    header.spawn((
                        Text::new("Player Inventory"),
                        TextFont {
                            font_size: 80.0,
                            ..default()
                        },
                    ));
                });

            let inventory_context = DisplayCaseContext {
                capacity: inventory.max_capacity,
                capacity_text: Some("Item Capacity"),
            };

            inventory.display_case =
                Some(display_case::spawn_display_case(parent, &inventory_context));
        });
    // We spawned base inventory UI, now lets update it with items
    commands.trigger_targets(UpdateInventoryUIEvent, player);
}
