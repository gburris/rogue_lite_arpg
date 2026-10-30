use bevy::prelude::*;

use super::{
    EquipmentInUse, Equippable, Equipped, Mainhand, MainhandOf, Offhand, OffhandOf,
    StopUsingEquipment,
};
use crate::items::ItemOf;
use crate::prelude::Character;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_equip).add_observer(on_unequip);
}

#[derive(EntityEvent)]
pub struct Equip {
    #[event_target]
    pub item: Entity,
    pub holder: Entity,
}

fn on_equip(
    equipped: On<Equip>,
    mut commands: Commands,
    mut item_query: Query<(&Equippable, &mut Visibility, Has<ItemOf>)>,
    mut holder_query: Query<(Option<&Mainhand>, Option<&Offhand>), With<Character>>,
) {
    let (equippable, mut visibility, in_inventory) = item_query
        .get_mut(equipped.item)
        .expect("Added Equipped to non-equippable item");

    let (mainhand, offhand) = holder_query
        .get_mut(equipped.holder)
        .expect("Added Equipment to holder that is not a character");

    commands
        .entity(equipped.item)
        .insert((Equipped, ChildOf(equipped.holder)));

    // Without this condition items get "re-added" to item relationship and order gets rearranged
    if !in_inventory {
        commands.entity(equipped.item).insert(ItemOf {
            holder: equipped.holder,
        });
    }

    if let Some(current) = equippable.slot.equipped(mainhand, offhand) {
        commands.trigger(Unequip { item: current });
    }

    let mut item_commands = commands.entity(equipped.item);
    equippable
        .slot
        .place_in_slot(&mut item_commands, equipped.holder);

    // Make sure item is now visible, since it is hidden while in inventory
    *visibility = Visibility::Visible;
}

/// Request to unequip an item. The holder is resolved via `ItemOf`.
#[derive(EntityEvent)]
pub struct Unequip {
    #[event_target]
    pub item: Entity,
}

fn on_unequip(
    unequip: On<Unequip>,
    mut commands: Commands,
    mut items: Query<(&ItemOf, &mut Visibility, Has<EquipmentInUse>), With<Equipped>>,
) {
    let Ok((item_of, mut visibility, in_use)) = items.get_mut(unequip.item) else {
        // Expected race: equipment was despawned prior to unequip.
        info!("Equipment was despawned prior to unequip");
        return;
    };
    if in_use {
        commands.trigger(StopUsingEquipment {
            item: unequip.item,
            holder: item_of.holder,
        });
    }
    *visibility = Visibility::Hidden;
    commands
        .entity(unequip.item)
        .remove::<(Equipped, MainhandOf, OffhandOf, ChildOf)>();
}
