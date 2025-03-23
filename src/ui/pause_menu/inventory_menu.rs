use bevy::prelude::*;

use crate::{
    enemy::Enemy,
    items::inventory::*,
    npc::NPC,
    player::Player,
    ui::{
        constants::{BACKGROUND_COLOR, DARK_GRAY_COLOR, FOOTER_HEIGHT},
        display_case::{self, UpdateDisplayCaseEvent},
        menu_helpers::menu_header,
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
                justify_content: JustifyContent::SpaceBetween,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0), // space between header and item list
                ..default()
            },
            BackgroundColor::from(BACKGROUND_COLOR),
        ))
        .with_children(|ChildOf| {
            menu_header(ChildOf, "INVENTORY");

            inventory.display_case = Some(display_case::spawn_display_case(ChildOf));

            ChildOf
                .spawn((
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
                ))
                .with_children(|footer| {
                    footer.spawn((
                        Text::new("Left click to equip/consume"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                    ));
                    footer.spawn((
                        Text::new("Right click to drop"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                    ));

                    footer.spawn((
                        Text::new(format!("Total coins: {:.1}", inventory.coins)),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                    ));

                    // Spacer between left and right side of footer
                    footer.spawn(Node {
                        flex_grow: 1.0,
                        ..default()
                    });

                    footer.spawn((
                        Text::new("Press ESC to unpause"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                    ));
                });
        });
    // We spawned base inventory UI, now lets update it with items
    commands.trigger_targets(UpdateDisplayCaseEvent, player);
}
