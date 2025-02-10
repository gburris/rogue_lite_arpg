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
pub struct InventoryCapacityText;

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
        });
    // We spawned base inventory UI, now lets update it with items
    commands.trigger_targets(UpdateInventoryUIEvent, player);
}
