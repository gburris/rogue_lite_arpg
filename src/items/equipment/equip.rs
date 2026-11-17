use bevy::prelude::*;

use super::slot::{MainhandOf, OffhandOf};
use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_equip).add_observer(on_unequip);
}

/// Request to equip `item` on `holder` in the item's [`Equippable`] slot, unequipping whatever is
/// there.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct Equip {
    #[event_target]
    pub item: Entity,
    pub holder: Entity,
}

fn on_equip(
    equipped: On<Equip>,
    mut commands: Commands,
    mut item_query: Query<(&Equippable, &mut Visibility, Has<ItemOf>)>,
    holder_query: Query<(Option<&Mainhand>, Option<&Offhand>), With<Character>>,
) {
    let (equippable, mut visibility, in_inventory) = item_query
        .get_mut(equipped.item)
        .expect("Added Equipped to non-equippable item");

    let (mainhand, offhand) = holder_query
        .get(equipped.holder)
        .expect("Added Equipment to holder that is not a character");

    if let Some(current) = equippable.slot.equipped(mainhand, offhand) {
        commands.trigger(Unequip { item: current });
    }

    let mut item_commands = commands.entity(equipped.item);
    item_commands.insert(ChildOf(equipped.holder));
    // Without this condition items get "re-added" to item relationship and order gets rearranged
    if !in_inventory {
        item_commands.insert(ItemOf {
            holder: equipped.holder,
        });
    }
    equippable
        .slot
        .place_in_slot(&mut item_commands, equipped.holder);

    // Make sure item is now visible, since it is hidden while in inventory
    *visibility = Visibility::Visible;
}

/// Request to unequip an item. The holder is resolved via `ItemOf`.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct Unequip {
    #[event_target]
    pub item: Entity,
}

fn on_unequip(
    unequip: On<Unequip>,
    mut commands: Commands,
    mut items: Query<(&mut Visibility, Has<ActionInProgress>), With<Equipped>>,
) {
    let Ok((mut visibility, in_progress)) = items.get_mut(unequip.item) else {
        // Expected race: equipment was despawned prior to unequip.
        info!("Equipment was despawned prior to unequip");
        return;
    };
    if in_progress {
        commands.trigger(StopAction {
            source: unequip.item,
        });
    }
    *visibility = Visibility::Hidden;
    commands
        .entity(unequip.item)
        .remove::<(Equipped, MainhandOf, OffhandOf, ChildOf)>();
}
