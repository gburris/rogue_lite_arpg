use bevy::prelude::*;

use crate::{
    economy::Purse,
    labels::states::PausedState,
    prelude::{Enemy, NPC, Player},
    ui::{
        constants::{BACKGROUND_COLOR, DARK_GRAY_COLOR, FOOTER_HEIGHT},
        display_case::{self, UpdateDisplayCase},
        primitives::{menu_header, text},
    },
};

#[derive(Component)]
pub struct InventoryMenu;

pub fn spawn_inventory_menu(
    mut commands: Commands,
    player: Single<(Entity, &Purse), (With<Player>, Without<Enemy>, Without<NPC>)>,
) {
    commands.spawn((
        InventoryMenu,
        DespawnOnExit(PausedState::Inventory),
        Node {
            width: percent(100.0),
            height: percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            flex_direction: FlexDirection::Column,
            row_gap: px(20.0), // space between header and item list
            ..default()
        },
        BackgroundColor::from(BACKGROUND_COLOR),
        children![
            menu_header("INVENTORY"),
            display_case::display_case(player.0),
            (
                Node {
                    width: percent(100.0),
                    height: FOOTER_HEIGHT,
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    padding: px(40.0).horizontal(),
                    column_gap: px(20.0),
                    ..default()
                },
                BackgroundColor::from(DARK_GRAY_COLOR),
                children![
                    text("Left click to equip/consume", 24.0),
                    text("Right click to drop", 24.0),
                    text(format!("Total coins: {:.1}", player.1.amount), 24.0),
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
    commands.trigger(UpdateDisplayCase { entity: player.0 });
}
