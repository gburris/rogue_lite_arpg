use core::fmt;

use bevy::prelude::*;

use super::equipment_pose::DirectionalPoses;
use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_equip).add_observer(on_unequip);
}

/// An item that can be equipped in a slot and used from it.
#[derive(Component, Clone, Debug)]
#[require(ActionSource)]
pub struct Equippable {
    pub slot: EquipmentSlot,
    pub poses: &'static DirectionalPoses,
}

impl Equippable {
    /// Equips into `slot`, resting in `poses` while not in use.
    pub fn new(slot: EquipmentSlot, poses: &'static DirectionalPoses) -> Self {
        Self { slot, poses }
    }
}

/// Present on an item while it sits in one of its holder's equipment slots. Required by the slot
/// relationships, and removed with them on unequip.
#[derive(Component, Clone, Debug, Default)]
pub struct Equipped;

/// Goes on the equipment marking where it should be equipped
///
/// Note: We pass this by value a lot, don't add data to it without consideration for passing this
/// by reference.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum EquipmentSlot {
    #[default]
    Mainhand,
    Offhand,
}

impl EquipmentSlot {
    pub const ALL: [Self; 2] = [Self::Mainhand, Self::Offhand];

    /// Item currently equipped in this slot, if any.
    pub fn equipped(
        self,
        mainhand: Option<&Mainhand>,
        offhand: Option<&Offhand>,
    ) -> Option<Entity> {
        match self {
            Self::Mainhand => mainhand.map(Mainhand::get),
            Self::Offhand => offhand.map(Offhand::get),
        }
    }

    /// Insert the relationship that places `item` in this slot on `holder`.
    pub fn place_in_slot(self, item: &mut EntityCommands, holder: Entity) {
        match self {
            Self::Mainhand => item.insert(MainhandOf(holder)),
            Self::Offhand => item.insert(OffhandOf(holder)),
        };
    }
}

impl fmt::Display for EquipmentSlot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let variant_name = match *self {
            Self::Mainhand => "Main Hand",
            Self::Offhand => "Off Hand",
        };
        write!(f, "{variant_name}")
    }
}

#[derive(Component, Clone)]
#[relationship(relationship_target = Mainhand)]
#[require(Equipped)]
pub struct MainhandOf(pub Entity);

#[derive(Component, Clone, Debug)]
#[relationship_target(relationship = MainhandOf)]
pub struct Mainhand(Entity);

impl Mainhand {
    pub fn get(&self) -> Entity {
        self.0
    }
}

#[derive(Component, Clone)]
#[relationship(relationship_target = Offhand)]
#[require(Equipped)]
pub struct OffhandOf(pub Entity);

#[derive(Component, Clone, Debug)]
#[relationship_target(relationship = OffhandOf)]
pub struct Offhand(Entity);

impl Offhand {
    pub fn get(&self) -> Entity {
        self.0
    }
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
    mut items: Query<(&mut Visibility, Has<ActionSourceUse>), With<Equipped>>,
) {
    let Ok((mut visibility, in_use)) = items.get_mut(unequip.item) else {
        // Expected race: equipment was despawned prior to unequip.
        info!("Equipment was despawned prior to unequip");
        return;
    };
    if in_use {
        commands.trigger(StopAction {
            source: unequip.item,
        });
    }
    *visibility = Visibility::Hidden;
    commands
        .entity(unequip.item)
        .remove::<(Equipped, MainhandOf, OffhandOf, ChildOf)>();
}
