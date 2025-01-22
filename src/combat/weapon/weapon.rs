use bevy::prelude::*;

use crate::{
    combat::projectile::{components::ProjectileBundle, systems::spawn_projectile},
    player::systems::AimPosition,
};

#[derive(Component, Default)]
pub struct Weapon;

#[derive(Component)]
#[require(Weapon)]
pub struct ProjectileWeapon {
    pub projectile: ProjectileBundle,
    pub spread: f32,
}

// We can use the same event for swords, fists, potions thrown, bows, staffs etc
// and add different observers to different respective entities
#[derive(Event)]
pub struct UseEquipmentEvent {
    pub holder: Entity, // entity holding the equipment
}

// "fired" implies this is a projectile weapon
pub fn on_weapon_fired(
    fired_trigger: Trigger<UseEquipmentEvent>,
    mut commands: Commands,
    weapon_query: Query<&ProjectileWeapon>,
    holder_query: Query<(&Transform, &AimPosition)>,
) {
    let Ok(projectile_weapon) = weapon_query.get(fired_trigger.entity()) else {
        warn!("Tried to fire weapon that is not a projectile weapon");
        return;
    };

    let Ok((holder_transform, holder_aim)) = holder_query.get(fired_trigger.holder) else {
        warn!("Tried to fire weapon with holder missing aim position or transform");
        return;
    };

    spawn_projectile(
        &mut commands,
        holder_transform,
        holder_aim.position,
        &projectile_weapon.projectile,
    );
}
