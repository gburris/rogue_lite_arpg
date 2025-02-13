use avian2d::prelude::ColliderAabb;
use bevy::prelude::*;
use rand::Rng;

use crate::{
    combat::damage::events::DamageDealtEvent, despawn::components::LiveDuration,
    labels::layer::ZLayer, player::Player,
};

const RED_COLOR: bevy::prelude::Color = Color::srgb(1.0, 0.0, 0.0);
const DAMAGE_TEXT_OFFSET: f32 = 10.0;

pub fn on_damage_overlay_amount(
    damage_trigger: Trigger<DamageDealtEvent>,
    mut commands: Commands,
    damaged_query: Query<&ColliderAabb>,
    player: Single<Entity, With<Player>>,
) {
    if damage_trigger.entity() == *player {
        return; // No damage overlay for the player, player has a health bar
    }

    let entity_height = if let Ok(collider) = damaged_query.get(damage_trigger.entity()) {
        collider.max.y - collider.min.y
    } else {
        32.0 // assume entity is 32 pixels tall if no collider can be found
    };

    // Create a quaternion for the random rotation
    let random_rotation = Quat::from_axis_angle(Vec3::Z, random_angle(30.0));

    // Get rotation assuming sprite is facing "UP" (y axis)
    let rotated_vector = (random_rotation * Vec3::Y).truncate();

    // Text height is relative to center of entity, so we get half of entity height and add a buffer
    let text_height = (entity_height / 2.0) + DAMAGE_TEXT_OFFSET;

    // Scale the direction vector by the desired text height to place the damage text above the entity
    let text_position = (rotated_vector.normalize() * text_height).extend(ZLayer::VisualEffect.z());

    let damage = damage_trigger.damage;
    let rounded_damage = (damage * 10.0).round() / 10.0; // Round to 1 decimal place
    let formatted_damage = if rounded_damage.fract() == 0.0 {
        format!("{:.0}", rounded_damage) // Display as a whole number
    } else {
        format!("{:.1}", rounded_damage) // Display with one decimal place
    };

    commands.entity(damage_trigger.entity()).with_child((
        Text2d::new(formatted_damage),
        TextColor::from(RED_COLOR),
        LiveDuration::new(0.4),
        Transform::from_translation(text_position),
    ));
}

// Generate a random angle between -angle_range and angle_range degrees (convert to radians)
fn random_angle(angle_range: f32) -> f32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(-angle_range..angle_range).to_radians()
}
