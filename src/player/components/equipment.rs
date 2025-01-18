use bevy::prelude::*;

#[derive(Component)]
pub struct PlayerEquipmentSlots {
    pub mainhand: Option<Entity>,
    pub head: Option<Entity>,
}

impl Default for PlayerEquipmentSlots {
    fn default() -> Self {
        PlayerEquipmentSlots {
            mainhand: None,
            head: None,
        }
    }
}
