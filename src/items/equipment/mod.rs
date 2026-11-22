use bevy::prelude::*;

mod equipment_pose;
mod slot;

pub use self::equipment_pose::{DEFAULT_EQUIPMENT_POSES, DirectionalPoses, EquipmentPose};
pub use self::slot::{Equip, EquipmentSlot, Equippable, Equipped, Mainhand, Offhand, Unequip};
use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(slot::plugin);

    app.add_systems(
        Update,
        // Always run this system InGame and InMenu so weapon transforms update as inventory is
        // interacted with
        equipment_pose::update_equipment_transforms.in_set(MainSystems::Shared),
    );
}
