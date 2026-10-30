use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::picking::hover::HoverMap;
use bevy::prelude::*;

use self::display_case::{UpdateDisplayCase, display_case};
use crate::prelude::*;

mod display_case;
mod item_display;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Inventory), spawn_inventory_menu);

    app.add_systems(
        Update,
        update_scroll_position
            .run_if(in_state(Menu::Inventory))
            .in_set(MainSystems::Menu),
    );

    app.add_observer(display_case::on_display_case_updated);
}

#[derive(Component)]
struct InventoryMenu;

fn spawn_inventory_menu(
    mut commands: Commands,
    player: Single<(Entity, &Purse), (With<Player>, Without<Enemy>, Without<Npc>)>,
) {
    commands.spawn((
        InventoryMenu,
        DespawnOnExit(Menu::Inventory),
        GlobalZIndex(2),
        Node {
            width: percent(100.0),
            height: percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            flex_direction: FlexDirection::Column,
            row_gap: px(20.0), // space between header and item list
            ..default()
        },
        children![
            menu_header("INVENTORY"),
            display_case(player.0),
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

const LINE_HEIGHT: f32 = 35.;

/// Updates the scroll position of scrollable nodes in response to mouse input
fn update_scroll_position(
    mut mouse_wheel_messages: MessageReader<MouseWheel>,
    hover_map: Res<HoverMap>,
    mut scrolled_node_query: Query<&mut ScrollPosition>,
) {
    for mouse_wheel_message in mouse_wheel_messages.read() {
        let dy = match mouse_wheel_message.unit {
            MouseScrollUnit::Line => mouse_wheel_message.y * LINE_HEIGHT,
            MouseScrollUnit::Pixel => mouse_wheel_message.y,
        };

        for (_pointer, pointer_map) in hover_map.iter() {
            for (entity, _hit) in pointer_map {
                if let Ok(mut scroll_position) = scrolled_node_query.get_mut(*entity) {
                    scroll_position.y -= dy;
                }
            }
        }
    }
}
