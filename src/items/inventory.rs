use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::{Item, ItemKind};
use crate::prelude::*;

/// Relationship linking an item to its holder (inventory or equipment owner).
#[derive(Component, Clone)]
#[relationship(relationship_target = Items)]
pub struct ItemOf {
    /// Entity holding this item.
    pub holder: Entity,
}

#[derive(Component, Clone, Debug)]
#[relationship_target(relationship = ItemOf, linked_spawn)]
pub struct Items(Vec<Entity>);

#[derive(Component)]
pub struct ItemCapacity(pub usize);

/// Reads the items characters carry.
#[derive(SystemParam)]
pub struct CarriedItems<'w, 's> {
    holders: Query<'w, 's, &'static Items>,
    items: Query<'w, 's, &'static Item>,
}

impl CarriedItems<'_, '_> {
    /// The first item of `kind` that `holder` carries.
    pub fn first_of_kind(&self, holder: Entity, kind: ItemKind) -> Option<Entity> {
        let carried = self.holders.get(holder).ok()?;
        carried
            .iter()
            .find(|&item| self.items.get(item).is_ok_and(|item| item.kind == kind))
    }
}

pub(super) fn on_item_added_to_inventory(
    item_added: On<Add<ItemOf>>,
    mut commands: Commands,
    item_query: Query<&ItemOf>,
    holder_query: Query<(Option<&Items>, Option<&ItemCapacity>)>,
) -> Result {
    let holder_entity = item_query.get(item_added.entity)?;

    let (items, capacity) = holder_query.get(holder_entity.holder)?;

    let capacity = capacity.map_or(usize::MAX, |c| c.0);

    if items.map_or(0, bevy::prelude::RelationshipTarget::len) >= capacity {
        commands.trigger(DropItem {
            item: item_added.entity,
        });
    }

    Ok(())
}

/// What an item does when consumed.
#[derive(Component, Clone)]
pub enum Consumable {
    /// Heals the consumer by this amount.
    Heal(f32),
}

#[derive(EntityEvent)]
pub struct Consume {
    pub entity: Entity,
    pub item: Entity,
}

/// A healing potion. Potions live in the inventory and are consumed directly
/// via `Consume` (the same path the player uses); they are never equipped.
pub fn health_potion(sprites: &SpriteAssets) -> impl Scene {
    let image = sprites.health_potion.clone();
    bsn! {
        Name::new("Health Potion")
        ~Item::new(40, ItemKind::Potion)
        ~Consumable::Heal(50.0)
        Sprite { image }
    }
}

pub(super) fn on_consume(
    consume: On<Consume>,
    mut commands: Commands,
    consumable_query: Query<&Consumable>,
) {
    let item = consume.item;

    if let Ok(consumable) = consumable_query.get(item) {
        match consumable {
            Consumable::Heal(amount) => {
                commands.trigger(AttemptHeal {
                    entity: consume.entity,
                    amount: *amount,
                });
            }
        }
        commands.entity(item).despawn();
    }
}
