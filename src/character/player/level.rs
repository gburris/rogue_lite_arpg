use bevy::prelude::*;

use super::Player;
use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            on_player_experience_change.in_set(SimulationSystems::Derive),
            animate_level_up.in_set(VfxSystems::Animate),
        ),
    )
    .add_observer(on_level_up);
}

#[derive(Event)]
struct PlayerLeveledUp;

#[derive(Component, Default, Clone)]
struct LevelUpEffect;

#[derive(Component, Default, Clone)]
struct LevelUpText;

/// Radius of the level up ring when its animation starts.
const LEVEL_UP_RING_START_RADIUS: f32 = 5.0;
/// Radius of the level up ring when its animation ends.
const LEVEL_UP_RING_END_RADIUS: f32 = 40.0;
const MAX_RING_SCALE: f32 = LEVEL_UP_RING_END_RADIUS / LEVEL_UP_RING_START_RADIUS;
const LEVEL_UP_ROTATION_SPEED: f32 = 2.0;
const LEVEL_UP_ANIMATION_DURATION: f32 = 1.2;
const LEVEL_UP_TEXT_MAX_HEIGHT: f32 = 100.0;

fn on_player_experience_change(mut commands: Commands, mut player: Single<&mut Player>) {
    while player.attempt_level_up() {
        commands.trigger(PlayerLeveledUp);
    }
}

fn on_level_up(
    _: On<PlayerLeveledUp>,
    mut commands: Commands,
    player: Single<Entity, With<Player>>,
) {
    let player = *player;
    commands.spawn_scene(bsn! { @ring_effect() ChildOf(player) });
    commands.spawn_scene(bsn! { @level_up_text() ChildOf(player) });
}

/// A golden ring that grows, spins and fades around the player.
fn ring_effect() -> impl Scene {
    bsn! {
        LevelUpEffect
        // Each ring owns its material, so fading one leaves the others alone.
        Mesh2d(asset_value(Circle::new(LEVEL_UP_RING_START_RADIUS)))
        MeshMaterial2d<ColorMaterial>(asset_value(Color::srgba(1.0, 0.9, 0.0, 0.7)))
        Transform::from_translation({Vec2::ZERO.extend(ZLayer::BehindSprite.z())})
        Lifespan::new(LEVEL_UP_ANIMATION_DURATION)
    }
}

/// "Level up!" rising above the player's head.
fn level_up_text() -> impl Scene {
    bsn! {
        LevelUpText
        Text2d("Level up!")
        TextFont { font_size: FontSize::Px(24.0) }
        TextColor(Color::srgb(1.0, 0.84, 0.0))
        Transform::from_xyz(0.0, 60.0, {ZLayer::BehindSprite.z()})
        Lifespan::new(LEVEL_UP_ANIMATION_DURATION)
    }
}

fn animate_level_up(
    mut effect_query: Query<
        (&mut Transform, &MeshMaterial2d<ColorMaterial>, &Lifespan),
        (With<LevelUpEffect>, Without<LevelUpText>),
    >,
    mut text_query: Query<(&mut Transform, &Lifespan), With<LevelUpText>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Animate ring effect
    for (mut transform, material, duration) in &mut effect_query {
        let progress = duration.0.fraction();

        // Scale up and rotate
        transform.scale = Vec3::splat(MAX_RING_SCALE * progress);
        transform.rotate_z(LEVEL_UP_ROTATION_SPEED * duration.0.elapsed_secs());

        // Fade out
        if let Some(mut material) = materials.get_mut(&material.0) {
            material.color.set_alpha(1.0 - progress);
        }
    }

    // Animate text
    for (mut transform, duration) in &mut text_query {
        transform.translation.y = LEVEL_UP_TEXT_MAX_HEIGHT * duration.0.fraction();
    }
}

/// The player's upgradeable attributes.
#[derive(Component, Clone, Copy)]
pub struct PlayerAttributes {
    pub agility: u32,   // Movement speed, roll range
    pub strength: u32,  // Melee swing damage
    pub dexterity: u32, // Critical Strike Chance
    pub intellect: u32, // Spell damage
    pub luck: u32,      // Drop rate
}

impl Default for PlayerAttributes {
    fn default() -> Self {
        Self {
            agility: 1,
            strength: 1,
            dexterity: 1,
            intellect: 1,
            luck: 99,
        }
    }
}

/// One of the [`PlayerAttributes`] fields, for shop UI and attribute changes.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Attribute {
    #[default]
    Agility,
    Strength,
    Dexterity,
    Intellect,
    Luck,
}

impl Attribute {
    /// Every attribute, in the order menus list them.
    pub const ALL: [Self; 5] = [
        Self::Agility,
        Self::Strength,
        Self::Dexterity,
        Self::Intellect,
        Self::Luck,
    ];

    pub fn description(self) -> &'static str {
        match self {
            Self::Agility => "Movement speed, roll range",
            Self::Strength => "Melee swing damage",
            Self::Dexterity => "Critical Strike Chance",
            Self::Intellect => "Spell damage",
            Self::Luck => "Drop rate",
        }
    }

    pub fn value(self, attributes: &PlayerAttributes) -> u32 {
        match self {
            Self::Agility => attributes.agility,
            Self::Strength => attributes.strength,
            Self::Dexterity => attributes.dexterity,
            Self::Intellect => attributes.intellect,
            Self::Luck => attributes.luck,
        }
    }
}
