use bevy::prelude::*;

use self::display_case::{UpdateDisplayCase, display_case};
use crate::prelude::*;

mod display_case;
mod item_display;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Inventory), spawn_inventory_menu);

    app.add_observer(display_case::on_display_case_updated);
}

#[derive(Component, Default, Clone)]
struct InventoryMenu;

fn spawn_inventory_menu(
    mut commands: Commands,
    player: Single<(Entity, &Purse), (With<Player>, Without<Enemy>, Without<Npc>)>,
) {
    let (player, purse) = player.into_inner();

    commands.spawn_scene(bsn! {
        @menu_screen(Menu::Inventory, "INVENTORY")
        InventoryMenu
        Node { justify_content: JustifyContent::SpaceBetween }
        Children [
            @display_case(player)
            --
            @menu_footer()
            Children [
                @text("Left click to equip/consume", BODY_FONT_SIZE)
                --
                @text("Right click to drop", BODY_FONT_SIZE)
                --
                @text({format!("Total coins: {}", purse.amount)}, BODY_FONT_SIZE)
                --
                @spacer()
                --
                @unpause_hint()
            ]
        ]
    });

    // We spawned base inventory UI, now lets update it with items
    commands.trigger(UpdateDisplayCase { entity: player });
}
