use crate::{
    combat::{damage::DamageDealtEvent, health::HealedEvent},
    configuration::ZLayer,
    despawn::components::LiveDuration,
};
use avian2d::prelude::ColliderAabb;
use bevy::prelude::*;
use rand::Rng;

const RED_COLOR: Color = Color::srgb(1.0, 0.0, 0.0);
const GREEN_COLOR: Color = Color::srgb(0.0, 0.8, 0.0);
const HEALTH_TEXT_OFFSET: f32 = 10.0;

fn spawn_health_change_text(
    commands: &mut Commands,
    entity: Entity,
    amount: f32,
    color: Color,
    collider_query: &Query<&ColliderAabb>,
) {
    let entity_height = if let Ok(collider) = collider_query.get(entity) {
        collider.max.y - collider.min.y
    } else {
        32.0 // assume entity is 32 pixels tall if no collider can be found
    };

    // Create a quaternion for the random rotation
    let random_rotation = Quat::from_axis_angle(Vec3::Z, random_angle(30.0));

    // Get rotation assuming sprite is facing "UP" (y axis)
    let rotated_vector = (random_rotation * Vec3::Y).truncate();

    // Text height is relative to center of entity, so we get half of entity height and add a buffer
    let text_height = (entity_height / 2.0) + HEALTH_TEXT_OFFSET;

    // Scale the direction vector by the desired text height to place the text above the entity
    let text_position = (rotated_vector.normalize() * text_height).extend(ZLayer::AboveSprite.z());

    let rounded_amount = (amount * 10.0).round() / 10.0; // Round to 1 decimal place
    let formatted_amount = if rounded_amount.fract() == 0.0 {
        format!("{:.0}", rounded_amount) // Display as a whole number
    } else {
        format!("{:.1}", rounded_amount) // Display with one decimal place
    };

    commands.entity(entity).with_child((
        Text2d::new(formatted_amount),
        TextColor::from(color),
        LiveDuration::new(0.4),
        Transform::from_translation(text_position),
    ));
}

// Generate a random angle between -angle_range and angle_range degrees (convert to radians)
fn random_angle(angle_range: f32) -> f32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(-angle_range..angle_range).to_radians()
}

pub fn on_damage_overlay_amount(
    damage_trigger: Trigger<DamageDealtEvent>,
    mut commands: Commands,
    damaged_query: Query<&ColliderAabb>,
) {
    spawn_health_change_text(
        &mut commands,
        damage_trigger.entity(),
        damage_trigger.damage,
        RED_COLOR,
        &damaged_query,
    );
}

pub fn on_healing_overlay_amount(
    healing_trigger: Trigger<HealedEvent>,
    mut commands: Commands,
    healed_query: Query<&ColliderAabb>,
) {
    spawn_health_change_text(
        &mut commands,
        healing_trigger.entity(),
        healing_trigger.amount,
        GREEN_COLOR,
        &healed_query,
    );
}
