use bevy::prelude::*;

use crate::{
    enemy::Enemy,
    items::inventory::*,
    npc::NPC,
    player::Player,
    ui::{
        constants::{BACKGROUND_COLOR, DARK_GRAY_COLOR, FOOTER_HEIGHT},
        display_case::{self, UpdateDisplayCaseEvent},
        primitives::{menu_header, text},
    },
};

#[derive(Component)]
pub struct InventoryMenu;

pub fn spawn_inventory_menu(
    mut commands: Commands,
    player: Single<(Entity, &mut Inventory), (With<Player>, Without<Enemy>, Without<NPC>)>,
) {
    commands.spawn((
        InventoryMenu,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(20.0), // space between header and item list
            ..default()
        },
        BackgroundColor::from(BACKGROUND_COLOR),
        children![
            menu_header("INVENTORY"),
            display_case::display_case(player.0),
            (
                Node {
                    width: Val::Percent(100.0),
                    height: FOOTER_HEIGHT,
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    padding: UiRect::horizontal(Val::Px(40.0)),
                    column_gap: Val::Px(20.0),
                    ..default()
                },
                BackgroundColor::from(DARK_GRAY_COLOR),
                children![
                    text("Left click to equip/consume", 24.0),
                    text("Right click to drop", 24.0),
                    text(format!("Total coins: {:.1}", player.1.coins), 24.0),
                    // Spacer between left and right side of footer
                    Node {
                        flex_grow: 1.0,
                        ..default()
                    },
                    text("Press ESC to unpause", 24.0)
                ],
            )
        ],
    ));

    // We spawned base inventory UI, now lets update it with items
    commands.trigger_targets(UpdateDisplayCaseEvent, player.0);
}
