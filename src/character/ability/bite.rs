use avian2d::prelude::*;
use bevy::prelude::*;

use super::Ability;
use crate::prelude::*;

/// Seconds a bite lasts with its hitbox out: the jaws stay open through a lunge, so a bite started
/// just short of its reach still lands on a target the biter closes on.
const BITE_HITBOX_SECONDS: f32 = 0.3;

/// A snap of the jaws: a brief round hitbox in front of the biter, along its aim, whose front
/// edge sits at the ability's [`ActionReach`].
#[derive(Component, Clone, Copy, Debug)]
#[require(Ability::Bite)]
pub struct Bite {
    damage: f32,
    life_steal: f32,
    hitbox_radius: f32,
    mouth_height: f32,
}

impl Bite {
    /// Bites for `damage` with a hitbox of `hitbox_radius`.
    pub fn new(damage: f32, hitbox_radius: f32) -> Self {
        Self {
            damage,
            life_steal: 0.0,
            hitbox_radius,
            mouth_height: 0.0,
        }
    }

    /// Centers the hitbox `mouth_height` above the biter's transform before it is pushed along
    /// the aim.
    pub fn with_mouth_height(self, mouth_height: f32) -> Self {
        Self {
            mouth_height,
            ..self
        }
    }

    /// Heals the biter by `fraction` of the damage each landed bite deals.
    pub fn with_life_steal(self, fraction: f32) -> Self {
        Self {
            life_steal: fraction,
            ..self
        }
    }
}

/// A bite ability carrying `bite` that reaches `reach`.
pub fn bite(reach: f32, bite: Bite) -> impl Scene {
    bsn! {
        ~{bite}
        ~{ActionReach(reach)}
        ~{ActionUseStyle::Timed { seconds: BITE_HITBOX_SECONDS }}
        Attack
        on(on_bite_took_effect)
    }
}

/// Snaps a bite along the holder's aim. The hitbox decides whether it lands.
fn on_bite_took_effect(
    took_effect: On<ActionTookEffect>,
    mut commands: Commands,
    bites: Query<(&Bite, &ActionReach)>,
    biting_characters: Query<(&Vision, &StatMultipliers, &CombatSide)>,
) -> Result {
    let (bite, &ActionReach(reach)) = bites.get(took_effect.source)?;
    let holder = took_effect.holder;
    let Ok((vision, stat_multipliers, side)) = biting_characters.get(holder) else {
        return Ok(());
    };
    let Some(direction) = vision.aim_direction.try_normalize() else {
        return Ok(());
    };
    let mouth_offset = Vec2::Y * bite.mouth_height + direction * (reach - bite.hitbox_radius);
    let damage = Damage::Single(bite.damage * stat_multipliers.damage);
    let mut hitbox = commands.spawn((
        HitBox::new(holder, damage, direction),
        Collider::circle(bite.hitbox_radius),
        side.hitbox_layers(),
        Transform::from_translation(mouth_offset.extend(0.0)),
        ChildOf(holder),
        SpawnedForUse {
            source: took_effect.source,
        },
    ));
    if bite.life_steal > 0.0 {
        hitbox.insert(LifeSteal(bite.life_steal));
    }
    Ok(())
}
