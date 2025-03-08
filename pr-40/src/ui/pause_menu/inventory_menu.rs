use bevy::prelude::*;

use crate::{
    enemy::Enemy,
    items::inventory::*,
    npc::NPC,
    player::Player,
    ui::{
        constants::BACKGROUND_COLOR,
        display_case::{self, UpdateDisplayCaseEvent},
        menu_helpers::spawn_header,
    },
};

#[derive(Component)]
pub struct InventoryMenu;

pub fn spawn_inventory_menu(
    mut commands: Commands,
    player: Single<(Entity, &mut Inventory), (With<Player>, Without<Enemy>, Without<NPC>)>,
) {
    let (player, mut inventory) = player.into_inner();

    commands
        .spawn((
            InventoryMenu,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0), // space between header and item list
                ..default()
            },
            BackgroundColor::from(BACKGROUND_COLOR),
        ))
        .with_children(|parent| {
            spawn_header(parent, "INVENTORY");

            inventory.display_case = Some(display_case::spawn_display_case(parent));
        });
    // We spawned base inventory UI, now lets update it with items
    commands.trigger_targets(UpdateDisplayCaseEvent, player);
}
