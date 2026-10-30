use bevy::prelude::*;

mod equip;
mod equipment_transform;
mod equippable;
mod slot;
mod use_equipped;

use crate::prelude::*;

pub mod prelude {
    pub use super::equip::{Equip, Unequip};
    pub use super::equipment_transform::{
        DEFAULT_EQUIPMENT_POSES, DirectionalPoses, EquipmentPose,
    };
    pub use super::equippable::{Equippable, Equipped};
    pub use super::slot::{EquipmentSlot, Mainhand, Offhand};
    pub use super::use_equipped::{
        EquipmentInUse, EquipmentRead, EquipmentUsage, EquipmentUseFailed, EquipmentUseFailure,
        EquipmentUseFinished, EquipmentUseSucceeded, StartUsingEquipment, StopUsingEquipment,
    };
}

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(equip::plugin)
        .add_observer(use_equipped::on_equipment_use_ended);

    app.add_systems(
        Update,
        (
            // Always run this system InGame and InMenu so weapon transforms update as inventory is
            // interacted with
            equipment_transform::update_equipment_transforms.in_set(MainSystems::Shared),
            use_equipped::tick_equippable_cooldown.in_set(SimulationSystems::Upkeep),
        ),
    );
}
