use bevy::prelude::*;

use crate::{
    combat::spells::{components::Spell, spell_factory::SpellFactory},
    configuration::assets::SpriteAssets,
    player::systems::AimPosition,
};

use super::weapon::Weapon;

#[derive(Event)]
pub struct MainHandActivated;

#[derive(Component, Default)]
pub struct ProjectileWeapon {
    pub spread: f32,
}

pub fn on_main_hand_activated(
    trigger: Trigger<MainHandActivated>,
    mut commands: Commands,
    mut weapon_query: Query<(&Parent, &mut Weapon, Option<&ProjectileWeapon>)>,
    holder_query: Query<(&Transform, &AimPosition)>,
    sprites: &Res<SpriteAssets>,
    texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
) {
    if let Ok((attacker, mut weapon, shooter)) = weapon_query.get_mut(trigger.entity()) {
        // Parent needs to have an aim position
        if let Ok((caster_transform, caster_aim_pos)) = holder_query.get(attacker.get()) {
            // Fire projectile weapon
            if shooter.is_some() && weapon.attack_rate.finished() {
                SpellFactory::spawn_spell(
                    &mut commands,
                    Spell::Fireball,
                    caster_transform.translation.truncate(),
                    caster_aim_pos.position,
                    &sprites,
                    texture_atlas_layouts,
                );
            } // else swing melee weapon here

            weapon.attack_rate.reset();
        }
    }
}
