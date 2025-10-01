use avian2d::prelude::Collider;
use bevy::prelude::*;

use crate::{
    labels::sets::{InGameSet, MainSet},
    utility::Lifespan,
};

mod consumable;
pub mod equipment;
pub mod lootable;
mod magnet;
mod mainhand_factory;
mod offhand_factory;

pub use consumable::{health_potion, Consumable, ConsumeEvent};
pub use magnet::Magnet;
pub use mainhand_factory::*;
pub use offhand_factory::*;

pub fn plugin(app: &mut App) {
    app.add_plugins(equipment::EquipmentPlugin);

    app.add_systems(
        FixedUpdate,
        magnet::update_magnet_locations.in_set(MainSet::InGame),
    )
    .add_observer(on_item_added);

    app.add_systems(
        Update,
        (lootable::glow_and_rotate_lootables.in_set(InGameSet::Vfx),),
    )
    .add_observer(lootable::on_drop_event)
    .add_observer(consumable::on_consume_event);
}

fn on_item_added(trigger: Trigger<OnAdd, Item>, mut commands: Commands) {
    // We do this to avoid having to manually add this observer to every item we create
    commands
        .entity(trigger.target())
        .observe(lootable::on_lootable_item_interaction);
}

/// This is the base component for all items in the game. If you have a new concept that will be
/// shared by all items, add it as a field here.
///
/// Ex.  All items can be dropped, so drop-related info can go here
#[derive(Component)]
#[require(Visibility::Hidden)]
pub struct Item {
    pub value: i32,
    pub item_type: ItemType,
    pub drop_glow_effect: f32,
    pub drop_rotation_timer: f32,
    pub drop_rate: f32,
}

impl Default for Item {
    fn default() -> Self {
        Item {
            value: 10,
            item_type: ItemType::Melee,
            drop_glow_effect: 0.0,
            drop_rotation_timer: 0.0,
            drop_rate: 0.0,
        }
    }
}

#[derive(Component, Clone)]
#[relationship(relationship_target = Items)]
pub struct ItemOf(pub Entity);

#[derive(Component, Clone, Debug)]
#[relationship_target(relationship = ItemOf, linked_spawn)]
pub struct Items(Vec<Entity>);

impl Item {
    pub fn new(value: i32, item_type: ItemType) -> Self {
        Item {
            value,
            item_type,
            drop_rate: 1.2,
            ..default()
        }
    }
}

pub enum ItemType {
    Melee,
    Staff,
    Potion,
    Tome,
}

#[derive(Component)]
pub struct HealingTome {
    pub healing: (f32, f32),
}

#[derive(Component)]
#[require(Lifespan::new(1.0))]
pub struct HealingTomeSpellVisualEffect;

#[derive(Component)]
pub struct Shield {
    pub hitbox: Collider,
}

//This component tags items that are active continiously while being used
//e.g. Holding right will keep a shield up
#[derive(Component)]
pub struct Holdable;
