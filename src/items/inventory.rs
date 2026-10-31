use bevy::prelude::*;

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

pub(super) fn on_item_added_to_inventory(
    item_added: On<Add, ItemOf>,
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
