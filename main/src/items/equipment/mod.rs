mod equip;
mod equipment_transform;
mod equippable;
mod plugin;
mod unequip;
mod use_equipped;

// Only expose the things outside modules need!!!

// Components!!
pub use equipment_transform::EquipmentTransform;
pub use equippable::EquipmentSlot;
pub use equippable::Equippable;
pub use equippable::Equipped;

// Equipment events to "get shit done"
pub use use_equipped::EquipmentUseFailedEvent;
pub use use_equipped::EquipmentUseFailure;
pub use use_equipped::UseEquipmentEvent;

// Observers to be added to respective equipment/weapons that want this functionality
pub use use_equipped::on_equipment_activated;
pub use use_equipped::on_equipment_deactivated;
pub use use_equipped::on_healing_tome_cast;
pub use use_equipped::on_shield_block;
pub use use_equipped::on_weapon_fired;
pub use use_equipped::on_weapon_melee;

pub use plugin::EquipmentPlugin;
