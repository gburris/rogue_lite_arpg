use bevy::{prelude::*, utils::HashMap};

use crate::{
    combat::{
        status_effects::handle_statuses::*, weapon::projectile_weapon::on_main_hand_activated,
    },
    labels::sets::InGameSet,
};

use super::{
    spells::spell_factory::{spawn_fire_projectile, ProjectileSpawners},
    weapon::weapon::tick_weapon_attack_rate,
};

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_main_hand_activated).add_systems(
            Update,
            tick_weapon_attack_rate.in_set(InGameSet::Simulation),
        );

        let mut projectile_spawners = ProjectileSpawners(HashMap::new());


        let sys = app.register_system(spawn_fire_projectile);
        projectile_spawners.0.insert(
            "fire_spell".into(),
            ,
        );

        app.insert_resource(projectile_spawners);
    }
}
