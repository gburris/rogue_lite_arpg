use bevy::prelude::*;

use crate::{
    character::Character,
    items::{
        ItemOf,
        equipment::{Equipped, Mainhand, MainhandOf, Offhand, OffhandOf},
    },
};

use super::{EquipmentInUse, Equippable, StopUsingEquipment};

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

    if let Some(current) = equippable.slot.get_equipped(mainhand, offhand) {
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
) -> Result {
    let Ok((item_of, mut visibility, in_use)) = items.get_mut(unequip.item) else {
        // Expected race: equipment was despawned prior to unequip.
        info!("Equipment was despawned prior to unequip");
        return Ok(());
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
    Ok(())
}

#[cfg(test)]
mod tests {
    use avian2d::prelude::{Collider, CollisionLayers};
    use bevy::prelude::*;

    use crate::{character::Character, prelude::*};

    use super::*;

    #[derive(Resource, Default)]
    struct StopSeen(Vec<(Entity, Entity)>);

    fn setup_world(in_use: bool) -> (World, Entity, Entity) {
        let mut world = World::new();
        world.init_resource::<StopSeen>();
        world.add_observer(on_unequip);
        world.add_observer(
            |stop: On<StopUsingEquipment>, mut commands: Commands, mut seen: ResMut<StopSeen>| {
                seen.0.push((stop.item, stop.holder));
                commands
                    .entity(stop.item)
                    .remove::<(EquipmentInUse, Collider, CollisionLayers)>();
            },
        );

        let holder = world.spawn((Character, Mana::new(10.0, 0.0))).id();
        let mut item = world.spawn((
            Equipped,
            ItemOf { holder },
            MainhandOf(holder),
            Visibility::Hidden,
        ));
        if in_use {
            item.insert((
                EquipmentInUse,
                Collider::rectangle(10.0, 40.0),
                CollisionLayers::default(),
            ));
        }
        let item = item.id();
        (world, holder, item)
    }

    /// Triggers unequip and flushes both the unequip and the queued stop.
    fn trigger_unequip(world: &mut World, item: Entity) {
        world.trigger(Unequip { item });
        world.flush();
        world.flush();
    }

    /// Mid-use unequip delegates transient cleanup to the stop observer.
    #[test]
    fn unequip_mid_use_delegates_cleanup_to_stop() {
        let (mut world, holder, item) = setup_world(true);

        trigger_unequip(&mut world, item);

        assert_eq!(world.resource::<StopSeen>().0, vec![(item, holder)]);
        assert_eq!(*world.get::<Visibility>(item).unwrap(), Visibility::Hidden);
        assert!(world.get::<Equipped>(item).is_none());
        assert!(world.get::<MainhandOf>(item).is_none());
        assert!(world.get::<EquipmentInUse>(item).is_none());
        assert!(world.get::<Collider>(item).is_none());
    }

    /// Unequipping an idle item removes it quietly without queuing a stop.
    #[test]
    fn unequip_idle_item_queues_no_stop() {
        let (mut world, _, item) = setup_world(false);

        trigger_unequip(&mut world, item);

        assert!(world.resource::<StopSeen>().0.is_empty());
        assert!(world.get::<Equipped>(item).is_none());
    }
}
