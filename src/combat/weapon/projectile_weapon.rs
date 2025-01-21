use bevy::prelude::*;

use crate::{
    combat::{projectile::components::ProjectileBundle, spells::spell_factory::SpellFactory},
    player::systems::AimPosition,
};

use super::{events::WeaponAttackTrigger, weapon::Weapon};

#[derive(Component)]
pub struct ProjectileWeapon {
    pub projectile: ProjectileBundle,
    pub spread: f32,
}

pub fn on_weapon_attack(
    trigger: Trigger<WeaponAttackTrigger>,
    mut commands: Commands,
    mut weapon_query: Query<(&Parent, &mut Weapon, Option<&ProjectileWeapon>)>,
    holder_query: Query<(&Transform, &AimPosition)>,
) {
    if let Ok((attacker, mut weapon, shooter)) = weapon_query.get_mut(trigger.entity()) {
        // Parent needs to have an aim position
        if let Ok((caster_transform, caster_aim_pos)) = holder_query.get(attacker.get()) {
            // Fire projectile weapon
            if shooter.is_some() && weapon.attack_rate.finished() {
                SpellFactory::spawn_spell(
                    &mut commands,
                    caster_transform.translation.truncate(),
                    caster_aim_pos.position,
                    &shooter.unwrap().projectile,
                );
            } // else swing melee weapon here

            weapon.attack_rate.reset();
        }
    }
}
