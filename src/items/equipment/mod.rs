use bevy::prelude::*;

mod equip;
mod equipment_transform;
mod slot;

use crate::prelude::*;

pub mod prelude {
    pub use super::equip::{Equip, Unequip};
    pub use super::equipment_transform::{
        DEFAULT_EQUIPMENT_POSES, DirectionalPoses, EquipmentPose,
    };
    pub use super::slot::{EquipmentSlot, Equippable, Equipped, Mainhand, Offhand};
}

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(equip::plugin);

    app.add_systems(
        Update,
        // Always run this system InGame and InMenu so weapon transforms update as inventory is
        // interacted with
        equipment_transform::update_equipment_transforms.in_set(MainSystems::Shared),
    );
}
