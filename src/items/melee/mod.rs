mod attack;
mod swing;
mod weapons;

pub mod prelude {
    pub use super::{axe, freeze_axe, sword};
}

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

pub use self::attack::ActiveMeleeAttack;
pub use self::weapons::{axe, freeze_axe, sword};
use crate::prelude::*;

/// Our pixel weapons all face upwards currently, so we must rotate them 90 degrees for attacks to
/// occur in the direction we expect. This value will need to be updated if our assets change
pub const MELEE_WEAPON_ROTATION: f32 = std::f32::consts::FRAC_PI_2;

/// Farthest target distance at which an AI holder swings a melee weapon.
pub const MELEE_WEAPON_REACH: f32 = 68.0;

pub(super) fn plugin(app: &mut App) {
    app.add_input_context::<MeleeWeapon>();

    app.add_systems(
        Update,
        (
            swing::process_melee_attacks.before(AiSet::Perception),
            attack::end_melee_attacks,
        )
            .chain()
            .in_set(SimulationSystems::Logic),
    );

    app.add_systems(
        Update,
        attack::handle_melee_collisions.in_set(InGameSystems::Collision),
    );
}

//Repesent a melee weapon
#[derive(Component, Clone, Debug)]
#[require(ContextPriority::<MeleeWeapon>::new(1))]
struct MeleeWeapon {
    // Time it takes (seconds) to complete the attack, smaller = faster
    attack_time: f32,
    damage: (f32, f32),
    hitbox: Collider,
    attack_type: swing::MeleeSwingType,
    hold_distance: f32,
}

impl MeleeWeapon {
    /// Collision layers that hit the holder's opponents.
    pub fn collision_layers(side: CombatSide) -> CollisionLayers {
        CollisionLayers::new(GameCollisionLayer::HitBox, side.target_hurtbox())
    }
}
