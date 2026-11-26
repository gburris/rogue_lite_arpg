use avian2d::prelude::*;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::prelude::*;

/// An innate move a character can use, carried out by a capability component of the same name,
/// such as [`Bite`], on one of its [`Abilities`].
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[require(ActionSource)]
pub enum Ability {
    /// Drink a carried potion.
    Heal,
    /// Loose an innate projectile with no weapon.
    Breath,
    /// Damage a target in contact reach.
    Bite,
    /// Raise the stats of every ally in reach with a war cry.
    Rally,
}

/// Links an ability entity to the character that uses it.
#[derive(Component, Clone)]
#[relationship(relationship_target = Abilities)]
pub struct AbilityOf {
    pub holder: Entity,
}

/// A character's ability entities, each carrying one capability component and whatever it
/// applies or fires.
#[derive(Component, Clone)]
#[relationship_target(relationship = AbilityOf, linked_spawn)]
pub struct Abilities(Vec<Entity>);

/// Checks in debug builds that no other ability of the new ability's holder is of its kind, since
/// [`ActionKey::Ability`] names abilities by kind.
pub(super) fn on_ability_added(
    added: On<Add<AbilityOf>>,
    abilities: Query<(&AbilityOf, &Ability)>,
    holders: Query<&Abilities>,
) {
    let Ok((&AbilityOf { holder }, &ability)) = abilities.get(added.entity) else {
        return;
    };
    let Ok(holder_abilities) = holders.get(holder) else {
        return;
    };
    // The new ability may not be in `Abilities` yet, so only the others are counted.
    let other_same_kind_count = holder_abilities
        .iter()
        .filter(|&other| other != added.entity)
        .filter(|&other| abilities.get(other).is_ok_and(|(_, &kind)| kind == ability))
        .count();
    debug_assert!(
        other_same_kind_count == 0,
        "character {holder:?} has {other_same_kind_count} more {ability:?} abilities; only the \
         first would ever be used"
    );
}

/// Seconds a bite's hitbox lasts, so a bite started just outside its reach can still hit a target
/// that moves into range.
const BITE_HITBOX_SECONDS: f32 = 0.3;

/// A snap of the jaws: a brief round hitbox in front of the biter, along its aim, whose front
/// edge sits at the ability's [`Reach`].
#[derive(Component, Clone, Copy, Debug)]
#[require(Ability::Bite)]
pub(super) struct Bite {
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
pub(super) fn bite(reach: f32, bite: Bite) -> impl Scene {
    bsn! {
        ~{bite}
        ~{Reach(reach)}
        ~{ActionUseStyle::Timed { seconds: BITE_HITBOX_SECONDS }}
        Attack
        on(on_bite_took_effect)
    }
}

/// Snaps a bite along the holder's aim. The hitbox decides whether it lands.
fn on_bite_took_effect(
    took_effect: On<ActionTookEffect>,
    mut commands: Commands,
    bites: Query<(&Bite, &Reach)>,
    biting_characters: Query<(&Vision, &StatMultipliers, &CombatSide)>,
) -> Result {
    let (bite, &Reach(reach)) = bites.get(took_effect.source)?;
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
        HitBox::new(damage, direction),
        CreditedAttackOf { attacker: holder },
        Collider::circle(bite.hitbox_radius),
        side.hitbox_layers(),
        Transform::from_translation(mouth_offset.extend(0.0)),
        ChildOf(holder),
        ActionUsePartOf {
            action_use: took_effect.action_use,
        },
    ));
    if bite.life_steal > 0.0 {
        hitbox.insert(LifeSteal(bite.life_steal));
    }
    Ok(())
}

/// An innate projectile attack fired from its ability entity's [`Projectiles`] templates along the
/// holder's aim.
#[derive(Component, Clone, Copy, Debug, Default)]
#[require(Ability::Breath)]
struct Breath;

/// A breath ability that fires its [`Projectiles`] templates at targets within `reach`. The reach
/// only tells a brain when to breathe; a projectile may fly past it.
pub(super) fn breath(reach: f32) -> impl Scene {
    bsn! {
        Breath
        ~{Reach(reach)}
        Attack
        on(on_breath_took_effect)
    }
}

/// Fires the breath's projectile templates from the holder along its aim.
fn on_breath_took_effect(
    took_effect: On<ActionTookEffect>,
    mut commands: Commands,
    breathers: Query<(&Transform, &Vision)>,
) {
    let Ok((transform, vision)) = breathers.get(took_effect.holder) else {
        return;
    };
    commands.trigger(FireProjectiles {
        source: took_effect.source,
        holder: took_effect.holder,
        position: transform.translation.xy(),
        aim_direction: vision.aim_direction,
    });
}

/// Drinking a carried potion to recover health.
#[derive(Component, Clone, Copy, Debug, Default)]
#[require(Ability::Heal)]
pub struct Heal;

/// A heal ability that drinks the holder's first carried potion when it takes effect.
pub(super) fn heal() -> impl Scene {
    bsn! {
        Heal
        ~{RequiredItem(ItemKind::Potion)}
        on(on_heal_took_effect)
    }
}

/// Drinks the holder's first carried potion, if it still carries one.
fn on_heal_took_effect(
    took_effect: On<ActionTookEffect>,
    mut commands: Commands,
    heals: Query<&RequiredItem, With<Heal>>,
    carried_items: CarriedItems,
) -> Result {
    let &RequiredItem(kind) = heals.get(took_effect.source)?;
    if let Some(potion) = carried_items.first_of_kind(took_effect.holder, kind) {
        commands.trigger(Consume {
            entity: took_effect.holder,
            item: potion,
        });
    }
    Ok(())
}

/// Radius of the shockwave sheet's ring at its widest, in sheet pixels.
const RALLY_SHOUT_RING_RADIUS: f32 = 87.0;

/// A war cry that gives every other ally within the ability's [`Reach`] its ability entity's
/// [`Effects`].
#[derive(Component, Clone, Copy, Debug, Default)]
#[require(Ability::Rally)]
pub struct Rally;

/// A [`Rally`] ability whose war cry raises the speed and damage of allies within `reach` by
/// `raise` for `seconds`.
pub(super) fn speed_and_damage_rally(reach: f32, raise: f32, seconds: f32) -> impl Scene {
    bsn! {
        Rally
        ~{Reach(reach)}
        on(on_rally_took_effect)
        Effects [
            Rallied
            Lifespan::new(seconds)
            --
            @stat_modifier(Stat::Speed, raise)
            Lifespan::new(seconds)
            --
            @stat_modifier(Stat::Damage, raise)
            Lifespan::new(seconds)
        ]
    }
}

/// Gives every other ally in reach the rally's [`Effects`] and sends a shockwave out from the
/// holder's feet.
fn on_rally_took_effect(
    took_effect: On<ActionTookEffect>,
    mut commands: Commands,
    sprites: Res<SpriteAssets>,
    sprite_layouts: Res<SpriteSheetLayouts>,
    rallies: Query<(&Effects, &Reach), With<Rally>>,
    allies: Allies,
) -> Result {
    let (effects, &Reach(reach)) = rallies.get(took_effect.source)?;
    let holder = took_effect.holder;
    for ally in allies.within(holder, reach) {
        commands.trigger(ApplyEffects::new(effects, ally.entity));
    }
    commands.spawn_scene(bsn! {
        @rally_shout_vfx(&sprites, &sprite_layouts, reach)
        ChildOf(holder)
    });
    Ok(())
}

/// A one-shot shockwave ring racing out along the ground from the holder's feet to `reach`.
fn rally_shout_vfx(
    sprites: &SpriteAssets,
    sprite_layouts: &SpriteSheetLayouts,
    reach: f32,
) -> impl Scene {
    let scale = reach / RALLY_SHOUT_RING_RADIUS;
    bsn! {
        @one_shot_vfx(sprites.rally_shout.clone(), sprite_layouts.rally_shout.clone(), 8, 0.071)
        Transform {
            translation: Vec3::new(0.0, CHARACTER_FEET_POS_OFFSET, {ZLayer::BehindSprite.z()}),
            scale: Vec3::new(scale, scale, 1.0),
        }
    }
}

/// Finds a character's allies: the other living characters on its side, gathered on each call.
#[derive(SystemParam)]
pub(super) struct Allies<'w, 's> {
    characters: Query<
        'w,
        's,
        (
            Entity,
            &'static Transform,
            &'static CombatSide,
            &'static Health,
        ),
    >,
}

/// An ally found by [`Allies`], and its distance from the character that asked.
#[derive(Clone, Copy, Debug)]
pub(super) struct Ally {
    pub entity: Entity,
    pub distance: f32,
}

impl Allies<'_, '_> {
    /// The closest ally of `entity`, if any is alive.
    pub(super) fn closest(&self, entity: Entity) -> Option<Ally> {
        self.allies_of(entity)
            .min_by(|ally, other| ally.distance.total_cmp(&other.distance))
    }

    /// The allies of `entity` at most `reach` away, in no particular order.
    pub(super) fn within(&self, entity: Entity, reach: f32) -> impl Iterator<Item = Ally> {
        self.allies_of(entity)
            .filter(move |ally| ally.distance <= reach)
    }

    fn allies_of(&self, entity: Entity) -> impl Iterator<Item = Ally> {
        let caller = self.characters.get(entity).ok();
        caller
            .into_iter()
            .flat_map(move |(_, caller_transform, caller_side, _)| {
                let position = caller_transform.translation.xy();
                self.characters
                    .iter()
                    .filter(move |(ally, _, side, health)| {
                        *ally != entity && side == &caller_side && !health.is_dead()
                    })
                    .map(move |(ally, transform, ..)| Ally {
                        entity: ally,
                        distance: transform.translation.xy().distance(position),
                    })
            })
    }
}
